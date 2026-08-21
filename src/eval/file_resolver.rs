//! 文件路径解析与歧义检测。
//!
//! 负责 SCSS/Sass 文件路径解析逻辑，包括：
//! - partial/non-partial 优先级
//! - 扩展名补全（.scss/.sass/.css）
//! - index 文件解析
//! - import-only 文件
//! - 文件歧义检测（多种文件冲突时报错）

use super::*;
use crate::error::{Result, SassError};
use std::path::{Path, PathBuf};

impl Evaluator {
    /// 解析 @import/@use/@forward 的 url 为文件路径。
    ///
    /// 优先级顺序：partial > non-partial，.scss > .sass > .css，import-only > index
    pub(crate) fn resolve_file(
        base: Option<&PathBuf>,
        url: &str,
        load_paths: &[PathBuf],
    ) -> Option<PathBuf> {
        let span = crate::__tracing::debug_span!("resolve_file", url = url);
        let _enter = span.enter();
        let base_dir = base
            .as_ref()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        // 先尝试相对于当前文件目录解析
        if let Some(path) = Self::try_resolve_dir(&base_dir, url) {
            return Some(path);
        }
        // 回退到 load paths
        for lp in load_paths {
            if let Some(path) = Self::try_resolve_dir(lp, url) {
                return Some(path);
            }
        }
        None
    }

    /// 在指定目录下尝试解析 url 对应的文件。
    fn try_resolve_dir(dir: &Path, url: &str) -> Option<PathBuf> {
        let url_path = std::path::Path::new(url);
        let parent = url_path.parent().unwrap_or(std::path::Path::new(""));
        let filename = url_path
            .file_stem()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| url.to_string());
        let candidates = [
            dir.join(parent).join(format!("_{filename}.scss")),
            dir.join(parent).join(format!("{filename}.scss")),
            dir.join(parent).join(format!("_{filename}.sass")),
            dir.join(parent).join(format!("{filename}.sass")),
            dir.join(parent).join(format!("_{filename}.css")),
            dir.join(parent).join(format!("{filename}.css")),
            dir.join(parent).join(format!("_{filename}.import.scss")),
            dir.join(parent).join(format!("{filename}.import.scss")),
            dir.join(url).join("_index.scss"),
            dir.join(url).join("index.scss"),
            dir.join(url).join("_index.sass"),
            dir.join(url).join("index.sass"),
        ];
        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }
        None
    }

    /// 检查文件解析歧义：多种文件冲突场景检测。
    ///
    /// Sass 规范要求以下情况报错：
    /// 1. partial vs non-partial（`_file.scss` 和 `file.scss` 同时存在）
    /// 2. extension 冲突（`file.scss` 和 `file.sass` 同时存在）
    /// 3. index 冲突（`dir/_index.scss` 和 `dir/index.scss` 同时存在）
    /// 4. import-only 冲突（`file.import.scss` 和 `file.import.sass`，或 `_file.import.scss` 和 `file.import.scss`）
    pub(crate) fn check_resolve_ambiguity(
        base: Option<&PathBuf>,
        url: &str,
        load_paths: &[PathBuf],
    ) -> Result<()> {
        let span = crate::__tracing::debug_span!("check_resolve_ambiguity", url = url);
        let _enter = span.enter();
        let base_dir = base
            .as_ref()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        for dir in std::iter::once(&base_dir).chain(load_paths.iter()) {
            let url_path = std::path::Path::new(url);
            let parent = url_path.parent().unwrap_or(std::path::Path::new(""));
            let filename = url_path
                .file_stem()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| url.to_string());

            let has_explicit_ext = url_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "scss" || e == "sass" || e == "css")
                .unwrap_or(false);

            let conflicts = if has_explicit_ext {
                Self::check_explicit_ext_conflicts(dir, parent, &filename)
            } else {
                Self::check_no_ext_conflicts(dir, parent, &filename, url)
            };

            if !conflicts.is_empty() {
                let mut all_files: Vec<String> = Vec::new();
                for c in &conflicts {
                    for f in c {
                        let s = f.display().to_string();
                        let s = s.strip_prefix("./").unwrap_or(&s).to_string();
                        if !all_files.contains(&s) {
                            all_files.push(s);
                        }
                    }
                }
                all_files.sort();
                return Err(SassError::Eval(format!(
                    "It's not clear which file to import. Found:\n  {}",
                    all_files.join("\n  ")
                )));
            }
        }
        Ok(())
    }

    /// url 带了明确扩展名：只检测 partial vs non-partial（同扩展名）
    fn check_explicit_ext_conflicts(
        dir: &Path,
        parent: &Path,
        filename: &str,
    ) -> Vec<Vec<PathBuf>> {
        let mut conflicts = Vec::new();
        for ext in &["scss", "sass", "css"] {
            let partial = dir.join(parent).join(format!("_{filename}.{ext}"));
            let non_partial = dir.join(parent).join(format!("{filename}.{ext}"));
            if partial.exists() && non_partial.exists() {
                conflicts.push(vec![partial, non_partial]);
            }
            let partial_io = dir.join(parent).join(format!("_{filename}.import.{ext}"));
            let non_partial_io = dir.join(parent).join(format!("{filename}.import.{ext}"));
            if partial_io.exists() && non_partial_io.exists() {
                conflicts.push(vec![partial_io, non_partial_io]);
            }
        }
        conflicts
    }

    /// url 未带扩展名：检测所有冲突类型
    fn check_no_ext_conflicts(
        dir: &Path,
        parent: &Path,
        filename: &str,
        url: &str,
    ) -> Vec<Vec<PathBuf>> {
        let mut conflicts = Vec::new();
        // partial vs non-partial（同扩展名）
        for ext in &["scss", "sass", "css"] {
            let partial = dir.join(parent).join(format!("_{filename}.{ext}"));
            let non_partial = dir.join(parent).join(format!("{filename}.{ext}"));
            if partial.exists() && non_partial.exists() {
                conflicts.push(vec![partial, non_partial]);
            }
        }
        // 同 partial 状态下 scss vs sass 冲突
        for is_partial in &[true, false] {
            let prefix = if *is_partial { "_" } else { "" };
            let scss = dir.join(parent).join(format!("{prefix}{filename}.scss"));
            let sass = dir.join(parent).join(format!("{prefix}{filename}.sass"));
            if scss.exists() && sass.exists() {
                conflicts.push(vec![scss, sass]);
            }
        }
        // import-only: scss vs sass 冲突
        for is_partial in &[true, false] {
            let prefix = if *is_partial { "_" } else { "" };
            let scss_io = dir.join(parent).join(format!("{prefix}{filename}.import.scss"));
            let sass_io = dir.join(parent).join(format!("{prefix}{filename}.import.sass"));
            if scss_io.exists() && sass_io.exists() {
                conflicts.push(vec![scss_io, sass_io]);
            }
        }
        // index: partial vs non-partial 冲突
        let index_dir = dir.join(url);
        for ext in &["scss", "sass"] {
            let partial_idx = index_dir.join(format!("_index.{ext}"));
            let non_partial_idx = index_dir.join(format!("index.{ext}"));
            if partial_idx.exists() && non_partial_idx.exists() {
                conflicts.push(vec![partial_idx, non_partial_idx]);
            }
        }
        conflicts
    }
}
