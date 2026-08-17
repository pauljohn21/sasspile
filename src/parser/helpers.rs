//! Parser helper methods — text reading utilities.

use crate::token::Token;
use super::Parser;

impl Parser {
    /// Read until `{` or `;` and return the text
    pub fn read_until_brace(&mut self) -> String {
        let mut s = String::new();
        while !self.is_at_end() && !matches!(self.peek(), Token::LBrace | Token::Semicolon) {
            match self.peek() {
                Token::Ident(n) => s.push_str(n),
                Token::Number(v, u) => {
                    s.push_str(&format!("{}{}", v, u.as_deref().unwrap_or("")));
                }
                Token::String(val, _) => {
                    s.push('"');
                    s.push_str(val);
                    s.push('"');
                }
                _ => {}
            }
            self.advance();
        }
        s.trim().to_string()
    }

    /// Read until `;` and return the text
    pub fn read_until_semicolon(&mut self) -> String {
        let mut s = String::new();
        while !self.is_at_end() && !matches!(self.peek(), Token::Semicolon) {
            match self.peek() {
                Token::Ident(n) => { s.push_str(n); s.push(' '); }
                Token::Dot => s.push('.'),
                Token::Hash => s.push('#'),
                Token::Ampersand => s.push('&'),
                _ => {}
            }
            self.advance();
        }
        s.trim().to_string()
    }

    /// Read a string literal or identifier
    pub fn read_string_or_ident(&mut self) -> String {
        match self.advance() {
            Token::String(s, _) => s.clone(),
            Token::Ident(s) => s.clone(),
            _ => String::new(),
        }
    }

    /// Read a list of member names (for @forward show/hide)
    pub fn read_member_list(&mut self) -> Vec<String> {
        let mut members = Vec::new();
        while !self.is_at_end() && !matches!(self.peek(), Token::Semicolon) {
            if let Token::Ident(s) = self.advance() {
                members.push(s.clone());
            }
            if matches!(self.peek(), Token::Comma) { self.advance(); }
        }
        members
    }
}
