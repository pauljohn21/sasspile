//! CSS 序列化器——CssNode 树 → CSS 字符串。

pub mod node;

pub use node::CssNode;

use crate::OutputStyle;

/// 序列化器。
pub struct Serializer;

/// 选择器 token——用于组合器验证。
#[derive(Debug)]
enum SelToken {
    /// 普通选择器片段。
    Selector(String),
    /// 组合器（>, +, ~）。
    Combinator(char),
    /// 伪类内部的选择器列表（需递归检查）。
    /// (伪类名, 内部选择器字符串, 是否允许前导组合器)
    PseudoInner(String, String, bool),
}

impl Serializer {
    /// 序列化 CssNode 列表为 CSS 字符串。
    pub fn serialize(nodes: &[CssNode], style: OutputStyle) -> String {
        let flattened = Self::flatten_nodes(nodes);
        let merged = Self::merge_at_rules(flattened);
        let css = match style {
            OutputStyle::Expanded => Self::serialize_expanded(&merged, 0),
            OutputStyle::Compressed => Self::serialize_compressed(&merged),
        };
        // 当输出包含非 ASCII 字符时，Dart Sass 在 expanded 模式下添加 @charset 前缀
        if css.chars().any(|c| !c.is_ascii()) {
            match style {
                OutputStyle::Expanded => format!("@charset \"UTF-8\";\n{css}"),
                OutputStyle::Compressed => format!("@charset\"UTF-8\";{css}"),
            }
        } else {
            css
        }
    }

    /// 合并相邻的 @media/@supports 块（相同 query）。
    fn merge_at_rules(nodes: Vec<CssNode>) -> Vec<CssNode> {
        let mut result: Vec<CssNode> = Vec::new();
        for node in nodes {
            match &node {
                CssNode::AtRule {
                    name,
                    params,
                    children,
                    has_body: true,
                } => {
                    // 检查是否与 result 中最后一个节点同名同 query
                    if let Some(last) = result.last() {
                        if let CssNode::AtRule {
                            name: last_name,
                            params: last_params,
                            children: last_children,
                            has_body: true,
                        } = last
                        {
                            if last_name == name && last_params == params {
                                // 合并 children
                                let mut merged = last_children.clone();
                                merged.extend(children.clone());
                                if let Some(last_mut) = result.last_mut() {
                                    *last_mut = CssNode::AtRule {
                                        name: name.clone(),
                                        params: params.clone(),
                                        children: merged,
                                        has_body: true,
                                    };
                                }
                                continue;
                            }
                        }
                    }
                    result.push(node);
                }
                _ => result.push(node),
            }
        }
        result
    }

