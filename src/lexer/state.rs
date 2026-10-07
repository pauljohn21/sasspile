use crate::types::Token;

/// Pending operator state for two-character operators (==, !=, <=, >=).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingOp {
    Eq, // saw '=', waiting for next char to decide ==
    Lt, // saw '<', waiting for next char to decide <=
    Gt, // saw '>', waiting for next char to decide >=
    Ne, // saw '!', waiting for next char to decide !=
}

/// Pending slash state: saw '/', waiting for next char to decide:
/// '//' → line comment, '/*' → block comment, otherwise division operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingSlash {
    Slash,
}

#[derive(Debug, Clone, Default)]
pub struct LexerState {
    pub pos: u32,
    pub in_double_quote: bool,
    pub in_single_quote: bool,
    pub interp_depth: u32,
    pub in_line_comment: bool,
    pub in_block_comment: bool,
    pub buf: String,
    pub prev_char: Option<char>,
    pub escaped: bool,
    pending_op: Option<PendingOp>,
    pending_slash: Option<PendingSlash>,
}

impl LexerState {
    pub fn new() -> Self { Self::default() }

    pub fn reset(&mut self) {
        self.pos = 0;
        self.in_double_quote = false;
        self.in_single_quote = false;
        self.interp_depth = 0;
        self.in_line_comment = false;
        self.in_block_comment = false;
        self.buf.clear();
        self.prev_char = None;
        self.escaped = false;
        self.pending_op = None;
        self.pending_slash = None;
    }

    /// Take a pending slash state, returning true if there was one pending.
    pub fn take_pending_slash(&mut self) -> bool {
        self.pending_slash.take().is_some()
    }

