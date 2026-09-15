//! Parse Stage Helpers
//!
//! 为 parse 阶段提供 AstBuilder 和 feed 函数
//! 实际算子链在 pipeline.rs 中通过 scan_map + flat_map 组装

use tracing;

use crate::ast::{Node, Token};

/// AST 构建器状态 — 累积 Token 直到可产出完整 Node
#[derive(Debug, Clone)]
pub struct AstBuilder {
    /// Token 缓冲区
    buffer: Vec<Token>,
    /// 嵌套深度 (由 { } 决定)
    depth: u32,
}

impl AstBuilder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            depth: 0,
        }
    }

    /// 喂入 Token, 产出 0..N 个完整 Node
    pub fn feed(&mut self, token: Token) -> Vec<Node> {
        self.buffer.push(token.clone());
        self.update_depth(&token);

        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!(?self.buffer, "parse.feed state");
        }

        // 顶层指令识别 (depth==0 时全量识别 @import/@use/@forward/@mixin)
        if self.depth == 0 {
            if let Some(node) = self.try_flush_directive() {
                return vec![node];
            }
        }

        // 行内指令识别 (任意深度):@include / @if / @for 可以在规则体内部出现
        if let Some(node) = self.try_flush_inline_directive() {
            return vec![node];
        }

        // 顶层变量声明识别:$varname : value ;
        if self.depth == 0 {
            if let Some(node) = self.try_flush_variable() {
                return vec![node];
            }
        }

        if self.depth == 0 && self.buffer.contains(&Token::RBrace) {
            return self.flush_rule();
        }
        Vec::new()
    }

    /// 尝试从 buffer 中解析顶层 @import / @use / @forward /
    /// @mixin / @include / @if / @for / @each 等指令
    fn try_flush_directive(&mut self) -> Option<Node> {
        // 检测 "At, Ident(instr)" 起始:至少 2 个 token
        if self.buffer.len() < 2 {
            return None;
        }
        if self.buffer[0] != Token::At {
            return None;
        }
        let instr = match &self.buffer[1] {
            Token::Ident(s) => s.clone(),
            _ => return None,
        };

        match instr.as_str() {
            "import" | "use" | "forward" => self.flush_directive_import_like(&instr),
            "mixin" => self.flush_directive_mixin(),
            "include" => self.flush_directive_include(),
            "if" => self.flush_directive_if(),
            "for" => self.flush_directive_for(),
            "each" => self.flush_directive_each(),
            _ => None,
        }
    }

    /// 尝试从 buffer 任意位置解析行内指令 (@include / @if / @for / @mixin)
    ///
    /// 与 try_flush_directive 不同，本方法从 buffer 中找到第一个 Token::At，
    /// 然后检查其后是否为行内指令名称。这样能在规则体内部 (depth>0) 识别指令。
    fn try_flush_inline_directive(&mut self) -> Option<Node> {
        // 在 buffer 中找到第一个 At
        let at_pos = self.buffer.iter().position(|t| *t == Token::At)?;

        // At 之后必须有至少一个 token:指令名称
        if at_pos + 1 >= self.buffer.len() {
            return None;
        }
        let instr = match &self.buffer[at_pos + 1] {
            Token::Ident(s) => s.clone(),
            _ => return None,
        };

        // 行内指令白名单
        match instr.as_str() {
            "include" => self.flush_inline_from_at(at_pos, "include", false),
            "if" => self.flush_inline_from_at(at_pos, "if", true),
            "for" => self.flush_inline_from_at(at_pos, "for", true),
            "each" => self.flush_inline_from_at(at_pos, "each", true),
            _ => None,
        }
    }

    /// 从 buffer 中指定 at_pos 位置的 At token 开始解析行内指令
    ///
    /// has_body: 若为 true，指令以 LBrace ... RBrace 结束;否则以 Semicolon 结束
    fn flush_inline_from_at(&mut self, at_pos: usize, instr: &str, has_body: bool) -> Option<Node> {
        if has_body {
            // 找 LBrace 然后找匹配的 RBrace
            // buffer[at_pos..] 中找 LBrace
            let lbrace_off = self.buffer[at_pos..].iter().position(|t| *t == Token::LBrace)?;
            let lbrace_abs = at_pos + lbrace_off;
            let rbrace_off = self.buffer[lbrace_abs..].iter().rposition(|t| *t == Token::RBrace)?;
            let rbrace_abs = lbrace_abs + rbrace_off;
            let all_tokens: Vec<_> = self.buffer.drain(at_pos..=rbrace_abs).collect();

            // 复用已有的 body 解析逻辑
            match instr {
                "if" => Self::parse_if_body(&all_tokens),
                "for" => Self::parse_for_body(&all_tokens),
                _ => None,
            }
        } else {
            // 找 Semicolon 或 Newline 结束
            let end_off = self.buffer[at_pos..]
                .iter()
                .position(|t| matches!(t, Token::Semicolon | Token::Newline))
                .map(|p| at_pos + p)?;

            let all_tokens: Vec<_> = self.buffer.drain(at_pos..=end_off).collect();
            // all_tokens[0] = At, all_tokens[1] = Ident(instr), all_tokens[2..] = 参数
            match instr {
                "include" => Self::parse_include_tokens(&all_tokens),
                _ => None,
            }
        }
    }

    /// 解析 @include 调用 token 序列: At Ident("include") Ident(name)? args Semicolon
    fn parse_include_tokens(tokens: &[Token]) -> Option<Node> {
        if tokens.len() < 3 {
            return None;
        }
        let mut idx = 2;
        let name = if let Token::Ident(n) = &tokens.get(idx)? {
            let name = n.clone();
            idx += 1;
            name
        } else {
            return None;
        };

        let args = if matches!(tokens.get(idx), Some(Token::LParen)) {
            AstBuilder::parse_arg_list_at(tokens, idx)
        } else {
            Vec::new()
        };

        Some(Node::MixinCall { name, args })
    }

    /// 解析 @if 指令: At Ident("if") cond LBrace ... RBrace
    fn parse_if_body(tokens: &[Token]) -> Option<Node> {
        if tokens.len() < 4 {
            return None;
        }
        let lbrace_rel = tokens.iter().position(|t| *t == Token::LBrace)?;
        let condition = tokens[2..lbrace_rel]
            .iter()
            .filter_map(|t| match t {
                Token::Ident(s) => Some(s.as_str()),
                Token::Number(s) => Some(s.as_str()),
                Token::Op(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        let rbrace_rel = tokens.iter().rposition(|t| *t == Token::RBrace)?;
        let body_tokens = &tokens[lbrace_rel + 1..rbrace_rel];
        let body = Self::parse_block(body_tokens);

        Some(Node::If {
            condition,
            then_branch: body,
            else_branch: None,
        })
    }

    /// 解析 @for 指令: At Ident("for") Dollar Ident(var) Ident("from") value Ident("to") value LBrace body RBrace
    fn parse_for_body(tokens: &[Token]) -> Option<Node> {
        if tokens.len() < 8 {
            return None;
        }
        let lbrace_rel = tokens.iter().position(|t| *t == Token::LBrace)?;

        // 解析变量名: Dollar Ident(var_name)
        let mut idx = 2; // skip At Ident("for")
        if tokens.get(idx) != Some(&Token::Dollar) {
            return None;
        }
        idx += 1;
        let var = if let Token::Ident(v) = tokens.get(idx)? {
            v.clone()
        } else {
            return None;
        };
        idx += 1;

        // 找 "from" 和 "to"
        let from_off = tokens[idx..lbrace_rel]
            .iter()
            .position(|t| matches!(t, Token::Ident(s) if s == "from"))
            .map(|p| p + idx)?;
        let to_off = tokens[from_off..lbrace_rel]
            .iter()
            .position(|t| matches!(t, Token::Ident(s) if s == "to"))
            .map(|p| p + from_off)?;

        let from = tokens[from_off + 1..to_off]
            .iter()
            .filter_map(|t| match t {
                Token::Number(s) => Some(s.as_str()),
                Token::Ident(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");
        let to = tokens[to_off + 1..lbrace_rel]
            .iter()
            .filter_map(|t| match t {
                Token::Number(s) => Some(s.as_str()),
                Token::Ident(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let rbrace_rel = tokens.iter().rposition(|t| *t == Token::RBrace)?;
        let body_tokens = &tokens[lbrace_rel + 1..rbrace_rel];
        let body = Self::parse_block(body_tokens);

        Some(Node::For { var, from, to, body })
    }

    /// 从 tokens[pos..] 开始解析参数列表 (LParen ... RParen)
    fn parse_arg_list_at(tokens: &[Token], mut pos: usize) -> Vec<String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut depth = 0u32;
        while pos < tokens.len() {
            match &tokens[pos] {
                Token::LParen => depth += 1,
                Token::RParen => {
                    if depth > 0 {
                        depth -= 1;
                    }
                    let c = current.trim().to_string();
                    if !c.is_empty() {
                        args.push(c);
                    }
                    break;
                }
                Token::Comma if depth == 0 => {
                    let c = current.trim().to_string();
                    if !c.is_empty() {
                        args.push(c);
                    }
                    current = String::new();
                }
                Token::Ident(s) => current.push_str(s),
                Token::Number(s) => {
                    if !current.is_empty() {
                        current.push(' ');
                    }
                    current.push_str(s);
                }
                Token::String(s) => {
                    current.push('"');
                    current.push_str(s);
                    current.push('"');
                }
                Token::Dollar => current.push('$'),
                _ => {}
            }
            pos += 1;
        }
        args
    }

    /// 解析 @import / @use / @forward 指令: At Ident(...) ; Semicolon
    fn flush_directive_import_like(&mut self, instr: &str) -> Option<Node> {
        // 寻找 Semicolon 或 Newline 作为指令结束
        let end_rel = self.buffer[2..]
            .iter()
            .position(|t| matches!(t, Token::Semicolon | Token::Newline))
            .map(|p| p + 2)?;

        let directive_tokens = self.buffer.drain(..=end_rel).collect::<Vec<_>>();

        // 提取路径:String 引号内的部分
        let path = directive_tokens
            .iter()
            .skip(2) // 跳过 At, Ident(instr)
            .find_map(|t| match t {
                Token::String(s) => Some(s.clone()),
                Token::Ident(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_default();

        match instr {
            "import" => Some(Node::Import { path }),
            "use" => Some(Node::Use { path }),
            "forward" => Some(Node::Forward { path }),
            _ => None,
        }
    }

    /// 解析 @mixin 定义: At "mixin" Ident(name)? "(" params ")" "{" body "}"
    fn flush_directive_mixin(&mut self) -> Option<Node> {
        // 向前看,寻找 LBrace
        let lbrace_rel = self.buffer.iter().position(|t| *t == Token::LBrace)?;
        // 找匹配 RBrace (假设简单情况:顶层)
        let rbrace_offset = self.buffer[lbrace_rel..]
            .iter()
            .rposition(|t| *t == Token::RBrace)?;
        let rbrace_rel = lbrace_rel + rbrace_offset;

        let all_tokens: Vec<_> = self.buffer.drain(..=rbrace_rel).collect();
        // 解析: At Ident("mixin") [Ident(name)] [LParen params RParen] LBrace body RBrace
        if all_tokens.len() < 3 {
            return None;
        }
        // 期望 [2] = Ident(name) 或直接 LParen
        let mut idx = 2;
        let name = if matches!(all_tokens.get(idx), Some(Token::Ident(_))) {
            if let Token::Ident(n) = &all_tokens[idx] {
                let name = n.clone();
                idx += 1;
                name
            } else {
                return None;
            }
        } else {
            String::new()
        };

        let params = if matches!(all_tokens.get(idx), Some(Token::LParen)) {
            self.parse_param_list(&all_tokens[idx..])
        } else {
            Vec::new()
        };

        // 找到 LBrace/RBrace 在 all_tokens 中的实际位置,提取 body 并解析
        let lbrace_idx = all_tokens.iter().position(|t| *t == Token::LBrace)?;
        let rbrace_idx = all_tokens.iter().rposition(|t| *t == Token::RBrace)?;
        let body_tokens = &all_tokens[lbrace_idx + 1..rbrace_idx];
        let body = Self::parse_block(body_tokens);

        Some(Node::MixinDef {
            name,
            params,
            body,
        })
    }

    /// 解析 @include 调用: At "include" Ident(name)? "(" args ")" ; Semicolon
    fn flush_directive_include(&mut self) -> Option<Node> {
        let end_rel = self.buffer[2..]
            .iter()
            .position(|t| matches!(t, Token::Semicolon | Token::Newline))
            .map(|p| p + 2)?;

        let all_tokens = self.buffer.drain(..=end_rel).collect::<Vec<_>>();
        // At Ident("include") Ident(name)? [LParen args RParen] Semicolon
        let mut idx = 2;
        let name = if let Token::Ident(n) = &all_tokens.get(idx)? {
            let name = n.clone();
            idx += 1;
            name
        } else {
            return None;
        };

        let args = if matches!(all_tokens.get(idx), Some(Token::LParen)) {
            self.parse_arg_list(&all_tokens[idx..])
        } else {
            Vec::new()
        };

        Some(Node::MixinCall { name, args })
    }

    fn flush_directive_if(&mut self) -> Option<Node> {
        // 简单占位 — 顶层 @if: 找到匹配的 RBrace 直到包含完整的 then {}
        let lbrace_rel = self.buffer.iter().position(|t| *t == Token::LBrace)?;
        let rbrace_offset = self.buffer[lbrace_rel..]
            .iter()
            .rposition(|t| *t == Token::RBrace)?;
        let rbrace_rel = lbrace_rel + rbrace_offset;

        let all_tokens: Vec<_> = self.buffer.drain(..=rbrace_rel).collect();
        // 提取条件: At Ident("if") cond LBrace ... RBrace
        if all_tokens.len() < 4 {
            return None;
        }
        // condition 是 At 和 LBrace 之间的 tokens
        let cond_start = 2usize;
        let cond_end = lbrace_rel;
        let condition = all_tokens[cond_start..cond_end]
            .iter()
            .filter_map(|t| match t {
                Token::Ident(s) => Some(s.as_str()),
                Token::Number(s) => Some(s.as_str()),
                Token::Op(s) => Some(s.as_str()),
                Token::Char(c) => Some(Box::leak(c.to_string().into_boxed_str())),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        Some(Node::If {
            condition,
            then_branch: Vec::new(),
            else_branch: None,
        })
    }

    fn flush_directive_for(&mut self) -> Option<Node> {
        let lbrace_rel = self.buffer.iter().position(|t| *t == Token::LBrace)?;
        let rbrace_offset = self.buffer[lbrace_rel..]
            .iter()
            .rposition(|t| *t == Token::RBrace)?;
        let rbrace_rel = lbrace_rel + rbrace_offset;
        let all_tokens: Vec<_> = self.buffer.drain(..=rbrace_rel).collect();

        // @for $i from X to Y { ... }
        // tokens: At Ident("for") Dollar Ident("i") Ident("from") value ... LBrace
        if all_tokens.len() < 5 {
            return None;
        }
        let mut idx = 2;
        if all_tokens.get(idx) != Some(&Token::Dollar) {
            return None;
        }
        idx += 1;
        let var = if let Token::Ident(v) = all_tokens.get(idx)? {
            v.clone()
        } else {
            return None;
        };
        // "from" keyword
        let from_idx = all_tokens[idx..]
            .iter()
            .position(|t| matches!(t, Token::Ident(s) if s == "from"))
            .map(|p| p + idx)?;
        let to_idx = all_tokens[from_idx..]
            .iter()
            .position(|t| matches!(t, Token::Ident(s) if s == "to"))
            .map(|p| p + from_idx)?;

        let from = all_tokens[from_idx + 1..to_idx]
            .iter()
            .filter_map(|t| match t {
                Token::Number(s) => Some(s.as_str()),
                Token::Ident(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");
        let to = all_tokens[to_idx + 1..lbrace_rel]
            .iter()
            .filter_map(|t| match t {
                Token::Number(s) => Some(s.as_str()),
                Token::Ident(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        Some(Node::For {
            var,
            from,
            to,
            body: Vec::new(),
        })
    }

    fn flush_directive_each(&mut self) -> Option<Node> {
        // @each $x in $list { ... } — 暂不完全实现
        let lbrace_rel = self.buffer.iter().position(|t| *t == Token::LBrace)?;
        let rbrace_offset = self.buffer[lbrace_rel..]
            .iter()
            .rposition(|t| *t == Token::RBrace)?;
        let rbrace_rel = lbrace_rel + rbrace_offset;
        let _all_tokens: Vec<_> = self.buffer.drain(..=rbrace_rel).collect();
        // 简化: 返回空的 MixinCall 占位
        tracing::debug!("@each not yet implemented fully");
        Some(Node::Directive {
            name: "each".into(),
            args: String::new(),
        })
    }

    /// 解析参数列表 (位于 LParen ... RParen 之间) -> name 序列
    fn parse_param_list(&mut self, tokens: &[Token]) -> Vec<String> {
        // 期望: LParen [Dollar Ident]* RParen
        let mut params = Vec::new();
        let mut i = 1; // skip LParen
        while i < tokens.len() {
            match &tokens[i] {
                Token::Dollar => {
                    if let Token::Ident(name) = &tokens.get(i + 1).unwrap_or(&Token::Semicolon) {
                        params.push(format!("${name}"));
                        i += 2;
                        continue;
                    }
                }
                Token::RParen => break,
                _ => {}
            }
            i += 1;
        }
        params
    }

    /// 解析调用参数列表 (位于 LParen ... RParen 之间) -> value 序列
    fn parse_arg_list(&mut self, tokens: &[Token]) -> Vec<String> {
        // 期望: LParen [value [, value]*] RParen
        let mut args = Vec::new();
        let mut current = String::new();
        let mut depth = 0u32;
        for t in tokens {
            match t {
                Token::LParen => depth += 1,
                Token::RParen => {
                    if depth > 0 {
                        depth -= 1;
                    }
                    let c = current.trim().to_string();
                    if !c.is_empty() {
                        args.push(c);
                    }
                    break;
                }
                Token::Comma if depth == 0 => {
                    let c = current.trim().to_string();
                    if !c.is_empty() {
                        args.push(c);
                    }
                    current = String::new();
                }
                Token::Ident(s) => current.push_str(s),
                Token::Number(s) => {
                    if !current.is_empty() {
                        current.push(' ');
                    }
                    current.push_str(s);
                }
                Token::String(s) => {
                    current.push('"');
                    current.push_str(s);
                    current.push('"');
                }
                Token::Dollar => current.push('$'),
                _ => {}
            }
        }
        args
    }

    fn update_depth(&mut self, token: &Token) {
        match token {
            Token::LBrace => self.depth += 1,
            Token::RBrace => {
                if self.depth > 0 {
                    self.depth -= 1;
                }
            }
            _ => {}
        }
    }

    /// 尝试从 buffer 中解析顶层 `$var : value ;` 变量声明
    fn try_flush_variable(&mut self) -> Option<Node> {
        // 模式: Dollar Ident PropName (Whitespace) Colon (Whitespace) (value)+ Semicolon
        if self.buffer.len() < 4 {
            return None;
        }
        if self.buffer[0] != Token::Dollar {
            return None;
        }
        let var_name = match &self.buffer[1] {
            Token::Ident(s) => s.clone(),
            _ => return None,
        };

        // 跳过 Whitespace 找到 Colon
        let mut idx = 2;
        while idx < self.buffer.len() && self.buffer[idx] == Token::Whitespace {
            idx += 1;
        }
        if idx >= self.buffer.len() || self.buffer[idx] != Token::Colon {
            return None;
        }
        idx += 1;
        // 跳过 Whitespace 到值
        while idx < self.buffer.len() && self.buffer[idx] == Token::Whitespace {
            idx += 1;
        }
        if idx >= self.buffer.len() {
            return None;
        }

        // 寻找 Semicolon 作为声明结束
        let semi_rel = self.buffer[idx..]
            .iter()
            .position(|t| matches!(t, Token::Semicolon | Token::Newline))?;

        let value_end = idx + semi_rel;
        let value_tokens = &self.buffer[idx..value_end];

        // Trace inside try_flush_variable
        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!(?self.buffer, var_name, value_end, "try_flush_variable attempt");
        }

        let value = value_tokens
            .iter()
            .filter_map(|t| match t {
                Token::Ident(s) => Some(s.as_str()),
                Token::Number(s) => Some(s.as_str()),
                Token::String(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // 检查是否同时含有 LBrace (不是合法变量)
        if self.buffer[..=value_end].contains(&Token::LBrace) {
            return None;
        }

        let _ = self.buffer.drain(..=value_end);
        Some(Node::Variable { name: var_name, value })
    }

    fn flush_rule(&mut self) -> Vec<Node> {
        let tokens = self.buffer.drain(..).collect::<Vec<_>>();

        if let Some(lbrace_idx) = tokens.iter().position(|t| *t == Token::LBrace) {
            let selector_tokens = &tokens[..lbrace_idx];
            let selector = selector_tokens
                .iter()
                .filter_map(|t| match t {
                    Token::Ident(s) => Some(s.as_str()),
                    Token::Dot => Some("."),
                    Token::Hash => Some("#"),
                    Token::Colon => Some(":"),
                    Token::Interpolation(s) => Some(s.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");

            let body_tokens = &tokens[lbrace_idx + 1..];
            let body = Self::parse_declarations(body_tokens);

            vec![Node::Rule { selector, body }]
        } else {
            Vec::new()
        }
    }

    fn parse_declarations(tokens: &[Token]) -> Vec<Node> {
        let mut decls = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // 属性名: 允许 Ident / Dollar / Interpolation / String 片段 (如 `--#{$p}-x`)
            let mut prop_parts = Vec::new();
            while i < tokens.len() {
                match &tokens[i] {
                    Token::Ident(s) => prop_parts.push(s.clone()),
                    Token::Dollar => prop_parts.push("$".to_string()),
                    Token::Interpolation(s) => prop_parts.push(s.clone()),
                    Token::String(s) => prop_parts.push(s.clone()),
                    _ => break,
                }
                i += 1;
            }
            let prop = prop_parts.join("");
            if prop.is_empty() {
                i += 1;
                continue;
            }

            if i >= tokens.len() || !matches!(tokens[i], Token::Colon) {
                i += 1;
                continue;
            }
            i += 1; // 跳过 Colon
              // 值:到 Semicolon / Whitespace / Newline 止
            let mut value_parts = Vec::new();
            while i < tokens.len() {
                match &tokens[i] {
                    Token::Semicolon | Token::Newline => break,
                    Token::Ident(s) => value_parts.push(s.clone()),
                    Token::Number(s) => value_parts.push(s.clone()),
                    Token::String(s) => value_parts.push(s.clone()),
                    Token::Dollar => value_parts.push("$".to_string()),
                    Token::Interpolation(s) => value_parts.push(s.clone()),
                    Token::Whitespace => {}
                    _ => {}
                }
                i += 1;
            }
            let value = value_parts.join("");
            decls.push(Node::Declaration { prop, value });
            if i < tokens.len() && !matches!(tokens[i], Token::Newline) {
                i += 1; // 跳过 Semicolon
            }
        }
        decls
    }

    /// 解析 block 内容 ( Declaration | Rule 混合)
    ///
    /// 用于 @mixin body / @for body / @if body 这些既包含简单声明又包含嵌套规则的场景。
    /// 识别规则:从当前位置扫描到 LBrace → selector;然后找匹配的 RBrace → 递归 parse 内部 declarations。
    fn parse_block(tokens: &[Token]) -> Vec<Node> {
        let mut out = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            // 1. 检测是否是 selector: 从当前位置扫描直到 LBrace 或 Colon 或 Semicolon
            let mut selector_parts = Vec::new();
            let mut scan = i;
            let mut found_lbrace = false;
            let mut lbrace_pos = 0;
            while scan < tokens.len() {
                match &tokens[scan] {
                    Token::LBrace => {
                        found_lbrace = true;
                        lbrace_pos = scan;
                        break;
                    }
                    Token::Colon | Token::Semicolon | Token::Newline if selector_parts.is_empty() => {
                        // 还没积累 selector 就遇到 Colon/Semicolon,这是 Declaration 模式
                        break;
                    }
                    Token::Ident(s) => selector_parts.push(s.clone()),
                    Token::Dot => selector_parts.push(".".to_string()),
                    Token::Hash => selector_parts.push("#".to_string()),
                    Token::Dollar => selector_parts.push("$".to_string()),
                    Token::Interpolation(s) => selector_parts.push(s.clone()),
                    Token::Number(s) => selector_parts.push(s.clone()),
                    _ => break,
                }
                scan += 1;
            }

            if found_lbrace && !selector_parts.is_empty() {
                // Rule 模式: selector { body }
                // 找匹配的 RBrace
                let mut depth = 1i32;
                let mut rb = lbrace_pos + 1;
                while rb < tokens.len() && depth > 0 {
                    match &tokens[rb] {
                        Token::LBrace => depth += 1,
                        Token::RBrace => depth -= 1,
                        _ => {}
                    }
                    rb += 1;
                }
                let body_tokens = &tokens[lbrace_pos + 1..rb - 1];
                let body = Self::parse_block(body_tokens);
                let selector = selector_parts.join("");
                out.push(Node::Rule { selector, body });
                i = rb;
            } else {
                // Declaration 模式或直接跳过
                // 声明: prop : value ;
                let mut prop_parts = Vec::new();
                while i < tokens.len() {
                    match &tokens[i] {
                        Token::Ident(s) => prop_parts.push(s.clone()),
                        Token::Dollar => prop_parts.push("$".to_string()),
                        Token::Interpolation(s) => prop_parts.push(s.clone()),
                        Token::String(s) => prop_parts.push(s.clone()),
                        _ => break,
                    }
                    i += 1;
                }
                let prop = prop_parts.join("");
                if prop.is_empty() {
                    i += 1;
                    continue;
                }
                if i >= tokens.len() || !matches!(tokens[i], Token::Colon) {
                    i += 1;
                    continue;
                }
                i += 1; // Colon
                let mut value_parts = Vec::new();
                while i < tokens.len() {
                    match &tokens[i] {
                        Token::Semicolon | Token::Newline => break,
                        Token::Ident(s) => value_parts.push(s.clone()),
                        Token::Number(s) => value_parts.push(s.clone()),
                        Token::String(s) => value_parts.push(s.clone()),
                        Token::Dollar => value_parts.push("$".to_string()),
                        Token::Interpolation(s) => value_parts.push(s.clone()),
                        Token::Whitespace => {}
                        _ => {}
                    }
                    i += 1;
                }
                let value = value_parts.join("");
                out.push(Node::Declaration { prop, value });
                if i < tokens.len() && !matches!(tokens[i], Token::Newline) {
                    i += 1;
                }
            }
        }
        out
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}
