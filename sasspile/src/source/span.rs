//! Source span definitions.
//!
//! A `SourceSpan` represents a byte range within a source file,
//! used for error reporting and source maps.

use std::ops::Range;

/// Byte range in source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub start: u32,
    pub end: u32,
}

impl SourceSpan {
    /// Create a new span from start and end byte offsets.
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Length of the span in bytes.
    pub fn len(&self) -> usize {
        (self.end.saturating_sub(self.start)) as usize
    }

    /// Whether the span is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Check if the span contains a byte offset.
    pub fn contains(&self, offset: u32) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Merge with another span to cover both.
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<Range<usize>> for SourceSpan {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start as u32,
            end: range.end as u32,
        }
    }
}

impl From<SourceSpan> for Range<usize> {
    fn from(span: SourceSpan) -> Self {
        span.start as usize..span.end as usize
    }
}
