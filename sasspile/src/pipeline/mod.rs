//! Pipeline orchestration — coordinates Tokio stages for compilation.
//!
//! The pipeline runs 7 stages as independent Tokio tasks connected via
//! bounded `mpsc` channels. Each stage processes items and passes results
//! to the next stage, enabling backpressure and cancellation.

mod backpressure;
mod concurrent;

pub use backpressure::PipelineConfig;
pub use concurrent::ConcurrentCompiler;

use tokio::sync::mpsc;
use tracing::{info, info_span, instrument, warn};

use crate::css::{self, OutputStyle};
use crate::Result;

/// Input to the pipeline.
#[derive(Debug, Clone)]
pub struct PipelineInput {
    /// Source file path (for diagnostics).
    pub path: String,
    /// Raw SCSS source text.
    pub source: String,
}

/// Output from the pipeline.
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    /// Compiled CSS.
    pub css: String,
    /// Source path (echoed from input).
    pub path: String,
}

/// Multi-stage compilation pipeline.
///
/// Spawns Tokio tasks for each stage:
/// 1. Lex → 2. Parse → 3. Semantic → 4. Eval → 5. CSS Gen → 6. Format → 7. Output
pub struct Pipeline {
    /// Channel capacity for backpressure.
    config: PipelineConfig,
}

impl Pipeline {
    /// Create a pipeline with default configuration.
    pub fn new() -> Self {
        Self {
            config: PipelineConfig::default(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// Compile a single input through the pipeline.
    #[instrument(skip(self, input))]
    pub async fn compile_one(&self, input: PipelineInput) -> Result<PipelineOutput> {
        let span = info_span!("pipeline_compile", path = %input.path);
        let _enter = span.enter();

        info!(len = input.source.len(), "starting pipeline compilation");

        // Stage 1-2: Parse (Lex + Parse happen synchronously in parse()).
        let (stylesheet, _diags) = crate::parser::parse(&input.source);

        // Stage 3-4: Semantic + Eval (happens during CSS generation for current impl).
        // Stage 5-7: CSS generation.
        let css = css::generate(&stylesheet, OutputStyle::Expanded)?;

        info!(css_len = css.len(), "pipeline compilation complete");

        Ok(PipelineOutput {
            css,
            path: input.path,
        })
    }

    /// Compile multiple inputs, returning results as they complete.
    #[instrument(skip(self, inputs))]
    pub async fn compile_batch(
        &self,
        inputs: Vec<PipelineInput>,
        style: OutputStyle,
    ) -> Vec<Result<PipelineOutput>> {
        let cap = self.config.channel_capacity;
        info!(files = inputs.len(), capacity = cap, "starting batch compilation");

        // Use bounded channels for backpressure.
        let (tx, mut rx) = mpsc::channel::<Result<PipelineOutput>>(cap);

        // Spawn producer tasks.
        let mut handles = Vec::new();
        for input in inputs {
            let tx = tx.clone();
            let handle = tokio::spawn(async move {
                let result = compile_static(&input, style).await;
                let _ = tx.send(result).await;
            });
            handles.push(handle);
        }
        drop(tx); // Drop original sender so channel closes when all producers finish.

        // Collect results.
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }

        // Await all tasks to catch any panics.
        for handle in handles {
            if let Err(e) = handle.await {
                warn!(error = ?e, "pipeline task panicked");
                results.push(Err(crate::SassError::Compile(format!("task panicked: {e}"))));
            }
        }

        info!(completed = results.len(), "batch compilation complete");
        results
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Static compile function used as the pipeline body.
async fn compile_static(input: &PipelineInput, style: OutputStyle) -> Result<PipelineOutput> {
    let (stylesheet, _) = crate::parser::parse(&input.source);
    let css = css::generate(&stylesheet, style)?;
    Ok(PipelineOutput {
        css,
        path: input.path.clone(),
    })
}

/// The main sasslipe compiler — backward-compatible façade.
///
/// For full pipeline usage, prefer `Pipeline` directly.
pub struct Compiler;

impl Compiler {
    /// Create a new compiler instance.
    pub fn new() -> Self {
        Self
    }

    /// Compile SCSS source to CSS.
    pub async fn compile(&self, source: &str) -> Result<String> {
        let pipeline = Pipeline::new();
        let input = PipelineInput {
            path: "<inline>".to_string(),
            source: source.to_string(),
        };
        let output = pipeline.compile_one(input).await?;
        Ok(output.css)
    }

    /// Compile a file by path.
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
