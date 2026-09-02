use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};

impl Evaluator {
    pub(crate) fn eval_include(
        name: &str,
        args: &[Arg],
        content: &Option<Vec<Node>>,
        env: Env,
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
            let ns_mixin = env.get_namespace(ns)
                .and_then(|module| module.all_mixins().find(|(k, _)| *k == mixin_name).map(|(_, m)| m.clone()));
            if let Some(mixin) = ns_mixin {
                return Self::exec_mixin(&mixin, args, content, env);
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
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        // @content 上下文快照——调用者环境（AGENTS.md 允许的 @content 例外）
        let content_env = env.clone();
        // 绑定参数——move env，消除 env.clone()
        let mut mixin_env = Self::bind_params(&mixin.params, args, env)?.incr_depth();
        // 合并 mixin 定义时捕获的命名空间
        for (ns, exports) in &mixin.captured_namespaces {
            if !mixin_env.get_namespace(ns).is_some() {
                mixin_env = mixin_env.add_namespace(ns.clone(), (**exports).clone());
            }
            // 将命名空间模块中的函数注入到 mixin 环境，使 mixin 体可直接调用
            for (fname, fdef) in exports.all_functions() {
                if mixin_env.get_function(fname).is_none() {
                    mixin_env = mixin_env.define_local_function(fname.clone(), fdef.clone());
                }
            }
        }
        // 注入 @content 块
        let mixin_env = if let Some(content_nodes) = content {
            mixin_env.set_content(content_nodes.clone(), content_env.clone())
        } else {
            mixin_env
        };
        // 求值 mixin body——move mixin_env，返回 css（env 丢弃，mixin 作用域不传播）
        let (css, _) = Self::eval_nodes(&mixin.body, mixin_env)?;
        // 返回 content_env 作为调用者 env（mixin 内部变量不泄漏到外层）
        Ok((css, content_env))
    }

    pub(crate) fn bind_params(params: &[Param], args: &[Arg], env: Env) -> Result<Env> {
        // 先求值所有参数，分离位置参数和关键字参数，展开 spread
        let mut positional: Vec<Value> = Vec::new();
        let mut keyword: HashMap<String, Value> = HashMap::new();
        for arg in args {
            let val = Self::eval_value(&arg.value, &env)?;
            if arg.spread {
                match &val {
                    Value::List(items, _, _) => {
                        positional.extend(items.iter().cloned());
                    }
                    Value::Map(pairs) => {
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

        // 直接 move env，链式 bind 绑定参数
        let mut new_env = env;
        let mut pos_idx = 0;
        for param in params.iter() {
            if param.rest {
                let rest: Vec<Value> = positional[pos_idx..].to_vec();
                new_env = new_env.bind(param.name.clone(), Value::List(rest, Separator::Comma, false));
                break;
            }
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
        let has_user_func = env.get_function(name).is_some();
        crate::__tracing::warn!(name = name, has_user_func, "call_function: checking user func");
        if let Some(func) = env.get_function(name) {
            return Self::call_user_function(func, pos_args, kw_args, env.clone());
        }
        // 在命名空间模块中查找同名函数
        for exports in env.get_namespaces().values() {
            if let Some(func) = exports.all_functions().find(|(k, _)| *k == name).map(|(_, f)| f) {
                return Self::call_user_function(func, pos_args, kw_args, env.clone());
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
        env: Env,
    ) -> Result<Value> {
        let span = crate::__tracing::info_span!(
            "call_user_function",
            n_params = func.params.len(),
            n_args = pos_args.len()
        );
        let _enter = span.enter();
        // 保存 local 表快照（函数作用域不传播）
        let saved_local_vars = env.get_local_vars().clone();
        let saved_local_mixins = env.get_local_mixins().clone();
        let saved_local_functions = env.get_local_functions().clone();
        let saved_forwarded_vars = env.get_forwarded_vars().clone();
        let saved_forwarded_mixins = env.get_forwarded_mixins().clone();
        let saved_forwarded_functions = env.get_forwarded_functions().clone();

        let mut func_env = env.incr_depth();
        // 合并函数定义时捕获的命名空间
        for (ns, exports) in &func.captured_namespaces {
            if func_env.get_namespace(ns).is_none() {
                func_env = func_env.add_namespace(ns.clone(), (**exports).clone());
            }
        }
        let mut pos_idx = 0;
        for param in func.params.iter() {
            if param.rest {
                let rest: Vec<Value> = pos_args[pos_idx..].to_vec();
                func_env = func_env.bind(param.name.clone(), Value::List(rest, Separator::Comma, false));
                break;
            }
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
        let mut return_val = Value::Null;
        for node in &func.body {
            let (out, e) = Self::eval_node(node, func_env)?;
            func_env = e;
            for css in &out {
                if let CssNode::Return(val) = css {
                    return_val = val.clone();
                    break;
                }
            }
        }
        // exit_scope 恢复外层作用域（仅传播命名空间变量和 !global 变量）
        let _restored = func_env.exit_scope(
            saved_local_vars, saved_local_mixins, saved_local_functions,
            saved_forwarded_vars, saved_forwarded_mixins, saved_forwarded_functions,
        );
        Ok(return_val)
    }

    // —— @at-root ——
    // 官方文档：@at-root 默认只脱离 style rules（父选择器），保留 @media 等 at-rules。
    // query 参数控制行为：without: media → 脱离 @media；without: all → 脱离所有；with: rule → 只保留 style rules。
    // 实际的 query 解析和分流在 RuleBuilder::push 和 eval_at_rule 中完成。
    pub(crate) fn eval_at_root(
        query: &Option<String>,
        body: &[Node],
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_at_root", query = ?query);
        let _enter = span.enter();
        let (css, new_env) = Self::eval_nodes(body, env)?;
        Ok((vec![CssNode::AtRoot(css, query.clone())], new_env))
    }

    // —— @规则 ——
    pub(crate) fn eval_at_rule(
        name: &str,
        params: &Option<String>,
        body: &Option<Vec<Node>>,
        env: Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_at_rule", name = name, has_body = body.is_some());
        let _enter = span.enter();

        let (children, has_body, new_env) = match body {
            Some(nodes) => {
                // at-rule body 内允许声明（如 @font-face { font-family: ...; }）
                // 使用 @ 前缀标记 AtRule 来源，让 eval_rule 能区分
                let env = if env.current_selector.is_none() {
                    env.with_selector(format!("@{name}"))
                } else { env };
                let (css, e) = Self::eval_nodes(nodes, env)?;
                (css, true, e)
            }
            None => (Vec::new(), false, env),
        };

        // 对 @media/@supports 参数做插值和表达式求值
        let eval_params = params
            .as_ref()
            .map(|p| Self::eval_at_params(name, p, &new_env));

        // 分流 AtRoot 节点——需要提升到 at-rule 外面的内容
        // 官方语义：
        // - without: media/supports → 仅脱离匹配的 at-rule
        // - without: all → 脱离所有 at-rules 和 style rules
        // - with: rule → 排除所有 at-rules，只保留 style rules
        // partition: predicate=true → inside（保留），predicate=false → outside（提升）
        let (inside, outside): (Vec<CssNode>, Vec<CssNode>) = children.into_iter().partition(|node| {
            match node {
                CssNode::AtRoot(_, Some(q)) => {
                    // without: all → 脱离所有 at-rules
                    let without_all = q.contains("without: all") || q.contains("without:all");
                    // with: rule → 排除所有 at-rules
                    let with_rule = q.contains("with: rule") || q.contains("with:rule");
                    if without_all || with_rule {
                        return false; // 提升到外面
                    }
                    // without: media → 仅对 @media 生效
                    if matches!(name, "media") {
                        return !(q.contains("without: media") || q.contains("without:media"));
                    }
                    // without: supports → 仅对 @supports 生效
                    if matches!(name, "supports") {
                        return !(q.contains("without: supports") || q.contains("without:supports"));
                    }
                    // without: container → 仅对 @container 生效
                    if matches!(name, "container") {
                        return !(q.contains("without: container") || q.contains("without:container"));
                    }
                    true
                }
                _ => true,
            }
        });

        // 将 outside 中的 AtRoot nodes 展开——父选择器已在 RuleBuilder 中处理
        let outside_flat: Vec<CssNode> = outside.into_iter().flat_map(|node| {
            match node {
                CssNode::AtRoot(nodes, _) => {
                    // without: all 时不需要嵌套父选择器——直接提升到 root
                    // without: media 时需要嵌套在父选择器下
                    // 但 AtRoot nodes 内部已经包含了父选择器信息（在 eval_rule 中保留的）
                    // 这里直接展开即可——RuleBuilder 已处理了嵌套
                    nodes
                }
                other => vec![other],
            }
        }).collect();

        // 当 @media/@supports/@container 在规则内部时，提升到外层
        if matches!(name, "media" | "supports" | "container")
            && let Some(sel) = new_env.get_selector()
                && !sel.is_empty() {
                    let mut result = outside_flat;
                    // @media/@supports/@container 空块不保留
                    if !inside.is_empty() {
                        result.push(CssNode::AtRule {
                            name: name.to_string(),
                            params: eval_params,
                            children: inside,
                            has_body,
                        });
                    }
                    return Ok((result, new_env));
                }

        // @media/@supports/@container 空块不保留；其他 at-rule（如 @keyframes）保留空块
        // 对于非 @media 类的 at-rule（如 @keyframes），先输出 at-rule 再输出 outside 提升的内容
        let skip_empty = matches!(name, "media" | "supports" | "container");
        let is_media_like = skip_empty;
        let result = if is_media_like {
            // @media 类：先 outside 后 at-rule（但上面已经 return 了，这里不会到）
            let mut r = outside_flat;
            if !skip_empty || !inside.is_empty() {
                r.push(CssNode::AtRule {
                    name: name.to_string(),
                    params: eval_params,
                    children: inside,
                    has_body,
                });
            }
            r
        } else {
            // 非 @media 类（如 @keyframes）：先 at-rule 后 outside
            let mut r = vec![CssNode::AtRule {
                name: name.to_string(),
                params: eval_params,
                children: inside,
                has_body,
            }];
            r.extend(outside_flat);
            r
        };
        Ok((result, new_env))
    }

    // —— 辅助 ——
    pub(crate) fn is_truthy(v: &Value) -> bool {
        !matches!(v, Value::Bool(false) | Value::Null)
    }
}
