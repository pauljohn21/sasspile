//! Shared Context 模块 — CompilerContext + 模块缓存双机制
//!
//! rxrust Shared 模式 (Section 5):
//! - CompilerContext: scan(CompilerCtx::new(), apply) 传播
//! - module_cache: MutRc<HashMap<PathBuf, EvaluatedModule>> (Local scope)
//! - 多订阅者共享: publish(Local::subject()).ref_count()

pub mod context;
pub mod module_cache;

pub use context::CompilerContext;
pub use module_cache::{ModuleCacheKey, ModuleCache, SharedModule};

use crate::ast::Node;

/// Mixin 定义
#[derive(Debug, Clone)]
pub struct MixinDef {
    pub params: Vec<String>,
    pub body: Vec<Node>,
}
