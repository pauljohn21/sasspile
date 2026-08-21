//! meta 模块的高级操作——`meta.apply`、`meta.load-css` mixin 及 `meta.get-mixin`、
//! `meta.module-functions`/`meta.module-mixins`/`meta.module-variables` 反射函数。

use super::*;
use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::parse::ast::{MixinRefData, Node};

impl Evaluator {
    /// `meta.apply($mixin, $args...)` mixin——动态调用 mixin 引用。
    ///
    /// 接收 `MixinRef` 作为第一个参数，剩余参数传递给目标 mixin。
    /// 支持 `@content` 传递。
    pub(crate) fn eval_meta_apply(
        args: &[Arg],
        content: &Option<Vec<Node>>,
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_meta_apply", n_args = args.len());
        let _enter = span.enter();

        if args.is_empty() {
            return Err(SassError::Eval("Missing argument $mixin.".into()));
        }

        // 求值第一个参数（mixin 引用）
        let mixin_ref_val = Self::eval_value(&args[0].value, env)?;
        let mixin_ref = match &mixin_ref_val {
            Value::MixinRef(data) => data.clone(),
            _ => {
                return Err(SassError::Eval(format!(
                    "$mixin: {} is not a mixin reference.",
                    mixin_ref_val
                )));
            }
        };

        // 收集剩余参数
        let remaining_args = &args[1..];

        // 从 MixinRefData 构造 MixinDef 并执行
        let mixin_def = MixinDef {
            params: mixin_ref.params.clone(),
            body: mixin_ref.body.clone(),
            captured_namespaces: Self::resolve_captured_namespaces(
                &mixin_ref.captured_ns_keys,
                env,
            ),
        };

        Self::exec_mixin(&mixin_def, remaining_args, content, env)
    }

    /// `meta.load-css($module, $with: ())` mixin——动态加载模块 CSS。
    ///
    /// 加载指定模块并将其 CSS 输出注入当前上下文。
    pub(crate) fn eval_meta_load_css(
        args: &[Arg],
        env: &Env,
    ) -> Result<(Vec<CssNode>, Env)> {
        let span = crate::__tracing::info_span!("eval_meta_load_css", n_args = args.len());
        let _enter = span.enter();

        if args.is_empty() {
            return Err(SassError::Eval("Missing argument $module.".into()));
        }

        // 求值模块名参数
        let module_val = Self::eval_value(&args[0].value, env)?;
        let module_name = match &module_val {
            Value::String(s, _) => s.clone(),
            _ => {
                return Err(SassError::Eval(format!(
                    "$module: {} is not a string.",
                    module_val
                )));
            }
        };

        // 求值 $with 配置参数（可选）
        let with_config: Vec<(String, Value)> = if args.len() > 1 {
            let with_val = Self::eval_value(&args[1].value, env)?;
            match &with_val {
                Value::Map(pairs) => pairs
                    .iter()
                    .filter_map(|(k, v)| {
                        if let Value::String(key, _) = k {
                            Some((key.clone(), v.clone()))
                        } else {
                            None
                        }
                    })
                    .collect(),
                Value::Null => vec![],
                _ => {
                    return Err(SassError::Eval(format!(
                        "$with: {} is not a map.",
                        with_val
                    )));
                }
            }
        } else {
            vec![]
        };

        // 内建模块（sass:math 等）——无 CSS 输出
        if module_name.starts_with("sass:") {
            return Ok((vec![], env.clone()));
        }

        // 解析文件路径
        let base = env.base_path.as_ref();
        let load_paths = env.get_load_paths();
        let path = Self::resolve_file(base, &module_name, load_paths)
            .ok_or_else(|| SassError::Module(format!("Can't find stylesheet to import: {module_name}")))?;

        // 检查是否已加载
        if env.loaded_modules.contains(&path) {
            return Ok((vec![], env.clone()));
        }

        // 加载模块
        let exports = Self::load_module(&path, &with_config, env)?;
        let css = exports.css.clone();
        let env_with_cache = merge_module_cache(env, &path, &exports);

        Ok((css, env_with_cache))
    }

