//! Unit tests for the lexer module.

use rxrust::prelude::*;

use lightforger::lexer::{Token, TokenKind, LexerState, TokenStream, lex, normalize_newlines};

/// Helper: collect a TokenStream into Vec<Token>
fn collect_tokens(stream: TokenStream) -> Vec<Token> {
    let result = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let r = result.clone();
    stream.subscribe(move |tok| r.borrow_mut().push(tok));
    match std::rc::Rc::try_unwrap(result) {
        Ok(cell) => cell.into_inner(),
        Err(_) => Vec::new(),
    }
}


#[test]
fn test_token_new() {
    let tok = Token::new(TokenKind::Ident, 0, "color");
    assert_eq!(tok.kind, TokenKind::Ident);
    assert_eq!(tok.pos, 0);
    assert_eq!(tok.text, "color");
}

#[test]
fn test_token_with_pos() {
    let tok = Token::with_pos(TokenKind::Number, 5, "42", 1, 6);
    assert_eq!(tok.line, 1);
    assert_eq!(tok.col, 6);
    assert_eq!(tok.text, "42");
}

#[test]
fn test_token_char_kind() {
    let tok = Token::new(TokenKind::Char('{'), 10, "{");
    assert_eq!(tok.char_kind(), Some('{'));

    let tok2 = Token::new(TokenKind::Ident, 0, "x");
    assert_eq!(tok2.char_kind(), None);
}

#[test]
fn test_lexer_state_default() {
    let state = LexerState::new();
    assert_eq!(state.pos, 0);
    assert_eq!(state.line, 1);
    assert_eq!(state.col, 1);
}

#[test]
fn test_lexer_state_advance() {
    let mut state = LexerState::new();
    state.advance('a');
    assert_eq!(state.pos, 1);
    assert_eq!(state.line, 1);
    assert_eq!(state.col, 2);

    state.advance('b');
    assert_eq!(state.pos, 2);
    assert_eq!(state.col, 3);
}

#[test]
fn test_lexer_state_newline() {
    let mut state = LexerState::new();
    state.advance('a');
    state.advance('b');
    state.advance('\n');
    assert_eq!(state.pos, 3);
    assert_eq!(state.line, 2);
    assert_eq!(state.col, 1);
}

#[test]
fn test_lexer_state_peek() {
    let mut state = LexerState::new();
    assert_eq!(state.peek(), 0);
    state.advance('a');
    assert_eq!(state.peek(), 1);
}

// ── Integration tests: lex() → Token stream ──

#[test]
fn test_lex_abc_produces_ident_token() {
    // "abc" is now lexed as a single Ident token
    let char_stream = Local::from_iter("abc ".chars()).box_it_clone();
    let tokens = collect_tokens(lex(char_stream));
    // Should contain an Ident("abc") token plus the space char
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Ident && t.text == "abc"),
        "expected Ident('abc') in {:?}", tokens);
}

#[test]
fn test_lex_var_token() {
    // "$x" → Variable("$x")
    let char_stream = Local::from_iter("$x ".chars()).box_it_clone();
    let tokens = collect_tokens(lex(char_stream));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Variable && t.text == "$x"),
        "expected Variable('$x') in {:?}", tokens);
}

#[test]
fn test_lex_newline_crlf() {
    // \r\n should normalize to single \n via normalize_newlines
    let normalized = normalize_newlines("\r\n");
    assert_eq!(normalized, "\n");
    let chars: Vec<char> = normalized.chars().collect();
    let char_stream = Local::from_iter(chars).box_it_clone();
    let tokens = collect_tokens(lex(char_stream));
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Char('\n'));
}

#[test]
fn test_lex_newline_formfeed() {
    // \x0C (form feed) → \n via normalize_newlines
    let normalized = normalize_newlines("\x0C");
    assert_eq!(normalized, "\n");
    let chars: Vec<char> = normalized.chars().collect();
    let char_stream = Local::from_iter(chars).box_it_clone();
    let tokens = collect_tokens(lex(char_stream));
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Char('\n'));
}

#[test]
fn test_lex_empty() {
    let char_stream = Local::from_iter(std::iter::empty::<char>()).box_it_clone();
    let tokens = collect_tokens(lex(char_stream));
    assert!(tokens.is_empty());
}

#[test]
fn test_error_construction() {
    use lightforger::Error;
    let e = Error::lexer("bad char @");
    assert_eq!(e.to_string(), "lexer error: bad char @");
    let e2 = Error::parser("unexpected }");
    assert_eq!(e2.to_string(), "parser error: unexpected }");
}

#[test]
fn test_normalize_newlines_plain() {
    assert_eq!(normalize_newlines("a\r\nb"), "a\nb");
    assert_eq!(normalize_newlines("a\rb"), "a\nb");
    assert_eq!(normalize_newlines("a\x0Cb"), "a\nb");
    assert_eq!(normalize_newlines("no change"), "no change");
}
