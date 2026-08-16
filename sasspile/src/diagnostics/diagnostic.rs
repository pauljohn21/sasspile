//! Diagnostic types for error reporting.

use crate::source::SourceSpan;

use super::Level;

/// A structured diagnostic with location and notes.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity.
    pub level: Level,
    /// Error code (e.g., "E001").
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Source location.
    pub span: Option<SourceSpan>,
    /// Additional notes.
    pub notes: Vec<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: Level::Error,
            code: code.into(),
            message: message.into(),
            span: None,
            notes: Vec::new(),
        }
    }

    /// Create a new warning diagnostic.
    pub fn warn(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: Level::Warn,
            code: code.into(),
            message: message.into(),
            span: None,
            notes: Vec::new(),
        }
    }

    /// Create a new info diagnostic.
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: Level::Info,
            code: code.into(),
            message: message.into(),
            span: None,
            notes: Vec::new(),
        }
    }

    /// Attach a source span.
    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    /// Add a note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.level {
            Level::Error => "error",
            Level::Warn => "warning",
            Level::Info => "info",
        };
        write!(f, "[{}] {}: {}", self.code, prefix, self.message)?;
        for note in &self.notes {
            write!(f, "\n  note: {}", note)?;
        }
        Ok(())
    }
}

/// Collection of diagnostics produced during compilation.
///
/// Use [`Diagnostics::errors`] to iterate, [`Diagnostics::has_errors`] to check
/// for failures, and [`Diagnostics::counts`] for a summary tuple `(errors, warns, infos)`.
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    pub items: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Create an empty collection.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Push a diagnostic.
    pub fn push(&mut self, diag: Diagnostic) {
        self.items.push(diag);
    }

    /// Return true if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.level == Level::Error)
    }

    /// Return true if empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Return the length.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Return the count of diagnostics at each level.
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut errors = 0;
        let mut warns = 0;
        let mut infos = 0;
        for d in &self.items {
            match d.level {
                Level::Error => errors += 1,
                Level::Warn => warns += 1,
                Level::Info => infos += 1,
            }
        }
        (errors, warns, infos)
    }

    /// Return all diagnostic items.
    pub fn errors(&self) -> &[Diagnostic] {
        &self.items
    }
}
