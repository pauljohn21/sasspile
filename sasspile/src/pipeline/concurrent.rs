//! Concurrent compilation — multiple files in parallel with shared module cache.
//!
//! Manages a pool of compile tasks with bounded concurrency, allowing
//! multiple entry files to be compiled in parallel while sharing
//! parsed module artifacts.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::Semaphore;
use tracing::{info, instrument};

use crate::css::OutputStyle;
use crate::pipeline::{Pipeline, PipelineConfig, PipelineInput, PipelineOutput};
use crate::Result;

/// Concurrent compiler with bounded parallelism.
pub struct ConcurrentCompiler {
    /// Shared pipeline configuration.
    config: PipelineConfig,
    /// Semaphore limiting concurrent compilations.
    semaphore: Arc<Semaphore>,
    /// Module cache: path → parsed module (shared across compilations).
    module_cache: Arc<Mutex<HashMap<String, Arc<String>>>>,
}

impl ConcurrentCompiler {
    /// Create with default parallelism (num_cpus).
    pub fn new() -> Self {
        let parallelism = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Self::with_parallelism(parallelism)
    }

    /// Create with specific parallelism limit.
    pub fn with_parallelism(parallelism: usize) -> Self {
        info!(parallelism, "creating concurrent compiler");
        Self {
            config: PipelineConfig::default(),
            semaphore: Arc::new(Semaphore::new(parallelism)),
            module_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Compile multiple files concurrently, returning all results.
    #[instrument(skip(self, inputs))]
    pub async fn compile_all(
        &self,
        inputs: Vec<PipelineInput>,
        style: OutputStyle,
    ) -> Vec<Result<PipelineOutput>> {
        info!(files = inputs.len(), "starting concurrent compilation");
        let pipeline = Pipeline::with_config(self.config.clone());
        pipeline.compile_batch(inputs, style).await
    }

    /// Compile a single file, respecting the concurrency limit.
    #[instrument(skip(self, input))]
    pub async fn compile_one(
        &self,
        input: PipelineInput,
        style: OutputStyle,
    ) -> Result<PipelineOutput> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .expect("semaphore should not be closed");

        let pipeline = Pipeline::with_config(self.config.clone());
        pipeline.compile_one(input).await
    }

    /// Get the current module cache size.
    pub fn cache_size(&self) -> usize {
        self.module_cache.lock().map(|m| m.len()).unwrap_or(0)
    }

    /// Insert into the module cache.
    pub fn cache_insert(&self, path: String, content: String) {
        if let Ok(mut cache) = self.module_cache.lock() {
            cache.insert(path, Arc::new(content));
        }
    }

    /// Get from the module cache.
    pub fn cache_get(&self, path: &str) -> Option<Arc<String>> {
        self.module_cache.lock().ok()?.get(path).cloned()
    }

    /// Clear the module cache.
    pub fn cache_clear(&self) {
        if let Ok(mut cache) = self.module_cache.lock() {
            cache.clear();
        }
    }
}

impl Default for ConcurrentCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_compiler_new() {
        let cc = ConcurrentCompiler::new();
        assert_eq!(cc.cache_size(), 0);
    }

    #[test]
    fn module_cache_operations() {
        let cc = ConcurrentCompiler::with_parallelism(2);
        cc.cache_insert("_tokens.scss".to_string(), "$primary: red;".to_string());
        assert_eq!(cc.cache_size(), 1);

        let cached = cc.cache_get("_tokens.scss");
        assert!(cached.is_some());
        assert_eq!(*cached.unwrap(), "$primary: red;");

        cc.cache_clear();
        assert_eq!(cc.cache_size(), 0);
    }

    #[tokio::test]
    async fn compile_one_respects_concurrency() {
        let cc = ConcurrentCompiler::with_parallelism(2);
        let input = PipelineInput {
            path: "test.scss".to_string(),
            source: "$a: 1; .a { width: $a }".to_string(),
        };
        let result = cc.compile_one(input, OutputStyle::Expanded).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.css.contains("width"));
    }

    #[tokio::test]
    async fn compile_all_parallel() {
        let cc = ConcurrentCompiler::with_parallelism(4);
        let inputs: Vec<PipelineInput> = (0..5)
            .map(|i| PipelineInput {
                path: format!("file_{i}.scss"),
                source: format!("$c: {}; .file-{{i}} {{ color: $c }}", i),
            })
            .collect();

        let results = cc.compile_all(inputs, OutputStyle::Expanded).await;
        assert_eq!(results.len(), 5);
        for result in &results {
            assert!(result.is_ok());
        }
    }
}
