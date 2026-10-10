use rx_scss::types::*;
use rxrust::prelude::*;

#[cfg(test)]
mod lexer_number_with_unit {
    use super::*;
    use rx_scss::lexer::scan;

    fn collect_tokens(source: &str) -> Vec<Token> {
        let stream = scan(source);
        rx_scss::collect_boxed(stream).unwrap_or_default()
    }

    #[test]
    fn test_px_unit() {
        let toks = collect_tokens("16px");
        assert!(toks.contains(&Token::Number(16.0, Some("px".into()))));
    }

    #[test]
    fn test_em_unit() {
        let toks = collect_tokens("1.5em");
        assert!(toks.contains(&Token::Number(1.5, Some("em".into()))));
    }

    #[test]
    fn test_rem_unit() {
        let toks = collect_tokens("2rem");
        assert!(toks.contains(&Token::Number(2.0, Some("rem".into()))));
    }

    #[test]
    fn test_percent_unit() {
        let toks = collect_tokens("100%");
        assert!(toks.contains(&Token::Number(100.0, Some("%".into()))));
    }

    #[test]
    fn test_s_unit() {
        let toks = collect_tokens("0.3s");
        assert!(toks.contains(&Token::Number(0.3, Some("s".into()))));
    }

    #[test]
    fn test_deg_unit() {
        let toks = collect_tokens("45deg");
        assert!(toks.contains(&Token::Number(45.0, Some("deg".into()))));
    }

    #[test]
    fn test_number_without_unit() {
        let toks = collect_tokens("42");
        assert!(toks.contains(&Token::Number(42.0, None)));
    }
}

#[cfg(test)]
mod lexer_comments {
    use super::*;
    use rx_scss::lexer::scan;

    fn collect_tokens(source: &str) -> Vec<Token> {
        let stream = scan(source);
        rx_scss::collect_boxed(stream).unwrap_or_default()
    }

    #[test]
    fn test_line_comment_suppressed() {
        let toks = collect_tokens("// this is a comment\n$var: red;");
        // The comment should produce no tokens; $var and red should be present
        assert!(!toks.contains(&Token::Ident("// this is a comment".into())));
        assert!(toks.contains(&Token::Ident("var".into())));
    }

    #[test]
    fn test_block_comment_suppressed() {
        let toks = collect_tokens("/* block comment */ $x: 1;");
        assert!(!toks.contains(&Token::Ident("block".into())));
        assert!(toks.contains(&Token::Ident("x".into())));
    }

    #[test]
    fn test_code_between_comments() {
        let toks = collect_tokens("$a: 1; // comment\n$b: 2;");
        assert!(toks.contains(&Token::Ident("a".into())));
        assert!(toks.contains(&Token::Ident("b".into())));
    }
}

#[cfg(test)]
mod lexer_strings {
    use super::*;
    use rx_scss::lexer::scan;

    fn collect_tokens(source: &str) -> Vec<Token> {
        let stream = scan(source);
        rx_scss::collect_boxed(stream).unwrap_or_default()
    }

    #[test]
    fn test_double_quoted_string() {
        let toks = collect_tokens("\"hello world\"");
        assert!(toks.contains(&Token::Str("hello world".into())));
    }

    #[test]
    fn test_single_quoted_string() {
        let toks = collect_tokens("'hello'");
        assert!(toks.contains(&Token::Str("hello".into())));
    }

    #[test]
    fn test_string_with_escape() {
        let toks = collect_tokens("\"hello\\\"world\"");
        assert!(toks.contains(&Token::Str("hello\"world".into())));
    }

    #[test]
    fn test_string_in_property() {
        let toks = collect_tokens("content: \";\"");
        assert!(toks.contains(&Token::Str(";".into())));
    }
}

#[cfg(test)]
mod lexer_operators {
    use super::*;
    use rx_scss::lexer::scan;

    fn collect_tokens(source: &str) -> Vec<Token> {
        let stream = scan(source);
        rx_scss::collect_boxed(stream).unwrap_or_default()
    }

