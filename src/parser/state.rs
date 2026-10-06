use crate::types::*;
use std::mem;

#[derive(Debug)]
pub struct ParserState {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub brace_stack: Vec<char>,
    pub scope_id_counter: u64,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Clone)]
pub struct ParseError { pub pos: u32, pub message: String }

impl ParserState {
    pub fn new() -> Self {
        Self { tokens: Vec::new(), pos: 0, brace_stack: Vec::new(), scope_id_counter: 1, errors: Vec::new() }
    }
    pub fn push_token(&mut self, tok: Token) { self.tokens.push(tok); }
    pub fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    pub fn peek_n(&self, n: usize) -> Option<&Token> { self.tokens.get(self.pos + n) }
    pub fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = mem::replace(&mut self.tokens[self.pos], Token::Eof);
            self.pos += 1;
            Some(t)
        } else { None }
    }
    pub fn next_scope_id(&mut self) -> u64 { self.scope_id_counter += 1; self.scope_id_counter }
}
