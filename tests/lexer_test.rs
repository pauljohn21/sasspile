use rx_scss::lexer::{scan, LexerState};
use rx_scss::types::Token;
use std::sync::{Arc, Mutex};
use rxrust::prelude::*;

fn collect_tokens(input: &str) -> Vec<Token> {
    let stream = scan(input);
    let result = Arc::new(Mutex::new(Vec::new()));
    let r = result.clone();
    let collected = stream.collect::<Vec<_>>();
    collected.subscribe(move |toks| {
        *r.lock().unwrap() = toks;
    });
    let guard = result.lock().unwrap();
    guard.clone()
}

#[test]
fn token_coverage() {
    let input = "$color: #369; .a { &:hover { color: $color; } }";
    let toks = collect_tokens(input);
    assert!(!toks.is_empty());
}

#[test]
fn at_rule_recognition() {
    let toks = collect_tokens("@media screen { }");
    assert!(toks.contains(&Token::AtMedia));
}

#[test]
fn basic_ident_number() {
    let toks = collect_tokens("color red");
    let idents: Vec<_> = toks.iter().filter(|t| matches!(t, Token::Ident(_))).collect();
    assert!(!idents.is_empty());
}

#[test]
fn lexer_state_feed() {
    let mut s = LexerState::new();
    let t = s.feed('$');
    assert_eq!(t, vec![Token::Dollar]);
}

#[test]
fn operator_double_char() {
    let toks = collect_tokens("== != <= >=");
    let has_eq = toks.contains(&Token::Eq);
    let has_ne = toks.contains(&Token::Ne);
    assert!(has_eq, "expected Eq in {:?}", toks);
    assert!(has_ne, "expected Ne in {:?}", toks);
}