    #[test]
    fn test_eq_operator() {
        let toks = collect_tokens("$x == 1");
        assert!(toks.contains(&Token::Eq));
    }

    #[test]
    fn test_ne_operator() {
        let toks = collect_tokens("$x != 1");
        assert!(toks.contains(&Token::Ne));
    }

    #[test]
    fn test_le_operator() {
        let toks = collect_tokens("$x <= 5");
        assert!(toks.contains(&Token::Le));
    }

    #[test]
    fn test_ge_operator() {
        let toks = collect_tokens("$x >= 5");
        assert!(toks.contains(&Token::Ge));
    }

    #[test]
    fn test_lt_operator() {
        let toks = collect_tokens("$x < 5");
        assert!(toks.contains(&Token::Lt));
    }

    #[test]
    fn test_gt_operator() {
        let toks = collect_tokens("$x > 5");
        assert!(toks.contains(&Token::Gt));
    }
}

#[cfg(test)]
mod lexer_at_rules {
    use super::*;
    use rx_scss::lexer::scan;

    fn collect_tokens(source: &str) -> Vec<Token> {
        let stream = scan(source);
        rx_scss::collect_boxed(stream).unwrap_or_default()
    }

    #[test]
    fn test_at_import() {
        let toks = collect_tokens("@import \"variables\";");
        assert!(toks.contains(&Token::AtImport));
    }

    #[test]
    fn test_at_else() {
        let toks = collect_tokens("@else { }");
        assert!(toks.contains(&Token::AtElse));
    }

    #[test]
    fn test_at_at_root() {
        let toks = collect_tokens("@at-root .foo { }");
        assert!(toks.contains(&Token::AtAtRoot));
    }

    #[test]
    fn test_at_error() {
        let toks = collect_tokens("@error \"bad\";");
        assert!(toks.contains(&Token::AtError));
    }

    #[test]
    fn test_at_charset() {
        let toks = collect_tokens("@charset \"UTF-8\";");
        assert!(toks.contains(&Token::AtCharset));
    }

    #[test]
    fn test_at_keyframes() {
        let toks = collect_tokens("@keyframes slide { }");
        assert!(toks.contains(&Token::AtKeyframes));
    }

    #[test]
    fn test_logical_and_keyword() {
        let toks = collect_tokens("a and b");
        assert!(toks.contains(&Token::And), "toks: {:?}", toks);
        assert!(!toks.iter().any(|t| matches!(t, Token::Ident(s) if s == "and")));
    }

    #[test]
    fn test_logical_or_keyword() {
        let toks = collect_tokens("a or b");
        assert!(toks.contains(&Token::Or), "toks: {:?}", toks);
        assert!(!toks.iter().any(|t| matches!(t, Token::Ident(s) if s == "or")));
    }

    #[test]
    fn test_unary_minus_before_var() {
        let toks = collect_tokens("-$x");
        // Should produce [Minus, Dollar, Ident("x")]
        assert!(toks.contains(&Token::Minus), "minus: {:?}", toks);
        assert!(toks.contains(&Token::Dollar), "dollar: {:?}", toks);
        assert!(toks.iter().any(|t| matches!(t, Token::Ident(s) if s == "x")), "ident x: {:?}", toks);
    }

    #[test]
    fn test_selector_interpolation_tokens() {
        let toks = collect_tokens(".#{$klass}");
        // We expect: [Dot, InterpolationStart, Dollar, Ident("klass"), InterpolationEnd]
        let has_interp = toks.contains(&Token::InterpolationStart);
        let has_dot = toks.contains(&Token::Dot);
        let has_interp_end = toks.contains(&Token::InterpolationEnd);
        assert!(has_dot, "should have Dot: {:?}", toks);
        assert!(has_interp, "should have InterpolationStart: {:?}", toks);
        assert!(has_interp_end, "should have InterpolationEnd: {:?}", toks);
    }
}