    /// 展平嵌套规则。
    fn flatten_nodes(nodes: &[CssNode]) -> Vec<CssNode> {
        let mut result = Vec::new();
        for node in nodes {
            match node {
                CssNode::Rule {
                    selector,
                    declarations,
                    children,
                } => {
                    if !declarations.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: declarations.clone(),
                            children: vec![],
                        });
                    }
                    result.extend(Self::flatten_children(selector, children));
                }
                other => result.push(other.clone()),
            }
        }
        result
    }

    fn flatten_children(_parent: &str, children: &[CssNode]) -> Vec<CssNode> {
        let mut result = Vec::new();
        for child in children {
            match child {
                CssNode::Rule {
                    selector,
                    declarations,
                    children: nested,
                } => {
                    // 选择器已由 Evaluator 合并——不再二次合并
                    if !declarations.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: declarations.clone(),
                            children: vec![],
                        });
                    }
                    result.extend(Self::flatten_children(selector, nested));
                }
                other => result.push(other.clone()),
            }
        }
        result
    }

    fn serialize_expanded(nodes: &[CssNode], depth: usize) -> String {
        let indent = "  ".repeat(depth);
        let mut result = String::new();
        for (i, n) in nodes.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            Self::write_node_expanded(&mut result, n, &indent, depth);
        }
        if depth == 0 {
            result.push('\n');
        }
        result
    }

    fn serialize_compressed(nodes: &[CssNode]) -> String {
        let mut result = String::new();
        for n in nodes {
            Self::write_node_compressed(&mut result, n);
        }
        result
    }

    /// 直接写入 String 缓冲区——避免 format! + collect + join 的多重分配。
    fn write_node_expanded(buf: &mut String, node: &CssNode, indent: &str, depth: usize) {
        match node {
            CssNode::Declaration {
                property,
                value,
                important,
            } => {
                buf.push_str(indent);
                buf.push_str(property);
                buf.push_str(": ");
                buf.push_str(value);
                if *important {
                    buf.push_str(" !important");
                }
                buf.push(';');
            }
            CssNode::Comment(text) => {
                buf.push_str(indent);
                buf.push_str("/* ");
                buf.push_str(text);
                buf.push_str(" */");
            }
            CssNode::AtRoot(nodes) => {
                buf.push_str(&Self::serialize_expanded(nodes, depth));
            }
            CssNode::Rule {
                selector,
                declarations,
                children,
            } => {
                let selector = Self::sanitize_selector(selector);
                if selector.is_empty() {
                    return;
                }
                let inner = "  ".repeat(depth + 1);
                buf.push_str(indent);
                buf.push_str(&selector);
                buf.push_str(" {\n");
                for decl in declarations {
                    if let CssNode::Declaration {
                        property,
                        value,
                        important,
                    } = decl
                    {
                        buf.push_str(&inner);
                        buf.push_str(property);
                        buf.push_str(": ");
                        buf.push_str(value);
                        if *important {
                            buf.push_str(" !important");
                        }
                        buf.push(';');
                        buf.push('\n');
                    }
                }
                if !children.is_empty() {
                    let child_css = Self::serialize_expanded(children, depth + 1);
                    if !child_css.is_empty() {
                        buf.push_str(&child_css);
                        buf.push('\n');
                    }
                }
                buf.push_str(indent);
                buf.push('}');
            }
            CssNode::AtRule {
                has_body: true,
                name,
                params,
                children,
            } => {
                let p = params.as_deref().unwrap_or("");
                if children.is_empty() {
                    buf.push_str(indent);
                    buf.push('@');
                    buf.push_str(name);
                    if !p.is_empty() {
                        buf.push(' ');
                        buf.push_str(p);
                    }
                    buf.push_str(" {}");
                } else {
                    buf.push_str(indent);
                    buf.push('@');
                    buf.push_str(name);
                    if !p.is_empty() {
                        buf.push(' ');
                        buf.push_str(p);
                    }
                    buf.push_str(" {\n");
                    let child_css = Self::serialize_expanded(children, depth + 1);
                    if !child_css.is_empty() {
                        buf.push_str(&child_css);
                        buf.push('\n');
                    }
                    buf.push_str(indent);
                    buf.push('}');
                }
            }
            CssNode::AtRule {
                has_body: false,
                name,
                params,
                ..
            } => {
                let p = params.as_deref().unwrap_or("");
                buf.push_str(indent);
                buf.push('@');
                buf.push_str(name);
                if !p.is_empty() {
                    buf.push(' ');
                    buf.push_str(p);
                }
                buf.push(';');
            }
            CssNode::Raw(text) => {
                buf.push_str(text);
            }
            CssNode::Return(_) => {}
        }
    }

    fn write_node_compressed(buf: &mut String, node: &CssNode) {
        match node {
            CssNode::Declaration {
                property,
                value,
                important,
            } => {
                buf.push_str(property);
                buf.push(':');
                buf.push_str(value);
                if *important {
                    buf.push_str(" !important");
                }
                buf.push(';');
            }
            CssNode::Comment(_) => {}
            CssNode::AtRoot(nodes) => {
                buf.push_str(&Self::serialize_compressed(nodes));
            }
            CssNode::Rule {
                selector,
                declarations,
                children,
            } => {
                let sel = Self::sanitize_selector(selector);
                if sel.is_empty() {
                    return;
                }
                buf.push_str(&sel);
                buf.push('{');
                for decl in declarations {
                    Self::write_node_compressed(buf, decl);
                }
                for kid in children {
                    Self::write_node_compressed(buf, kid);
                }
                buf.push('}');
            }
            CssNode::AtRule {
                has_body: true,
                name,
                params,
                children,
            } => {
                let p = params.as_deref().unwrap_or("");
                buf.push('@');
                buf.push_str(name);
                if !p.is_empty() {
                    buf.push(' ');
                    buf.push_str(p);
                }
                buf.push('{');
                for kid in children {
                    Self::write_node_compressed(buf, kid);
                }
                buf.push('}');
            }
            CssNode::AtRule {
                has_body: false,
                name,
                params,
                ..
            } => {
                let p = params.as_deref().unwrap_or("");
                buf.push('@');
                buf.push_str(name);
                if !p.is_empty() {
                    buf.push(' ');
                    buf.push_str(p);
                }
                buf.push(';');
            }
            CssNode::Raw(text) => {
                buf.push_str(text);
            }
            CssNode::Return(_) => {}
        }
    }

    /// 净化选择器——处理占位符 `%xxx` 在伪类中的移除 + 组合器验证 + 相邻复合选择器规范化。
    fn sanitize_selector(selector: &str) -> String {
        // 先规范化属性选择器（引号去除、修饰符空格）
        let selector = Self::normalize_attr_selectors(selector);
        // 处理相邻复合选择器（[a]b → [a] b）
        let selector = Self::normalize_adjacent_compounds(&selector);
        // 组合器验证——无效组合器返回空字符串
        if Self::has_bogus_combinators(&selector) {
            return String::new();
        }
        if !selector.contains('%') {
            return selector;
        }
        // 顶层逗号分隔——移除纯占位符部分
        let parts: Vec<&str> = selector
            .split(',')
            .filter(|s| !s.trim().starts_with('%'))
            .collect();
        let mut result = parts.join(",").trim().to_string();
        // 处理伪类内的占位符
        for pseudo in &["is", "not", "where", "matches"] {
            let pattern = format!(":{pseudo}(%");
            while let Some(pos) = result.find(&pattern) {
                let paren_start = pos + pattern.len() - 2; // pattern=":is(%" → -2 得到 ( 的位置
                let mut depth = 1;
                let chars: Vec<char> = result.chars().collect();
                let mut i = paren_start + 1;
                while i < chars.len() && depth > 0 {
                    if chars[i] == '(' {
                        depth += 1;
                    } else if chars[i] == ')' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        i += 1;
                    }
                }
                let end = i;
                let inner = &result[paren_start + 1..end];
                let args: Vec<&str> = inner.split(',').filter(|s| !s.trim().is_empty()).collect();
                let real_args: Vec<&str> = args
                    .iter()
                    .filter(|s| !s.trim().starts_with('%'))
                    .cloned()
                    .collect();
                if real_args.is_empty() {
                    if *pseudo == "not" {
                        // :not(%placeholder) → 移除 :not()
                        // 如果前面有选择器（如 a:not(%b)），直接移除 :not → a
                        // 如果 :not 是整个选择器（如 :not(%b)），替换为 *（匹配所有元素）
                        let before = &result[..pos];
                        let after = &result[end + 1..];
                        if before.trim().is_empty() {
                            result = format!("*{after}");
                        } else {
                            result = format!("{before}{after}");
                        }
                    } else {
                        return String::new();
                    }
                } else {
                    let new_inner = real_args
                        .iter()
                        .map(|s| s.trim())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let before = &result[..paren_start];
                    let after = &result[end + 1..];
                    result = format!("{before}({new_inner}){after}");
                }
            }
        }
        result
    }

    /// 检查选择器是否包含无效组合器（bogus combinators）。
    ///
    /// 规则：
    /// - 顶层/:has() 内：允许单个前导组合器（`> a`），但禁止多个前导组合器、连续组合器、尾部组合器
    /// - :is/:where/:not/matches 内：禁止任何前导组合器（只能有完整选择器）
    /// - 所有上下文：禁止连续组合器和尾部组合器
    fn has_bogus_combinators(selector: &str) -> bool {
        Self::check_bogus_in_selector(selector, true)
    }

    /// 递归检查选择器中的无效组合器。
    /// `allow_leading_combinator` 控制是否允许单个前导组合器。
    fn check_bogus_in_selector(selector: &str, allow_leading_combinator: bool) -> bool {
        // 对逗号分隔的每个选择器部分单独检查
        for part in selector.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            // 将选择器分解为 token 序列，同时提取伪类内部内容
            let tokens = Self::tokenize_selector_with_pseudo(part);
            if Self::tokens_have_bogus(&tokens, allow_leading_combinator) {
                return true;
            }
        }
        false
    }

    /// 将选择器字符串分解为 token 向量，识别伪类内部内容。
    fn tokenize_selector_with_pseudo(selector: &str) -> Vec<SelToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = selector.chars().collect();
        let mut i = 0;
        let mut current = String::new();
        let mut in_brackets = false;

        while i < chars.len() {
            let c = chars[i];
            if c == '[' {
                in_brackets = true;
                current.push(c);
            } else if c == ']' {
                in_brackets = false;
                current.push(c);
            } else if c == '(' && !in_brackets {
                // 找到伪类内部——提取完整内容
                if !current.trim().is_empty() {
                    tokens.push(SelToken::Selector(current.trim().to_string()));
                    current = String::new();
                }
                // 向前查看伪类名（向前回看）
                let pseudo_name = Self::find_pseudo_name(&tokens);
                // 提取括号内容
                let mut depth = 1;
                let mut inner = String::new();
                i += 1;
                while i < chars.len() && depth > 0 {
                    if chars[i] == '(' {
                        depth += 1;
                    } else if chars[i] == ')' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        inner.push(chars[i]);
                    }
                    i += 1;
                }
                // :is/:where/:not/matches 不允许前导组合器
                // :has 允许前导组合器（同顶层）
                let allow_leading = pseudo_name.as_deref() == Some("has");
                tokens.push(SelToken::PseudoInner(
                    pseudo_name.unwrap_or_default(),
                    inner,
                    allow_leading,
                ));
                continue; // 已经推进了 i
            } else if in_brackets {
                current.push(c);
            } else if c == '>' || c == '+' || c == '~' {
                if !current.trim().is_empty() {
                    tokens.push(SelToken::Selector(current.trim().to_string()));
                    current = String::new();
                }
                tokens.push(SelToken::Combinator(c));
            } else if c.is_whitespace() {
                if !current.trim().is_empty() {
                    tokens.push(SelToken::Selector(current.trim().to_string()));
                    current = String::new();
                }
            } else {
                current.push(c);
            }
            i += 1;
        }
        if !current.trim().is_empty() {
            tokens.push(SelToken::Selector(current.trim().to_string()));
        }
        tokens
    }

    /// 从 token 序列中找出最近伪类的名称。
    fn find_pseudo_name(tokens: &[SelToken]) -> Option<String> {
        // 回看 token 序列，找到 :pseudoName 模式
        for window in tokens.windows(2) {
            if let SelToken::Selector(name) = &window[0] {
                if let SelToken::Selector(colon) = &window[1] {
                    if colon == ":" {
                        return Some(name.clone());
                    }
                }
            }
        }
        // 检查最后一个 token 是否是 `:name` 形式
        if let Some(SelToken::Selector(last)) = tokens.last() {
            if last.starts_with(':') {
                return Some(last[1..].to_string());
            }
        }
        None
    }

    /// 检查 token 序列是否包含无效组合器。
    fn tokens_have_bogus(tokens: &[SelToken], allow_leading_combinator: bool) -> bool {
        if tokens.is_empty() {
            return false;
        }
        // 检查尾部组合器
        if matches!(tokens.last(), Some(SelToken::Combinator(_))) {
            return true;
        }
        // 检查前导组合器
        if let Some(SelToken::Combinator(_)) = tokens.first() {
            if !allow_leading_combinator {
                return true;
            }
            // 单个前导组合器允许，但第二个不能是组合器
            if tokens.len() >= 2 {
                if let SelToken::Combinator(_) = tokens[1] {
                    return true; // 连续组合器
                }
            }
        }
        // 检查中间连续组合器
        for window in tokens.windows(2) {
            if let (SelToken::Combinator(_), SelToken::Combinator(_)) = (&window[0], &window[1]) {
                return true;
            }
        }
        // 递归检查伪类内部
        for token in tokens {
            if let SelToken::PseudoInner(name, inner, allow_leading) = token {
                // 处理逗号分隔的多个选择器
                for part in inner.split(',') {
                    let part = part.trim();
                    if part.is_empty() {
                        continue;
                    }
                    let inner_tokens = Self::tokenize_selector_with_pseudo(part);
                    if Self::tokens_have_bogus(&inner_tokens, *allow_leading) {
                        return true;
                    }
                }
                // 多个逗号分隔的伪类选择器之间也需要有完整的选择器
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                if parts.len() > 1 {
                    for part in &parts {
                        let inner_tokens = Self::tokenize_selector_with_pseudo(part);
                        if Self::tokens_have_bogus(&inner_tokens, *allow_leading) {
                            return true;
                        }
                    }
                }
                let _ = name; // 仅 :has 允许前导组合器
            }
        }
        false
    }

    /// 规范化相邻复合选择器——在属性选择器后跟类型选择器时添加空格。
    /// 例如：`[a]b` → `[a] b`（属性选择器后紧跟类型选择器/通配符需加空格）
    /// 但 `[a].b` `[a]#b` `[a]:hover` 不需要加空格（单复合选择器内）
    fn normalize_adjacent_compounds(selector: &str) -> String {
        let chars: Vec<char> = selector.chars().collect();
        let mut result = String::new();
        let mut i = 0;
        while i < chars.len() {
            result.push(chars[i]);
            // 检查是否需要插入空格
            if i + 1 < chars.len() {
                let curr = chars[i];
                let next = chars[i + 1];
                if curr == ']' && !next.is_whitespace() {
                    // ] 后紧跟类型选择器（字母）或通配符 * 时加空格
                    // 但 . # : [ 后是同一复合选择器的延续，不加空格
                    if next == '*' || next.is_ascii_alphabetic() {
                        result.push(' ');
                    }
                }
            }
            i += 1;
        }
        result
    }

    /// 规范化属性选择器——去除合法标识符的引号，在修饰符前加空格。
    /// 例如：`[a="b"i]` → `[a=b i]`
    fn normalize_attr_selectors(selector: &str) -> String {
        let chars: Vec<char> = selector.chars().collect();
        let mut result = String::new();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '[' {
                // 找到匹配的 ]
                let start = i;
                let mut depth = 1;
                i += 1;
                while i < chars.len() && depth > 0 {
                    if chars[i] == '[' {
                        depth += 1;
                    } else if chars[i] == ']' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        i += 1;
                    }
                }
                if i < chars.len() {
                    // 提取属性选择器内容
                    let inner: String = chars[start + 1..i].iter().collect();
                    let normalized = Self::normalize_attr_content(&inner);
                    result.push('[');
                    result.push_str(&normalized);
                    result.push(']');
                    i += 1; // 跳过 ]
                } else {
                    // 未闭合的 [——直接复制剩余
                    result.extend(&chars[start..]);
                    break;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    /// 规范化属性选择器内容。
    /// `a="b"i` → `a=b i`（当 b 是合法标识符时去引号，修饰符前加空格）
    fn normalize_attr_content(inner: &str) -> String {
        let chars: Vec<char> = inner.chars().collect();
        let mut result = String::new();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '"' || chars[i] == '\'' {
                let quote = chars[i];
                // 找到结束引号
                let val_start = i + 1;
                let mut j = val_start;
                while j < chars.len() && chars[j] != quote {
                    j += 1;
                }
                if j < chars.len() {
                    // 提取引号内的值
                    let val: String = chars[val_start..j].iter().collect();
                    // 检查是否是合法 CSS 标识符（但 --foo 需保留引号）
                    let is_ident = !val.is_empty()
                        && !val.starts_with("--")
                        && val.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '-')
                        && val.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
                    if is_ident {
                        // 去除引号
                        result.push_str(&val);
                    } else {
                        // 保留引号
                        result.push(quote);
                        result.push_str(&val);
                        result.push(quote);
                    }
                    // 检查后面是否有修饰符（紧跟的字母）
                    let after = j + 1;
                    if after < chars.len() && chars[after].is_ascii_alphabetic() {
                        // 在修饰符前加空格
                        result.push(' ');
                    }
                    i = j + 1;
                } else {
                    // 未闭合的引号——直接复制
                    result.push(chars[i]);
                    i += 1;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }
}
