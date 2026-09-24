//! TokenKind — 单行分类 (纯函数, &str → enum)
//!
//! 这是 group_by 的 key selector,只做一次分类,不做解析/处理

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    VariableDef,
    AtEachSingle,
    AtEachMulti,
    AtForSingle,
    AtForMulti,
    AtIfStart,
    AtElseIf,
    AtElse,
    AtMixinDef,
    AtInclude,
    AtExtend,
    AtUse,
    AtForward,
    AtKeyframes,
    PlaceholderDef,
    RuleBlock,
    RuleStart,
    CloseBrace,
    EmptyLine,
    PlainLine,
}

impl TokenKind {
    /// 分类 — 纯函数, &str → enum
    pub fn classify(token: &str) -> Self {
        let t = token.trim();

        if t.is_empty() {
            return Self::EmptyLine;
        }
        if t == "}" {
            return Self::CloseBrace;
        }
        if t.starts_with('$') && t.contains(':') {
            return Self::VariableDef;
        }
        if let Some(rest) = t.strip_prefix("@") {
            return Self::classify_at_rule(rest);
        }
        if let Some(brace_idx) = Self::find_block_brace(t) {
            let sel = &t[..brace_idx];
            return if sel.starts_with('%') {
                Self::PlaceholderDef
            } else if t.ends_with('}') {
                Self::RuleBlock
            } else {
                Self::RuleStart
            };
        }
        Self::PlainLine
    }

    fn classify_at_rule(rest: &str) -> Self {
        if rest.starts_with("each ") {
            if Self::is_single_line_block(rest) { Self::AtEachSingle }
            else { Self::AtEachMulti }
        } else if rest.starts_with("for ") {
            if Self::is_single_line_block(rest) { Self::AtForSingle }
            else { Self::AtForMulti }
        } else if rest.starts_with("if ") {
            Self::AtIfStart
        } else if rest.starts_with("else if ") || rest.starts_with("elseif ") {
            Self::AtElseIf
        } else if rest == "else" || rest.starts_with("else ") {
            Self::AtElse
        } else if rest.starts_with("mixin ") {
            Self::AtMixinDef
        } else if rest.starts_with("include ") {
            Self::AtInclude
        } else if rest.starts_with("extend ") {
            Self::AtExtend
        } else if rest.starts_with("use ") {
            Self::AtUse
        } else if rest.starts_with("forward ") {
            Self::AtForward
        } else if rest.starts_with("keyframes ") {
            Self::AtKeyframes
        } else {
            Self::PlainLine
        }
    }

    fn is_single_line_block(s: &str) -> bool {
        matches!((s.find('{'), s.rfind('}')), (Some(o), Some(c)) if c > o)
    }

    fn find_block_brace(t: &str) -> Option<usize> {
        t.char_indices()
            .find(|(i, c)| *c == '{' && *i > 0 && t.as_bytes().get(*i - 1) == Some(&b' '))
            .map(|(i, _)| i)
    }
}
