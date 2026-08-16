//! Source position definitions.

/// Position in source file (line and column, 1-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourcePosition {
    /// Line number, 1-based.
    pub line: u32,
    /// Column number, 1-based.
    pub column: u32,
    /// Byte offset from start of file.
    pub offset: u32,
}

impl SourcePosition {
    /// Create a new position.
    pub fn new(line: u32, column: u32, offset: u32) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

impl Default for SourcePosition {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
