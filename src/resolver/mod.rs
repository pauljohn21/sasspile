//! Module resolver — decouples file loading/tokenize/parse from the eval layer.
//!
//! The `ModuleResolver` trait abstracts the process of resolving a module URL
//! (from `@use` or `@import`) into a parsed AST.  The default `FileResolver`
//! implementation handles filesystem-based resolution with module caching and
//! circular-reference detection.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{Expr, Stmt};
use crate::error::{SassError, SourcePos};

/// A resolved module — the AST plus metadata.
pub struct ResolvedModule {
    /// Parsed AST statements.
    pub ast: Vec<Stmt>,
    /// Whether the source was a `.css` file (treated as raw CSS).
    pub is_css: bool,
    /// Raw file content (only set for `.css` files).
    pub raw_content: Option<String>,
    /// The resolved filesystem path of the module.
    pub source_path: PathBuf,
}

/// Trait for resolving module URLs into parsed AST.
///
/// This abstraction breaks the eval → parser circular dependency by
/// allowing the eval layer to delegate file loading and parsing to
/// an external resolver.
pub trait ModuleResolver {
    /// Resolve `url` relative to `base_dir` into a parsed module.
    fn resolve(&mut self, url: &str, base_dir: &Path) -> Result<ResolvedModule, SassError>;

    /// Parse an expression string (used for interpolation contexts).
    ///
    /// This breaks the eval → parser dependency for interpolation evaluation.
    fn parse_expr(&mut self, source: &str) -> Result<Expr, SassError>;
}

/// Default filesystem-based resolver with module caching and circular-ref detection.
pub struct FileResolver {
    /// Cache of already-parsed module ASTs, keyed by resolved file path.
    cache: HashMap<PathBuf, Vec<Stmt>>,
    /// Set of paths currently being resolved (for circular reference detection).
    loading: HashSet<PathBuf>,
}

impl FileResolver {
    /// Create a new empty `FileResolver`.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            loading: HashSet::new(),
        }
    }
}

impl Default for FileResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleResolver for FileResolver {
    fn resolve(&mut self, url: &str, base_dir: &Path) -> Result<ResolvedModule, SassError> {
        let span = tracing::info_span!(
            "module_resolve",
            stage = "eval",
            url = %url,
            resolved_path = tracing::field::Empty,
            is_css = tracing::field::Empty,
        );
        let _enter = span.enter();

        // Strip leading ./ or ../
        let rel = url.trim_start_matches("./").trim_start_matches("../");

        // Build candidates — underscore prefix goes on the last path component.
        // e.g. "mixins/banner" → "mixins/_banner.scss", not "_mixins/banner.scss"
        let underscored = {
            let mut s = String::new();
            let parts: Vec<&str> = rel.rsplitn(2, '/').collect();
            if parts.len() == 2 {
                s.push_str(parts[1]);
                s.push('/');
                s.push('_');
                s.push_str(parts[0]);
            } else {
                s.push('_');
                s.push_str(parts[0]);
            }
            s
        };

        let candidates = [
            base_dir.join(rel),
            base_dir.join(format!("{}.scss", rel)),
            base_dir.join(format!("{}.css", rel)),
            base_dir.join(format!("{}.scss", underscored)),
            base_dir.join(format!("{}.css", underscored)),
        ];

        let file_path = candidates.iter().find(|p| p.is_file()).cloned();

        let path = match file_path {
            Some(p) => p,
            None => {
                tracing::debug!(
                    stage = "eval",
                    module = "resolver",
                    url = %url,
                    "file not found for module"
                );
                return Err(SassError::eval(
                    format!("Cannot find module: {}", url),
                    SourcePos::default(),
                ));
            }
        };

        let is_css = path.extension().and_then(|e| e.to_str()) == Some("css");

        // Circular reference detection
        if self.loading.contains(&path) {
            tracing::warn!(
                stage = "eval",
                module = "resolver",
                path = %path.display(),
                "circular @use/@import detected, skipping"
            );
            return Err(SassError::eval(
                format!("Circular @use/@import detected: {}", path.display()),
                SourcePos::default(),
            ));
        }

        // Check cache — return cloned AST if already parsed
        if let Some(cached) = self.cache.get(&path) {
            tracing::debug!(
                stage = "eval",
                module = "resolver",
                path = %path.display(),
                "module loaded from cache"
            );
            let content = if is_css {
                Some(std::fs::read_to_string(&path).unwrap_or_default())
            } else {
                None
            };
            return Ok(ResolvedModule {
                ast: cached.clone(),
                is_css,
                raw_content: content,
                source_path: path,
            });
        }

        // Mark as loading to detect circular references
        self.loading.insert(path.clone());

        let content = std::fs::read_to_string(&path).map_err(|e| {
            SassError::parse(
                format!("Failed to read {}: {}", path.display(), e),
                SourcePos { file: path.display().to_string(), line: 0, column: 0 },
            )
        })?;

        let file_name = path.display().to_string();

        if is_css {
            // CSS files are treated as raw content — no tokenize/parse needed
            tracing::debug!(
                stage = "eval",
                module = "resolver",
                path = %path.display(),
                is_css = true,
                "CSS module resolved"
            );
            self.loading.remove(&path);
            return Ok(ResolvedModule {
                ast: Vec::new(),
                is_css: true,
                raw_content: Some(content),
                source_path: path,
            });
        }

        // SCSS — tokenize + parse
        let tokens = crate::lexer::tokenize(&content, &file_name).map_err(|e| {
            SassError::parse(
                format!("{}: {}", path.display(), e),
                SourcePos { file: file_name.clone(), line: 0, column: 0 },
            )
        })?;
        let ast = crate::parser::parse(tokens).map_err(|e| {
            SassError::parse(
                format!("{}: {}", path.display(), e),
                SourcePos { file: path.display().to_string(), line: 0, column: 0 },
            )
        })?;

        tracing::debug!(
            stage = "eval",
            module = "resolver",
            path = %path.display(),
            is_css = false,
            stmt_count = ast.len(),
            "SCSS module resolved"
        );

        // Cache the parsed AST
        self.cache.insert(path.clone(), ast.clone());
        self.loading.remove(&path);

        Ok(ResolvedModule {
            ast,
            is_css: false,
            raw_content: None,
            source_path: path,
        })
    }

    fn parse_expr(&mut self, source: &str) -> Result<Expr, SassError> {
        let span = tracing::debug_span!(
            "parse_expr",
            stage = "eval",
            module = "resolver",
            expr = %source,
        );
        let _enter = span.enter();

        let tokens = crate::lexer::tokenize(source, "interpolation")?;
        let mut parser = crate::parser::Parser::new(tokens);
        let mut expr_parser = crate::parser::expr::ExprParser::new(&mut parser);
        let expr = expr_parser.parse_expr()?;
        Ok(expr)
    }
}