    pub fn feed(&mut self, ch: char) -> Vec<Token> {
        self.pos += 1;
        let mut out = Vec::new();

        // Drain pending slash first: decide if '/' is comment start or division
        if self.pending_slash.is_some() {
            self.pending_slash = None;
            if ch == '/' {
                // Line comment: discard the pending '/' and enter comment mode
                self.in_line_comment = true;
                self.prev_char = None;
                return out;
            } else if ch == '*' {
                // Block comment: discard the pending '/' and enter comment mode
                self.in_block_comment = true;
                self.prev_char = None;
                return out;
            } else {
                // Division operator: emit the pending Slash token, then process current char
                out.push(Token::Slash);
                // Fall through to process `ch` below (don't return)
            }
        }

        // Drain pending operator (lookahead from previous char)
        if let Some(pending) = self.pending_op.take() {
            let consumed = self.resolve_pending(pending, ch, &mut out);
            if consumed {
                return out;
            }
        }

        // Line comment: consume until newline
        if self.in_line_comment {
            if ch == '\n' {
                self.in_line_comment = false;
                self.prev_char = Some(ch);
            }
            return out;
        }

        // Block comment: look for */
        if self.in_block_comment {
            if self.prev_char == Some('*') && ch == '/' {
                self.in_block_comment = false;
            }
            self.prev_char = Some(ch);
            return out;
        }

        // Inside string literal
        if self.in_double_quote || self.in_single_quote {
            self.feed_string_char(ch, &mut out);
            return out;
        }

        // Special: detect #{ interpolation start BEFORE flushing buffer
        if ch == '{' && self.buf.ends_with('#') {
            let prefix_len = self.buf.len() - 1;
            if prefix_len > 0 {
                let prefix = self.buf[..prefix_len].to_string();
                self.buf.clear();
                self.flush_raw(&mut out, prefix);
            } else {
                self.buf.clear();
            }
            out.push(Token::InterpolationStart);
            self.interp_depth += 1;
            self.prev_char = Some('{');
            return out;
        }

        // Does char accumulate with current buffer?
        if accumulates(ch, &self.buf) {
            self.buf.push(ch);
            self.prev_char = Some(ch);
            return out;
        }

        // Char does NOT accumulate — flush buffer first
        self.flush_buf(&mut out);

        // Process non-accumulating char
        match ch {
            c if c.is_whitespace() => {
                // skip whitespace
            }
            '\'' => {
                self.in_single_quote = true;
                self.escaped = false;
            }
            '"' => {
                self.in_double_quote = true;
                self.escaped = false;
            }
            '/' => {
                // Defer slash to next char: divide vs // line comment vs /* block comment
                self.pending_slash = Some(PendingSlash::Slash);
                self.prev_char = Some(ch);
                return out;
            }
            '*' => out.push(Token::Star),
            '{' => out.push(Token::LBrace),
            '}' => {
                if self.interp_depth > 0 {
                    self.interp_depth -= 1;
                    out.push(Token::InterpolationEnd);
                    // The '}' is the interpolation terminator — do NOT also emit RBrace
                } else {
                    out.push(Token::RBrace);
                }
            }
            '(' => out.push(Token::LParen),
            ')' => out.push(Token::RParen),
            '[' => out.push(Token::LBracket),
            ']' => out.push(Token::RBracket),
            ':' => out.push(Token::Colon),
            ';' => out.push(Token::Semicolon),
            ',' => out.push(Token::Comma),
            '.' => {
                self.buf.push('.');
                self.prev_char = Some(ch);
                return out;
            }
            '&' => out.push(Token::Ampersand),
            '+' => out.push(Token::Plus),
            '-' => out.push(Token::Minus),
            '%' => out.push(Token::Percent),
            '=' => {
                self.pending_op = Some(PendingOp::Eq);
                self.prev_char = Some(ch);
                return out;
            }
            '!' => {
                self.pending_op = Some(PendingOp::Ne);
                self.prev_char = Some(ch);
                return out;
            }
            '<' => {
                self.pending_op = Some(PendingOp::Lt);
                self.prev_char = Some(ch);
                return out;
            }
            '>' => {
                self.pending_op = Some(PendingOp::Gt);
                self.prev_char = Some(ch);
                return out;
            }
            '?' => out.push(Token::Question),
            '#' => {
                self.buf.push('#');
                self.prev_char = Some(ch);
                return out;
            }
            '@' => {
                self.buf.push('@');
                self.prev_char = Some(ch);
                return out;
            }
            '$' => out.push(Token::Dollar),
            _ => {}
        }

        self.prev_char = Some(ch);
        out
    }

    /// Resolve a pending operator. Returns true if the current char was consumed.
    fn resolve_pending(&mut self, pending: PendingOp, ch: char, out: &mut Vec<Token>) -> bool {
        match pending {
            PendingOp::Eq => {
                if ch == '=' {
                    out.push(Token::Eq);
                    self.prev_char = Some('=');
                    true
                } else {
                    self.buf.push('=');
                    self.prev_char = Some('=');
                    false
                }
            }
            PendingOp::Ne => {
                if ch == '=' {
                    out.push(Token::Ne);
                    self.prev_char = Some('=');
                    true
                } else {
                    out.push(Token::Bang);
                    self.prev_char = Some(ch);
                    false
                }
            }
            PendingOp::Lt => {
                if ch == '=' {
                    out.push(Token::Le);
                    self.prev_char = Some('=');
                    true
                } else {
                    out.push(Token::Lt);
                    self.prev_char = Some(ch);
                    false
                }
            }
            PendingOp::Gt => {
                if ch == '=' {
                    out.push(Token::Ge);
                    self.prev_char = Some('=');
                    true
                } else {
                    out.push(Token::Gt);
                    self.prev_char = Some(ch);
                    false
                }
            }
        }
    }

    fn feed_string_char(&mut self, ch: char, out: &mut Vec<Token>) {
        if self.escaped {
            self.buf.push(ch);
            self.escaped = false;
            self.prev_char = Some(ch);
            return;
        }
        if ch == '\\' {
            self.escaped = true;
            self.prev_char = Some(ch);
            return;
        }
        let quote = if self.in_double_quote { '"' } else { '\'' };
        if ch == quote {
            let content = self.buf.clone();
            self.buf.clear();
            if self.in_double_quote {
                self.in_double_quote = false;
            } else {
                self.in_single_quote = false;
            }
            out.push(Token::Str(content));
        } else {
            self.buf.push(ch);
            self.prev_char = Some(ch);
        }
    }

