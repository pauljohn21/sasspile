//! Sass indented syntax support.
//!
//! Generates virtual Indent/Dedent tokens from indentation in .sass files.

use tracing::instrument;

use crate::source::SourceSpan;

use super::token::{Token, TokenKind};

/// Tracks indentation levels for Sass syntax.
pub struct IndentTracker {
    /// Stack of indentation levels.
    levels: Vec<usize>,
}

impl IndentTracker {
    /// Create a new indent tracker.
    pub fn new() -> Self {
        Self {
            levels: vec![0],
        }
    }

    /// Process a line and return tokens to emit before its content.
    ///
    /// Input: raw line content (no newline).
    /// Returns: Indent/Dedent tokens based on indentation change.
    #[instrument(skip(self))]
    pub fn process_line(&mut self, line: &str) -> Vec<Token> {
        let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        let last = *self.levels.last().unwrap_or(&0);
        let mut tokens = Vec::new();

        if indent > last {
            // Indent
            self.levels.push(indent);
            let span = SourceSpan::new(0, indent as u32);
            tokens.push(Token::new(TokenKind::Indent, span));
        } else {
            // Dedent (possibly multiple levels)
            while let Some(&lvl) = self.levels.last() {
                if lvl > indent {
                    self.levels.pop();
                    let span = SourceSpan::new(0, indent as u32);
                    tokens.push(Token::new(TokenKind::Dedent, span));
                } else {
                    break;
                }
            }
        }
        tokens
    }

    /// Generate final dedent tokens at end of file.
    pub fn finalize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        for _ in 1..self.levels.len() {
            self.levels.pop();
            tokens.push(Token::new(TokenKind::Dedent, SourceSpan::new(0, 0)));
        }
        tokens
    }
}

impl Default for IndentTracker {
    fn default() -> Self {
        Self::new()
    }
}
