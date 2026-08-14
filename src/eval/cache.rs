//! 全局缓存——避免重复 lex/parse/eval。
//!
//! - AST 缓存：文件路径 → 解析后的 AST（Arc，因为 Ast 不含 Rc）
//! - ModuleExports 缓存：文件路径 → eval 后的导出（仅无 config 时）
//!
//! 相同路径的文件内容在单次进程运行中不变，可安全缓存。
//! 无 config 的 @use 永远从相同的空环境开始求值，结果可复用。
//!
//! 注意：ModuleExports 内部含 Rc，不能跨线程，故用 thread_local + RefCell。

use super::ModuleExports;
use crate::parse::ast::Ast;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::{LazyLock, Mutex};

/// 全局 AST 缓存：文件路径 → 解析后的 AST。
static AST_CACHE: LazyLock<Mutex<HashMap<PathBuf, Arc<Ast>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// 全局 ModuleExports 缓存（thread_local，因内部含 Rc）。
// 仅当 config 为空时缓存/查询（有 config 时每次结果不同）。
thread_local! {
    static MODULE_CACHE: RefCell<HashMap<PathBuf, Rc<ModuleExports>>> = RefCell::new(HashMap::new());
}

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

/// 获取缓存的 ModuleExports（如果存在）。
///
/// 仅当 config 为空时有效——无 config 的 @use 每次求值结果相同。
pub(crate) fn get_cached_module(path: &PathBuf) -> Option<Rc<ModuleExports>> {
    MODULE_CACHE.with(|cache| cache.borrow().get(path).cloned())
}

/// 插入 ModuleExports 到缓存（仅用于无 config 的场景）。
pub(crate) fn put_cached_module(path: PathBuf, exports: ModuleExports) -> Rc<ModuleExports> {
    let rc = Rc::new(exports);
    MODULE_CACHE.with(|cache| {
        cache.borrow_mut().insert(path, rc.clone());
    });
    rc
}