    pub fn flush_buf_checked(&mut self, out: &mut Vec<Token>) {
        if let Some(pending) = self.pending_op.take() {
            match pending {
                PendingOp::Eq => self.buf.push('='),
                PendingOp::Ne => { out.push(Token::Bang); }
                PendingOp::Lt => out.push(Token::Lt),
                PendingOp::Gt => out.push(Token::Gt),
            }
        }
        self.flush_buf(out);
    }

    fn flush_buf(&mut self, out: &mut Vec<Token>) {
        if self.buf.is_empty() { return; }
        let raw = self.buf.clone();
        self.buf.clear();

        // Lone '-' is a minus operator, not an identifier.
        // Vendor-prefixed identifiers like -webkit- never reach here as "-",
        // because their leading letter arrives while buf == "-" and accumulates.
        if raw == "-" {
            out.push(Token::Minus);
            return;
        }

        if raw.starts_with('@') {
            match raw.as_str() {
                "@media" => out.push(Token::AtMedia),
                "@supports" => out.push(Token::AtSupports),
                "@if" => out.push(Token::AtIf),
                "@else" => out.push(Token::AtElse),
                "@for" => out.push(Token::AtFor),
                "@each" => out.push(Token::AtEach),
                "@while" => out.push(Token::AtWhile),
                "@mixin" => out.push(Token::AtMixin),
                "@include" => out.push(Token::AtInclude),
                "@function" => out.push(Token::AtFunction),
                "@return" => out.push(Token::AtReturn),
                "@use" => out.push(Token::AtUse),
                "@forward" => out.push(Token::AtForward),
                "@import" => out.push(Token::AtImport),
                "@extend" => out.push(Token::AtExtend),
                "@at-root" => out.push(Token::AtAtRoot),
                "@content" => out.push(Token::AtContent),
                "@warn" => out.push(Token::AtWarn),
                "@debug" => out.push(Token::AtDebug),
                "@error" => out.push(Token::AtError),
                "@charset" => out.push(Token::AtCharset),
                "@namespace" => out.push(Token::AtNamespace),
                "@keyframes" => out.push(Token::AtKeyframes),
                "@font-face" => out.push(Token::AtFontFace),
                "@page" => out.push(Token::AtPage),
                "@custom-media" => out.push(Token::AtCustomMedia),
                "@custom-selector" => out.push(Token::AtCustomSelector),
                other => out.push(Token::IdentAt(other.into())),
            }
        } else if raw.starts_with('#') && raw.len() > 1 {
            let color_raw = &raw[1..];
            out.push(Token::HashId(color_raw.into()));
        } else if let Some((num, unit)) = parse_number_with_unit(&raw) {
            out.push(Token::Number(num, Some(unit)));
        } else if let Ok(n) = raw.parse::<f64>() {
            out.push(Token::Number(n, None));
        } else {
            // Recognize logical keywords
            match raw.as_str() {
                "and" => out.push(Token::And),
                "or" => out.push(Token::Or),
                _ => out.push(Token::Ident(raw)),
            }
        }
    }

