//! CssNode 定义。

use crate::eval::value::Value;

/// CSS 输出节点。
#[derive(Debug, Clone)]
pub enum CssNode {
    Rule {
        selector: String,
        declarations: Vec<CssNode>,
        children: Vec<CssNode>,
    },
    Declaration {
        property: String,
        value: String,
        important: bool,
    },
    Comment(String),
    AtRule {
        name: String,
        params: String,
        children: Vec<CssNode>,
        has_body: bool,
    },
    AtRoot(Vec<CssNode>),
    Return(Value),
}
