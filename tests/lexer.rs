//! Lexer tests — tests tokenization of SCSS source code.

use sasspile::tokenize;
use sasspile::token::Token;

fn tok(input: &str) -> Vec<Token> {
    let spans = tokenize(input, "test_input").expect("tokenize should succeed");
    spans.into_iter().map(|ts| ts.token).collect()
}

#[test]
fn test_tokenize_empty() {
    let tokens = tok("");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::Eof));
}

#[test]
fn test_tokenize_simple_selector() {
    let tokens = tok("a { }");
    assert_eq!(tokens.len(), 4);
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "a"));
    assert!(matches!(tokens[1], Token::LBrace));
    assert!(matches!(tokens[2], Token::RBrace));
}

#[test]
fn test_tokenize_declaration() {
    let tokens = tok("color: red;");
    assert_eq!(tokens.len(), 5); // color, :, red, ;, Eof
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "color"));
    assert!(matches!(tokens[1], Token::Colon));
    assert!(matches!(&tokens[2], Token::Ident(s) if s == "red"));
    assert!(matches!(tokens[3], Token::Semicolon));
}

#[test]
fn test_tokenize_number_with_unit() {
    let tokens = tok("10px");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::Number(v, Some(u)) if (v - 10.0).abs() < 1e-9 && u == "px"));
}

#[test]
fn test_tokenize_number_without_unit() {
    let tokens = tok("42");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::Number(v, None) if (v - 42.0).abs() < 1e-9));
}

#[test]
fn test_tokenize_decimal_number() {
    let tokens = tok("1.5em");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::Number(v, Some(u)) if (v - 1.5).abs() < 1e-9 && u == "em"));
}

#[test]
fn test_tokenize_percentage() {
    let tokens = tok("50%");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::Number(v, Some(u)) if (v - 50.0).abs() < 1e-9 && u == "%"));
}

#[test]
fn test_tokenize_variable() {
    let tokens = tok("$color");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::Variable(s) if s == "color"));
}

#[test]
fn test_tokenize_at_rule() {
    let tokens = tok("@mixin");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::AtRule(s) if s == "mixin"));
}

#[test]
fn test_tokenize_at_include() {
    let tokens = tok("@include foo");
    assert_eq!(tokens.len(), 3);
    assert!(matches!(&tokens[0], Token::AtRule(s) if s == "include"));
    assert!(matches!(&tokens[1], Token::Ident(s) if s == "foo"));
}

#[test]
fn test_tokenize_hex_color() {
    let tokens = tok("#fff");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::HexColor(v) if *v == 0xfff));
}

#[test]
fn test_tokenize_hex_color_long() {
    let tokens = tok("#aabbcc");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::HexColor(v) if *v == 0xaabbcc));
}

#[test]
fn test_tokenize_interpolation() {
    let tokens = tok("#{1 + 2}");
    assert_eq!(tokens.len(), 6); // #{, 1, +, 2, }, Eof
    assert!(matches!(tokens[0], Token::InterpolationStart));
    assert!(matches!(&tokens[1], Token::Number(_, None)));
    assert!(matches!(tokens[2], Token::Plus));
    assert!(matches!(&tokens[3], Token::Number(_, None)));
    // `}` is RBrace (parser decides if it's interpolation end)
    assert!(matches!(tokens[4], Token::RBrace));
}

#[test]
fn test_tokenize_quoted_string() {
    let tokens = tok("\"hello\"");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::String(s, '"') if s == "hello"));
}

#[test]
fn test_tokenize_single_quoted_string() {
    let tokens = tok("'hello'");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(&tokens[0], Token::String(s, '\'') if s == "hello"));
}

#[test]
fn test_tokenize_line_comment() {
    let tokens = tok("// this is a comment\na");
    assert_eq!(tokens.len(), 3);
    assert!(matches!(&tokens[0], Token::LineComment(c) if c.contains("this is a comment")));
    assert!(matches!(&tokens[1], Token::Ident(s) if s == "a"));
}