    /// Flush a raw string (already extracted from buf) without touching self.buf.
    fn flush_raw(&mut self, out: &mut Vec<Token>, raw: String) {
        if raw == "-" {
            out.push(Token::Minus);
            return;
        }
        if raw.starts_with('@') {
            match raw.as_str() {
                "@media" => out.push(Token::AtMedia),
                "@import" => out.push(Token::AtImport),
                "@if" => out.push(Token::AtIf),
                "@else" => out.push(Token::AtElse),
                "@for" => out.push(Token::AtFor),
                "@each" => out.push(Token::AtEach),
                "@mixin" => out.push(Token::AtMixin),
                "@include" => out.push(Token::AtInclude),
                "@function" => out.push(Token::AtFunction),
                "@return" => out.push(Token::AtReturn),
                "@use" => out.push(Token::AtUse),
                "@forward" => out.push(Token::AtForward),
                "@extend" => out.push(Token::AtExtend),
                "@at-root" => out.push(Token::AtAtRoot),
                "@content" => out.push(Token::AtContent),
                "@warn" => out.push(Token::AtWarn),
                "@debug" => out.push(Token::AtDebug),
                "@error" => out.push(Token::AtError),
                "@charset" => out.push(Token::AtCharset),
                "@keyframes" => out.push(Token::AtKeyframes),
                "@font-face" => out.push(Token::AtFontFace),
                "@page" => out.push(Token::AtPage),
                "@supports" => out.push(Token::AtSupports),
                "@while" => out.push(Token::AtWhile),
                "@namespace" => out.push(Token::AtNamespace),
                "@custom-media" => out.push(Token::AtCustomMedia),
                "@custom-selector" => out.push(Token::AtCustomSelector),
                other => out.push(Token::IdentAt(other.into())),
            }
        } else if raw.starts_with('#') && raw.len() > 1 {
            let color_raw = &raw[1..];
            out.push(Token::HashId(color_raw.into()));
        } else if raw == "." {
            out.push(Token::Dot);
        } else if let Some((num, unit)) = parse_number_with_unit(&raw) {
            out.push(Token::Number(num, Some(unit)));
        } else if let Ok(n) = raw.parse::<f64>() {
            out.push(Token::Number(n, None));
        } else {
            match raw.as_str() {
                "and" => out.push(Token::And),
                "or" => out.push(Token::Or),
                _ => out.push(Token::Ident(raw)),
            }
        }
    }

    pub fn in_string(&self) -> bool {
        self.in_double_quote || self.in_single_quote
    }
}

/// Returns true if `ch` should be accumulated into the current buffer.
fn accumulates(ch: char, buf: &str) -> bool {
    // Digit always accumulates
    if ch.is_ascii_digit() { return true; }
    // Letters accumulate (could be identifier or unit suffix)
    if ch.is_ascii_alphabetic() { return true; }
    // Underscore and hyphen accumulate (identifier or flag)
    if ch == '_' || ch == '-' { return true; }
    // '.' accumulates only if buf doesn't have one AND buf starts with digit or is empty
    if ch == '.' {
        if buf.contains('.') { return false; }
        return buf.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true);
    }
    // '%' accumulates only when buf starts with digit or '-' followed by digit
    // (percent unit like 100% or -50%)
    if ch == '%' {
        let mut chars = buf.chars();
        let first = chars.next();
        let is_percent_number = match first {
            None => false,
            Some('-') | Some('+') => chars.next().map(|c| c.is_ascii_digit()).unwrap_or(false),
            Some(c) => c.is_ascii_digit(),
        };
        return is_percent_number;
    }
    // '#' starts/continues a hex color
    if ch == '#' { return true; }
    // '@' accumulates only if buf already starts with @ (building @-rule name)
    if ch == '@' { return buf.starts_with('@'); }
    false
}

/// Parse a string like "16px", "1.5em", "100%", "2.5s" into (number, unit).
/// Returns None if the string is purely numeric or doesn't contain a valid split.
fn parse_number_with_unit(raw: &str) -> Option<(f64, String)> {
    if raw.starts_with('#') || raw.starts_with('@') {
        return None;
    }
    let digits_end = raw.chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .count();
    if digits_end == 0 || digits_end >= raw.len() {
        return None;
    }
    let num_part = &raw[..digits_end];
    let unit_part = &raw[digits_end..];
    if unit_part.is_empty() {
        return None;
    }
    let n = num_part.parse::<f64>().ok()?;
    Some((n, unit_part.to_string()))
}
