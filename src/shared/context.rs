//! CompilerContext — 编译器的全局上下文
//!
//! 5.4: CompilerContext MUST 通过 scan(CompilerCtx::new(), apply) 在管道内传播
//! 5.5: module_cache MUST 使用 MutRc<HashMap<PathBuf, EvaluatedModule>>

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::ast::Node;

/// Mixin 定义 — 存储在 CompilerContext.mixins 中
#[derive(Debug, Clone)]
pub struct MixinDef {
    /// 参数名列表 (如 vec!["$color", "$size"])
    pub params: Vec<String>,
    /// mixin 体 (AST 节点列表)
    pub body: Vec<Node>,
}

/// 评估后的模块 — 每个 @use / @import 对应一个
#[derive(Debug, Clone)]
pub struct EvaluatedModule {
    /// 模块路径
    pub path: PathBuf,
    /// 模块产出的顶层节点
    pub nodes: Vec<Node>,
    /// 模块导出的变量
    pub variables: HashMap<String, String>,
    /// 模块导出的 mixins
    pub mixins: HashMap<String, Vec<Node>>,
}

impl EvaluatedModule {
    pub fn empty() -> Self {
        Self {
            path: PathBuf::new(),
            nodes: Vec::new(),
            variables: HashMap::new(),
            mixins: HashMap::new(),
        }
    }

    pub fn with_nodes(path: PathBuf, nodes: Vec<Node>) -> Self {
        Self {
            path,
            nodes,
            variables: HashMap::new(),
            mixins: HashMap::new(),
        }
    }
}

/// 编译器上下文 — 在 scan 闭包间传播
#[derive(Debug, Clone)]
pub struct CompilerContext {
    /// 已加载模块的缓存 (MutRc pattern for Local Observable scopes)
    pub module_cache: Rc<RefCell<HashMap<PathBuf, EvaluatedModule>>>,
    /// 全局变量 (Bootstrap-style !global)
    pub global_variables: Rc<RefCell<HashMap<String, String>>>,
    /// 当前文件路径栈 (循环引用检测)
    pub path_stack: Vec<PathBuf>,
    /// mixin 注册表
    pub mixins: Rc<RefCell<HashMap<String, MixinDef>>>,
}

impl CompilerContext {
    pub fn new() -> Self {
        Self {
            module_cache: Rc::new(RefCell::new(HashMap::new())),
            global_variables: Rc::new(RefCell::new(HashMap::new())),
            path_stack: Vec::new(),
            mixins: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// 尝试从缓存获取模块
    pub fn try_get_module(&self, path: &PathBuf) -> Option<EvaluatedModule> {
        self.module_cache.borrow().get(path).cloned()
    }

    /// 插入模块到缓存，返回插入后的共享 Rc
    pub fn insert_module(&self, key: PathBuf, module: EvaluatedModule) {
        self.module_cache.borrow_mut().insert(key, module);
    }

    /// 检查模块是否已在缓存中 (用于 @use 缓存命中)
    pub fn is_module_cached(&self, path: &PathBuf) -> bool {
        self.module_cache.borrow().contains_key(path)
    }

    /// 在 subscribe 时生成 Shared 上下文 — 多订阅者共享同一模块缓存
    ///
    /// 5.6: MUST 通过 publish(Local::subject()).ref_count() 实现多订阅者模块共享
    pub fn shared(&self) -> Rc<RefCell<HashMap<PathBuf, EvaluatedModule>>> {
        self.module_cache.clone()
    }
}

impl Default for CompilerContext {
    fn default() -> Self {
        Self::new()
    }
}
