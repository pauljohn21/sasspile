//! Plain CSS 模式限制检查。
//!
//! `.css` 文件加载时 `env.plain_css = true`，
//! 此时禁止 Sass 特有的表达式和语句。

use crate::error::{Result, SassError};
use crate::eval::error_msgs::{
    err_plain_css_at_rule, err_plain_css_sass_var, err_plain_css_silent_comment,
};
use crate::parse::ast::{BinOpKind, Node, Value};

impl super::Evaluator {
    /// Plain CSS 模式下检查值表达式是否合法。
    ///
    /// 递归检查子表达式，在第一个违规处报错。
    pub(crate) fn check_plain_css_value(value: &Value) -> Result<()> {
        match value {
            // 数值、颜色、布尔、null、Calc、MixinRef — 允许
            Value::Number(..)
            | Value::Color(..)
            | Value::Bool(..)
            | Value::Null
            | Value::Calc(..)
            | Value::MixinRef(..) => Ok(()),

            // 变量引用 — 禁止
            Value::Variable(_) => Err(SassError::Eval(
                "Sass variables aren't allowed in plain CSS.".into(),
            )),

            // 插值 — 禁止
            Value::Interp(_) => Err(SassError::Eval(
                "Interpolation isn't allowed in plain CSS.".into(),
            )),

            // 二元运算 — 禁止（但 and/or 在 if 条件中已由 partial_eval_condition 处理）
            Value::BinOp(b) => {
                Self::check_plain_css_value(&b.left)?;
                Self::check_plain_css_value(&b.right)?;
                match b.op {
                    BinOpKind::And | BinOpKind::Or => Ok(()),
                    _ => Err(SassError::Eval(
                        "Operators aren't allowed in plain CSS.".into(),
                    )),
                }
            }

            // 一元运算 — 检查操作数
            Value::UnaryOp(_, v) => Self::check_plain_css_value(v),

            // 括号 — 禁止（但 calc() 内部的括号由 Calc 字符串处理）
            Value::Paren(_) => Err(SassError::Eval(
                "Parentheses aren't allowed in plain CSS.".into(),
            )),

            // 字符串 — 检查是否包含插值，以及是否为 & 父选择器
            Value::String(s, _) => {
                if s.contains("#{") {
                    Err(SassError::Eval(
                        "Interpolation isn't allowed in plain CSS.".into(),
                    ))
                } else if s == "&" {
                    Err(SassError::Eval(
                        "The parent selector isn't allowed in plain CSS.".into(),
                    ))
                } else {
                    Ok(())
                }
            }

            // 列表 — 检查每个元素
            Value::List(elements, _, _) => {
                elements
                    .iter()
                    .try_for_each(|e| Self::check_plain_css_value(e))?;
                Ok(())
            }

            // Map — 禁止
            Value::Map(_) => Err(SassError::Eval("expected \")\".".into())),

            // 函数调用 — 检查是否为允许的 CSS 原生函数
            Value::Call(name, args) => Self::check_plain_css_call(name, args),

            // 剩余参数展开 — 检查内部值
            Value::Spread(v) => Self::check_plain_css_value(v),
        }
    }

    /// 检查函数调用在 plain CSS 模式下是否合法。
    ///
    /// CSS 原生函数（rgb, hsl, var, calc 等）允许透传，
    /// Sass 内建函数（index, length, map-get 等）禁止。
    fn check_plain_css_call(name: &str, args: &[crate::parse::ast::Arg]) -> Result<()> {
        let lower = name.to_lowercase();
        // if/css 有自己的 plain CSS 逻辑，不拦截
        // sass() 在 plain CSS 中禁止
        if lower == "if" || lower == "css" {
            return Ok(());
        }
        if lower == "sass" {
            return Err(SassError::Eval(
                "sass() conditions aren't allowed in plain CSS".into(),
            ));
        }
        // 检查参数中的命名参数（关键字参数含 $ 变量名）— 禁止
        if args.iter().any(|arg| arg.name.is_some()) {
            return Err(SassError::Eval(
                "Sass variables aren't allowed in plain CSS.".into(),
            ));
        }
        // 检查 spread 参数（args...）— 禁止
        if args.iter().any(|arg| arg.spread) {
            return Err(SassError::Eval("expected \")\".".into()));
        }
        // CSS 原生函数 — 允许，但检查参数中是否有违规
        if Self::is_css_function(&lower) {
            args.iter()
                .try_for_each(|arg| Self::check_plain_css_value(&arg.value))?;
            return Ok(());
        }
        // 已知 Sass 内建函数（非 CSS 原生）— 禁止
        if Self::is_known_builtin(&lower) {
            return Err(SassError::Eval(
                "This function isn't allowed in plain CSS.".into(),
            ));
        }
        // 未知函数 — 可能是用户自定义函数，允许通过
        // eval_value 的正常流程会处理（找到则调用，找不到则 CSS 透传）
        args.iter()
            .try_for_each(|arg| Self::check_plain_css_value(&arg.value))?;
        Ok(())
    }

