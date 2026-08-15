//! HRX parser implementation.
//!
//! Parses the HRX format into an [`Archive`](crate::models::Archive).
//!
//! # HRX Format Rules (as used by sass-spec)
//!
//! 1. Each entry starts with a boundary line: `<===> <path>`
//! 2. An empty `<===>` followed by `====...` (80 `=` chars) indicates a directory boundary
//! 3. Lines starting with `#` are comments (ignored during parsing)
//! 4. File contents continue until the next boundary marker or EOF
//! 5. Maximum line length is 1MB

use crate::error::{HrxError, Result};
use crate::models::{Archive, DirEntry, Entry, FileEntry, BOUNDARY_MARKER};
use tracing::{debug, info, trace, warn};

/// Parse an HRX archive from a string.
pub fn parse(input: &str) -> Result<Archive> {
    info!("parsing HRX input ({} bytes)", input.len());
    let parser = Parser::new(input);
    parser.parse()
}

/// Internal parser state.
struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line_num: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line_num: 0,
        }
    }

    fn parse(mut self) -> Result<Archive> {
        let mut entries: Vec<Entry> = Vec::new();
        let mut pending_dir: Option<DirEntry> = None;

        // Skip any leading comments and whitespace
        self.skip_comments_and_whitespace();

        while !self.at_end() {
            trace!(line = self.line_num, pos = self.pos, "parsing next entry");

            let line = match self.peek_line() {
                Some(line) => line,
                None => break,
            };

            // Check if it's a boundary marker
            if let Some(stripped) = line.strip_prefix(BOUNDARY_MARKER) {
                let boundary_content = stripped.trim();

                if boundary_content.is_empty() {
                    // Could be a directory boundary comment or empty path
                    self.consume_line(); // consume the <===> line

                    // Check if next line is the directory boundary marker
                    if let Some(next) = self.peek_line() {
                        if next.trim() == "================================================================================" {
                            // It's a directory boundary - flush pending dir
                            info!(line = self.line_num, "found directory boundary");
                            if let Some(dir) = pending_dir.take() {
                                entries.push(Entry::Dir(dir));
                            }
                            self.consume_line(); // consume the === line
                            self.skip_comments_and_whitespace();
                            continue;
                        } else if next.starts_with(BOUNDARY_MARKER) {
                            // Two <===> in a row - could be empty boundary + real entry
                            warn!(line = self.line_num, "empty boundary before entry");
                            self.skip_comments_and_whitespace();
                            continue;
                        } else if next.starts_with('#') {
                            // Comment line
                            self.consume_line();
                            continue;
                        }
                    }

                    warn!(line = self.line_num, "empty boundary without directory separator");
                    self.skip_comments_and_whitespace();
                    continue;
                }

                // It's a file entry: <===> path
                let path = boundary_content.to_string();
                debug!(line = self.line_num, path, "found file entry");
                self.consume_line(); // consume the boundary line

                // Read file contents until next boundary or EOF
                let contents = self.read_file_contents();

                let file = FileEntry::new(path.clone(), contents);

                // Determine if this belongs to a pending directory
                if let Some(ref mut dir) = pending_dir {
                    // Check if file is a child of this directory
                    let dir_prefix = format!("{}/", dir.path);
                    if path.starts_with(&dir_prefix) {
                        let subpath = &path[dir_prefix.len()..];
                        dir.children.push(Entry::File(FileEntry::new(subpath, file.contents)));
                        trace!(path, dir = dir.path, "added file to pending directory");
                    } else {
                        // Different path - flush pending dir and create new entry
                        let dir = pending_dir.take().unwrap();
                        entries.push(Entry::Dir(dir));
                        pending_dir = None;
                        entries.push(Entry::File(file));
                    }
                } else {
                    entries.push(Entry::File(file));
                }
            } else {
                // Non-boundary line at top level - this shouldn't happen per spec
                warn!(
                    line = self.line_num,
                    content = line.chars().take(50).collect::<String>(),
                    "unexpected non-boundary line, skipping"
                );
                self.consume_line();
            }
        }

        // Flush any remaining pending directory
        if let Some(dir) = pending_dir {
            entries.push(Entry::Dir(dir));
        }

        info!(
            file_count = entries.iter().filter(|e| e.is_file()).count(),
            dir_count = entries.iter().filter(|e| e.is_dir()).count(),
            "parsing complete"
        );

        Ok(Archive::from_entries(entries))
    }

    /// Reads the contents of a file until the next boundary marker or EOF.
    fn read_file_contents(&mut self) -> String {
        let start = self.pos;
        let mut last_content_end = self.pos;

        while !self.at_end() {
            let line = match self.peek_line() {
                Some(l) => l,
                None => break,
            };

            // Stop at boundary marker
            if line.starts_with(BOUNDARY_MARKER) {
                break;
            }

            // Track the end of content (excluding trailing blank lines)
            if !line.trim().is_empty() {
                last_content_end = self.pos + line.len();
            }

            self.consume_line();
        }

        // Return contents, trimming trailing newlines
        let contents = &self.input[start..last_content_end];
        let result = contents.trim_end().to_string();
        trace!(bytes = result.len(), "read file contents");
        result
    }

    /// Returns the current line without consuming it.
    fn peek_line(&self) -> Option<&'a str> {
        if self.pos >= self.input.len() {
            return None;
        }
        let remaining = &self.input[self.pos..];
        match remaining.find('\n') {
            Some(idx) => Some(&remaining[..idx]),
            None => Some(remaining),
        }
    }

    /// Consumes the current line (up to and including the newline).
    fn consume_line(&mut self) {
        let remaining = &self.input[self.pos..];
        match remaining.find('\n') {
            Some(idx) => {
                self.pos += idx + 1;
                self.line_num += 1;
            }
            None => {
                self.pos = self.input.len();
                self.line_num += 1;
            }
        }
    }

    /// Returns true if we've reached the end of input.
    fn at_end(&self) -> bool {
        self.pos >= self.input.len() || self.input[self.pos..].trim().is_empty()
    }

    /// Skips comment lines and blank lines.
    fn skip_comments_and_whitespace(&mut self) {
        while !self.at_end() {
            if let Some(line) = self.peek_line() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    self.consume_line();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}

/// Parse an HRX archive from a byte slice.
pub fn parse_bytes(input: &[u8]) -> Result<Archive> {
    let text = std::str::from_utf8(input).map_err(|e| {
        HrxError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid UTF-8: {}", e),
        ))
    })?;
    parse(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let input = "<===> input.scss\na { color: red }\n\n<===> output.css\na {\n  color: red;\n}\n";
        let archive = parse(input).unwrap();
        assert!(!archive.is_empty());
        assert_eq!(archive.len(), 2);
    }

    #[test]
    fn test_parse_with_dir_boundary() {
        let input = "\
<===> unbracketed/input.scss
a {b: is-bracketed(foo bar)}

<===> unbracketed/output.css
a {b: false}

<===>
================================================================================
<===> bracketed/input.scss
a {b: is-bracketed([foo bar])}

<===> bracketed/output.css
a {b: true}
";
        let archive = parse(input).unwrap();
        assert!(!archive.is_empty());
    }
}
