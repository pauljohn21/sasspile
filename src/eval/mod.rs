//! 求值器——纯函数式 try_fold + 不可变环境。

use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::*;
use crate::lex::Lexer;
use crate::lex::token::Token;
use tracing::{instrument, trace, warn};

use im::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// 模块导出——加载的文件模块的成员。
#[derive(Debug, Clone, Default)]
struct ModuleExports {
vars: HashMap<String, Value>,
mixins: HashMap<String, MixinDef>,
functions: HashMap<String, FunctionDef>,
#[allow(dead_code)]
css: Vec<CssNode>,
}

/// 不可变求值环境。
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// 变量绑定（扁平，用作用域前缀模拟）。
    vars: HashMap<String, Value>,
    /// mixin 定义。
    mixins: HashMap<String, MixinDef>,
    /// 用户函数定义。
    functions: HashMap<String, FunctionDef>,
    /// @content 内容块（Rc 共享，避免深拷贝）。
    content: Option<Rc<Vec<Node>>>,
    /// @content 的环境（Rc 共享，避免深拷贝）。
    content_env: Option<Rc<Env>>,
    /// 已加载的内建模块名集合。
    builtin_modules: Vec<String>,
    /// 命名空间模块（文件加载的模块）。
    namespaces: HashMap<String, Rc<ModuleExports>>,
    /// 当前文件路径（用于解析相对 @use/@import）。
    base_path: Option<PathBuf>,
    /// 递归深度计数器。
    depth: usize,
    /// @extend 收集的继承关系 (extender, target)——Rc 共享避免深拷贝。
    extends: Rc<Vec<(String, String)>>,
    /// 当前选择器上下文（进入规则体时设置）。
    current_selector: Option<String>,
}

/// mixin 定义存储。
#[derive(Debug, Clone)]
struct MixinDef {
    params: Vec<Param>,
    body: Vec<Node>,
}

/// 函数定义存储。
#[derive(Debug, Clone)]
struct FunctionDef {
    params: Vec<Param>,
    body: Vec<Node>,
}

impl Env {
    pub fn new_env() -> Self { Self::default() }
    /// 递增深度。
    pub fn incr_depth(&self) -> Self {
        let mut new = self.clone();
        new.depth += 1;
        new
    }
    pub fn bind(&self, name: String, value: Value) -> Self {
        let mut new = self.clone();
        new.vars.insert(name, value);
        new
    }
    pub fn lookup(&self, name: &str) -> Option<&Value> { self.vars.get(name) }
    pub fn has_var(&self, name: &str) -> bool { self.vars.contains_key(name) }
    pub fn define_mixin(&self, name: String, def: MixinDef) -> Self {
        let mut new = self.clone();
        new.mixins.insert(name, def);
        new
    }
    pub fn get_mixin(&self, name: &str) -> Option<&MixinDef> { self.mixins.get(name) }
    pub fn define_function(&self, name: String, def: FunctionDef) -> Self {
        let mut new = self.clone();
        new.functions.insert(name, def);
        new
    }
    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> { self.functions.get(name) }
    /// 设置 @content 内容块。
    pub fn set_content(&self, content: Vec<Node>, content_env: Env) -> Self {
        let mut new = self.clone();
        new.content = Some(Rc::new(content));
        new.content_env = Some(Rc::new(content_env));
        new
    }
    /// 获取 @content 内容块。
    pub fn get_content(&self) -> Option<(&[Node], &Env)> {
        self.content.as_ref().map(|c| c.as_slice()).zip(
            self.content_env.as_ref().map(|e| e.as_ref())
        )
    }
    /// 注册已加载内建模块。
    pub fn add_module(&self, name: String) -> Self {
        let mut new = self.clone();
        if !new.builtin_modules.contains(&name) {
            new.builtin_modules.push(name);
        }
        new
    }
    /// 检查内建模块是否已加载。
    pub fn has_module(&self, name: &str) -> bool { self.builtin_modules.iter().any(|m| m == name) }
    /// 添加命名空间模块。
    pub fn add_namespace(&self, ns: String, exports: ModuleExports) -> Self {
        let mut new = self.clone();
        new.namespaces.insert(ns, Rc::new(exports));
        new
    }
    /// 获取命名空间模块。
    pub fn get_namespace(&self, ns: &str) -> Option<&ModuleExports> {
        self.namespaces.get(ns).map(|rc| rc.as_ref())
    }
    /// 设置基础路径。
    pub fn with_base_path(&self, path: PathBuf) -> Self {
        let mut new = self.clone();
        new.base_path = Some(path);
        new
    }
    /// 添加 @extend 关系。
    pub fn add_extend(&self, extender: String, target: String) -> Self {
        let mut new = self.clone();
        let mut extends = (*self.extends).clone();
        extends.push((extender, target));
        new.extends = Rc::new(extends);
        new
    }
    /// 获取所有 @extend 关系。
    pub fn get_extends(&self) -> &[(String, String)] { &self.extends }
    /// 设置当前选择器。
    pub fn with_selector(&self, sel: String) -> Self {
        let mut new = self.clone();
        new.current_selector = Some(sel);
        new
    }
    /// 获取当前选择器。
    pub fn get_selector(&self) -> Option<&str> { self.current_selector.as_deref() }
}

/// 求值器。
pub struct Evaluator;

/// 最大递归深度——防止无限递归导致内存爆炸。
const MAX_DEPTH: usize = 200;

impl Evaluator {
    /// 求值 AST 入口。
    pub fn evaluate(ast: &Ast) -> Result<Vec<CssNode>> {
        let (mut css, final_env) = Self::eval_nodes(&ast.nodes, &Env::default())?;
        let extends = final_env.get_extends().to_vec();
        if !extends.is_empty() {
            Self::apply_extends(&mut css, &extends);
        }
        Ok(css)
    }

    /// 求值 AST 入口（带基础路径，支持文件加载）。
    pub fn evaluate_with_path(ast: &Ast, base_path: PathBuf) -> Result<Vec<CssNode>> {
        let env = Env::default().with_base_path(base_path);
        let (mut css, final_env) = Self::eval_nodes(&ast.nodes, &env)?;
        let extends = final_env.get_extends().to_vec();
        if !extends.is_empty() {
            Self::apply_extends(&mut css, &extends);
        }
        Ok(css)
    }

    /// 求值节点列表——try_fold。
    #[instrument(skip(nodes, env), fields(depth = env.depth, n = nodes.len()))]
    fn eval_nodes(nodes: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        if env.depth > MAX_DEPTH {
            warn!(depth = env.depth, "recursion limit exceeded");
            return Err(SassError::Eval("递归深度超过限制（可能是无限循环）".into()));
        }
        nodes.iter().try_fold((Vec::new(), env.clone()), |(mut css, env), node| {
            let (mut out, new_env) = Self::eval_node(node, &env)?;
            css.append(&mut out);
            Ok((css, new_env))
        })
    }

