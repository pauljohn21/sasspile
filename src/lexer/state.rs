use crate::types::Token;

#[derive(Debug, Clone, Default)]
pub struct LexerState {
    pub pos: u32,
    pub in_double_quote: bool,
    pub in_single_quote: bool,
    pub interp_depth: u32,
    pub in_comment: bool,
    pub buf: String,
}

impl LexerState {
    pub fn new() -> Self { Self::default() }
    pub fn reset(&mut self) {
        self.pos = 0;
        self.in_double_quote = false;
        self.in_single_quote = false;
        self.interp_depth = 0;
        self.in_comment = false;
        self.buf.clear();
    }
    pub fn feed(&mut self, ch: char) -> Vec<Token> {
        self.pos += 1;
        let mut out = Vec::new();
        if self.buf.is_empty() && !self.in_double_quote && !self.in_single_quote && !self.in_comment {
            match ch {
                '\'' => { self.in_single_quote = true; self.buf.push(ch); }
                '"' => { self.in_double_quote = true; self.buf.push(ch); }
                '#' => { out.push(Token::InterpolationStart); self.interp_depth += 1; }
                '@' => {
                    // Check if @ was emitted by a previous flush; buffer if needed
                    self.buf.push('@');
                }
                '$' => { out.push(Token::Dollar); }
                '{' => { out.push(Token::LBrace); }
                '}' => { out.push(Token::RBrace); }
                '(' => { out.push(Token::LParen); }
                ')' => { out.push(Token::RParen); }
                '[' => { out.push(Token::LBracket); }
                ']' => { out.push(Token::RBracket); }
                ':' => { out.push(Token::Colon); }
                ';' => { out.push(Token::Semicolon); }
                ',' => { out.push(Token::Comma); }
                '.' => { out.push(Token::Dot); }
                '&' => { out.push(Token::Ampersand); }
                '+' => { out.push(Token::Plus); }
                '-' => { out.push(Token::Minus); }
                '*' => { out.push(Token::Star); }
                '/' => { out.push(Token::Slash); }
                '%' => { out.push(Token::Percent); }
                '=' => { out.push(Token::Eq); }
                '!' => { out.push(Token::Ne); }
                '<' => { out.push(Token::Lt); }
                '>' => { out.push(Token::Gt); }
                c if c.is_whitespace() => { out.push(Token::Whitespace); }
                c if c.is_ascii_alphabetic() || c == '_' => { self.buf.push(ch); }
                c if c.is_ascii_digit() => { self.buf.push(ch); }
                _ => {}
            }
        } else if self.in_double_quote || self.in_single_quote {
            self.buf.push(ch);
        } else {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                self.buf.push(ch);
            } else {
                self.flush_buf(&mut out);
                let sub = self.feed(ch);
                out.extend(sub);
            }
        }
        out
    }
    pub fn flush_buf_checked(&mut self, out: &mut Vec<Token>) { self.flush_buf(out); }
    fn flush_buf(&mut self, out: &mut Vec<Token>) {
        if self.buf.is_empty() { return; }
        let raw = self.buf.clone();
        self.buf.clear();
        if raw.starts_with('@') {
            match raw.as_str() {
                "@media" => out.push(Token::AtMedia),
                "@supports" => out.push(Token::AtSupports),
                "@if" => out.push(Token::AtIf),
                "@for" => out.push(Token::AtFor),
                "@each" => out.push(Token::AtEach),
                "@while" => out.push(Token::AtWhile),
                "@mixin" => out.push(Token::AtMixin),
                "@include" => out.push(Token::AtInclude),
                "@function" => out.push(Token::AtFunction),
                "@return" => out.push(Token::AtReturn),
                "@use" => out.push(Token::AtUse),
                "@forward" => out.push(Token::AtForward),
                "@extend" => out.push(Token::AtExtend),
                "@warn" => out.push(Token::AtWarn),
                "@debug" => out.push(Token::AtDebug),
                other => out.push(Token::IdentAt(other.into())),
            }
        } else if let Ok(n) = raw.parse::<f64>() {
            out.push(Token::Number(n, None));
        } else if raw.starts_with('#') && raw.len() > 1 {
            out.push(Token::HashId(raw[1..].into()));
        } else {
            out.push(Token::Ident(raw));
        }
    }
}
