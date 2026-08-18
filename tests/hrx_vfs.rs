//! HRX in-memory VFS — resolves modules from parsed HRX file contents.
//!
//! Enables multi-file spec tests by providing a `ModuleResolver` backed by
//! a `HashMap<String, String>` of file path → content extracted from HRX.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use sasspile::ast::{Expr, Stmt};
use sasspile::error::{SassError, SourcePos};
use sasspile::resolver::{ModuleResolver, ResolvedModule};

/// In-memory file system populated from an HRX archive.
#[derive(Debug, Clone)]
pub struct HrxVfs {
    /// Normalized path → file content.
    pub files: HashMap<String, String>,
    /// The entry-point file path (e.g. `"input.scss"` or `"subdir/input.scss"`).
    pub input_path: String,
}

impl HrxVfs {
    /// Look up a file by path.
    pub fn get(&self, path: &str) -> Option<&str> {
        self.files.get(path).map(|s| s.as_str())
    }

    /// Get the input SCSS content.
    pub fn input(&self) -> &str {
        self.files
            .get(&self.input_path)
            .map(|s| s.as_str())
            .unwrap_or("")
    }
}

/// Build an `HrxVfs` from HRX-parsed files for a given test-case directory.
///
/// `files` is the raw `(path, content)` list from `hrx_parser::parse_hrx`.
/// `dir` is the test-case's directory prefix (e.g. `""` for root, `"subdir"` for nested).
pub fn build_vfs(files: &[(String, String)], dir: &str) -> HrxVfs {
    let mut vfs = HashMap::new();
    let input_path = if dir.is_empty() {
        "input.scss".to_string()
    } else {
        format!("{}/input.scss", dir)
    };

    for (path, content) in files {
        // Only include files that belong to this test case's directory
        let belongs = if dir.is_empty() {
            // Root: include files with no '/' or whose parent is ""
            !path.contains('/')
                || path
                    .rsplit('/')
                    .nth(1)
                    .map(|p| p.is_empty())
                    .unwrap_or(false)
        } else {
            // Nested: include files under `dir/`
            path.starts_with(&format!("{}/", dir))
        };
        if belongs {
            // Strip the dir prefix so paths are relative to the test case
            let normalized = if dir.is_empty() {
                path.clone()
            } else {
                path.strip_prefix(&format!("{}/", dir))
                    .unwrap_or(path)
                    .to_string()
            };
            vfs.insert(normalized, content.clone());
        }
    }

    HrxVfs {
        files: vfs,
        input_path: if dir.is_empty() {
            "input.scss".to_string()
        } else {
            "input.scss".to_string()
        },
    }
}

/// A `ModuleResolver` backed by `HrxVfs` — resolves `@use`/`@import` from memory.
pub struct VfsResolver {
    /// The in-memory file system.
    pub vfs: HrxVfs,
    /// AST cache keyed by normalized path.
    ast_cache: HashMap<String, Vec<Stmt>>,
    /// Paths currently being loaded (circular reference detection).
    loading: HashSet<String>,
}

impl VfsResolver {
    pub fn new(vfs: HrxVfs) -> Self {
        Self {
            vfs,
            ast_cache: HashMap::new(),
            loading: HashSet::new(),
        }
    }

    /// Generate candidate paths for a module URL, following Sass resolution rules.
    fn candidate_paths(url: &str) -> Vec<String> {
        // Strip leading ./ or ../
        let rel = url.trim_start_matches("./").trim_start_matches("../");

        // Build underscored variant (only last path component gets `_`)
        let underscored = {
            if let Some(idx) = rel.rfind('/') {
                let (dir, file) = rel.split_at(idx + 1);
                format!("{}_{}", dir, file)
            } else {
                format!("_{}", rel)
            }
        };

        vec![
            rel.to_string(),
            format!("{}.scss", rel),
            format!("{}.css", rel),
            format!("{}.scss", underscored),
            format!("{}.css", underscored),
        ]
    }
}

