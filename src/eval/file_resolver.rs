//! 文件路径解析——@use/@import/@forward 的文件查找。
//!
//! 优先级：partial > non-partial，.scss > .sass > .css，
//! import-only > index

use std::path::{Path, PathBuf};

/// 解析 @import/@use/@forward 的 url 为文件路径。
///
/// 优先在当前文件目录查找，然后回退到 load paths。
pub fn resolve_file(
    base: Option<&PathBuf>,
    url: &str,
    load_paths: &[PathBuf],
) -> Option<PathBuf> {
    let base_dir = base
        .as_ref()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // 先尝试相对于当前文件目录
    if let Some(path) = try_resolve_dir(&base_dir, url) {
        return Some(path);
    }
    // 回退到 load paths
    for lp in load_paths {
        if let Some(path) = try_resolve_dir(lp, url) {
            return Some(path);
        }
    }
    None
}

/// 在指定目录下尝试解析 url 对应的文件。
fn try_resolve_dir(dir: &Path, url: &str) -> Option<PathBuf> {
    let url_path = Path::new(url);
    let parent = url_path.parent().unwrap_or(Path::new(""));
    let filename = url_path
        .file_stem()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| url.to_string());

    // 候选列表——按优先级排序
    let candidates = [
        dir.join(parent).join(format!("_{filename}.scss")),
        dir.join(parent).join(format!("{filename}.scss")),
        dir.join(parent).join(format!("_{filename}.sass")),
        dir.join(parent).join(format!("{filename}.sass")),
        dir.join(parent).join(format!("_{filename}.css")),
        dir.join(parent).join(format!("{filename}.css")),
        dir.join(parent).join(format!("_{filename}.import.scss")),
        dir.join(parent).join(format!("{filename}.import.scss")),
        dir.join(parent).join(format!("_{filename}.import.sass")),
        dir.join(parent).join(format!("{filename}.import.sass")),
        // index 文件
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
