use std::path::PathBuf;
use crate::pipeline::{from_path, from_string};
use crate::serialize::Options;
use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerConfig {
    SingleThread,
    ThreadPool(usize),
}

pub struct CompileBuilder {
    pub(crate) syntax: InputSyntax,
    pub(crate) include_paths: Vec<PathBuf>,
    pub(crate) serialize_style: OutputStyle,
    pub(crate) scheduler_config: Option<SchedulerConfig>,
}

impl Default for CompileBuilder {
    fn default() -> Self {
        Self {
            syntax: InputSyntax::Scss,
            include_paths: Vec::new(),
            serialize_style: OutputStyle::Expanded,
            scheduler_config: None,
        }
    }
}

impl CompileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn syntax(mut self, syntax: InputSyntax) -> Self {
        self.syntax = syntax;
        self
    }

    pub fn scss(mut self) -> Self {
        self.syntax = InputSyntax::Scss;
        self
    }

    pub fn css(mut self) -> Self {
        self.syntax = InputSyntax::Css;
        self
    }

    pub fn sass(mut self) -> Self {
        self.syntax = InputSyntax::Sass;
        self
    }

    pub fn include_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.include_paths.push(path.into());
        self
    }

    pub fn include_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.include_paths = paths;
        self
    }

    pub fn output_style(mut self, style: OutputStyle) -> Self {
        self.serialize_style = style;
        self
    }

    pub fn expanded(mut self) -> Self {
        self.serialize_style = OutputStyle::Expanded;
        self
    }

    pub fn compressed(mut self) -> Self {
        self.serialize_style = OutputStyle::Compressed;
        self
    }

    pub fn nested(mut self) -> Self {
        self.serialize_style = OutputStyle::Nested;
        self
    }

    pub fn scheduler(mut self, config: SchedulerConfig) -> Self {
        self.scheduler_config = Some(config);
        self
    }

    pub fn build(self) -> CompileSession {
        CompileSession {
            syntax: self.syntax,
            include_paths: self.include_paths,
            options: Options {
                style: self.serialize_style,
                suppress_charset: false,
            },
            scheduler_config: self.scheduler_config,
        }
    }

    pub fn compile_string(&self, source: &str) -> Result<String, CompileError> {
        let options = Options {
            style: self.serialize_style,
            suppress_charset: false,
        };
        crate::pipeline::from_string_with_paths(source, &options, self.include_paths.clone())
    }

    pub fn compile_file(&self, path: impl AsRef<std::path::Path>) -> Result<String, CompileError> {
        let options = Options {
            style: self.serialize_style,
            suppress_charset: false,
        };
        // Build include paths: file's parent dir first, then user-specified paths
        let mut include_paths = self.include_paths.clone();
        if let Some(parent) = path.as_ref().parent() {
            let parent_buf = parent.to_path_buf();
            if !include_paths.contains(&parent_buf) {
                include_paths.push(parent_buf);
            }
        }
        crate::pipeline::from_string_with_paths(
            &std::fs::read_to_string(path.as_ref()).map_err(|e| CompileError::Io(e.to_string()))?,
            &options,
            include_paths,
        )
    }
}

#[allow(dead_code)]
pub struct CompileSession {
    pub(crate) syntax: InputSyntax,
    pub(crate) include_paths: Vec<PathBuf>,
    pub(crate) options: Options,
    pub(crate) scheduler_config: Option<SchedulerConfig>,
}

impl CompileSession {
    pub fn compile(&self, source: &str) -> Result<String, CompileError> {
        from_string(source, &self.options)
    }

    pub fn compile_from_path(&self, path: impl AsRef<std::path::Path>) -> Result<String, CompileError> {
        from_path(path.as_ref(), &self.options)
    }
}

pub fn compile(source: &str) -> Result<String, CompileError> {
    let builder = CompileBuilder::new();
    builder.compile_string(source)
}

pub fn compile_with_options(source: &str, style: OutputStyle) -> Result<String, CompileError> {
    let builder = CompileBuilder::new().output_style(style);
    builder.compile_string(source)
}


