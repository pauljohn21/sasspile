//! CssNode 定义——求值阶段的中间表示。

/// CSS 节点——求值器产出的中间表示。
#[derive(Debug, Clone, PartialEq)]
pub enum CssNode {
    /// 样式规则——`selector { ... }`。
    Rule {
        /// 选择器文本。
        selector: String,
        /// 声明列表。
        declarations: Vec<CssNode>,
        /// 子规则和 @规则。
        children: Vec<CssNode>,
    },

    /// 属性声明——`property: value;`。
    Declaration {
        /// 属性名。
        property: String,
        /// 属性值。
        value: String,
        /// 是否 important。
        important: bool,
    },

    /// @规则——`@media`, `@keyframes` 等。
    AtRule {
        /// 规则名。
        name: String,
        /// 参数。
        params: Option<String>,
        /// 子节点。
        children: Vec<CssNode>,
    },

    /// 注释。
    Comment(String),
}

impl std::fmt::Display for CssNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CssNode::Declaration {
                property,
                value,
                important,
            } => {
                if *important {
                    write!(f, "{property}: {value} !important;")
                } else {
                    write!(f, "{property}: {value};")
                }
            }
            CssNode::Comment(text) => write!(f, "/* {text} */"),
            CssNode::Rule { selector, .. } => write!(f, "{selector} {{ ... }}"),
            CssNode::AtRule { name, .. } => write!(f, "@{name} {{ ... }}"),
        }
    }
}