    /// 从捕获的命名空间键列表和当前环境解析命名空间映射。
    fn resolve_captured_namespaces(
        ns_keys: &[String],
        env: &Env,
    ) -> HashMap<String, Rc<ModuleExports>> {
        let mut result = HashMap::new();
        for key in ns_keys {
            if let Some(exports) = env.namespaces.get(key) {
                result.insert(key.clone(), exports.clone());
            }
        }
        result
    }

    /// `meta.get-mixin($name, $module: null)` 函数——返回 mixin 引用。
    pub(crate) fn meta_get_mixin(
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
        env: &Env,
    ) -> Result<Value> {
        let span = crate::__tracing::info_span!("meta_get_mixin");
        let _enter = span.enter();

        let name_arg = pos_args.first()
            .or_else(|| kw_args.get("name"))
            .or_else(|| kw_args.get("$name"));
        let module_arg = pos_args.get(1)
            .or_else(|| kw_args.get("module"))
            .or_else(|| kw_args.get("$module"));
        let name = match name_arg {
            Some(Value::String(s, _)) => s.clone(),
            Some(v) => return Err(SassError::Eval(format!("$name: {v} is not a string."))),
            None => return Err(SassError::Eval("Missing argument $name.".into())),
        };
        let module_ns: Option<String> = match module_arg {
            Some(Value::String(s, _)) => Some(s.clone()),
            Some(Value::Null) | None => None,
            Some(v) => return Err(SassError::Eval(format!("$module: {v} is not a string."))),
        };
        // dash-insensitive 查找：- 和 _ 等价
        let lookup_name = name.replace('-', "_");
        // 先在模块命名空间中查找
        if let Some(ns) = &module_ns
            && let Some(module) = env.get_namespace(ns)
            && let Some(mixin) = module.all_mixins().find(|(k, _)| *k == &lookup_name || *k == &name).map(|(_, m)| m)
        {
            let ns_keys: Vec<String> = mixin.captured_namespaces.keys().cloned().collect();
            return Ok(Value::MixinRef(std::rc::Rc::new(
                MixinRefData {
                    name: name.clone(),
                    module: module_ns.clone(),
                    params: mixin.params.clone(),
                    body: mixin.body.clone(),
                    captured_ns_keys: ns_keys,
                }
            )));
        }
        // 全局查找
        let lookup_variants = [name.as_str(), &lookup_name, &name.replace('_', "-")];
        for variant in &lookup_variants {
            if let Some((params, body, ns_keys)) = env.get_mixin_ref_data(variant) {
                return Ok(Value::MixinRef(std::rc::Rc::new(
                    MixinRefData {
                        name: name.clone(),
                        module: None,
                        params,
                        body,
                        captured_ns_keys: ns_keys,
                    }
                )));
            }
        }
        Err(SassError::Eval(format!("There is no mixin named \"{name}\"")))
    }

    /// `meta.module-functions($module)` 函数——返回模块函数 map。
    pub(crate) fn meta_module_functions(
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
        env: &Env,
    ) -> Result<Value> {
        let span = crate::__tracing::info_span!("meta_module_functions");
        let _enter = span.enter();

        let ns_name = Self::extract_module_arg(pos_args, kw_args)?;
        let module = env.get_namespace(&ns_name)
            .ok_or_else(|| SassError::Eval(format!("There is no module with namespace \"{ns_name}\".")))?;
        let pairs: Vec<(Value, Value)> = module.all_functions()
            .map(|(name, _)| {
                (Value::String(name.clone(), true), Value::String(name.clone(), false))
            })
            .collect();
        Ok(Value::Map(pairs))
    }

    /// `meta.module-mixins($module)` 函数——返回模块 mixin map。
    pub(crate) fn meta_module_mixins(
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
        env: &Env,
    ) -> Result<Value> {
        let span = crate::__tracing::info_span!("meta_module_mixins");
        let _enter = span.enter();

        let ns_name = Self::extract_module_arg(pos_args, kw_args)?;
        let module = env.get_namespace(&ns_name)
            .ok_or_else(|| SassError::Eval(format!("There is no module with namespace \"{ns_name}\".")))?;
        let pairs: Vec<(Value, Value)> = module.all_mixins()
            .map(|(name, mixin)| {
                let ns_keys: Vec<String> = mixin.captured_namespaces.keys().cloned().collect();
                let ref_data = MixinRefData {
                    name: name.clone(),
                    module: Some(ns_name.clone()),
                    params: mixin.params.clone(),
                    body: mixin.body.clone(),
                    captured_ns_keys: ns_keys,
                };
                (Value::String(name.clone(), true), Value::MixinRef(std::rc::Rc::new(ref_data)))
            })
            .collect();
        Ok(Value::Map(pairs))
    }

