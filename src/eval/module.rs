//! 模块系统——@use / @forward / @import 的文件加载和求值。
//!
//! - `@use`：加载文件为命名空间模块，变量/mixin/函数不泄漏到调用者作用域
//! - `@forward`：加载文件并转发其成员到调用者的导出
//! - `@import`：加载文件并内联合并到当前作用域（旧语法兼容）

use crate::error::{Result, SassError};
use crate::eval::value::Value;
use crate::eval::env::{Env, ModuleExports};
use crate::css::CssNode;
use crate::parse::ast::ConfigVar;
use crate::source::Source;
use crate::parse::Parser;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::file_resolver::resolve_file;
use super::{eval_nodes, eval_value};

/// @use 指令处理。
pub fn eval_use(
    url: &str,
    namespace: &Option<String>,
    star: bool,
    config: &[ConfigVar],
    env: Env,
) -> Result<(Option<Vec<CssNode>>, Env)> {
    // 内建模块 sass:math/string/list/map/color/meta/selector
    if url.starts_with("sass:") {
        return Ok((None, env));
    }

    let base = env.base_path.clone();
    let load_paths = env.load_paths.clone();

    let path = match resolve_file(base.as_ref(), url, &load_paths) {
        Some(p) => p,
        None => return Ok((None, env)),
    };

    let already_loaded = env.loaded_modules.contains(&path);

    // 循环加载检测
    if already_loaded && !env.module_cache.contains_key(&path) {
        return Err(SassError::eval("Module loop: this module is already being loaded."));
    }

    // 已加载且带 config → 报错
    if already_loaded && !config.is_empty() {
        return Err(SassError::eval(
            "This module was already loaded, so it can't be configured using \"with\".",
        ));
    }

    let exports = if already_loaded {
        env.module_cache.get(&path).cloned().unwrap_or_default()
    } else {
        // 求值 config 变量
        let config_pairs: Vec<(String, Value)> = config
            .iter()
            .map(|c| {
                let val = eval_value(&c.value, &env);
                (c.name.clone(), val)
            })
            .collect();
        load_module(&path, &config_pairs, &env)?
    };

    // CSS 输出（首次加载才输出）
    let css = if already_loaded {
        None
    } else if exports.css.is_empty() {
        None
    } else {
        Some(exports.css.clone())
    };

    // 更新模块缓存
    let mut updated_cache = (*env.module_cache).clone();
    if !already_loaded {
        updated_cache.insert(path.clone(), ModuleExports { css: vec![], ..exports.clone() });
    }
    let env = Env {
        module_cache: Rc::new(updated_cache),
        ..env
    };

    // 标记为已加载
    if !already_loaded {
        let mut loaded = (*env.loaded_modules).clone();
        loaded.push(path.clone());
        let env = Env {
            loaded_modules: Rc::new(loaded),
            ..env
        };
        return finish_use(env, &exports, star, namespace, url, css);
    }

    finish_use(env, &exports, star, namespace, url, css)
}

fn finish_use(
    env: Env,
    exports: &ModuleExports,
    star: bool,
    namespace: &Option<String>,
    url: &str,
    css: Option<Vec<CssNode>>,
) -> Result<(Option<Vec<CssNode>>, Env)> {
    if star {
        // @use as * — 全局导入
        let env = bind_exports_global(env, exports);
        Ok((css, env))
    } else {
        // 计算命名空间
        let ns = namespace.clone().unwrap_or_else(|| {
            let url_stem = Path::new(url)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(url);
            let base = url_stem.split('.').next().unwrap_or(url_stem);
            base.trim_start_matches('_').to_string()
        });
        let env = env.define_namespace(ns, Rc::new(exports.clone()));
        Ok((css, env))
    }
}

