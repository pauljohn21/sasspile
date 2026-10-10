use crate::Token;
use std::mem;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct ParserState {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub brace_stack: Vec<char>,
    pub scope_id_counter: u64,
    pub errors: Vec<ParseError>,
    pub include_paths: Vec<PathBuf>,
    pub imports_seen: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ParseError { pub pos: u32, pub message: String }

impl ParserState {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            pos: 0,
            brace_stack: Vec::new(),
            scope_id_counter: 1,
            errors: Vec::new(),
            include_paths: Vec::new(),
            imports_seen: Vec::new(),
        }
    }
    pub fn with_include_paths(paths: Vec<PathBuf>) -> Self {
        Self { include_paths: paths, ..Self::new() }
    }
    pub fn push_token(&mut self, tok: Token) { self.tokens.push(tok); }
    pub fn peek(&self) -> Option<&Token> { self.tokens.get(self.pos) }
    pub fn peek_n(&self, n: usize) -> Option<&Token> { self.tokens.get(self.pos + n) }
    #[allow(clippy::should_implement_trait)]
    pub fn next_token(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = mem::replace(&mut self.tokens[self.pos], Token::Eof);
            self.pos += 1;
            Some(t)
        } else { None }
    }
    pub fn next_scope_id(&mut self) -> u64 { self.scope_id_counter += 1; self.scope_id_counter }
}
