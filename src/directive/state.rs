//! 统一编译状态 — 整个管线唯一的状态载体
//!
//! 设计:
//!   - 单一结构体 CompileState 存所有编译期状态
//!   - scan_map 闭包接收 &mut CompileState + 输入 token
//!   - 三阶段 pass 在单一闭包内按收集深度切换
//!   - 状态由 rxrust scan_map 框架线程化, 零 clone / 零 Arc<Mutex>

use std::collections::HashMap;
use tracing::info_span;

// ─── Mixin 定义 ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct MixinDef {
    pub params: Vec<(String, Option<String>)>,
    pub body: Vec<String>,
}

// ─── 模块系统 ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct Module {
    pub variables: HashMap<String, String>,
}

impl Module {
    pub fn parse_from_content(content: &str) -> Self {
        let mut module = Self::default();
        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("$") {
                if let Some((name, value)) = rest.split_once(":") {
                    let name = format!("${}", name.trim());
                    let value = value
                        .trim()
                        .trim_end_matches("!default")
                        .trim_end()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    module.variables.insert(name, value);
                }
            }
        }
        module
    }
}

// ─── 作用域 ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub variables: HashMap<String, String>,
    pub mixins: HashMap<String, MixinDef>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }
}

// ─── 当前收集阶段（scan_map 内部的三模式切换） ─────────────────────────────

#[derive(Clone, Debug, Default)]
pub enum Phase {
    #[default]
    Struct,   // 收集 @mixin def / @for/@each/@if body
    Expand,   // 展开 @include / 变量替换
    Resolve,  // 最终求解
}

// ─── 控制流收集状态 ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq)]
pub enum Collecting {
    #[default]
    None,

    /// @mixin 定义收集
    MixinDef {
        params: Vec<(String, Option<String>)>,
    },

    /// @for 循环收集
    For {
        var_name: String,
        values: Vec<String>,
        body: Vec<String>,
    },

    /// @each 循环收集
    Each {
        var_name: String,
        items: Vec<String>,
        body: Vec<String>,
    },

    /// @if 分支收集
    If {
        branch_taken: bool,
        body: Vec<String>,
    },
}

// ─── 统一编译状态 ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct CompileState {
    /// 当前作用域
    pub scope: Scope,
    /// @use 模块
    pub modules: HashMap<String, Module>,
    /// @forward 表
    pub forwarded: HashMap<String, Module>,
    /// 宏展开缓冲
    pub expand_buffer: Vec<String>,
    /// 嵌套深度
    pub nesting_depth: usize,
    /// 当前收集的控制流
    pub collecting: Collecting,
    /// 正在定义的 mixin 名称
    pub current_mixin_name: Option<String>,
    /// 当前管线阶段 (struct → expand → resolve)
    pub phase: Phase,
    /// 父选择器栈 (嵌套规则上下文, 索引 0 = 最外层)
    pub selector_stack: Vec<String>,
}

impl CompileState {
    pub fn new() -> Self {
        Self::default()
    }

    // ── 查询接口 ──────────────────────────────────────────────────────

    pub fn resolve_variable(&self, name: &str) -> Option<&str> {
        self.scope.variables.get(name).map(|v| v.as_str())
    }

    pub fn resolve_mixin(&self, name: &str) -> Option<&MixinDef> {
        self.scope.mixins.get(name)
    }

    // ── 写入接口 ──────────────────────────────────────────────────────

    pub fn set_variable(&mut self, name: String, value: String) {
        self.scope.variables.insert(name, value);
    }

    pub fn define_mixin(&mut self, name: String, def: MixinDef) {
        self.scope.mixins.insert(name, def);
    }

    pub fn load_module(&mut self, path: String, module: Module) {
        self.modules.insert(path, module);
    }

    /// 阶段推进到下一 pass
    pub fn advance_phase(&mut self) {
        self.phase = match self.phase {
            Phase::Struct => Phase::Expand,
            Phase::Expand => Phase::Resolve,
            Phase::Resolve => Phase::Resolve,
        };
    }
}