    /// 求值单个节点。
    #[instrument(skip(node, env), fields(depth = env.depth))]
    fn eval_node(node: &Node, env: &Env) -> Result<(Vec<CssNode>, Env)> {
        match node {
            Node::Rule { selector, body } => Self::eval_rule(selector, body, env),
            Node::Decl { property, value, important } => {
                let val = Self::eval_value(value, env)?;
                Ok((vec![CssNode::Declaration {
                    property: property.clone(),
                    value: val.to_string(),
                    important: *important,
                }], env.clone()))
            }
            Node::Variable { name, value, flags } => Self::eval_variable(name, value, flags, env),
            Node::Comment(text, silent) => {
                if *silent { Ok((vec![], env.clone())) }
                else { Ok((vec![CssNode::Comment(text.clone())], env.clone())) }
            }
            Node::If { branches, else_body } => Self::eval_if(branches, else_body, env),
            Node::For { var, from, to, inclusive, body } => Self::eval_for(var, from, to, *inclusive, body, env),
            Node::Each { vars, list, body } => Self::eval_each(vars, list, body, env),
            Node::While { cond, body } => Self::eval_while(cond, body, env),
            Node::MixinDef { name, params, body } => {
                let new_env = env.define_mixin(name.clone(), MixinDef { params: params.clone(), body: body.clone() });
                Ok((vec![], new_env))
            }
            Node::Include { name, args, content } => Self::eval_include(name, args, content, env),
            Node::Content => {
                if let Some((content_nodes, content_env)) = env.get_content() {
                    Self::eval_nodes(content_nodes, content_env)
                } else {
                    Ok((vec![], env.clone()))
                }
            }
            Node::FunctionDef { name, params, body } => {
                let new_env = env.define_function(name.clone(), FunctionDef { params: params.clone(), body: body.clone() });
                Ok((vec![], new_env))
            }
            Node::Return(_) => Ok((vec![], env.clone())), // @return 由函数调用处理
            Node::Use { url, namespace, star, config } => {
                // 内建模块 sass:math/string/list/map/color/meta/selector
                if url.starts_with("sass:") {
                    return Ok((vec![], env.add_module(url.clone())));
                }
                // 文件模块——解析路径并加载
                let base = env.base_path.as_ref();
                if let Some(path) = Self::resolve_file(base, url) {
                    let exports = Self::load_module(&path, config, env)?;
                    if *star {
                        let mut new_env = env.clone();
                        for (k, v) in &exports.vars { new_env = new_env.bind(k.clone(), v.clone()); }
                        for (k, v) in &exports.mixins { new_env = new_env.define_mixin(k.clone(), v.clone()); }
                        for (k, v) in &exports.functions { new_env = new_env.define_function(k.clone(), v.clone()); }
                        return Ok((vec![], new_env));
                    }
                    let ns = namespace.clone().unwrap_or_else(|| {
                        // 默认命名空间 = 文件名（不含扩展名和前缀 _）
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(url);
                        stem.trim_start_matches('_').to_string()
                    });
                    return Ok((vec![], env.add_namespace(ns, exports)));
                }
                // 找不到文件——静默跳过
                Ok((vec![], env.clone()))
            }
            Node::Forward { url, show: _, hide: _, prefix: _ } => {
                // @forward 'url' —— 转发模块成员到当前作用域（简化版：同 @import）
                let base = env.base_path.as_ref();
                if let Some(path) = Self::resolve_file(base, url) {
                    let exports = Self::load_module(&path, &[], env)?;
                    let mut new_env = env.clone();
                    for (k, v) in &exports.vars { new_env = new_env.bind(k.clone(), v.clone()); }
                    for (k, v) in &exports.mixins { new_env = new_env.define_mixin(k.clone(), v.clone()); }
                    for (k, v) in &exports.functions { new_env = new_env.define_function(k.clone(), v.clone()); }
                    return Ok((vec![], new_env));
                }
                Ok((vec![], env.clone()))
            }
Node::Import { url } => {
// @import 'url' —— 旧版内联：加载文件内容注入当前作用域
if url.starts_with("sass:") {
return Ok((vec![], env.add_module(url.clone())));
}
let base = env.base_path.as_ref();
if let Some(path) = Self::resolve_file(base, url) {
let exports = Self::load_module(&path, &[], env)?;
let mut new_env = env.clone();
for (k, v) in &exports.vars { new_env = new_env.bind(k.clone(), v.clone()); }
for (k, v) in &exports.mixins { new_env = new_env.define_mixin(k.clone(), v.clone()); }
for (k, v) in &exports.functions { new_env = new_env.define_function(k.clone(), v.clone()); }
return Ok((exports.css, new_env));
}
Ok((vec![], env.clone()))
}
            Node::Extend { selector, optional: _ } => {
                // @extend selector —— 收集继承关系
                if let Some(extender) = env.get_selector() {
                    let new_env = env.add_extend(extender.to_string(), selector.clone());
                    Ok((vec![], new_env))
                } else {
                    Ok((vec![], env.clone()))
                }
            }
            Node::AtRoot { query, body } => Self::eval_at_root(query, body, env),
            Node::AtRule { name, params, body } => Self::eval_at_rule(name, params, body, env),
            Node::Warn(_) | Node::Debug(_) => Ok((vec![], env.clone())),
            Node::Error(v) => {
                let msg = Self::eval_value(v, env)?;
                Err(SassError::Eval(msg.to_string()))
            }
        }
    }

