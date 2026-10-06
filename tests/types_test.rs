use rx_scss::types::*;

#[test]
fn token_coverage() {
    let tokens = vec![
        Token::Ident("body".into()),
        Token::Str("hello".into()),
        Token::Number(16.0, Some("px".into())),
        Token::HashId("main".into()),
        Token::Ampersand,
        Token::Dollar,
        Token::Colon,
        Token::Semicolon,
        Token::Comma,
        Token::Dot,
        Token::LParen,
        Token::RParen,
        Token::LBrace,
        Token::RBrace,
        Token::LBracket,
        Token::RBracket,
        Token::Eq,
        Token::Ne,
        Token::Le,
        Token::Ge,
        Token::Lt,
        Token::Gt,
        Token::Plus,
        Token::Minus,
        Token::Slash,
        Token::Star,
        Token::Percent,
        Token::AtMedia,
        Token::AtSupports,
        Token::AtIf,
        Token::AtFor,
        Token::AtEach,
        Token::AtWhile,
        Token::AtMixin,
        Token::AtInclude,
        Token::AtFunction,
        Token::AtReturn,
        Token::AtUse,
        Token::AtForward,
        Token::AtExtend,
        Token::AtWarn,
        Token::AtDebug,
        Token::InterpolationStart,
        Token::InterpolationEnd,
        Token::AtName,
        Token::IdentAt("@page".into()),
        Token::Whitespace,
        Token::Eof,
    ];
    assert!(tokens.len() >= 40, "Token should have 40+ variants, got {}", tokens.len());
}

#[test]
fn token_pos_returns_zero() {
    let tok = Token::Ident("x".into());
    assert_eq!(tok.pos(), 0);
}

#[test]
fn token_eq_variants() {
    assert_eq!(Token::AtMedia, Token::AtMedia);
    assert_ne!(Token::AtMedia, Token::AtSupports);
}

#[test]
fn binop_coverage() {
    let ops = vec![
        BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Div, BinOp::Mod,
        BinOp::Eq, BinOp::Ne, BinOp::Lt, BinOp::Gt, BinOp::Le, BinOp::Ge,
        BinOp::And, BinOp::Or,
    ];
    assert_eq!(ops.len(), 13);
}

#[test]
fn unaryop_coverage() {
    let ops = vec![UnaryOp::Neg, UnaryOp::Not];
    assert_eq!(ops.len(), 2);
}

#[test]
fn param_creation() {
    let p = Param::new("color");
    assert_eq!(p.name, "color");
    assert!(p.default_value.is_none());

    let p2 = Param::with_default("size", AstNode::Literal(Value::Number(10.0)));
    assert_eq!(p2.name, "size");
    assert!(p2.default_value.is_some());
}

#[test]
fn input_syntax_default() {
    assert_eq!(InputSyntax::default(), InputSyntax::Scss);
}

#[test]
fn output_style_default() {
    assert_eq!(OutputStyle::default(), OutputStyle::Expanded);
}

#[test]
fn compile_error_display() {
    let e = CompileError::Io("not found".into());
    assert!(e.to_string().contains("IO error"));

    let e2 = CompileError::Parse { pos: 42, message: "unexpected".into() };
    assert!(e2.to_string().contains("42"));

    let e3 = CompileError::Eval("bad".into());
    assert!(e3.to_string().contains("Eval error"));
}
