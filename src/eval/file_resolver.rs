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
    /// `is_import=true` 时 @import 优先使用 import-only 文件
    pub(crate) fn resolve_file(
        base: Option<&PathBuf>,
        url: &str,
        load_paths: &[PathBuf],
    ) -> Option<PathBuf> {
        Self::resolve_file_inner(base, url, load_paths, false)
    }

    /// 解析 @import 专用——优先 import-only 文件。
    pub(crate) fn resolve_file_import(
        base: Option<&PathBuf>,
        url: &str,
        load_paths: &[PathBuf],
    ) -> Option<PathBuf> {
        Self::resolve_file_inner(base, url, load_paths, true)
    }

    fn resolve_file_inner(
        base: Option<&PathBuf>,
        url: &str,
        load_paths: &[PathBuf],
        is_import: bool,
    ) -> Option<PathBuf> {
        let span = crate::__tracing::debug_span!("resolve_file", url = url, is_import);
        let _enter = span.enter();
        let base_dir = base
            .as_ref()
            .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));
        // 先尝试相对于当前文件目录解析
        match Self::try_resolve_dir(&base_dir, url, is_import) {
            Some(path) => return Some(path),
            None => {}
        }
        // 回退到 load paths
        load_paths.iter().find_map(|lp| Self::try_resolve_dir(lp, url, is_import))
    }

    /// 在指定目录下尝试解析 url 对应的文件。
    fn try_resolve_dir(dir: &Path, url: &str, is_import: bool) -> Option<PathBuf> {
        let url_path = std::path::Path::new(url);
        let parent = url_path.parent().unwrap_or(std::path::Path::new(""));
        let filename = url_path
            .file_stem()
            .map_or_else(|| url.to_string(), |f| f.to_string_lossy().to_string());
        // 规范化 parent 路径（处理 .. 等组件）
        let parent_normalized = normalize_path(&dir.join(parent));
        let url_normalized = normalize_path(&dir.join(url));
        // @import 时优先 import-only 文件；@use/@forward 时跳过 import-only 文件
        let import_only_pairs = match is_import {
            true => vec![
                parent_normalized.join(format!("_{filename}.import.scss")),
                parent_normalized.join(format!("{filename}.import.scss")),
                parent_normalized.join(format!("_{filename}.import.sass")),
                parent_normalized.join(format!("{filename}.import.sass")),
            ],
            false => vec![],
        };
        let mut candidates = import_only_pairs;
        candidates.extend([
            parent_normalized.join(format!("_{filename}.scss")),
            parent_normalized.join(format!("{filename}.scss")),
            parent_normalized.join(format!("_{filename}.sass")),
            parent_normalized.join(format!("{filename}.sass")),
            parent_normalized.join(format!("_{filename}.css")),
            parent_normalized.join(format!("{filename}.css")),
            url_normalized.join("_index.scss"),
            url_normalized.join("index.scss"),
            url_normalized.join("_index.sass"),
            url_normalized.join("index.sass"),
        ]);
        candidates.into_iter().find(|c| c.exists())
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
            .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));
        std::iter::once(&base_dir)
            .chain(load_paths.iter())
            .try_for_each(|dir| {
                let url_path = std::path::Path::new(url);
                let parent = url_path.parent().unwrap_or(std::path::Path::new(""));
                let filename = url_path
                    .file_stem()
                    .map_or_else(|| url.to_string(), |f| f.to_string_lossy().to_string());

                let has_explicit_ext = url_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e == "scss" || e == "sass" || e == "css");

                let conflicts = match has_explicit_ext {
                    true => Self::check_explicit_ext_conflicts(dir, parent, &filename),
                    false => Self::check_no_ext_conflicts(dir, parent, &filename, url),
                };

                match conflicts.is_empty() {
                    true => Ok(()),
                    false => {
                        let mut all_files: Vec<String> = conflicts
                            .iter()
                            .flat_map(|c| c.iter())
                            .map(|f| {
                                let s = f.display().to_string();
                                s.strip_prefix("./").unwrap_or(&s).to_string()
                            })
                            .collect::<std::collections::HashSet<_>>()
                            .into_iter()
                            .collect();
                        all_files.sort();
                        Err(SassError::Eval(format!(
                            "It's not clear which file to import. Found:\n  {}",
                            all_files.join("\n  ")
                        )))
                    }
                }
            })
    }

    /// url 带了明确扩展名：只检测 partial vs non-partial（同扩展名）
    fn check_explicit_ext_conflicts(
        dir: &Path,
        parent: &Path,
        filename: &str,
    ) -> Vec<Vec<PathBuf>> {
        ["scss", "sass", "css"].iter().flat_map(|ext| {
            let partial = dir.join(parent).join(format!("_{filename}.{ext}"));
            let non_partial = dir.join(parent).join(format!("{filename}.{ext}"));
            let partial_io = dir.join(parent).join(format!("_{filename}.import.{ext}"));
            let non_partial_io = dir.join(parent).join(format!("{filename}.import.{ext}"));
            [
                (partial.exists() && non_partial.exists()).then(|| vec![partial, non_partial]),
                (partial_io.exists() && non_partial_io.exists()).then(|| vec![partial_io, non_partial_io]),
            ]
        }).flatten().collect()
    }

    /// url 未带扩展名：检测所有冲突类型
    fn check_no_ext_conflicts(
        dir: &Path,
        parent: &Path,
        filename: &str,
        url: &str,
    ) -> Vec<Vec<PathBuf>> {
        // partial vs non-partial（同扩展名）
        let partial_conflicts: Vec<Vec<PathBuf>> = ["scss", "sass", "css"]
            .iter()
            .filter_map(|ext| {
                let partial = dir.join(parent).join(format!("_{filename}.{ext}"));
                let non_partial = dir.join(parent).join(format!("{filename}.{ext}"));
                (partial.exists() && non_partial.exists()).then(|| vec![partial, non_partial])
            })
            .collect();
        // 同 partial 状态下 scss vs sass 冲突
        let ext_conflicts: Vec<Vec<PathBuf>> = [true, false]
            .iter()
            .filter_map(|is_partial| {
                let prefix = match *is_partial { true => "_", false => "" };
                let scss = dir.join(parent).join(format!("{prefix}{filename}.scss"));
                let sass = dir.join(parent).join(format!("{prefix}{filename}.sass"));
                (scss.exists() && sass.exists()).then(|| vec![scss, sass])
            })
            .collect();
        // import-only: scss vs sass 冲突
        let import_conflicts: Vec<Vec<PathBuf>> = [true, false]
            .iter()
            .filter_map(|is_partial| {
                let prefix = match *is_partial { true => "_", false => "" };
                let scss_io = dir
                    .join(parent)
                    .join(format!("{prefix}{filename}.import.scss"));
                let sass_io = dir
                    .join(parent)
                    .join(format!("{prefix}{filename}.import.sass"));
                (scss_io.exists() && sass_io.exists()).then(|| vec![scss_io, sass_io])
            })
            .collect();
        // index: partial vs non-partial 冲突
        let index_dir = dir.join(url);
        let index_conflicts: Vec<Vec<PathBuf>> = ["scss", "sass"]
            .iter()
            .filter_map(|ext| {
                let partial_idx = index_dir.join(format!("_index.{ext}"));
                let non_partial_idx = index_dir.join(format!("index.{ext}"));
                (partial_idx.exists() && non_partial_idx.exists())
                    .then(|| vec![partial_idx, non_partial_idx])
            })
            .collect();
        partial_conflicts
            .into_iter()
            .chain(ext_conflicts)
            .chain(import_conflicts)
            .chain(index_conflicts)
            .collect()
    }
}

/// 规范化路径——处理 `..` 和 `.` 组件，不要求路径存在。
fn normalize_path(path: &Path) -> PathBuf {
    path.components().fold(Vec::new(), |mut acc, c| {
        match c {
            std::path::Component::ParentDir => {
                match acc.last().is_some_and(|last| {
                    !matches!(last, std::path::Component::ParentDir)
                        && !matches!(last, std::path::Component::RootDir)
                }) {
                    true => { acc.pop(); }
                    false => { acc.push(c); }
                }
            }
            std::path::Component::CurDir => {}
            other => acc.push(other),
        }
        acc
    }).iter().collect()
}