/// @forward 指令处理。
pub fn eval_forward(
    url: &str,
    prefix: &Option<String>,
    config: &[ConfigVar],
    env: Env,
    show: &[String],
    hide: &[String],
) -> Result<(Option<Vec<CssNode>>, Env)> {
    let base = env.base_path.clone();
    let load_paths = env.load_paths.clone();

    let path = match resolve_file(base.as_ref(), url, &load_paths) {
        Some(p) => p,
        None => return Err(SassError::eval("Can't find stylesheet to import.")),
    };

    let already_loaded = env.loaded_modules.contains(&path);

    if already_loaded && !env.module_cache.contains_key(&path) {
        return Err(SassError::eval("Module loop: this module is already being loaded."));
    }

    if already_loaded && !config.is_empty() {
        return Err(SassError::eval(
            "This module was already loaded, so it can't be configured using \"with\".",
        ));
    }

    let exports = if already_loaded {
        env.module_cache.get(&path).cloned().unwrap_or_default()
    } else {
        let config_pairs: Vec<(String, Value)> = config
            .iter()
            .map(|c| {
                let val = eval_value(&c.value, &env);
                (c.name.clone(), val)
            })
            .collect();
        load_module(&path, &config_pairs, &env)?
    };

    let css = if already_loaded {
        None
    } else if exports.css.is_empty() {
        None
    } else {
        Some(exports.css.clone())
    };

    // 更新缓存
    let mut updated_cache = (*env.module_cache).clone();
    if !already_loaded {
        updated_cache.insert(path.clone(), ModuleExports { css: vec![], ..exports.clone() });
    }
    let env = Env {
        module_cache: Rc::new(updated_cache),
        ..env
    };

    if !already_loaded {
        let mut loaded = (*env.loaded_modules).clone();
        loaded.push(path);
        let env = Env {
            loaded_modules: Rc::new(loaded),
            ..env
        };
        // 将导出的成员转发到当前环境
        let env = bind_exports_forward(env, &exports, prefix.as_deref(), show, hide);
        return Ok((css, env));
    }

    let env = bind_exports_forward(env, &exports, prefix.as_deref(), show, hide);
    Ok((css, env))
}

/// @import 指令处理——内联模式。
pub fn eval_import(url: &str, modifier: &str, env: Env) -> Result<(Option<Vec<CssNode>>, Env)> {
    // sass: 内建模块
    if url.starts_with("sass:") {
        return Ok((None, env));
    }

    // CSS @import 透传（带 modifier 或 .css 扩展名）
    if !modifier.is_empty() || url.ends_with(".css") {
        let css = vec![CssNode::AtRule {
            name: "import".to_string(),
            params: format!("\"{url}\" {modifier}").trim().to_string(),
            children: vec![],
            has_body: false,
        }];
        return Ok((Some(css), env));
    }

    let base = env.base_path.clone();
    let load_paths = env.load_paths.clone();

    let path = match resolve_file(base.as_ref(), url, &load_paths) {
        Some(p) => p,
        None => {
            // 找不到文件——输出为 CSS @import
            let css = vec![CssNode::AtRule {
                name: "import".to_string(),
                params: format!("\"{url}\""),
                children: vec![],
                has_body: false,
            }];
            return Ok((Some(css), env));
        }
    };

    // @import 内联加载——继承当前环境
    load_import(&path, env)
}

/// 加载文件模块——读取、词法分析、语法分析、求值，返回导出。
fn load_module(
    path: &Path,
    config: &[(String, Value)],
    caller_env: &Env,
) -> Result<ModuleExports> {
    if caller_env.depth > 50 {
        return Ok(ModuleExports::default());
    }

    let source_text = std::fs::read_to_string(path)
        .map_err(|e| SassError::eval(format!("Cannot read {}: {e}", path.display())))?;

    let is_plain_css = path.extension().and_then(|e| e.to_str()) == Some("css");

    let lexed = Source::new(&source_text).lex()?;
    let ast = Parser::new(lexed.tokens).parse_body()?;

    // 创建模块环境
    let mut env = Env {
        base_path: Some(path.to_path_buf()),
        load_paths: caller_env.load_paths.clone(),
        depth: caller_env.depth + 1,
        plain_css: is_plain_css,
        module_cache: caller_env.module_cache.clone(),
        loaded_modules: {
            let mut loaded = (*caller_env.loaded_modules).clone();
            loaded.push(path.to_path_buf());
            Rc::new(loaded)
        },
        ..Env::root(None, Vec::new())
    };

    // 注入 with() 配置变量
    for (name, value) in config {
        if !matches!(value, Value::Null) {
            env = env.define_var(&name.replace('-', "_"), value.clone());
        }
    }

    let (module_css, final_env) = eval_nodes(&ast, env)?;

    let css = if is_plain_css {
        vec![CssNode::AtRoot(module_css)]
    } else {
        module_css
    };

    Ok(ModuleExports {
        variables: final_env.local_vars,
        mixins: final_env.local_mixins,
        functions: final_env.local_functions,
        css,
    })
}

