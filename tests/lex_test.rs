use sasspile::lex::Lexer;
use sasspile::lex::token::Token;

fn lex(input: &str) -> Vec<Token> {
    Lexer::new(input)
        .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace) | Ok(Token::Eof)))
        .map(|t| t.unwrap())
        .collect()
}

#[test]
fn test_ident() {
    assert_eq!(lex("color"), vec![Token::Ident("color".to_string())]);
}

#[test]
fn test_number_with_unit() {
    assert_eq!(lex("16px"), vec![Token::Number("16px".to_string())]);
}

#[test]
fn test_number_decimal() {
    assert_eq!(lex("3.14"), vec![Token::Number("3.14".to_string())]);
}

#[test]
fn test_string() {
    assert_eq!(
        lex("\"hello\""),
        vec![Token::String("hello".to_string(), '"')]
    );
}

#[test]
fn test_interp() {
    assert_eq!(lex("#{1 + 2}"), vec![Token::Interp("1 + 2".to_string())]);
}

#[test]
fn test_amp() {
    assert_eq!(
        lex("&:hover"),
        vec![Token::Amp, Token::Colon, Token::Ident("hover".to_string())]
    );
}

#[test]
fn test_dot_dot_dot() {
    assert_eq!(lex("..."), vec![Token::DotDotDot]);
}

#[test]
fn test_at_rule() {
    assert_eq!(lex("@media"), vec![Token::AtRule("media".to_string())]);
}

#[test]
fn test_dollar() {
    assert_eq!(lex("$color"), vec![Token::Dollar("color".to_string())]);
}

#[test]
fn test_hash() {
    assert_eq!(lex("#ff0000"), vec![Token::Hash("ff0000".to_string())]);
}

#[test]
fn test_line_comment() {
    let tokens = lex("// comment");
    assert_eq!(tokens, vec![Token::Comment("comment".to_string(), true)]);
}

#[test]
fn test_block_comment() {
    let tokens = lex("/* hello */");
    assert_eq!(tokens, vec![Token::Comment("hello".to_string(), false)]);
}

#[test]
fn test_operators() {
    let tokens = lex("== != <= >=");
    assert_eq!(
        tokens,
        vec![Token::Eq, Token::NotEq, Token::LessEq, Token::GreaterEq]
    );
}

#[test]
fn test_keywords() {
    let tokens = lex("true false null and or not");
    assert_eq!(
        tokens,
        vec![
            Token::True,
            Token::False,
            Token::Null,
            Token::And,
            Token::Or,
            Token::Not
        ]
    );
}

#[test]
fn test_full_selector() {
    let tokens = lex("a:hover");
    assert_eq!(
        tokens,
        vec![
            Token::Ident("a".to_string()),
            Token::Colon,
            Token::Ident("hover".to_string()),
        ]
    );
}
