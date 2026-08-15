//! Compilation pipeline orchestration.

use tracing::{info, instrument};

use crate::Result;

/// The main sasslipe compiler.
///
/// Each compilation stage runs as a separate Tokio task, connected via
/// `mpsc` channels for data flow and `watch` channels for reactive updates.
pub struct Compiler {
    // TODO: add channel senders/receivers for each stage
}

impl Compiler {
    /// Create a new compiler instance.
    #[instrument]
    pub fn new() -> Self {
        info!("creating sasslipe compiler");
        Self {}
    }

    /// Compile SCSS source to CSS.
    #[instrument(skip(self, source))]
    pub async fn compile(&self, source: &str) -> Result<String> {
        info!(len = source.len(), "starting compilation");
        // TODO: plug in pipeline stages
        Ok(String::new())
    }

    /// Compile a file by path.
    #[instrument(skip(self))]
    pub async fn compile_file(&self, path: &str) -> Result<String> {
        let source = tokio::fs::read_to_string(path)
            .await
            .map_err(crate::SassError::Io)?;
        self.compile(&source).await
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Re-exports ─────────────────────────────────────────────────────

pub use Compiler as SassCompiler;
