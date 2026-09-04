//! 选择器 AST——将 CSS 选择器字符串解析为类型化的代数结构。
//!
//! 类型层级：
//! ```text
//! Selector              — 逗号分隔列表
//!   ComplexSelector     — 组合器分隔序列
//!     CompoundSelector  — 无空格简单选择器序列
//!       SimpleSelector  — 最小单元（Type/Class/Id/...）
//! ```

use std::fmt;

// ─── AST 类型定义 ─────────────────────────────────────────────────

/// 顶层选择器——逗号分隔的复杂选择器列表。
#[derive(Debug, Clone, PartialEq)]
pub struct Selector(pub Vec<ComplexSelector>);

/// 复杂选择器——组合器分隔的复合选择器序列。
/// 第一个元素的 combinator 始终为 None（除非有前导组合器如 `> a`）。
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
    /// (可选组合器, 复合选择器) 对序列。
    pub compounds: Vec<(Option<Combinator>, CompoundSelector)>,
}

/// 复合选择器——无空格的简单选择器序列。
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundSelector(pub Vec<SimpleSelector>);

/// 简单选择器——最小不可分割的选择器单元。
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleSelector {
    /// `*`
    Universal,
    /// `div`、`a`、`span`
    Type(String),
    /// `.btn`
    Class(String),
    /// `#main`
    Id(String),
    /// `[type="text"]`
    Attribute {
        name: String,
        op: Option<String>,
        value: Option<String>,
        modifier: Option<String>,
    },
    /// `:hover`、`:nth-child(2n+1)`
    PseudoClass {
        name: String,
        arg: Option<String>,
    },
    /// `::before`
    PseudoElement {
        name: String,
        arg: Option<String>,
    },
    /// `%button`（占位符）
    Placeholder(String),
}

/// 组合器。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// 空格（后代）
    Descendant,
    /// `>`（直接子元素）
    Child,
    /// `+`（紧邻兄弟）
    Adjacent,
    /// `~`（一般兄弟）
    Sibling,
}

// ─── Display 实现 ────────────────────────────────────────────────

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.0.iter().map(|c| c.to_string()).collect();
        write!(f, "{}", parts.join(", "))
    }
}

impl fmt::Display for ComplexSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (comb, compound)) in self.compounds.iter().enumerate() {
            match (i, comb) {
                (0, Some(c)) => match c {
                    Combinator::Descendant => write!(f, " ")?,
                    Combinator::Child => write!(f, "> ")?,
                    Combinator::Adjacent => write!(f, "+ ")?,
                    Combinator::Sibling => write!(f, "~ ")?,
                },
                (0, None) => {}
                (_, Some(Combinator::Descendant)) | (_, None) => write!(f, " ")?,
                (_, Some(Combinator::Child)) => write!(f, " > ")?,
                (_, Some(Combinator::Adjacent)) => write!(f, " + ")?,
                (_, Some(Combinator::Sibling)) => write!(f, " ~ ")?,
            }
            write!(f, "{compound}")?;
        }
        Ok(())
    }
}

impl fmt::Display for CompoundSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for sel in &self.0 {
            write!(f, "{sel}")?;
        }
        Ok(())
    }
}

impl fmt::Display for SimpleSelector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Universal => write!(f, "*"),
            Self::Type(s) => write!(f, "{s}"),
            Self::Class(s) => write!(f, ".{s}"),
            Self::Id(s) => write!(f, "#{s}"),
            Self::Attribute { name, op, value, modifier } => {
                write!(f, "[{name}")?;
                if let Some(op) = op {
                    write!(f, "{op}")?;
                    if let Some(val) = value {
                        write!(f, "{val}")?;
                    }
                }
                write!(f, "]")?;
                if let Some(m) = modifier {
                    write!(f, " {m}")?;
                }
                Ok(())
            }
            Self::PseudoClass { name, arg } => {
                write!(f, ":{name}")?;
                if let Some(arg) = arg {
                    write!(f, "({arg})")?;
                }
                Ok(())
            }
            Self::PseudoElement { name, arg } => {
                write!(f, "::{name}")?;
                if let Some(arg) = arg {
                    write!(f, "({arg})")?;
                }
                Ok(())
            }
            Self::Placeholder(s) => write!(f, "%{s}"),
        }
    }
}
