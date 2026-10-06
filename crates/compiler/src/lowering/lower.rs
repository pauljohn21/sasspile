//! lower_to_ast — 降级转换的核心函数

use super::context::LoweringContext;
use crate::parser::SassAstNode;
use crate::reactive::{AstNode, CssStmt, Value};
use crate::Error;

/// 将单个 SassAstNode 递归转换为 AstNode
pub fn lower_to_ast(
    node: SassAstNode,
    ctx: &mut LoweringContext,
) -> Result<AstNode, Error> {
    match node {
        // ── 简单样式声明：直接转换 ──
        SassAstNode::StyleDecl { prop, value } => {
            Ok(AstNode::StyleDecl { property: prop, value })
        }

        // ── 变量声明：处理 !default 语义 ──
        SassAstNode::VariableDecl { name, value, has_default } => {
            let value_str = node_to_string(&value, ctx)?;
            let value_node = make_value_node(&value_str);

            // !default 语义：变量已存在时丢弃
            if has_default && ctx.has_variable(&name) {
                // 返回一个 Placeholder，表示该声明被丢弃
                return Ok(AstNode::Placeholder);
            }

            // 记录变量到环境
            ctx.set_variable(&name, &value_str);

            Ok(AstNode::VariableDecl {
                name,
                value: value_node,
            })
        }

        // ── 嵌套规则：处理父选择器展开 ──
        SassAstNode::Rule { selector, inner } => {
            let expanded_selector = expand_parent_selector(&selector, ctx);

            // 压入当前选择器到祖先栈
            ctx.push_selector(expanded_selector.clone());

            // 递归降级子节点
            let lowered_inner = lower_inner_nodes(inner, ctx)?;

            // 弹出当前选择器
            ctx.pop_selector();

            Ok(AstNode::RuleSet {
                selector: expanded_selector,
                inner: lowered_inner,
            })
        }

        // ── 插值表达式：展开为具体字符串 ──
        SassAstNode::Interpolated(expr) => {
            let resolved = resolve_interpolation(&expr, ctx)?;
            Ok(AstNode::Css(CssStmt::Decl {
                property: String::new(),
                value: resolved,
            }))
        }

        // ── Map 字面量 → Value::Map ──
        SassAstNode::MapLiteral(entries) => {
            let mut map_entries = Vec::with_capacity(entries.len());
            for (key_node, value_node) in entries {
                let key_str = node_to_string(&key_node, ctx)?;
                let value_str = node_to_string(&value_node, ctx)?;
                map_entries.push((key_str, make_value_node(&value_str)));
            }
            // Map 作为特殊的 AstNode 变量值返回
            // 这里用 CssStmt::Decl 包装表示一个值占位
            Ok(AstNode::Css(CssStmt::Decl {
                property: String::new(),
                value: format_map_literal(&map_entries),
            }))
        }

        // ── List 字面量 → Value::List ──
        SassAstNode::ListLiteral(items) => {
            let mut list_values = Vec::with_capacity(items.len());
            for item in items {
                let item_str = node_to_string(&item, ctx)?;
                list_values.push(make_value_node(&item_str));
            }
            let _ = list_values; // 暂不使用，未来接入 Value
            Ok(AstNode::Placeholder)
        }

        // ── 注释：透传为 Placeholder ──
        SassAstNode::Comment(_) => Ok(AstNode::Placeholder),

        // ── Raw 文本：作为 CssStmt::Decl 透传 ──
        SassAstNode::Raw(text) => {
            if text.trim().is_empty() {
                Ok(AstNode::Placeholder)
            } else {
                Ok(AstNode::Css(CssStmt::Decl {
                    property: String::new(),
                    value: text,
                }))
            }
        }

        // ── 父选择器：单独遇到时展开 ──
        SassAstNode::ParentSelector => {
            let parent = ctx.current_selector().unwrap_or_default();
            Ok(AstNode::Css(CssStmt::Decl {
                property: String::new(),
                value: parent.to_string(),
            }))
        }
    }
}