    /// 求值规则——按顺序穿插输出声明组和嵌套规则。
    fn eval_rule(selector: &str, body: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        // 对选择器中的 #{...} 插值求值
        let selector = if selector.contains("#{") {
            Self::eval_interp_str(selector, env)
        } else {
            selector.to_string()
        };
        let (css, new_env) = Self::eval_nodes(body, &env.with_selector(selector.clone()))?;

        let mut result = Vec::new();
        let mut current_decls = Vec::new();
        let mut root_nodes = Vec::new();

        for node in css {
            match node {
                CssNode::Declaration { .. } => current_decls.push(node),
                CssNode::AtRoot(nodes) => root_nodes.extend(nodes),
                CssNode::Rule { selector: child_sel, declarations: child_decls, children: child_kids } => {
                    // 遇到嵌套规则——先刷新当前声明组
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    // 合并选择器并输出嵌套规则
                    let combined = Self::combine_selectors(&selector, &child_sel);
                    if !child_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: combined.clone(),
                            declarations: child_decls,
                            children: vec![],
                        });
                    }
                    // 递归展平子规则的子规则
                    for kid in child_kids {
                        if let CssNode::Rule { selector: kid_sel, declarations: kid_decls, .. } = kid {
                            let kid_combined = Self::combine_selectors(&combined, &kid_sel);
                            if !kid_decls.is_empty() {
                                result.push(CssNode::Rule {
                                    selector: kid_combined,
                                    declarations: kid_decls,
                                    children: vec![],
                                });
                            }
                        } else {
                            result.push(kid);
                        }
                    }
                }
                other => {
                    // 其他节点（AtRule 等）——先刷新当前声明组
                    if !current_decls.is_empty() {
                        result.push(CssNode::Rule {
                            selector: selector.clone(),
                            declarations: std::mem::take(&mut current_decls),
                            children: vec![],
                        });
                    }
                    result.push(other);
                }
            }
        }

        // 输出最后的声明组
        if !current_decls.is_empty() {
            result.push(CssNode::Rule {
                selector: selector.clone(),
                declarations: current_decls,
                children: vec![],
            });
        }

        // 如果没有任何输出，保留空规则
        if result.is_empty() && root_nodes.is_empty() {
            result.push(CssNode::Rule {
                selector,
                declarations: vec![],
                children: vec![],
            });
        }

        // 添加 @at-root 节点
        result.extend(root_nodes);
        Ok((result, new_env))
    }

    /// 组合选择器——处理 & 替换和逗号分隔选择器。
    fn combine_selectors(parent: &str, child: &str) -> String {
        let parents: Vec<&str> = parent.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let children: Vec<&str> = child.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let mut result = Vec::new();
        for p in &parents {
            for c in &children {
                if c.contains('&') {
                    result.push(c.replace('&', p));
                } else if p.is_empty() {
                    result.push(c.to_string());
                } else {
                    result.push(format!("{p} {c}"));
                }
            }
        }
        result.join(", ")
    }

    /// 求值变量声明。
    fn eval_variable(name: &str, value: &Value, flags: &VarFlags, env: &Env) -> Result<(Vec<CssNode>, Env)> {
        if flags.default && env.has_var(name) {
            return Ok((vec![], env.clone()));
        }
        let val = Self::eval_value(value, env)?;
        Ok((vec![], env.bind(name.to_string(), val)))
    }

    /// 求值值表达式。
    fn eval_value(value: &Value, env: &Env) -> Result<Value> {
        match value {
            Value::Number(..) | Value::Color(..) | Value::Bool(..) | Value::Null | Value::Calc(..) => Ok(value.clone()),
            Value::String(s, quoted) => {
                // 处理插值在字符串中
                if s.contains('#') && s.contains('{') {
                    Ok(Value::String(Self::eval_interp_str(s, env), *quoted))
                } else {
                    Ok(value.clone())
                }
            }
            Value::Variable(name) => {
                // 检查是否为命名空间变量 module.var
                if let Some(dot) = name.find('.') {
                    let ns = &name[..dot];
                    let var_name = &name[dot + 1..];
                    if let Some(module) = env.get_namespace(ns) {
                        if let Some(val) = module.vars.get(var_name) {
                            return Ok(val.clone());
                        }
                    }
                }
                env.lookup(name)
                    .cloned()
                    .ok_or_else(|| SassError::UndefinedVariable(name.clone()))
            }
            Value::List(elements, sep, bracketed) => {
                let evaluated: Vec<Value> = elements.iter()
                    .map(|e| Self::eval_value(e, env))
                    .collect::<Result<_>>()?;
                // 空格分隔列表可能需要进一步处理
                Ok(Value::List(evaluated, sep.clone(), *bracketed))
            }
            Value::Map(pairs) => {
                let evaluated: Vec<(Value, Value)> = pairs.iter()
                    .map(|(k, v)| Ok((Self::eval_value(k, env)?, Self::eval_value(v, env)?)))
                    .collect::<Result<_>>()?;
                Ok(Value::Map(evaluated))
            }
            Value::Call(name, args) => {
                let evaluated_args: Vec<Value> = args.iter()
                    .map(|a| Self::eval_value(&a.value, env))
                    .collect::<Result<_>>()?;
                Self::call_function(name, &evaluated_args, env)
            }
            Value::Interp(s) => {
                Ok(Value::String(Self::eval_interp_str(s, env), false))
            }
            Value::BinOp(b) => Self::eval_binop(&b.op, &b.left, &b.right, env),
            Value::UnaryOp(op, v) => {
                let val = Self::eval_value(v, env)?;
                match op {
                    UnaryOp::Neg => match val {
                        Value::Number(n, u) => Ok(Value::Number(-n, u)),
                        _ => Err(SassError::Eval(format!("无法对 {val} 取负"))),
                    },
                    UnaryOp::Not => match val {
                        Value::Bool(b) => Ok(Value::Bool(!b)),
                        _ => Ok(Value::Bool(false)),
                    },
                }
            }
            Value::Spread(v) => Self::eval_value(v, env),
        }
    }

    /// 求值二元运算。
    fn eval_binop(op: &BinOpKind, left: &Value, right: &Value, env: &Env) -> Result<Value> {
        let l = Self::eval_value(left, env)?;
        let r = Self::eval_value(right, env)?;
        match op {
            BinOpKind::Add => Self::add(&l, &r),
            BinOpKind::Sub => Self::sub(&l, &r),
            BinOpKind::Mul => Self::mul(&l, &r),
            BinOpKind::Div => Self::div(&l, &r),
            BinOpKind::Mod => Self::modulo(&l, &r),
            BinOpKind::Eq => Ok(Value::Bool(Self::values_eq(&l, &r))),
            BinOpKind::NotEq => Ok(Value::Bool(!Self::values_eq(&l, &r))),
            BinOpKind::And => match l { Value::Bool(false) => Ok(Value::Bool(false)), _ => Ok(r) },
            BinOpKind::Or => match l { Value::Bool(true) => Ok(Value::Bool(true)), _ => Ok(r) },
            BinOpKind::Lt | BinOpKind::Gt | BinOpKind::LtEq | BinOpKind::GtEq => Self::compare(op, &l, &r),
        }
    }

    fn add(l: &Value, r: &Value) -> Result<Value> {
        let l = l.clone();
        let r = r.clone();
        match (l, r) {
            (Value::Number(a, u1), Value::Number(b, u2)) => {
                let unit = u1.or(u2);
                Ok(Value::Number(a + b, unit))
            }
            // 字符串拼接——结果引号跟随左侧
            (Value::String(a, qa), Value::String(b, _)) => Ok(Value::String(format!("{a}{b}"), qa)),
            (Value::String(a, qa), Value::Number(n, u)) => Ok(Value::String(format!("{a}{}{}", n, u.as_deref().unwrap_or("")), qa)),
            (Value::String(a, qa), Value::Color(c)) => Ok(Value::String(format!("{a}#{:02x}{:02x}{:02x}", c.r, c.g, c.b), qa)),
            (Value::String(a, qa), Value::Null) => Ok(Value::String(a, qa)),
            (Value::Number(n, u), Value::String(b, qb)) => Ok(Value::String(format!("{}{}{b}", n, u.as_deref().unwrap_or("")), qb)),
            (Value::Color(c), Value::String(b, qb)) => Ok(Value::String(format!("#{:02x}{:02x}{:02x}{b}", c.r, c.g, c.b), qb)),
            (Value::Null, Value::String(b, qb)) => Ok(Value::String(b, qb)),
            // 列表拼接
            (Value::List(mut items, sep, _), Value::List(items2, _, _)) => {
                items.extend(items2);
                Ok(Value::List(items, sep, false))
            }
            (Value::List(mut items, sep, _), other) => {
                items.push(other);
                Ok(Value::List(items, sep, false))
            }
            (other, Value::List(items, sep, false)) => {
                let mut new_items = vec![other];
                new_items.extend(items);
                Ok(Value::List(new_items, sep, false))
            }
            _ => Err(SassError::Eval("不支持的 + 运算".into())),
        }
    }
    fn sub(l: &Value, r: &Value) -> Result<Value> {
        let l = l.clone();
        let r = r.clone();
        match (l, r) {
            (Value::Number(a, u1), Value::Number(b, u2)) => {
                let unit = u1.or(u2);
                Ok(Value::Number(a - b, unit))
            }
            // 字符串拼接——用 - 连接
            (Value::String(a, qa), Value::String(b, _)) => Ok(Value::String(format!("{a}-{b}"), qa)),
            (Value::String(a, qa), Value::Number(n, u)) => Ok(Value::String(format!("{a}-{}{}", n, u.as_deref().unwrap_or("")), qa)),
            (Value::String(a, qa), Value::Color(c)) => Ok(Value::String(format!("{a}-#{:02x}{:02x}{:02x}", c.r, c.g, c.b), qa)),
            (Value::Number(n, u), Value::String(b, qb)) => Ok(Value::String(format!("{}{}-{b}", n, u.as_deref().unwrap_or("")), qb)),
            (Value::Color(c), Value::String(b, qb)) => Ok(Value::String(format!("#{:02x}{:02x}{:02x}-{b}", c.r, c.g, c.b), qb)),
            _ => Err(SassError::Eval("不支持的 - 运算".into())),
        }
    }
    fn mul(l: &Value, r: &Value) -> Result<Value> {
        match (l, r) {
            (Value::Number(a, u1), Value::Number(b, u2)) => {
                let unit = if u1.is_some() { u1.clone() } else { u2.clone() };
                Ok(Value::Number(a * b, unit))
            }
            _ => Err(SassError::Eval(format!("无法 {l} * {r}"))),
        }
    }
    fn div(l: &Value, r: &Value) -> Result<Value> {
        match (l, r) {
            (Value::Number(a, u1), Value::Number(b, _)) => {
                if *b == 0.0 { return Err(SassError::DivideByZero); }
                Ok(Value::Number(a / b, u1.clone()))
            }
            // 非数字 / —— 作为斜杠分隔列表保留（如 font: 16px/24px）
            _ => Ok(Value::List(vec![l.clone(), r.clone()], Separator::Slash, false)),
        }
    }
    fn modulo(l: &Value, r: &Value) -> Result<Value> {
        match (l, r) {
            (Value::Number(a, u), Value::Number(b, _)) => {
                if *b == 0.0 { return Err(SassError::DivideByZero); }
                Ok(Value::Number(a % b, u.clone()))
            }
            // Null RHS — % 不是运算符，作为字符串保留
            (l, Value::Null) => Ok(Value::List(vec![l.clone(), Value::String("%".to_string(), false)], Separator::Space, false)),
            // 非数字 % —— 作为空格分隔列表保留
            _ => Ok(Value::List(vec![l.clone(), r.clone()], Separator::Space, false)),
        }
    }
    fn compare(op: &BinOpKind, l: &Value, r: &Value) -> Result<Value> {
        match (l, r) {
            (Value::Number(a, _), Value::Number(b, _)) => {
                let result = match op {
                    BinOpKind::Lt => a < b,
                    BinOpKind::Gt => a > b,
                    BinOpKind::LtEq => a <= b,
                    BinOpKind::GtEq => a >= b,
                    _ => false,
                };
                Ok(Value::Bool(result))
            }
            _ => Err(SassError::Eval(format!("无法比较 {l} 和 {r}"))),
        }
    }

    /// inspect() 专用格式化——比 Display 更详细。
    fn inspect_value(v: &Value) -> String {
        match v {
            Value::List(elements, sep, bracketed) => {
                if elements.is_empty() {
                    if *bracketed { return "[]".to_string(); }
                    if matches!(sep, Separator::Comma) { return "()".to_string(); }
                    return String::new();
                }
                let sep_str = match sep {
                    Separator::Comma => ", ",
                    Separator::Space => " ",
                    Separator::Slash => " / ",
                    Separator::Undecided => " ",
                };
                let parts: Vec<String> = elements.iter().map(Self::inspect_value).collect();
                let inner = if elements.len() == 1 && matches!(sep, Separator::Comma) {
                    format!("{},", parts[0])
                } else {
                    parts.join(sep_str)
                };
                if *bracketed { format!("[{}]", inner) } else { inner }
            }
            Value::Map(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", Self::inspect_value(k), Self::inspect_value(v)))
                    .collect();
                if pairs.len() == 1 {
                    format!("({},)", parts.join(", "))
                } else {
                    format!("({})", parts.join(", "))
                }
            }
            Value::String(s, quoted) => {
                if *quoted { format!("\"{s}\"") } else { s.clone() }
            }
            Value::Null => "null".to_string(),
            _ => v.to_string(),
        }
    }

    fn values_eq(l: &Value, r: &Value) -> bool {
        match (l, r) {
            (Value::Number(a, _), Value::Number(b, _)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a, _), Value::String(b, _)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Color(a), Value::Color(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a, _, _), Value::List(b, _, _)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Self::values_eq(x, y))
            }
            (Value::Map(a), Value::Map(b)) => {
                a.len() == b.len() && a.iter().all(|(k, v)| {
                    b.iter().any(|(k2, v2)| Self::values_eq(k, k2) && Self::values_eq(v, v2))
                })
            }
            _ => false,
        }
    }

    /// 求值插值字符串 #{...}。
    fn eval_interp_str(s: &str, env: &Env) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '#' && chars.peek() == Some(&'{') {
                chars.next(); // 消费 {
                let mut expr = String::new();
                let mut depth = 1;
                while let Some(ch) = chars.next() {
                    if ch == '{' { depth += 1; expr.push(ch); }
                    else if ch == '}' { depth -= 1; if depth == 0 { break; } expr.push(ch); }
                    else { expr.push(ch); }
                }
                // 尝试求值表达式
                if let Ok(val) = Self::eval_simple_expr(&expr, env) {
                    result.push_str(&val.to_string());
                } else {
                    result.push_str(&expr);
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    /// 简单表达式求值（用于插值）。
    fn eval_simple_expr(expr: &str, env: &Env) -> Result<Value> {
        let expr = expr.trim();
        // 变量引用
        if let Some(name) = expr.strip_prefix('$') {
            return env.lookup(name).cloned().ok_or_else(|| SassError::UndefinedVariable(name.to_string()));
        }
        // 尝试作为数字
        if let Ok(n) = expr.parse::<f64>() {
            return Ok(Value::Number(n, None));
        }
        // 尝试词法分析 + 解析
        let tokens: Vec<_> = crate::lex::Lexer::new(expr)
            .filter(|t| !matches!(t.as_ref(), Ok(crate::lex::token::Token::Whitespace) | Ok(crate::lex::token::Token::Eof)))
            .collect::<crate::error::Result<Vec<_>>>()?;
        let mut parser = crate::parse::Parser::new(&tokens);
        let v = parser.parse_value()?;
        Self::eval_value(&v, env)
    }

    // —— 控制流 ——
    fn eval_if(branches: &[(Value, Vec<Node>)], else_body: &Option<Vec<Node>>, env: &Env) -> Result<(Vec<CssNode>, Env)> {
        for (cond, body) in branches {
            let c = Self::eval_value(cond, env)?;
            if Self::is_truthy(&c) {
                return Self::eval_nodes(body, env);
            }
        }
        if let Some(body) = else_body {
            Self::eval_nodes(body, env)
        } else {
            Ok((vec![], env.clone()))
        }
    }

    fn eval_for(var: &str, from: &Value, to: &Value, inclusive: bool, body: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        let from_val = Self::eval_value(from, env)?;
        let to_val = Self::eval_value(to, env)?;
        let (start, end) = match (from_val, to_val) {
            (Value::Number(s, _), Value::Number(e, _)) => (s as i64, e as i64),
            _ => return Err(SassError::Eval("@for 范围必须是数字".into())),
        };
        let mut css = Vec::new();
        let mut current_env = env.clone();
        if inclusive {
            for i in start..=end {
                if i - start > MAX_DEPTH as i64 {
                    return Err(SassError::Eval("@for 循环次数超过限制".into()));
                }
                current_env = current_env.bind(var.to_string(), Value::Number(i as f64, None));
                let (mut out, e) = Self::eval_nodes(body, &current_env)?;
                css.append(&mut out);
                current_env = e;
            }
        } else {
            for i in start..end {
                if i - start > MAX_DEPTH as i64 {
                    return Err(SassError::Eval("@for 循环次数超过限制".into()));
                }
                current_env = current_env.bind(var.to_string(), Value::Number(i as f64, None));
                let (mut out, e) = Self::eval_nodes(body, &current_env)?;
                css.append(&mut out);
                current_env = e;
            }
        }
        Ok((css, env.clone()))
    }

    fn eval_each(vars: &[String], list: &Value, body: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        let evaluated = Self::eval_value(list, env)?;
        // 对 Map，按 (key, value) 对迭代
        let items: Vec<Vec<Value>> = match &evaluated {
            Value::Map(pairs) if vars.len() >= 2 => {
                pairs.iter().map(|(k, v)| vec![k.clone(), v.clone()]).collect()
            }
            Value::Map(pairs) if vars.len() == 1 => {
                // 单变量遍历 Map：每对作为一个子列表
                pairs.iter().map(|(k, v)| vec![Value::List(vec![k.clone(), v.clone()], Separator::Space, false)]).collect()
            }
            Value::List(es, _, _) => es.iter().map(|e| vec![e.clone()]).collect(),
            Value::Map(pairs) => pairs.iter().flat_map(|(k, v)| vec![vec![k.clone()], vec![v.clone()]]).collect(),
            other => vec![vec![other.clone()]],
        };
        let mut css = Vec::new();
        let mut current_env = env.clone();
        for item_group in &items {
            if css.len() > 10000 {
                return Err(SassError::Eval("@each 输出节点过多".into()));
            }
            if vars.len() == 1 {
                let val = item_group.get(0).cloned().unwrap_or(Value::Null);
                current_env = current_env.bind(vars[0].clone(), val);
            } else {
                for (j, v) in vars.iter().enumerate() {
                    let val = item_group.get(j).cloned().unwrap_or(Value::Null);
                    current_env = current_env.bind(v.clone(), val);
                }
            }
            let (mut out, e) = Self::eval_nodes(body, &current_env)?;
            css.append(&mut out);
            current_env = e;
        }
        Ok((css, env.clone()))
    }

    fn eval_while(cond: &Value, body: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        let mut css = Vec::new();
        let mut current_env = env.clone();
        let mut iteration = 0;
        loop {
            iteration += 1;
            if iteration > MAX_DEPTH {
                return Err(SassError::Eval("@while 循环次数超过限制（可能是无限循环）".into()));
            }
            let c = Self::eval_value(cond, &current_env)?;
            if !Self::is_truthy(&c) { break; }
            let (mut out, e) = Self::eval_nodes(body, &current_env)?;
            css.append(&mut out);
            current_env = e;
            // 限制 CSS 输出大小
            if css.len() > 10000 {
                return Err(SassError::Eval("@while 输出节点过多".into()));
            }
        }
        Ok((css, env.clone()))
    }

    // —— Mixin / Function ——
    fn eval_include(name: &str, args: &[Arg], content: &Option<Vec<Node>>, env: &Env) -> Result<(Vec<CssNode>, Env)> {
        let mixin = env.get_mixin(name).ok_or_else(|| SassError::UndefinedMixin(name.to_string()))?.clone();
        // 绑定参数
        let mixin_env = Self::bind_params(&mixin.params, args, env)?.incr_depth();
        // 注入 @content 块
        let mixin_env = if let Some(content_nodes) = content {
            mixin_env.set_content(content_nodes.clone(), env.clone())
        } else {
            mixin_env
        };
        // 求值 mixin body
        let (css, _) = Self::eval_nodes(&mixin.body, &mixin_env)?;
        Ok((css, env.clone()))
    }

    fn bind_params(params: &[Param], args: &[Arg], env: &Env) -> Result<Env> {
        // 先求值所有参数，分离位置参数和关键字参数，展开 spread
        let mut positional: Vec<Value> = Vec::new();
        let mut keyword: HashMap<String, Value> = HashMap::new();
        for arg in args {
            let val = Self::eval_value(&arg.value, env)?;
            if arg.spread {
                // 展开 $args... 为多个位置参数
                if let Value::List(items, _, _) = val {
                    positional.extend(items);
                } else {
                    positional.push(val);
                }
            } else if let Some(name) = &arg.name {
                keyword.insert(name.clone(), val);
            } else {
                positional.push(val);
            }
        }

        let mut new_env = env.clone();
        let mut pos_idx = 0;
        for param in params.iter() {
            if param.rest {
                // 剩余参数——收集剩余位置参数
                let rest: Vec<Value> = positional[pos_idx..].to_vec();
                new_env = new_env.bind(param.name.clone(), Value::List(rest, Separator::Comma, false));
                pos_idx = positional.len();
                break;
            }
            // 优先用关键字参数
            if let Some(val) = keyword.get(&param.name) {
                new_env = new_env.bind(param.name.clone(), val.clone());
            } else if pos_idx < positional.len() {
                new_env = new_env.bind(param.name.clone(), positional[pos_idx].clone());
                pos_idx += 1;
            } else if let Some(default) = &param.default {
                let val = Self::eval_value(default, &new_env)?;
                new_env = new_env.bind(param.name.clone(), val);
            } else {
                new_env = new_env.bind(param.name.clone(), Value::Null);
            }
        }
        Ok(new_env)
    }

    /// 调用函数（内建或用户定义）。
    fn call_function(name: &str, args: &[Value], env: &Env) -> Result<Value> {
        // 用户函数
        if let Some(func) = env.get_function(name) {
            return Self::call_user_function(func, args, env);
        }
        // 模块限定函数 (math.abs, map.get, etc.)
        if name.contains('.') {
            return Self::call_module_function(name, args, env);
        }
        // 内建函数
        Self::call_builtin(name, args, env)
    }

    /// 应用 @extend 后处理——遍历 CSS 树，为目标选择器添加继承者。
    fn apply_extends(nodes: &mut [CssNode], extends: &[(String, String)]) {
        for node in nodes.iter_mut() {
            match node {
                CssNode::Rule { selector, children, .. } => {
                    // 应用 extend
                    for (extender, target) in extends {
                        let target_trimmed = target.trim();
                        if selector.contains(target_trimmed) {
                            if target_trimmed.starts_with('%') {
                                // 占位符：直接替换为目标
                                *selector = selector.replace(target_trimmed, extender);
                            } else {
                                // 普通选择器：添加继承者作为额外选择器
                                let new_sel = selector.replace(target_trimmed, extender);
                                if !new_sel.is_empty() && new_sel != *selector {
                                    if !selector.contains(&new_sel) {
                                        selector.push_str(", ");
                                        selector.push_str(&new_sel);
                                    }
                                }
                            }
                        }
                    }
                    // 递归处理子规则
                    Self::apply_extends(children, extends);
                    // 移除未被继承的占位符选择器部分
                    let parts: Vec<&str> = selector.split(',')
                        .filter(|s| !s.trim().starts_with('%'))
                        .collect();
                    *selector = parts.join(",").trim().to_string();
                }
                CssNode::AtRule { children, .. } => {
                    Self::apply_extends(children, extends);
                }
                CssNode::AtRoot(kids) => {
                    Self::apply_extends(kids, extends);
                }
                _ => {}
            }
        }
    }

    /// 解析模块 URL 到文件路径——_ 前缀加在文件名上，不是路径上。
    fn resolve_file(base: Option<&PathBuf>, url: &str) -> Option<PathBuf> {
        let base_dir = base
            .as_ref()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        // 拆分路径和文件名——_ 前缀只加在文件名上
        let url_path = std::path::Path::new(url);
        let parent = url_path.parent().unwrap_or(std::path::Path::new(""));
        let filename = url_path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| url.to_string());
        let candidates = [
            base_dir.join(parent).join(format!("_{filename}.scss")),
            base_dir.join(parent).join(format!("{filename}.scss")),
            base_dir.join(parent).join(format!("_{filename}.sass")),
            base_dir.join(parent).join(format!("{filename}.sass")),
            base_dir.join(url).join("_index.scss"),
            base_dir.join(url).join("index.scss"),
        ];
        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }
        None
    }

    /// 加载文件模块——读取、词法分析、语法分析、求值，返回导出。
    fn load_module(path: &Path, config: &[(String, Value)], caller_env: &Env) -> Result<ModuleExports> {
        // 防止循环导入导致栈溢出
        if caller_env.depth > 50 {
            return Ok(ModuleExports::default());
        }
        let source = std::fs::read_to_string(path)
            .map_err(|e| SassError::Module(format!("无法读取 {}: {e}", path.display())))?;
        let tokens: Vec<Token> = Lexer::new(&source)
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace) | Ok(Token::Eof)))
            .collect::<Result<Vec<_>>>()?;
        let ast = crate::parse::Parser::parse(&tokens)?;
        let mut env = Env::default().with_base_path(path.to_path_buf());
        env.depth = caller_env.depth + 1;
        // 注入 with() 配置变量（求值后注入，使 !default 尊重覆盖值）
        for (name, value) in config {
            let val = Self::eval_value(value, caller_env)?;
            env = env.bind(name.clone(), val);
        }