    /// `meta.module-variables($module)` 函数——返回模块变量 map。
    pub(crate) fn meta_module_variables(
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
        env: &Env,
    ) -> Result<Value> {
        let span = crate::__tracing::info_span!("meta_module_variables");
        let _enter = span.enter();

        let ns_name = Self::extract_module_arg(pos_args, kw_args)?;
        let module = env.get_namespace(&ns_name)
            .ok_or_else(|| SassError::Eval(format!("There is no module with namespace \"{ns_name}\".")))?;
        let pairs: Vec<(Value, Value)> = module.all_vars()
            .filter(|(name, _)| !name.starts_with('_'))
            .map(|(name, val)| (Value::String(name.clone(), true), val.clone()))
            .collect();
        Ok(Value::Map(pairs))
    }

    /// `meta.accepts-content($mixin)` 函数——检查 mixin 是否接受 @content。
    ///
    /// 遍历 mixin body 检查是否包含 `Node::Content` 节点。
    pub(crate) fn meta_accepts_content(
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
        _env: &Env,
    ) -> Result<Value> {
        let span = crate::__tracing::info_span!("meta_accepts_content");
        let _enter = span.enter();

        let mixin_arg = pos_args.first()
            .or_else(|| kw_args.get("mixin"))
            .or_else(|| kw_args.get("$mixin"));
        let mixin_ref = match mixin_arg {
            Some(Value::MixinRef(data)) => data.clone(),
            Some(v) => return Err(SassError::Eval(format!(
                "$mixin: {v} is not a mixin reference."
            ))),
            None => return Err(SassError::Eval("Missing argument $mixin.".into())),
        };

        // 检查 mixin body 中是否包含 Node::Content
        let accepts = body_has_content(&mixin_ref.body);
        Ok(Value::Bool(accepts))
    }

    /// 从参数中提取 $module 字符串。
    fn extract_module_arg(
        pos_args: &[Value],
        kw_args: &HashMap<String, Value>,
    ) -> Result<String> {
        let module_arg = pos_args.first()
            .or_else(|| kw_args.get("module"))
            .or_else(|| kw_args.get("$module"));
        match module_arg {
            Some(Value::String(s, _)) => Ok(s.clone()),
            Some(v) => Err(SassError::Eval(format!("$module: {v} is not a string."))),
            None => Err(SassError::Eval("Missing argument $module.".into())),
        }
    }
}

/// 合并模块缓存和 @extend 关系。
fn merge_module_cache(env: &Env, path: &std::path::Path, exports: &ModuleExports) -> Env {
    let mut new_loaded = (*env.loaded_modules).clone();
    new_loaded.insert(path.to_path_buf());
    new_loaded.extend((*exports.loaded_modules).clone().iter().cloned());
    let mut new_extends = (*env.extends).clone();
    new_extends.extend((*exports.extends).clone().iter().cloned());
    Env {
        loaded_modules: Rc::new(new_loaded),
        extends: Rc::new(new_extends),
        module_cache: exports.module_cache.clone(),
        ..env.clone()
    }
}

/// 递归检查 mixin body 中是否包含 `Node::Content` 节点。
/// 用于 `meta.accepts-content` 判断 mixin 是否接受 @content 块。
fn body_has_content(nodes: &[Node]) -> bool {
    nodes.iter().any(|n| match n {
        Node::Content => true,
        // 递归检查嵌套的控制流和规则体
        Node::If { branches, else_body } => {
            branches.iter().any(|(_, body)| body_has_content(body))
                || else_body.as_ref().is_some_and(|eb| body_has_content(eb))
        }
        Node::For { body, .. } => body_has_content(body),
        Node::Each { body, .. } => body_has_content(body),
        Node::While { body, .. } => body_has_content(body),
        Node::AtRoot { body, .. } => body_has_content(body),
        Node::Include { content, .. } => content.as_ref().is_some_and(|c| body_has_content(c)),
        _ => false,
    })
}
