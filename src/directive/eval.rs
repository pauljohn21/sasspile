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
    AtWhile,
    AtIfStart,
    AtElseIf,
    AtElse,
    AtMixinDef,
    AtInclude,
    AtIncludeMulti,
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
    /// 分类 — 纯函数, &str → enum (match 表达式 dispatch, 无 if/return 链)
    pub fn classify(token: &str) -> Self {
        let t = token.trim();

        match t {
            "" => Self::EmptyLine,
            "}" => Self::CloseBrace,
            _ if Self::is_variable(t) => Self::VariableDef,
            _ if t.starts_with('@') => Self::classify_at_rule(&t[1..]),
            _ => Self::classify_non_at_rule(t),
        }
    }

    /// 非 @rule 分类: block 块 or plain
    #[inline]
    fn classify_non_at_rule(t: &str) -> Self {
        let brace_idx = Self::find_block_brace(t);
        match brace_idx {
            Some(idx) => {
                let sel = &t[..idx];
                match (sel.starts_with('%'), t.ends_with('}')) {
                    (true, _) => Self::PlaceholderDef,
                    (_, true) => Self::RuleBlock,
                    _ => Self::RuleStart,
                }
            }
            None => Self::PlainLine,
        }
    }

    #[inline]
    fn is_variable(t: &str) -> bool {
        t.starts_with('$') && t.contains(':')
    }

    fn classify_at_rule(rest: &str) -> Self {
        // match 在 (首 token, 次 token) 元组上, 纯声明式 dispatch
        let mut tokens = rest.split(' ');
        let (first, second) = (tokens.next().unwrap_or(""), tokens.next().unwrap_or(""));
        match (first, second) {
            ("each", _) => Self::from_single_line(rest, Self::AtEachSingle, Self::AtEachMulti),
            ("for", _)  => Self::from_single_line(rest, Self::AtForSingle, Self::AtForMulti),
            ("while", _) => Self::AtWhile,
            ("if", _) => Self::AtIfStart,
            ("else", "if") | ("else", "elseif") | ("elseif", _) => Self::AtElseIf,
            ("else", _) => Self::AtElse,
            ("mixin", _) => Self::AtMixinDef,
            ("include", _) => Self::classify_include(rest),
            ("extend", _) => Self::AtExtend,
            ("use", _) => Self::AtUse,
            ("forward", _) => Self::AtForward,
            ("keyframes", _) => Self::AtKeyframes,
            _ => Self::PlainLine,
        }
    }

    /// 单行/多行分发: 有 `{...}` 同一行 → single, 否则 multi
    #[inline]
    fn from_single_line(s: &str, single: Self, multi: Self) -> Self {
        if Self::is_single_line_block(s) { single } else { multi }
    }

    /// @include 分类: 有 { 但无 } → Multi (content block 未闭); 否则 Single
    #[inline]
    fn classify_include(rest: &str) -> Self {
        let has_open = rest.contains('{');
        let has_close = rest.contains('}');
        if has_open && !has_close { Self::AtIncludeMulti } else { Self::AtInclude }
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