let (module_css, final_env) = Self::eval_nodes(&ast.nodes, &env)?;
Ok(ModuleExports {
vars: final_env.vars,
mixins: final_env.mixins,
functions: final_env.functions,
css: module_css,
})
    }

    /// 模块限定函数调用。
    fn call_module_function(name: &str, args: &[Value], env: &Env) -> Result<Value> {
        // 先检查文件加载的命名空间
        if let Some(dot) = name.find('.') {
            let ns = &name[..dot];
            let func_name = &name[dot + 1..];
            if let Some(module) = env.get_namespace(ns) {
                if let Some(func) = module.functions.get(func_name) {
                    return Self::call_user_function(func, args, env);
                }
            }
        }
        // 将模块限定名映射到内建函数
        let builtin_name = match name {
            // sass:math
            "math.abs" => "abs",
            "math.ceil" => "ceil",
            "math.floor" => "floor",
            "math.round" => "round",
            "math.max" => "max",
            "math.min" => "min",
            "math.percentage" => "percentage",
            "math.random" => "random",
            "math.pow" => "pow",
            "math.sqrt" => "sqrt",
            "math.sin" => "sin",
            "math.cos" => "cos",
            "math.tan" => "tan",
            "math.unit" => "unit",
            "math.is-unitless" => "is-unitless",
            "math.compatible" => "compatible",
            // sass:string
            "string.length" => "str-length",
            "string.index" => "str-index",
            "string.slice" => "str-slice",
            "string.to-upper-case" => "to-upper-case",
            "string.to-lower-case" => "to-lower-case",
            "string.insert" => "str-insert",
            "string.quote" => "quote",
            "string.unquote" => "unquote",
            // sass:map
            "map.get" => "map-get",
            "map.merge" => "map-merge",
            "map.remove" => "map-remove",
            "map.keys" => "map-keys",
            "map.values" => "map-values",
            "map.has-key" => "map-has-key",
            // sass:list
            "list.length" => "length",
            "list.nth" => "nth",
            "list.append" => "append",
            "list.join" => "join",
            "list.index" => "index",
            "list.separator" => "separator",
            "list.set-nth" => "set-nth",
            "list.zip" => "zip",
            "list.is-bracketed" => "is-bracketed",
            // sass:color
            "color.adjust" => "adjust-color",
            "color.change" => "change-color",
            "color.scale" => "scale-color",
            "color.mix" => "mix",
            "color.invert" => "invert",
            "color.grayscale" => "grayscale",
            "color.complement" => "complement",
            "color.channel" => "color-channel",
            // sass:meta
            "meta.type-of" => "type-of",
            "meta.inspect" => "inspect",
            "meta.keywords" => "keywords",
            "meta.get-function" => "get-function",
            "meta.call" => "call",
            "meta.mixin-exists" => "mixin-exists",
            "meta.function-exists" => "function-exists",
            "meta.global-variable-exists" => "global-variable-exists",
            "meta.variable-exists" => "variable-exists",
            // sass:selector
            "selector.append" => "selector-append",
            "selector.nest" => "selector-nest",
            "selector.is-super" => "selector-is-super",
            "selector.parse" => "selector-parse",
            "selector.simple-selectors" => "selector-simple-selectors",
            "selector.unify" => "selector-unify",
            "selector.extend" => "selector-extend",
            _ => name,
        };
        Self::call_builtin(builtin_name, args, env)
    }

    fn call_user_function(func: &FunctionDef, args: &[Value], env: &Env) -> Result<Value> {
        let mut func_env = env.incr_depth();
        for (i, param) in func.params.iter().enumerate() {
            let val = if let Some(arg) = args.get(i) {
                arg.clone()
            } else if let Some(default) = &param.default {
                Self::eval_value(default, &func_env)?
            } else {
                Value::Null
            };
            func_env = func_env.bind(param.name.clone(), val);
        }
        // 求值函数体，找 @return
        for node in &func.body {
            if let Node::Return(v) = node {
                return Self::eval_value(v, &func_env);
            }
            let (_, e) = Self::eval_node(node, &func_env)?;
            func_env = e;
        }
        Ok(Value::Null)
    }

    // —— @at-root ——
    fn eval_at_root(_query: &Option<String>, body: &[Node], env: &Env) -> Result<(Vec<CssNode>, Env)> {
        let (css, new_env) = Self::eval_nodes(body, env)?;
        // 包装为 AtRoot，信号 eval_rule 不嵌套
        Ok((vec![CssNode::AtRoot(css)], new_env))
    }

    // —— @规则 ——
    fn eval_at_rule(name: &str, params: &Option<String>, body: &Option<Vec<Node>>, env: &Env) -> Result<(Vec<CssNode>, Env)> {
        let children = match body {
            Some(nodes) => Self::eval_nodes(nodes, env)?.0,
            None => Vec::new(),
        };
        Ok((vec![CssNode::AtRule {
            name: name.to_string(),
            params: params.clone(),
            children,
        }], env.clone()))
    }

    // —— 辅助 ——
    fn is_truthy(v: &Value) -> bool {
        !matches!(v, Value::Bool(false) | Value::Null)
    }

    /// 内建函数分派。
    fn call_builtin(name: &str, args: &[Value], env: &Env) -> Result<Value> {
        match name {
            // math
            "abs" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.abs(), u.clone())),
                _ => Err(SassError::Eval("abs 需要 1 个数字参数".into())),
            },
            "ceil" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.ceil(), u.clone())),
                _ => Err(SassError::Eval("ceil 需要 1 个数字参数".into())),
            },
            "floor" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.floor(), u.clone())),
                _ => Err(SassError::Eval("floor 需要 1 个数字参数".into())),
            },
            "round" => match args {
                [Value::Number(n, u)] => Ok(Value::Number(n.round(), u.clone())),
                _ => Err(SassError::Eval("round 需要 1 个数字参数".into())),
            },
            "min" => args.iter().try_fold(Value::Number(f64::INFINITY, None), |acc, v| match (acc, v) {
                (Value::Number(a, _), Value::Number(b, u)) => Ok(Value::Number(a.min(*b), u.clone())),
                _ => Err(SassError::Eval("min 需要数字参数".into())),
            }),
            "max" => args.iter().try_fold(Value::Number(f64::NEG_INFINITY, None), |acc, v| match (acc, v) {
                (Value::Number(a, _), Value::Number(b, u)) => Ok(Value::Number(a.max(*b), u.clone())),
                _ => Err(SassError::Eval("max 需要数字参数".into())),
            }),
            "percentage" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n * 100.0, Some("%".into()))),
                _ => Err(SassError::Eval("percentage 需要 1 个数字参数".into())),
            },
            // string
            "str-length" => match args {
                [Value::String(s, _)] => Ok(Value::Number(s.chars().count() as f64, None)),
                _ => Err(SassError::Eval("str-length 需要 1 个字符串参数".into())),
            },
            "to-upper-case" => match args {
                [Value::String(s, q)] => Ok(Value::String(s.to_uppercase(), *q)),
                _ => Err(SassError::Eval("to-upper-case 需要 1 个字符串参数".into())),
            },
            "to-lower-case" => match args {
                [Value::String(s, q)] => Ok(Value::String(s.to_lowercase(), *q)),
                _ => Err(SassError::Eval("to-lower-case 需要 1 个字符串参数".into())),
            },
            "unquote" => match args {
                [Value::String(s, _)] => Ok(Value::String(s.clone(), false)),
                _ => Err(SassError::Eval("unquote 需要 1 个字符串参数".into())),
            },
            "quote" => match args {
                [Value::String(s, _)] => Ok(Value::String(s.clone(), true)),
                _ => Err(SassError::Eval("quote 需要 1 个字符串参数".into())),
            },
            // color
            "rgba" => Self::builtin_rgba(args),
            "rgb" => Self::builtin_rgba(args),
            "darken" => Self::builtin_darken(args),
            "lighten" => Self::builtin_lighten(args),
            "mix" => Self::builtin_mix(args),
            "invert" => match args {
                [Value::Color(c)] => Ok(Value::Color(Color::rgb(255 - c.r, 255 - c.g, 255 - c.b))),
                _ => Err(SassError::Eval("invert 需要 1 个颜色参数".into())),
            },
            "grayscale" => match args {
                [Value::Color(c)] => {
                    let avg = ((c.r as u16 + c.g as u16 + c.b as u16) / 3) as u8;
                    Ok(Value::Color(Color::rgba(avg, avg, avg, c.a)))
                }
                _ => Err(SassError::Eval("grayscale 需要 1 个颜色参数".into())),
            },
            // list
