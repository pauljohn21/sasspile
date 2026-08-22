//! AST Node 定义。

use crate::eval::value::Value;

/// 变量标志位。
#[derive(Debug, Clone, Default)]
pub struct VarFlags {
    pub default: bool,
    pub global: bool,
    pub important: bool,
}

/// 函数参数。
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default: Option<Value>,
    pub rest: bool,  // ...
}

/// 函数调用参数。
#[derive(Debug, Clone)]
pub struct Arg {
    /// 命名参数名。
    pub name: Option<String>,
    /// 参数值表达式。
    pub value: Value,
    /// 是否展开剩余参数（`...`）。
    pub spread: bool,
}

/// 配置变量——`@use`/`@forward` 的 `with ($x: val)` 参数。
#[derive(Debug, Clone)]
pub struct ConfigVar {
    /// 变量名（不含 $）。
    pub name: String,
    /// 变量值表达式。
    pub value: Value,
    /// `!default` 标志。
    pub is_default: bool,
}

/// AST 节点。
#[derive(Debug, Clone)]
pub enum Node {
    // CSS 输出
    Rule { selector: String, body: Vec<Node> },
    Decl { property: String, value: Value, important: bool },
    Comment(String),

    // 变量
    Variable { name: String, value: Value, flags: VarFlags },

    // 控制流
    If { branches: Vec<(Value, Vec<Node>)>, else_body: Option<Vec<Node>> },
    For { var: String, from: Value, to: Value, inclusive: bool, body: Vec<Node> },
    Each { vars: Vec<String>, list: Value, body: Vec<Node> },
    While { cond: Value, body: Vec<Node> },

    // mixin / function
    MixinDef { name: String, params: Vec<Param>, body: Vec<Node> },
    FunctionDef { name: String, params: Vec<Param>, body: Vec<Node> },
    Include { name: String, args: Vec<Arg>, content: Option<Vec<Node>> },
    Content,
    Return(Value),

    // 模块系统
    Use { url: String, namespace: Option<String>, star: bool, config: Vec<ConfigVar> },
    Forward { url: String, show: Vec<String>, hide: Vec<String>, prefix: Option<String>, config: Vec<ConfigVar> },
    Import { url: String, modifier: String },

    // 其他 at-rules
    AtRoot { query: Option<String>, body: Vec<Node> },
    AtRule { name: String, params: Option<String>, body: Option<Vec<Node>> },
    Extend { selector: String, optional: bool },
    Warn(Value),
    Debug(Value),
    Error(Value),
}

/// AST = 顶层语句序列。
pub type Ast = Vec<Node>;
