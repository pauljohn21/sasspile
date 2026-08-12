//! CSS 序列化器——CssNode 树 → CSS 字符串。

pub mod node;

pub use node::CssNode;

use crate::OutputStyle;

/// 序列化器。
pub struct Serializer;

impl Serializer {
    /// 序列化 CssNode 列表为 CSS 字符串。
    pub fn serialize(nodes: &[CssNode], style: OutputStyle) -> String {
        let flattened = Self::flatten_nodes(nodes);
        let css = match style {
            OutputStyle::Expanded => Self::serialize_expanded(&flattened, 0),
            OutputStyle::Compressed => Self::serialize_compressed(&flattened),
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

    /// 净化选择器——处理占位符 `%xxx` 在伪类中的移除。
    fn sanitize_selector(selector: &str) -> String {
        // 先规范化属性选择器（引号去除、修饰符空格）
        let selector = Self::normalize_attr_selectors(selector);
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
                        // :not(%placeholder) → *（因为占位符不存在，:not 匹配所有元素）
                        let before = &result[..pos];
                        let after = &result[end + 1..];
                        result = format!("{before}*{after}");
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