/// 加载 @import 文件——内联模式：继承当前环境的所有成员。
fn load_import(path: &Path, caller_env: Env) -> Result<(Option<Vec<CssNode>>, Env)> {
    if caller_env.depth > 50 {
        return Ok((None, caller_env));
    }

    let source_text = std::fs::read_to_string(path)
        .map_err(|e| SassError::eval(format!("Cannot read {}: {e}", path.display())))?;

    let is_plain_css = path.extension().and_then(|e| e.to_str()) == Some("css");

    let lexed = Source::new(&source_text).lex()?;
    let ast = Parser::new(lexed.tokens).parse_body()?;

    // 继承当前环境
    let saved_base_path = caller_env.base_path.clone();
    let saved_depth = caller_env.depth;
    let env = Env {
        base_path: Some(path.to_path_buf()),
        depth: caller_env.depth + 1,
        plain_css: is_plain_css,
        ..caller_env
    };

    let (css, mut final_env) = eval_nodes(&ast, env)?;

    final_env.base_path = saved_base_path;
    final_env.depth = saved_depth;
    final_env.plain_css = false;

    let css = if is_plain_css {
        vec![CssNode::AtRoot(css)]
    } else {
        css
    };

    Ok((Some(css), final_env))
}

/// @use as * — 将模块导出全部绑定到当前环境。
fn bind_exports_global(env: Env, exports: &ModuleExports) -> Env {
    let mut env = env;
    for (k, v) in &exports.variables {
        env = env.define_var(k, v.clone());
    }
    for (_, m) in &exports.mixins {
        env = env.define_mixin(m.clone());
    }
    for (_, f) in &exports.functions {
        env = env.define_function(f.clone());
    }
    env
}

/// @forward — 将模块导出转发到当前环境（带 show/hide/prefix 过滤）。
fn bind_exports_forward(
    env: Env,
    exports: &ModuleExports,
    prefix: Option<&str>,
    show: &[String],
    hide: &[String],
) -> Env {
    let mut env = env;
    let filter_active = !show.is_empty() || !hide.is_empty();

    for (k, v) in &exports.variables {
        if filter_active && !show.is_empty() && !show.contains(k) {
            continue;
        }
        if filter_active && hide.contains(k) {
            continue;
        }
        let name = if let Some(pfx) = prefix {
            format!("{pfx}{k}")
        } else {
            k.clone()
        };
        env = env.define_var(&name, v.clone());
    }

    for (name, m) in &exports.mixins {
        if filter_active && !show.is_empty() && !show.contains(name) {
            continue;
        }
        if filter_active && hide.contains(name) {
            continue;
        }
        let new_name = if let Some(pfx) = prefix {
            format!("{pfx}{name}")
        } else {
            name.clone()
        };
        let mut m = m.clone();
        m.name = new_name.clone();
        env = env.define_mixin(m);
    }

    for (name, f) in &exports.functions {
        if filter_active && !show.is_empty() && !show.contains(name) {
            continue;
        }
        if filter_active && hide.contains(name) {
            continue;
        }
        let new_name = if let Some(pfx) = prefix {
            format!("{pfx}{name}")
        } else {
            name.clone()
        };
        let mut f = f.clone();
        f.name = new_name.clone();
        env = env.define_function(f);
    }

    env
}
