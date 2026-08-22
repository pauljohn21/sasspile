//! Lexer 单元测试——验证插值、转义、注释、复合 token。

use scss_rs::lex::{Lexed, Token};
use scss_rs::source::Source;
use scss_rs::lex::token::QuoteStyle;

fn lex(input: &str) -> Vec<Token> {
    let source = Source::new(input);
    let lexed = Lexed::try_from(source).unwrap();
    lexed.tokens
}

#[test]
fn test_simple_ident() {
    let tokens = lex("color");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "color"));
}

#[test]
fn test_number_with_unit() {
    let tokens = lex("16px");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Number(n, Some(u)) if n == "16" && u == "px"));
}

#[test]
fn test_number_decimal() {
    let tokens = lex("3.14");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Number(n, None) if n == "3.14"));
}

#[test]
fn test_string_double() {
    let tokens = lex("\"hello\"");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::String(s, QuoteStyle::Double) if s == "hello"));
}

#[test]
fn test_string_single() {
    let tokens = lex("'world'");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::String(s, QuoteStyle::Single) if s == "world"));
}

#[test]
fn test_interp() {
    let tokens = lex("#{1 + 2}");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Interp(s) if s == "1 + 2"));
}

#[test]
fn test_interp_nested() {
    let tokens = lex("#{func({a: 1})}");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Interp(s) if s == "func({a: 1})"));
}

#[test]
fn test_interp_with_string() {
    let tokens = lex("#{\"not\"}");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Interp(s) if s.contains("not")));
}

#[test]
fn test_silent_comment_preserved() {
    let tokens = lex("// this is a comment\nbody { }");
    assert!(tokens.iter().any(|t| matches!(t, Token::SilentComment(s) if s.contains("comment"))));
}

#[test]
fn test_block_comment_preserved() {
    let tokens = lex("/* block comment */ body { }");
    assert!(tokens.iter().any(|t| matches!(t, Token::Comment(s) if s.contains("block comment"))));
}

#[test]
fn test_at_rule() {
    let tokens = lex("@media screen { }");
    assert!(tokens.iter().any(|t| matches!(t, Token::AtRule(s) if s == "media")));
}

#[test]
fn test_at_rule_with_dash() {
    let tokens = lex("@at-root { }");
    assert!(tokens.iter().any(|t| matches!(t, Token::AtRule(s) if s == "at-root")));
}

#[test]
fn test_compound_operators() {
    let tokens = lex("1 >= 2 <= 3 != 4");
    assert!(tokens.iter().any(|t| matches!(t, Token::Gte)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Lte)));
    assert!(tokens.iter().any(|t| matches!(t, Token::NotEq)));
}

#[test]
fn test_arrow() {
    let tokens = lex("$x => 1");
    assert!(tokens.iter().any(|t| matches!(t, Token::Arrow)));
}

#[test]
fn test_dot_dot_dot() {
    let tokens = lex("$args...");
    assert!(tokens.iter().any(|t| matches!(t, Token::DotDotDot)));
}

#[test]
fn test_bang() {
    let tokens = lex("!important");
    assert!(tokens.iter().any(|t| matches!(t, Token::Bang)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Ident(s) if s == "important")));
}

#[test]
fn test_escape_in_string() {
    let tokens = lex("\"a\\nb\"");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::String(s, _) if s == "anb"));
}

#[test]
fn test_escape_quote_in_string() {
    let tokens = lex("\"say \\\"hi\\\"\"");
    assert!(matches!(&tokens[0], Token::String(s, _) if s.contains("hi")));
}

#[test]
fn test_keywords() {
    let tokens = lex("true and false or not null");
    assert!(tokens.iter().any(|t| matches!(t, Token::True)));
    assert!(tokens.iter().any(|t| matches!(t, Token::And)));
    assert!(tokens.iter().any(|t| matches!(t, Token::False)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Not)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Null)));
}

#[test]
fn test_variable() {
    let tokens = lex("$primary");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Variable(s) if s == "primary"));
}

#[test]
fn test_hex_color() {
    let tokens = lex("#ff0000");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::HexColor(s) if s == "ff0000"));
}

#[test]
fn test_hex_color_short() {
    let tokens = lex("#fff");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::HexColor(s) if s == "fff"));
}

#[test]
fn test_ampersand() {
    let tokens = lex("&:hover");
    assert!(tokens.iter().any(|t| matches!(t, Token::Ampersand)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Colon)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Ident(s) if s == "hover")));
}

#[test]
fn test_negative_number() {
    let tokens = lex("-10px");
    assert!(matches!(&tokens[0], Token::Number(n, Some(u)) if n == "-10" && u == "px"));
}

#[test]
fn test_dot_number() {
    let tokens = lex(".5em");
    assert!(matches!(&tokens[0], Token::Number(n, Some(u)) if n == ".5" && u == "em"));
}

#[test]
fn test_scientific_notation() {
    let tokens = lex("1e3");
    assert!(matches!(&tokens[0], Token::Number(n, _) if n.contains("e")));
}

#[test]
fn test_dash_ident() {
    let tokens = lex("-webkit-transition");
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "-webkit-transition"));
}

#[test]
fn test_css_custom_property() {
    let tokens = lex("--my-var: 1px");
    assert!(tokens.iter().any(|t| matches!(t, Token::Ident(s) if s == "--")));
    assert!(tokens.iter().any(|t| matches!(t, Token::Ident(s) if s == "my-var")));
}

#[test]
fn test_comment_does_not_break_code() {
    let tokens = lex("// comment\nbody { color: red; }");
    assert!(tokens.iter().any(|t| matches!(t, Token::SilentComment(s) if s.contains("comment"))));
    assert!(tokens.iter().any(|t| matches!(t, Token::Ident(s) if s == "body")));
    assert!(tokens.iter().any(|t| matches!(t, Token::Ident(s) if s == "color")));
    assert!(tokens.iter().any(|t| matches!(t, Token::Ident(s) if s == "red")));
}