"length" | "list-length" => match args {
[Value::List(es, _, _)] => Ok(Value::Number(es.len() as f64, None)),
[Value::Map(pairs)] => Ok(Value::Number(pairs.len() as f64, None)),
[_] => Ok(Value::Number(1.0, None)),
_ => Err(SassError::Eval("length 需要 1 个参数".into())),
},
"nth" => match args {
[Value::List(es, _, _), Value::Number(n, _)] => {
let len = es.len() as i64;
let idx = *n as i64;
let actual = if idx > 0 { (idx as usize).saturating_sub(1) }
else if idx < 0 { ((len + idx) as usize).saturating_sub(1) }
else { return Err(SassError::Eval("nth 索引 0 无效（从 1 开始）".into())); };
es.get(actual).cloned().ok_or_else(|| SassError::Eval(format!("nth 索引 {idx} 超出范围")))
}
[Value::Map(pairs), Value::Number(n, _)] => {
let len = pairs.len() as i64;
let idx = *n as i64;
let actual = if idx > 0 { (idx as usize).saturating_sub(1) }
else if idx < 0 { ((len + idx) as usize).saturating_sub(1) }
else { return Err(SassError::Eval("nth 索引 0 无效".into())); };
pairs.get(actual).map(|(k, v)| Value::List(vec![k.clone(), v.clone()], Separator::Space, false))
.ok_or_else(|| SassError::Eval(format!("nth 索引 {idx} 超出范围")))
}
[other, Value::Number(1.0, _)] => Ok(other.clone()),
[other, Value::Number(-1.0, _)] => Ok(other.clone()),
_ => Err(SassError::Eval("nth 需要 (list, n) 参数".into())),
},
            // map
            "map-get" => match args {
                [Value::Map(pairs), key] => pairs.iter()
                    .find(|(k, _)| Self::values_eq(k, key))
                    .map(|(_, v)| v.clone())
                    .ok_or_else(|| SassError::Eval(format!("map-get: 键 {key} 不存在"))),
                _ => Err(SassError::Eval("map-get 需要 (map, key) 参数".into())),
            },
            "map-keys" => match args {
                [Value::Map(pairs)] => Ok(Value::List(pairs.iter().map(|(k, _)| k.clone()).collect(), Separator::Comma, false)),
                _ => Err(SassError::Eval("map-keys 需要 1 个 map 参数".into())),
            },
            "map-values" => match args {
                [Value::Map(pairs)] => Ok(Value::List(pairs.iter().map(|(_, v)| v.clone()).collect(), Separator::Comma, false)),
                _ => Err(SassError::Eval("map-values 需要 1 个 map 参数".into())),
            },
            "map-has-key" => match args {
                [Value::Map(pairs), key] => Ok(Value::Bool(pairs.iter().any(|(k, _)| Self::values_eq(k, key)))),
                _ => Err(SassError::Eval("map-has-key 需要 (map, key) 参数".into())),
            },
            // meta
            "type-of" => match args {
                [Value::Number(..)] => Ok(Value::String("number".into(), false)),
                [Value::String(..)] => Ok(Value::String("string".into(), false)),
                [Value::Color(..)] => Ok(Value::String("color".into(), false)),
                [Value::Bool(..)] => Ok(Value::String("bool".into(), false)),
                [Value::List(..)] => Ok(Value::String("list".into(), false)),
                [Value::Map(..)] => Ok(Value::String("map".into(), false)),
                [Value::Null] => Ok(Value::String("null".into(), false)),
                _ => Ok(Value::String("unknown".into(), false)),
            },
            "inspect" => match args {
                [v] => Ok(Value::String(Self::inspect_value(v), false)),
                _ => Err(SassError::Eval("inspect 需要 1 个参数".into())),
            },
            "if" => match args {
                [cond, t, f] => Ok(if Self::is_truthy(cond) { t.clone() } else { f.clone() }),
                _ => Err(SassError::Eval("if 需要 3 个参数".into())),
            },
            // string (additional)
            "str-slice" => match args {
                [Value::String(s, q), Value::Number(start, _)] => {
                    let start = *start as isize;
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as isize;
                    let start_idx = if start < 0 { (len + start).max(0) as usize } else { (start - 1).max(0) as usize };
                    let result: String = chars[start_idx.min(len as usize)..].iter().collect();
                    Ok(Value::String(result, *q))
                }
                [Value::String(s, q), Value::Number(start, _), Value::Number(end, _)] => {
                    let start = *start as isize;
                    let end = *end as isize;
                    let chars: Vec<char> = s.chars().collect();
                    let len = chars.len() as isize;
                    let start_idx = if start < 0 { (len + start).max(0) as usize } else { (start - 1).max(0) as usize };
                    let end_idx = if end < 0 { (len + end + 1).max(0) as usize } else { end.min(len) as usize };
                    let result: String = chars[start_idx.min(end_idx)..end_idx.min(len as usize)].iter().collect();
                    Ok(Value::String(result, *q))
                }
                _ => Err(SassError::Eval("str-slice 需要 2-3 个参数".into())),
            },
            "str-index" => match args {
                [Value::String(s, _), Value::String(needle, _)] => {
                    match s.find(needle) {
                        Some(pos) => Ok(Value::Number((s[..pos].chars().count() + 1) as f64, None)),
                        None => Ok(Value::Null),
                    }
                }
                _ => Err(SassError::Eval("str-index 需要 2 个字符串参数".into())),
            },
            "str-insert" => match args {
                [Value::String(s, q), Value::String(insert, _), Value::Number(idx, _)] => {
                    let idx = *idx as usize;
                    let chars: Vec<char> = s.chars().collect();
                    let pos = idx.min(chars.len()).min(idx.saturating_sub(1));
                    let mut result: Vec<char> = chars[..pos].to_vec();
                    result.extend(insert.chars());
                    result.extend(chars[pos..].iter());
                    Ok(Value::String(result.into_iter().collect(), *q))
                }
                _ => Err(SassError::Eval("str-insert 需要 3 个参数".into())),
            },
            "unique-id" => Ok(Value::String(format!("id{}", std::time::SystemTime::now().elapsed().unwrap_or_default().as_nanos()), false)),
            // math (additional)
            "math.div" | "div" => match args {
                [Value::Number(a, u1), Value::Number(b, _)] => {
                    if *b == 0.0 { return Err(SassError::DivideByZero); }
                    Ok(Value::Number(a / b, u1.clone()))
                }
                _ => Err(SassError::Eval("div 需要 2 个数字参数".into())),
            },
            "pow" => match args {
                [Value::Number(a, _), Value::Number(b, _)] => Ok(Value::Number(a.powf(*b), None)),
                _ => Err(SassError::Eval("pow 需要 2 个数字参数".into())),
            },
            "sqrt" => match args {
                [Value::Number(n, _)] => Ok(Value::Number(n.sqrt(), None)),
                _ => Err(SassError::Eval("sqrt 需要 1 个数字参数".into())),
            },
            "sin" => match args { [Value::Number(n, _)] => Ok(Value::Number(n.sin(), None)), _ => Err(SassError::Eval("sin 需要 1 个参数".into())) },
            "cos" => match args { [Value::Number(n, _)] => Ok(Value::Number(n.cos(), None)), _ => Err(SassError::Eval("cos 需要 1 个参数".into())) },
            "tan" => match args { [Value::Number(n, _)] => Ok(Value::Number(n.tan(), None)), _ => Err(SassError::Eval("tan 需要 1 个参数".into())) },
            "random" => match args {
                [] => Ok(Value::Number(Self::simple_random(), None)),
                [Value::Number(n, _)] => Ok(Value::Number((Self::simple_random() * n).floor() + 1.0, None)),
                _ => Err(SassError::Eval("random 需要 0-1 个参数".into())),
            },
            "unit" => match args {
                [Value::Number(_, Some(u))] => Ok(Value::String(u.clone(), false)),
                [Value::Number(_, None)] => Ok(Value::String("".into(), false)),
                _ => Err(SassError::Eval("unit 需要 1 个数字参数".into())),
            },
            "is-unitless" => match args {
                [Value::Number(_, None)] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(_))] => Ok(Value::Bool(false)),
                _ => Err(SassError::Eval("is-unitless 需要 1 个数字参数".into())),
            },
            "compatible" => match args {
                [Value::Number(_, None), _] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(u1)), Value::Number(_, Some(u2))] => Ok(Value::Bool(u1 == u2)),
                [Value::Number(_, Some(_)), Value::Number(_, None)] => Ok(Value::Bool(true)),
                _ => Err(SassError::Eval("compatible 需要 2 个数字参数".into())),
            },
            // color (additional)
            "color-channel" => match args {
                [Value::Color(c), Value::String(ch, _)] => match ch.as_str() {
                    "red" => Ok(Value::Number(c.r as f64, None)),
                    "green" => Ok(Value::Number(c.g as f64, None)),
                    "blue" => Ok(Value::Number(c.b as f64, None)),
                    "alpha" => Ok(Value::Number(c.a as f64, None)),
                    _ => Err(SassError::Eval(format!("未知颜色通道: {ch}"))),
                }
                _ => Err(SassError::Eval("color-channel 需要 (color, channel) 参数".into())),
            },
            "adjust-color" | "change-color" | "scale-color" => {
                args.first().cloned().ok_or_else(|| SassError::Eval("颜色函数需要至少 1 个参数".into()))
            }
            "complement" => match args {
                [Value::Color(c)] => Ok(Value::Color(Color::rgb(255 - c.r, 255 - c.g, 255 - c.b))),
                _ => Err(SassError::Eval("complement 需要 1 个颜色参数".into())),
            },
            // map (additional)
            "map-merge" => match args {
                [Value::Map(a), Value::Map(b)] => {
                    let mut merged = a.clone();
                    for (k, v) in b { merged.push((k.clone(), v.clone())); }
                    Ok(Value::Map(merged))
                }
                _ => Err(SassError::Eval("map-merge 需要 2 个 map 参数".into())),
            },
            "map-remove" => match args {
                [Value::Map(pairs), keys @ ..] => {
                    let filtered: Vec<(Value, Value)> = pairs.iter()
                        .filter(|(k, _)| !keys.iter().any(|key| Self::values_eq(k, key)))
                        .cloned()
                        .collect();
                    Ok(Value::Map(filtered))
                }
                _ => Err(SassError::Eval("map-remove 需要至少 1 个参数".into())),
            },
            // meta (additional)
            "mixin-exists" => Ok(Value::Bool(false)),
            "function-exists" => match args {
                [Value::String(name, _)] => Ok(Value::Bool(env.get_function(name).is_some())),
                _ => Ok(Value::Bool(false)),
            },
            "global-variable-exists" => match args {
                [Value::String(name, _)] => Ok(Value::Bool(env.has_var(name))),
                _ => Ok(Value::Bool(false)),
            },
            "variable-exists" => match args {
                [Value::String(name, _)] => Ok(Value::Bool(env.has_var(name))),
                _ => Ok(Value::Bool(false)),
            },
            "get-function" => match args {
                [Value::String(fname, _)] => Ok(Value::String(fname.clone(), false)),
                _ => Err(SassError::Eval("get-function 需要 1 个参数".into())),
            },
            "call" => match args {
                [Value::String(fname, _), rest @ ..] => Self::call_builtin(fname, rest, env),
                _ => Err(SassError::Eval("call 需要至少 1 个参数".into())),
            },
            "keywords" => match args {
                [_] => Ok(Value::Map(vec![])),
                _ => Err(SassError::Eval("keywords 需要 1 个参数".into())),
            },
            // list (additional)
            "append" => match args {
                [Value::List(items, sep, false), val] => {
                    let mut new_items = items.clone();
                    new_items.push(val.clone());
                    Ok(Value::List(new_items, sep.clone(), false))
                }
                [Value::List(items, sep, false), val, Value::String(s, _)] => {
                    let new_sep = match s.as_str() {
                        "comma" => Separator::Comma,
                        "space" => Separator::Space,
                        "slash" => Separator::Slash,
                        _ => sep.clone(),
                    };
                    let mut new_items = items.clone();
                    new_items.push(val.clone());
                    Ok(Value::List(new_items, new_sep, false))
                }
                [other, val] => Ok(Value::List(vec![other.clone(), val.clone()], Separator::Space, false)),
                _ => Err(SassError::Eval("append 需要 2-3 个参数".into())),
            },
            "join" => match args {
                [Value::List(a, sa, false), Value::List(b, sb, false)] => {
                    let sep = if a.is_empty() { sb.clone() } else { sa.clone() };
                    let mut items = a.clone();
                    items.extend(b.clone());
                    Ok(Value::List(items, sep, false))
                }
                [Value::List(a, sa, false), Value::List(b, sb, false), Value::String(s, _)] => {
                    let sep = match s.as_str() {
                        "comma" => Separator::Comma,
                        "space" => Separator::Space,
                        "slash" => Separator::Slash,
                        _ => if a.is_empty() { sb.clone() } else { sa.clone() },
                    };
                    let mut items = a.clone();
                    items.extend(b.clone());
                    Ok(Value::List(items, sep, false))
                }
                [a, b] => Ok(Value::List(vec![a.clone(), b.clone()], Separator::Space, false)),
                _ => Err(SassError::Eval("join 需要 2-4 个参数".into())),
            },
            "index" => match args {
                [Value::List(items, _, _), needle] => {
                    for (i, item) in items.iter().enumerate() {
                        if Self::values_eq(item, needle) {
                            return Ok(Value::Number((i + 1) as f64, None));
                        }
                    }
                    Ok(Value::Null)
                }
                [other, needle] => {
                    if Self::values_eq(other, needle) { Ok(Value::Number(1.0, None)) }
                    else { Ok(Value::Null) }
                }
                _ => Err(SassError::Eval("index 需要 2 个参数".into())),
            },
            "list-separator" | "separator" => match args {
                [Value::List(_, Separator::Comma, false)] => Ok(Value::String("comma".into(), false)),
                [Value::List(_, Separator::Space, false)] => Ok(Value::String("space".into(), false)),
                [Value::List(_, Separator::Slash, false)] => Ok(Value::String("slash".into(), false)),
                _ => Ok(Value::String("space".into(), false)),
            },
            "set-nth" => match args {
                [Value::List(items, sep, false), Value::Number(n, _), val] => {
                    let idx = *n as usize;
                    let mut new_items = items.clone();
                    if idx >= 1 && idx <= new_items.len() {
                        new_items[idx - 1] = val.clone();
                    }
                    Ok(Value::List(new_items, sep.clone(), false))
                }
                _ => Err(SassError::Eval("set-nth 需要 3 个参数".into())),
            },
            "is-bracketed" => match args {
                [Value::List(_, _, true)] => Ok(Value::Bool(true)),
                _ => Ok(Value::Bool(false)),
            },
            "zip" => match args {
                [Value::List(a, _, _), Value::List(b, _, _)] => {
                    let pairs: Vec<Value> = a.iter().zip(b.iter()).map(|(x, y)| {
                        Value::List(vec![x.clone(), y.clone()], Separator::Space, false)
                    }).collect();
                    Ok(Value::List(pairs, Separator::Comma, false))
                }
                _ => Err(SassError::Eval("zip 需要 2+ 个列表参数".into())),
            },
            // color (additional)
            "hsl" => match args {
                [Value::Number(h, _), Value::Number(s, _), Value::Number(l, _)] => {
                    Ok(Value::Color(Self::hsl_to_rgb(*h, *s / 100.0, *l / 100.0)))
                }
                [Value::Number(h, _), Value::Number(s, _), Value::Number(l, _), Value::Number(a, _)] => {
                    let mut c = Self::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    c.a = *a as f32;
                    Ok(Value::Color(c))
                }
                _ => Err(SassError::Eval("hsl 需要 3-4 个参数".into())),
            },
            "hsla" => match args {
                [Value::Number(h, _), Value::Number(s, _), Value::Number(l, _), Value::Number(a, _)] => {
                    let mut c = Self::hsl_to_rgb(*h, *s / 100.0, *l / 100.0);
                    c.a = *a as f32;
                    Ok(Value::Color(c))
                }
                _ => Err(SassError::Eval("hsla 需要 4 个参数".into())),
            },
            "adjust-hue" => match args {
                [Value::Color(c), Value::Number(deg, _)] => {
                    let (h, s, l) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    let new_h = (h + *deg).rem_euclid(360.0);
                    Ok(Value::Color(Self::hsl_to_rgb(new_h, s, l)))
                }
                _ => Err(SassError::Eval("adjust-hue 需要 (color, degrees) 参数".into())),
            },
            "saturate" => match args {
                [Value::Color(c), Value::Number(amount, _)] => {
                    let (h, s, l) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Color(Self::hsl_to_rgb(h, (s + *amount / 100.0).min(1.0), l)))
                }
                [Value::Number(n, _)] => Ok(Value::String(format!("saturate({})", n), false)),
                _ => Err(SassError::Eval("saturate 需要 (color, amount) 参数".into())),
            },
            "desaturate" => match args {
                [Value::Color(c), Value::Number(amount, _)] => {
                    let (h, s, l) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Color(Self::hsl_to_rgb(h, (s - *amount / 100.0).max(0.0), l)))
                }
                _ => Err(SassError::Eval("desaturate 需要 (color, amount) 参数".into())),
            },
            "transparentize" | "fade-out" => match args {
                [Value::Color(c), Value::Number(amount, _)] => {
                    Ok(Value::Color(Color::rgba(c.r, c.g, c.b, (c.a - *amount as f32).max(0.0))))
                }
                _ => Err(SassError::Eval("transparentize 需要 (color, amount) 参数".into())),
            },
            "opacify" | "fade-in" => match args {
                [Value::Color(c), Value::Number(amount, _)] => {
                    Ok(Value::Color(Color::rgba(c.r, c.g, c.b, (c.a + *amount as f32).min(1.0))))
                }
                _ => Err(SassError::Eval("opacify 需要 (color, amount) 参数".into())),
            },
            "alpha" | "opacity" => match args {
                [Value::Color(c)] => Ok(Value::Number(c.a as f64, None)),
                _ => Err(SassError::Eval("alpha 需要 1 个颜色参数".into())),
            },
            "red" => match args {
                [Value::Color(c)] => Ok(Value::Number(c.r as f64, None)),
                _ => Err(SassError::Eval("red 需要 1 个颜色参数".into())),
            },
            "green" => match args {
                [Value::Color(c)] => Ok(Value::Number(c.g as f64, None)),
                _ => Err(SassError::Eval("green 需要 1 个颜色参数".into())),
            },
            "blue" => match args {
                [Value::Color(c)] => Ok(Value::Number(c.b as f64, None)),
                _ => Err(SassError::Eval("blue 需要 1 个颜色参数".into())),
            },
            "hue" => match args {
                [Value::Color(c)] => {
                    let (h, _, _) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Number(h, Some("deg".into())))
                }
                _ => Err(SassError::Eval("hue 需要 1 个颜色参数".into())),
            },
            "saturation" => match args {
                [Value::Color(c)] => {
                    let (_, s, _) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Number(s * 100.0, Some("%".into())))
                }
                _ => Err(SassError::Eval("saturation 需要 1 个颜色参数".into())),
            },
            "lightness" => match args {
                [Value::Color(c)] => {
                    let (_, _, l) = Self::rgb_to_hsl(c.r, c.g, c.b);
                    Ok(Value::Number(l * 100.0, Some("%".into())))
                }
                _ => Err(SassError::Eval("lightness 需要 1 个颜色参数".into())),
            },
            // math (additional)
            "clamp" => match args {
                [Value::Number(min, _), Value::Number(val, _), Value::Number(max, _)] => {
                    Ok(Value::Number(val.max(*min).min(*max), None))
                }
                _ => Err(SassError::Eval("clamp 需要 3 个数字参数".into())),
            },
            "comparable" => match args {
                [Value::Number(_, u1), Value::Number(_, u2)] => {
                    Ok(Value::Bool(u1 == u2 || u1.is_none() || u2.is_none()))
                }
                _ => Err(SassError::Eval("comparable 需要 2 个数字参数".into())),
            },
            "unitless" => match args {
                [Value::Number(_, None)] => Ok(Value::Bool(true)),
                [Value::Number(_, Some(_))] => Ok(Value::Bool(false)),
                _ => Err(SassError::Eval("unitless 需要 1 个数字参数".into())),
            },
            // CSS 原生函数——原样保留
            "calc" | "clamp" | "env" | "var" => {
                let arg_str = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                Ok(Value::Calc(format!("{name}({arg_str})")))
            },
            // selector functions
            "selector-append" => {
                let parts: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                Ok(Value::String(parts.join(""), false))
            }
            "selector-nest" => {
                let parts: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                Ok(Value::String(parts.join(" "), false))
            }
            "selector-is-super" => match args {
                [Value::String(a, _), Value::String(b, _)] => {
                    Ok(Value::Bool(b.contains(a.as_str())))
                }
                _ => Ok(Value::Bool(false)),
            }
            "selector-parse" => match args {
                [Value::String(s, _)] => {
                    let parts: Vec<Value> = s.split(',').map(|p| Value::String(p.trim().to_string(), false)).collect();
                    Ok(Value::List(parts, Separator::Comma, false))
                }
                _ => Err(SassError::Eval("selector-parse 需要 1 个参数".into())),
            }
            "selector-simple-selectors" => match args {
                [Value::String(s, _)] => {
                    // 拆分复合选择器为简单选择器
                    let mut result = Vec::new();
                    let mut current = String::new();
                    for c in s.chars() {
                        if c == '.' || c == '#' || c == ':' || c == '[' {
                            if !current.is_empty() { result.push(Value::String(current.clone(), false)); }
                            current = c.to_string();
                        } else {
                            current.push(c);
                        }
                    }
                    if !current.is_empty() { result.push(Value::String(current, false)); }
                    Ok(Value::List(result, Separator::Comma, false))
                }
                _ => Err(SassError::Eval("selector-simple-selectors 需要 1 个参数".into())),
            }
            "selector-unify" => match args {
                [Value::String(a, _), Value::String(b, _)] => {
                    // 简化版：如果一个是另一个的前缀，返回另一个
                    if a.contains(b.as_str()) { Ok(Value::String(a.clone(), false)) }
                    else if b.contains(a.as_str()) { Ok(Value::String(b.clone(), false)) }
                    else { Ok(Value::String(format!("{a}{b}"), false)) }
                }
                _ => Ok(Value::Null),
            }
            "selector-extend" => match args {
                [Value::String(selector, _), Value::String(target, _), Value::String(extender, _)] => {
                    let result = if selector.contains(target.as_str()) {
                        format!("{selector}, {extender}")
                    } else {
                        selector.clone()
                    };
                    Ok(Value::String(result, false))
                }
                _ => Err(SassError::Eval("selector-extend 需要 3 个参数".into())),
            }
            // not a function → 原样输出
            _ => Err(SassError::UndefinedFunction(name.to_string())),
        }
}

    /// HSL → RGB 转换。
    fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Color {
        let h = h.rem_euclid(360.0);
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        let (r1, g1, b1) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        Color::rgb(
            ((r1 + m) * 255.0).round() as u8,
            ((g1 + m) * 255.0).round() as u8,
            ((b1 + m) * 255.0).round() as u8,
        )
    }

    /// RGB → HSL 转换。
    fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
        let r = r as f64 / 255.0;
        let g = g as f64 / 255.0;
        let b = b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < f64::EPSILON {
            return (0.0, 0.0, l);
        }
        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        let h = if max == r {
            ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0
        } else if max == g {
            ((b - r) / d + 2.0) * 60.0
        } else {
            ((r - g) / d + 4.0) * 60.0
        };
        (h, s, l)
    }

    /// 简单伪随机数——基于系统时间。
    fn simple_random() -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let val = (nanos % 1_000_000) as f64;
    val / 1_000_000.0
}