impl ModuleResolver for VfsResolver {
    fn resolve(&mut self, url: &str, _base_dir: &Path) -> Result<ResolvedModule, SassError> {
        let span = tracing::info_span!(
            "vfs_resolve",
            stage = "test",
            url = %url,
            resolved_path = tracing::field::Empty,
            is_css = tracing::field::Empty,
        );
        let _enter = span.enter();

        let candidates = Self::candidate_paths(url);

        // Find the first candidate that exists in VFS
        let resolved = candidates
            .iter()
            .find(|c| self.vfs.files.contains_key(*c));

        let path = match resolved {
            Some(p) => p.clone(),
            None => {
                tracing::debug!(
                    stage = "test",
                    module = "vfs_resolver",
                    url = %url,
                    candidates = ?candidates,
                    "module not found in VFS"
                );
                return Err(SassError::eval(
                    format!("Cannot find module in VFS: {}", url),
                    SourcePos::default(),
                ));
            }
        };

        let is_css = path.ends_with(".css");

        // Circular reference detection
        if self.loading.contains(&path) {
            tracing::warn!(
                stage = "test",
                module = "vfs_resolver",
                path = %path,
                "circular @use/@import detected"
            );
            return Err(SassError::eval(
                format!("Circular @use/@import detected: {}", path),
                SourcePos::default(),
            ));
        }

        // AST cache check
        if let Some(cached) = self.ast_cache.get(&path) {
            tracing::debug!(
                stage = "test",
                module = "vfs_resolver",
                path = %path,
                "module loaded from AST cache"
            );
            let raw = if is_css {
                self.vfs.get(&path).map(|s| s.to_string())
            } else {
                None
            };
            return Ok(ResolvedModule {
                ast: cached.clone(),
                is_css,
                raw_content: raw,
                source_path: PathBuf::from(&path),
            });
        }

        // Mark as loading
        self.loading.insert(path.clone());

        let content = self
            .vfs
            .get(&path)
            .ok_or_else(|| {
                SassError::eval(
                    format!("Module not found in VFS: {}", path),
                    SourcePos::default(),
                )
            })?
            .to_string();

        if is_css {
            tracing::debug!(
                stage = "test",
                module = "vfs_resolver",
                path = %path,
                is_css = true,
                "CSS module resolved from VFS"
            );
            self.loading.remove(&path);
            return Ok(ResolvedModule {
                ast: Vec::new(),
                is_css: true,
                raw_content: Some(content),
                source_path: PathBuf::from(&path),
            });
        }

        // SCSS — tokenize + parse
        let tokens = sasspile::tokenize(&content, &path).map_err(|e| {
            SassError::parse(
                format!("{}: {}", path, e),
                SourcePos { file: path.clone(), line: 0, column: 0 },
            )
        })?;
        let ast = sasspile::parse(tokens).map_err(|e| {
            SassError::parse(
                format!("{}: {}", path, e),
                SourcePos { file: path.clone(), line: 0, column: 0 },
            )
        })?;

        tracing::debug!(
            stage = "test",
            module = "vfs_resolver",
            path = %path,
            is_css = false,
            stmt_count = ast.len(),
            "SCSS module resolved from VFS"
        );

        self.ast_cache.insert(path.clone(), ast.clone());
        self.loading.remove(&path);

        Ok(ResolvedModule {
            ast,
            is_css: false,
            raw_content: None,
            source_path: PathBuf::from(&path),
        })
    }

    fn parse_expr(&mut self, source: &str) -> Result<Expr, SassError> {
        let span = tracing::debug_span!(
            "vfs_parse_expr",
            stage = "test",
            module = "vfs_resolver",
            expr = %source,
        );
        let _enter = span.enter();

        let expr = sasspile::parse_expression(source)?;
        Ok(expr)
    }
}
