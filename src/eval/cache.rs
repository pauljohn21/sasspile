//! 全局 AST 缓存——避免相同文件重复 lex + parse。
//!
//! 相同路径的文件内容在单次进程运行中不变，可安全缓存解析结果。
//! 缓存以 `Rc<Ast>` 共享，避免深拷贝。

use crate::parse::ast::Ast;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};

/// 全局 AST 缓存：文件路径 → 解析后的 AST。
///
/// 使用 `LazyLock` 延迟初始化，`Mutex` 保证线程安全。
static AST_CACHE: LazyLock<Mutex<HashMap<PathBuf, Arc<Ast>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 全局规范路径缓存：路径 → canonicalized 路径。
///
/// 避免对同一文件重复调用 `canonicalize()`（文件系统调用）。
static CANON_CACHE: LazyLock<Mutex<HashMap<PathBuf, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 获取缓存的 AST（如果存在）。
pub(crate) fn get_cached_ast(path: &PathBuf) -> Option<Arc<Ast>> {
    AST_CACHE.lock().ok()?.get(path).cloned()
}

/// 插入 AST 到缓存。
pub(crate) fn put_cached_ast(path: PathBuf, ast: Ast) -> Arc<Ast> {
    let rc = Arc::new(ast);
    if let Ok(mut cache) = AST_CACHE.lock() {
        cache.insert(path, rc.clone());
    }
    rc
}

/// 获取缓存的规范路径（如果存在）。
pub(crate) fn get_cached_canonical(path: &PathBuf) -> Option<PathBuf> {
    CANON_CACHE.lock().ok()?.get(path).cloned()
}

/// 获取或计算规范路径（带缓存）。
pub(crate) fn get_or_canonicalize(path: &PathBuf) -> PathBuf {
    if let Some(cached) = get_cached_canonical(path) {
        return cached;
    }
    let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
    if let Ok(mut cache) = CANON_CACHE.lock() {
        cache.insert(path.clone(), canonical.clone());
    }
    canonical
}

/// 清空缓存（主要是测试用——隔离不同测试的缓存状态）。
#[cfg(test)]
pub(crate) fn clear_cache() {
    if let Ok(mut cache) = AST_CACHE.lock() {
        cache.clear();
    }
    if let Ok(mut cache) = CANON_CACHE.lock() {
        cache.clear();
    }
}