fn builtin_rgba(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Number(r, _), Value::Number(g, _), Value::Number(b, _)] => {
                Ok(Value::Color(Color::rgb(*r as u8, *g as u8, *b as u8)))
            }
            [Value::Number(r, _), Value::Number(g, _), Value::Number(b, _), Value::Number(a, _)] => {
                Ok(Value::Color(Color::rgba(*r as u8, *g as u8, *b as u8, *a as f32)))
            }
            _ => Err(SassError::Eval("rgba 需要 3-4 个数字参数".into())),
        }
    }

    fn builtin_darken(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                let factor = 1.0 - (*amount as f32 / 100.0);
                Ok(Value::Color(Color::rgba(
                    (c.r as f32 * factor) as u8,
                    (c.g as f32 * factor) as u8,
                    (c.b as f32 * factor) as u8,
                    c.a,
                )))
            }
            _ => Err(SassError::Eval("darken 需要 (color, amount) 参数".into())),
        }
    }

    fn builtin_lighten(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(c), Value::Number(amount, _)] => {
                let factor = *amount as f32 / 100.0;
                Ok(Value::Color(Color::rgba(
                    (c.r as f32 + (255.0 - c.r as f32) * factor) as u8,
                    (c.g as f32 + (255.0 - c.g as f32) * factor) as u8,
                    (c.b as f32 + (255.0 - c.b as f32) * factor) as u8,
                    c.a,
                )))
            }
            _ => Err(SassError::Eval("lighten 需要 (color, amount) 参数".into())),
        }
    }

    fn builtin_mix(args: &[Value]) -> Result<Value> {
        match args {
            [Value::Color(a), Value::Color(b)] => Ok(Value::Color(Color::rgba(
                ((a.r as u16 + b.r as u16) / 2) as u8,
                ((a.g as u16 + b.g as u16) / 2) as u8,
                ((a.b as u16 + b.b as u16) / 2) as u8,
                (a.a + b.a) / 2.0,
            ))),
            [Value::Color(a), Value::Color(b), Value::Number(w, _)] => {
                let weight = *w as f32 / 100.0;
                Ok(Value::Color(Color::rgba(
                    (a.r as f32 * (1.0 - weight) + b.r as f32 * weight) as u8,
                    (a.g as f32 * (1.0 - weight) + b.g as f32 * weight) as u8,
                    (a.b as f32 * (1.0 - weight) + b.b as f32 * weight) as u8,
                    a.a * (1.0 - weight) + b.a * weight,
                )))
            }
            _ => Err(SassError::Eval("mix 需要 2-3 个参数".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_simple() {
        let ast = Ast { nodes: vec![Node::Rule {
            selector: "a".into(),
            body: vec![Node::Decl {
                property: "color".into(),
                value: Value::String("red".into(), false),
                important: false,
            }],
        }]};
        let css = Evaluator::evaluate(&ast).unwrap();
        assert_eq!(css.len(), 1);
    }

    #[test]
    fn test_eval_variable() {
        let ast = Ast { nodes: vec![
            Node::Variable { name: "x".into(), value: Value::Number(10.0, Some("px".into())), flags: VarFlags::default() },
            Node::Decl { property: "w".into(), value: Value::Variable("x".into()), important: false },
        ]};
        let css = Evaluator::evaluate(&ast).unwrap();
        assert_eq!(css.len(), 1);
        if let CssNode::Declaration { value, .. } = &css[0] {
            assert_eq!(value, "10px");
        }
    }
}