/// 递归降级内部节点列表
fn lower_inner_nodes(
    nodes: Vec<SassAstNode>,
    ctx: &mut LoweringContext,
) -> Result<Vec<AstNode>, Error> {
    let mut result = Vec::new();
    for node in nodes {
        let lowered = lower_to_ast(node, ctx)?;
        // 过滤掉 Placeholder（被丢弃的节点）
        if !matches!(lowered, AstNode::Placeholder) {
            result.push(lowered);
        }
    }
    Ok(result)
}

/// 展开父选择器：将 `&` 替换为完整祖先路径
fn expand_parent_selector(selector: &str, ctx: &LoweringContext) -> String {
    if selector.contains('&') {
        let parent = ctx.ancestor_path();
        if parent.is_empty() {
            selector.replace('&', "")
        } else {
            selector.replace('&', &parent)
        }
    } else if !ctx.selector_stack.is_empty() && !selector.starts_with('@') {
        // 非父选择器且非指令：拼接祖先路径
        let parent = ctx.ancestor_path();
        if parent.is_empty() {
            selector.to_string()
        } else {
            format!("{parent} {selector}")
        }
    } else {
        selector.to_string()
    }
}

/// 解析插值表达式
fn resolve_interpolation(expr: &str, ctx: &LoweringContext) -> Result<String, Error> {
    // 处理简单变量引用：$var（保持 $ 前缀查找）
    if expr.starts_with('$') {
        match ctx.get_variable(expr) {
            Some(value) => Ok(value.to_string()),
            None => Err(Error::lowering(format!(
                "undefined variable '{expr}' in interpolation"
            ))),
        }
    } else if expr.contains("#{") {
        // 处理复合插值：#{$var}suffix
        expand_complex_interpolation(expr, ctx)
    } else {
        // 无插值的纯文本
        Ok(expr.to_string())
    }
}

/// 展开复合插值表达式
fn expand_complex_interpolation(expr: &str, ctx: &LoweringContext) -> Result<String, Error> {
    let mut result = String::new();
    let mut chars = expr.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '#' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            // 提取变量名直到 '}'
            let mut var_name = String::new();
            while let Some(c) = chars.next() {
                if c == '}' {
                    break;
                }
                var_name.push(c);
            }
            // 解析变量（保持 $ 前缀查找）
            let value = if var_name.starts_with('$') {
                match ctx.get_variable(&var_name) {
                    Some(v) => v.to_string(),
                    None => {
                        return Err(Error::lowering(format!(
                            "undefined variable '{var_name}' in interpolation"
                        )))
                    }
                }
            } else {
                var_name
            };
            result.push_str(&value);
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

/// 将 SassAstNode 转换为字符串表示
fn node_to_string(node: &SassAstNode, ctx: &LoweringContext) -> Result<String, Error> {
    match node {
        SassAstNode::Raw(text) => Ok(text.clone()),
        SassAstNode::Interpolated(expr) => resolve_interpolation(expr, ctx),
        SassAstNode::VariableDecl { name, value, .. } => node_to_string(value, ctx).map(|v| format!("{name}: {v}")),
        SassAstNode::StyleDecl { value, .. } => Ok(value.clone()),
        _ => Ok(String::new()),
    }
}

/// 从字符串创建值节点
fn make_value_node(text: &str) -> Value {
    let trimmed = text.trim();
    // 尝试解析为数字
    if let Ok(num) = trimmed.parse::<f64>() {
        return Value::Number(num);
    }
    // 否则作为字符串
    Value::String(trimmed.to_string())
}

/// 格式化 Map 字面量为字符串
fn format_map_literal(entries: &[(String, Value)]) -> String {
    let parts: Vec<String> = entries
        .iter()
        .map(|(k, v)| format!("{k}: {v:?}"))
        .collect();
    format!("({})", parts.join(", "))
}
