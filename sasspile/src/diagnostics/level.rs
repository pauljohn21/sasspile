//! Diagnostic severity levels.

/// Severity of a diagnostic message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Informational (lowest severity).
    Info,
    /// Warning — potential issue.
    Warn,
    /// Error — compilation failure.
    Error,
}
