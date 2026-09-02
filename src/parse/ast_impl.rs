use super::ast::*;

/// AST 节点序列化——用于最小化工具将 AST 转回 SCSS 源码。
impl Node {
    /// 将 AST 节点序列化回 SCSS 源码——用于最小化工具。
    pub fn to_scss(&self, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        match self {
            Node::Rule { selector, body } => {
                let body: String = body
                    .iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                if body.is_empty() {
                    format!("{pad}{selector} {{}}")
                } else {
                    format!("{pad}{selector} {{\n{body}\n{pad}}}")
                }
            }
            Node::Decl {
                property,
                value,
                important,
            } => {
                let imp = if *important { " !important" } else { "" };
                format!("{pad}{property}: {value}{imp};")
            }
            Node::Variable { name, value, flags } => {
                let mut s = format!("{pad}${name}: {value}");
                if flags.default {
                    s.push_str(" !default");
                }
                if flags.global {
                    s.push_str(" !global");
                }
                s.push(';');
                s
            }
            Node::Comment(text, silent) => {
                if *silent {
                    format!("{pad}// {text}")
                } else {
                    format!("{pad}/* {text} */")
                }
            }
            // —— 控制流 ——
            Node::If {
                branches,
                else_body,
            } => {
                let mut s = branches.iter().enumerate().fold(
                    String::new(),
                    |mut acc, (i, (cond, body))| {
                        let kw = if i == 0 { "@if" } else { "@else if" };
                        let body_s: String = body
                            .iter()
                            .map(|n| n.to_scss(indent + 1))
                            .collect::<Vec<_>>()
                            .join("\n");
                        acc.push_str(&format!("{pad}{kw} {cond} {{\n{body_s}\n{pad}}}"));
                        if i < branches.len() - 1 || else_body.is_some() {
                            acc.push('\n');
                        }
                        acc
                    },
                );
                if let Some(eb) = else_body {
                    let body_s: String = eb
                        .iter()
                        .map(|n| n.to_scss(indent + 1))
                        .collect::<Vec<_>>()
                        .join("\n");
                    s.push_str(&format!("{pad}@else {{\n{body_s}\n{pad}}}"));
                }
                s
            }
            Node::For {
                var,
                from,
                to,
                inclusive,
                body,
            } => {
                let kw = if *inclusive { "through" } else { "to" };
                let body_s: String = body
                    .iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@for ${var} from {from} {kw} {to} {{\n{body_s}\n{pad}}}")
            }
            Node::Each { vars, list, body } => {
                let vars_s = vars
                    .iter()
                    .map(|v| format!("${v}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let body_s: String = body
                    .iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@each {vars_s} in {list} {{\n{body_s}\n{pad}}}")
            }
            Node::While { cond, body } => {
                let body_s: String = body
                    .iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@while {cond} {{\n{body_s}\n{pad}}}")
            }
            // —— Mixin / Function ——
            Node::MixinDef { name, params, body } => {
                let params_s = params
                    .iter()
                    .map(|p| {
                        let s = format!("${}", p.name);
                        if p.rest {
                            format!("{s}...")
                        } else if let Some(d) = &p.default {
                            format!("{s}: {d}")
                        } else {
                            s
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let body_s: String = body
                    .iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@mixin {name}({params_s}) {{\n{body_s}\n{pad}}}")
            }
            Node::Include {
                name,
                args,
                content,
            } => {
                let args_s = args
                    .iter()
                    .map(|a| {
                        let prefix = match &a.name {
                            Some(n) => format!("${n}: "),
                            None => String::new(),
                        };
                        let suffix = if a.spread { "..." } else { "" };
                        format!("{prefix}{}{suffix}", a.value)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let base = if args_s.is_empty() {
                    format!("{pad}@include {name};")
                } else {
                    format!("{pad}@include {name}({args_s});")
                };
                if let Some(content) = content {
                    let content_s: String = content
                        .iter()
                        .map(|n| n.to_scss(indent + 1))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{base}\n{pad}{{\n{content_s}\n{pad}}}")
                } else {
                    base
                }
            }
            Node::Content => format!("{pad}@content;"),
            Node::FunctionDef { name, params, body } => {
                let params_s = params
                    .iter()
                    .map(|p| {
                        let s = format!("${}", p.name);
                        if p.rest {
                            format!("{s}...")
                        } else if let Some(d) = &p.default {
                            format!("{s}: {d}")
                        } else {
                            s
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let body_s: String = body
                    .iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{pad}@function {name}({params_s}) {{\n{body_s}\n{pad}}}")
            }
            Node::Return(v) => format!("{pad}@return {v};"),
            // —— 模块系统 ——
            Node::Use {
                url,
                namespace,
                star,
                config,
            } => {
                let mut s = format!("{pad}@use \"{url}\"");
                if *star {
                    s.push_str(" as *");
                } else if let Some(ns) = namespace {
                    s.push_str(&format!(" as {ns}"));
                }
                if !config.is_empty() {
                    let cfg: String = config
                        .iter()
                        .map(|c| format!("${}: {}", c.name, c.value))
                        .collect::<Vec<_>>()
                        .join(", ");
                    s.push_str(&format!(" with ({cfg})"));
                }
                s.push(';');
                s
            }
            Node::Forward {
                url,
                show,
                hide,
                prefix,
                config: _,
            } => {
                let mut s = format!("{pad}@forward \"{url}\"");
                if let Some(p) = prefix {
                    s.push_str(&format!(" as {p}-*"));
                }
                if !show.is_empty() {
                    s.push_str(&format!(" show {}", show.join(", ")));
                }
                if !hide.is_empty() {
                    s.push_str(&format!(" hide {}", hide.join(", ")));
                }
                s.push(';');
                s
            }
            Node::Import { url, modifier } => {
                if modifier.is_empty() {
                    format!("{pad}@import \"{url}\";")
                } else {
                    format!("{pad}@import \"{url}\" {modifier};")
                }
            }
            // —— 其他指令 ——
            Node::Extend { selector, optional } => {
                let opt = if *optional { " !optional" } else { "" };
                format!("{pad}@extend {selector}{opt};")
            }
            Node::AtRoot { query, body } => {
                let body_s: String = body
                    .iter()
                    .map(|n| n.to_scss(indent + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Some(q) = query {
                    format!("{pad}@at-root {q} {{\n{body_s}\n{pad}}}")
                } else {
                    format!("{pad}@at-root {{\n{body_s}\n{pad}}}")
                }
            }
            Node::AtRule { name, params, body } => {
                let params_s = params.as_deref().unwrap_or("");
                match body {
                    Some(nodes) => {
                        let body_s: String = nodes
                            .iter()
                            .map(|n| n.to_scss(indent + 1))
                            .collect::<Vec<_>>()
                            .join("\n");
                        if params_s.is_empty() {
                            format!("{pad}@{name} {{\n{body_s}\n{pad}}}")
                        } else {
                            format!("{pad}@{name} {params_s} {{\n{body_s}\n{pad}}}")
                        }
                    }
                    None => {
                        if params_s.is_empty() {
                            format!("{pad}@{name};")
                        } else {
                            format!("{pad}@{name} {params_s};")
                        }
                    }
                }
            }
            Node::Warn(v) => format!("{pad}@warn {v};"),
            Node::Debug(v) => format!("{pad}@debug {v};"),
            Node::Error(v) => format!("{pad}@error {v};"),
        }
    }
}
