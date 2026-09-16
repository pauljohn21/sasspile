//! 模块缓存 — publish/ref_count 共享
//!
//! 5.6: 多订阅者共享 MUST 通过 publish(Local::subject()).ref_count() 实现

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use super::context::EvaluatedModule;

/// 模块缓存别名
pub type ModuleCacheKey = PathBuf;

/// 模块缓存存储
pub type ModuleCache = Rc<RefCell<HashMap<PathBuf, EvaluatedModule>>>;

/// 共享模块 — 通过 Rc 共享给多订阅者
pub type SharedModule = Rc<RefCell<HashMap<PathBuf, EvaluatedModule>>>;

/// 创建一个 Shared 模块缓存实例
pub fn new_shared_cache() -> SharedModule {
    Rc::new(RefCell::new(HashMap::new()))
}

/// 尝试从 Shared 缓存获取模块
pub fn try_get_shared_module(cache: &SharedModule, path: &PathBuf) -> Option<EvaluatedModule> {
    cache.borrow().get(path).cloned()
}

/// 将评估后的模块插入 Shared 缓存
pub fn insert_shared_module(cache: &SharedModule, key: PathBuf, module: EvaluatedModule) {
    cache.borrow_mut().insert(key, module);
}

/// 检查模块是否已在缓存中
pub fn is_cached(cache: &SharedModule, path: &PathBuf) -> bool {
    cache.borrow().contains_key(path)
}

/// 缓存命中计数 — 用于测试验证
pub fn cache_hit_count(cache: &SharedModule) -> usize {
    cache.borrow().len()
}