#[test]
fn test_tokenize_block_comment() {
    let tokens = tok("/* comment */a");
    assert_eq!(tokens.len(), 3);
    assert!(matches!(&tokens[0], Token::BlockComment(c) if c.contains("comment")));
    assert!(matches!(&tokens[1], Token::Ident(s) if s == "a"));
}

#[test]
fn test_tokenize_operators() {
    let tokens = tok("1 + 2 - 3 * 4 / 5 % 6");
    assert_eq!(tokens.len(), 12); // 6 numbers + 5 operators + Eof
    assert!(matches!(tokens[1], Token::Plus));
    assert!(matches!(tokens[3], Token::Minus));
    assert!(matches!(tokens[5], Token::Star));
    assert!(matches!(tokens[7], Token::Slash));
    assert!(matches!(tokens[9], Token::Percent));
}

#[test]
fn test_tokenize_comparison_ops() {
    let tokens = tok("1 < 2 <= 3 > 4 >= 5 == 6 != 7");
    // 7 numbers + 6 comparison ops + Eof = 14
    assert_eq!(tokens.len(), 14);
    assert!(matches!(tokens[1], Token::Lt));
    assert!(matches!(tokens[3], Token::LtEq));
    assert!(matches!(tokens[5], Token::Gt));
    assert!(matches!(tokens[7], Token::GtEq));
    assert!(matches!(tokens[9], Token::Eq));
    assert!(matches!(tokens[11], Token::NotEq));
}

#[test]
fn test_tokenize_ampersand() {
    let tokens = tok("&:hover");
    assert_eq!(tokens.len(), 4); // &, :, hover, Eof
    assert!(matches!(tokens[0], Token::Ampersand));
    assert!(matches!(tokens[1], Token::Colon));
    assert!(matches!(&tokens[2], Token::Ident(s) if s == "hover"));
}

#[test]
fn test_tokenize_bang_flags() {
    let tokens = tok("!default !global !optional");
    assert_eq!(tokens.len(), 4);
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "!default"));
    assert!(matches!(&tokens[1], Token::Ident(s) if s == "!global"));
    assert!(matches!(&tokens[2], Token::Ident(s) if s == "!optional"));
}

#[test]
fn test_tokenize_parens_brackets_braces() {
    let tokens = tok("()[]{}");
    assert_eq!(tokens.len(), 7); // (, ), [, ], {, }, Eof
    assert!(matches!(tokens[0], Token::LParen));
    assert!(matches!(tokens[1], Token::RParen));
    assert!(matches!(tokens[2], Token::LBracket));
    assert!(matches!(tokens[3], Token::RBracket));
    assert!(matches!(tokens[4], Token::LBrace));
    assert!(matches!(tokens[5], Token::RBrace));
}

#[test]
fn test_tokenize_complex_scss() {
    let input = "$color: #336699;\n.foo { color: $color; }";
    let tokens = tok(input);
    assert!(!tokens.is_empty());
    // $color variable
    assert!(matches!(&tokens[0], Token::Variable(s) if s == "color"));
    // colon
    assert!(matches!(tokens[1], Token::Colon));
    // hex color
    assert!(matches!(&tokens[2], Token::HexColor(v) if *v == 0x336699));
    // semicolon
    assert!(matches!(tokens[3], Token::Semicolon));
    // .foo -> Dot + Ident("foo")
    assert!(matches!(tokens[4], Token::Dot));
    assert!(matches!(&tokens[5], Token::Ident(s) if s == "foo"));
}

#[test]
fn test_tokenize_position_tracking() {
    let spans = tokenize("a\n  color: red;\n", "test_pos").expect("should succeed");
    let first = &spans[0];
    assert_eq!(first.pos.line, 1);
    assert_eq!(first.pos.column, 1);
    // Find the color ident (after newline + spaces)
    let color_span = spans
        .iter()
        .find(|ts| matches!(&ts.token, Token::Ident(s) if s == "color"));
    assert!(color_span.is_some());
    let color_span = color_span.unwrap();
    assert_eq!(color_span.pos.line, 2);
    assert_eq!(color_span.pos.column, 3);
}