    /// Plain CSS 模式下检查节点是否合法。
    ///
    /// 返回 `Ok(())` 表示允许，`Err(_)` 表示禁止。
    pub(crate) fn check_plain_css_node(node: &Node) -> Result<()> {
        match node {
            // 允许的节点
            Node::Rule { .. } | Node::Decl { .. } | Node::Comment(_, false) => Ok(()),
            Node::AtRule { name, .. } => {
                use crate::parse::at_rule_kinds::AtRuleKind;
                // Sass 内建 at-rule（@if/@for/@each/@while/@mixin/@include/@function 等）— 禁止
                if AtRuleKind::from_str(name).is_known() {
                    Err(err_plain_css_at_rule())
                } else {
                    // CSS 标准 at-rule 和未知 at-rule — 允许
                    Ok(())
                }
            }
            // 静默注释 — 禁止
            Node::Comment(_, true) => Err(err_plain_css_silent_comment()),
            // Sass 特有的 at-rules — 全部禁止
            Node::Variable { .. } => Err(err_plain_css_sass_var()),
            Node::If { .. } | Node::For { .. } | Node::Each { .. } | Node::While { .. } => {
                Err(err_plain_css_at_rule())
            }
            Node::MixinDef { .. } | Node::Include { .. } | Node::Content => {
                Err(err_plain_css_at_rule())
            }
            Node::FunctionDef { .. } | Node::Return(_) => Err(err_plain_css_at_rule()),
            Node::Extend { .. } => Err(err_plain_css_at_rule()),
            Node::AtRoot { .. } => Err(err_plain_css_at_rule()),
            Node::Warn(_) | Node::Debug(_) | Node::Error(_) => Err(err_plain_css_at_rule()),
            // Use/Forward/Import — 由模块系统处理，不拦截
            Node::Use { .. } | Node::Forward { .. } | Node::Import { .. } => Ok(()),
        }
    }

    /// 检查选择器是否在 plain CSS 模式下合法。
    ///
    /// 禁止：插值 `#{}`、占位符 `%foo`、父选择器后缀 `&b`
    pub(crate) fn check_plain_css_selector(selector: &str) -> Result<()> {
        // 插值 — 禁止
        if selector.contains("#{") {
            return Err(SassError::Eval(
                "Interpolation isn't allowed in plain CSS.".into(),
            ));
        }
        // 占位符选择器 — 禁止
        if selector.contains('%') {
            // 简单检查：% 后跟标识符字符
            let has_placeholder = selector.chars().enumerate().any(|(i, c)| {
                c == '%'
                    && i + 1 < selector.len()
                    && selector[i + 1..]
                        .chars()
                        .next()
                        .is_some_and(|nc| nc.is_alphanumeric() || nc == '-')
            });
            if has_placeholder {
                return Err(SassError::Eval(
                    "Placeholder selectors aren't allowed in plain CSS.".into(),
                ));
            }
        }
        // 父选择器后缀 — 禁止 & 直接跟标识符
        // 如 `&b` 但不是 `& > b` 或 `&.b`
        let chars: Vec<char> = selector.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if c == '&' && i + 1 < chars.len() {
                let next = chars[i + 1];
                // & 后直接跟字母/数字/连字符 = 后缀
                if next.is_alphanumeric() || next == '-' {
                    return Err(SassError::Eval(
                        "Parent selectors can't have suffixes in plain CSS.".into(),
                    ));
                }
            }
        }
        // 顶层前导组合器 — 禁止
        let trimmed = selector.trim_start();
        if trimmed.starts_with('>') || trimmed.starts_with('+') || trimmed.starts_with('~') {
            return Err(SassError::Eval(
                "Top-level leading combinators aren't allowed in plain CSS.".into(),
            ));
        }
        Ok(())
    }
}
