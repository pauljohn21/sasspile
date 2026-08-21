use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};

impl Evaluator {
    pub(crate) fn eval_include(
        name: &str,
        args: &[Arg],
        content: &Option<Vec<Node>>,
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_include", name = name, n_args = args.len());
        let _enter = span.enter();
        // meta.apply mixin——动态调用 mixin 引用
        if name == "meta.apply" {
            return Self::eval_meta_apply(args, content, env);
        }
        // meta.load-css mixin——动态加载模块 CSS
        if name == "meta.load-css" {
            return Self::eval_meta_load_css(args, env);
        }
        // 命名空间限定 mixin（如 midstream.b-a）
        if let Some(dot) = name.find('.') {
            let ns = &name[..dot];
            let mixin_name = &name[dot + 1..];
            if let Some(module) = env.get_namespace(ns)
                && let Some(mixin) = module.all_mixins().find(|(k, _)| *k == mixin_name).map(|(_, m)| m) {
                    return Self::exec_mixin(mixin, args, content, env);
                }
        }
        let mixin = env
            .get_mixin(name)
            .ok_or_else(|| SassError::UndefinedMixin(name.to_string()))?
            .clone();
        Self::exec_mixin(&mixin, args, content, env)
    }

    /// 执行 mixin——绑定参数、注入 @content、求值 body。
    pub(crate) fn exec_mixin(
        mixin: &MixinDef,
        args: &[Arg],
        content: &Option<Vec<Node>>,
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
// 绑定参数
let mixin_env = Self::bind_params(&mixin.params, args, env)?.incr_depth();
        // 合并 mixin 定义时捕获的命名空间
        let mut mixin_env = mixin_env;
        for (ns, exports) in &mixin.captured_namespaces {
            if !mixin_env.namespaces.contains_key(ns) {
                mixin_env.namespaces.insert(ns.clone(), exports.clone());
            }
            // 将命名空间模块中的函数注入到 mixin 环境，使 mixin 体可直接调用
            for (fname, fdef) in exports.all_functions() {
                if !mixin_env.local_functions.contains_key(fname) {
                    mixin_env = mixin_env.define_local_function(fname.clone(), fdef.clone());
                }
            }
        }
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

    pub(crate) fn bind_params(params: &[Param], args: &[Arg], env: &Env) -> Result<Env> {
        // 先求值所有参数，分离位置参数和关键字参数，展开 spread
        let mut positional: Vec<Value> = Vec::new();
        let mut keyword: HashMap<String, Value> = HashMap::new();
        for arg in args {
            let val = Self::eval_value(&arg.value, env)?;
            if arg.spread {
                // 展开 $args... 为多个参数
                match &val {
                    Value::List(items, _, _) => {
                        positional.extend(items.iter().cloned());
                    }
                    Value::Map(pairs) => {
                        // Map spread → 关键字参数（key=value 对）
                        for (k, v) in pairs {
                            if let Value::String(key, _) = k {
                                keyword.insert(key.clone(), v.clone());
                            }
                        }
                    }
                    _ => {
                        positional.push(val);
                    }
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
                new_env = new_env.bind(
                    param.name.clone(),
                    Value::List(rest, Separator::Comma, false),
                );
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
    pub(crate) fn call_function(name: &str, pos_args: &[Value], kw_args: &HashMap<String, Value>, env: &Env) -> Result<Value> {
        let span = crate::__tracing::info_span!("call_function", name = name, n_args = pos_args.len());
        let _enter = span.enter();
        // 用户函数
        if let Some(func) = env.get_function(name) {
            return Self::call_user_function(func, pos_args, kw_args, env);
        }
        // 在命名空间模块中查找同名函数（支持 mixin 体内调用同模块的私有函数）
        for (_, exports) in &env.namespaces {
            if let Some(func) = exports.all_functions().find(|(k, _)| *k == name).map(|(_, f)| f) {
                return Self::call_user_function(func, pos_args, kw_args, env);
            }
        }
        // 模块限定函数 (math.abs, map.get, etc.)
        if name.contains('.') {
            return Self::call_module_function(name, pos_args, kw_args, env);
        }
        // 内建函数
        Self::call_builtin(name, pos_args, kw_args, env)
    }

    pub(crate) fn call_user_function(
        func: &FunctionDef,
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
        env: &Env,
    ) -> Result<Value> {
        let span = crate::__tracing::info_span!(
            "call_user_function",
            n_params = func.params.len(),
            n_args = pos_args.len()
        );
        let _enter = span.enter();
        let mut func_env = env.incr_depth();
        // 合并函数定义时捕获的命名空间（使函数体可访问定义模块的 @use 命名空间）
        for (ns, exports) in &func.captured_namespaces {
            if !func_env.namespaces.contains_key(ns) {
                func_env.namespaces.insert(ns.clone(), exports.clone());
            }
        }
        let mut pos_idx = 0;
        for param in func.params.iter() {
            if param.rest {
                // 剩余参数——收集剩余位置参数
                let rest: Vec<Value> = pos_args[pos_idx..].to_vec();
                func_env = func_env.bind(param.name.clone(), Value::List(rest, Separator::Comma, false));
                break;
            }
            // 优先用关键字参数
            if let Some(val) = kw_args.get(&param.name) {
                func_env = func_env.bind(param.name.clone(), val.clone());
            } else if pos_idx < pos_args.len() {
                func_env = func_env.bind(param.name.clone(), pos_args[pos_idx].clone());
                pos_idx += 1;
            } else if let Some(default) = &param.default {
                let val = Self::eval_value(default, &func_env)?;
                func_env = func_env.bind(param.name.clone(), val);
            } else {
                func_env = func_env.bind(param.name.clone(), Value::Null);
            }
        }
        // 求值函数体，找 @return
        for node in &func.body {
            let (out, e) = Self::eval_node(node, &func_env)?;
            func_env = e;
            // 检查 Return 标记
            for css in &out {
                if let CssNode::Return(val) = css {
                    return Ok(val.clone());
                }
            }
        }
        Ok(Value::Null)
    }

    // —— @at-root ——
    pub(crate) fn eval_at_root(
        _query: &Option<String>,
        body: &[Node],
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let (css, new_env) = Self::eval_nodes(body, env)?;
        // 包装为 AtRoot，信号 eval_rule 不嵌套
        Ok((vec![CssNode::AtRoot(css)], new_env))
    }

    // —— @规则 ——
    pub(crate) fn eval_at_rule(
        name: &str,
        params: &Option<String>,
        body: &Option<Vec<Node>>,
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_at_rule", name = name, has_body = body.is_some());
        let _enter = span.enter();

        let (children, has_body) = match body {
            Some(nodes) => (Self::eval_nodes(nodes, env)?.0, true),
            None => (Vec::new(), false),
        };

        // 对 @media/@supports 参数做插值和表达式求值
        let eval_params = params
            .as_ref()
            .map(|p| Self::eval_at_params(name, p, env));

        // 当 @media/@supports/@container 在规则内部时，提升到外层：
        // 将声明包裹在当前选择器的规则中，嵌套规则保持原样（选择器已合并）。
        if matches!(name, "media" | "supports" | "container")
            && let Some(sel) = env.get_selector()
                && !sel.is_empty() {
                    let mut new_children = Vec::new();
                    let mut current_decls = Vec::new();
                    for child in children {
                        match &child {
                            CssNode::Declaration { .. } => current_decls.push(child),
                            _ => {
                                if !current_decls.is_empty() {
                                    new_children.push(CssNode::Rule {
                                        selector: sel.to_string(),
                                        declarations: std::mem::take(&mut current_decls),
                                        children: vec![],
                                    });
                                }
                                new_children.push(child);
                            }
                        }
                    }
                    if !current_decls.is_empty() {
                        new_children.push(CssNode::Rule {
                            selector: sel.to_string(),
                            declarations: current_decls,
                            children: vec![],
                        });
                    }
                    return Ok((
                        vec![CssNode::AtRule {
                            name: name.to_string(),
                            params: eval_params,
                            children: new_children,
                            has_body,
                        }],
                        env.clone(),
                    ));
                }

        Ok((
            vec![CssNode::AtRule {
                name: name.to_string(),
                params: eval_params,
                children,
                has_body,
            }],
            env.clone(),
        ))
    }

    // —— 辅助 ——
    pub(crate) fn is_truthy(v: &Value) -> bool {
        !matches!(v, Value::Bool(false) | Value::Null)
    }
}
