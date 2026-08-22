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
    pub name: Option<String>,  // named arg
    pub value: Value,
    pub spread: bool,  // ...
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
    Use { url: String, namespace: Option<String>, star: bool, config: Vec<(String, Value)> },
    Forward { url: String, show: Vec<String>, hide: Vec<String>, prefix: Option<String>, config: Vec<(String, Value)> },
    Import { url: String, modifier: Option<String> },

    // 其他 at-rules
    AtRoot { query: Option<String>, body: Vec<Node> },
    AtRule { name: String, params: String, body: Option<Vec<Node>> },
    Extend { selector: String, optional: bool },
    Warn(Value),
    Debug(Value),
    Error(Value),
}

/// AST = 顶层语句序列。
pub type Ast = Vec<Node>;
