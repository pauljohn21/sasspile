use super::*;
use crate::error::{Result, SassError};
use std::path::{Path, PathBuf};

impl Evaluator {
    pub(crate) fn resolve_file(
        base: Option<&PathBuf>,
        url: &str,
        load_paths: &[PathBuf],
    ) -> Option<PathBuf> {
        let span = tracing::debug_span!("resolve_file", url = url);
        let _enter = span.enter();
        let base_dir = base
            .as_ref()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        // 先尝试相对于当前文件目录解析
        if let Some(path) = Self::try_resolve_dir(&base_dir, url) {
            return Some(path);
        }
        // 回退到 load paths
        for lp in load_paths {
            if let Some(path) = Self::try_resolve_dir(lp, url) {
                return Some(path);
            }
        }
        None
    }

    /// 在指定目录下尝试解析 url 对应的文件。
    fn try_resolve_dir(dir: &Path, url: &str) -> Option<PathBuf> {
        let url_path = std::path::Path::new(url);
        let parent = url_path.parent().unwrap_or(std::path::Path::new(""));
        let filename = url_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| url.to_string());
        let candidates = [
            dir.join(parent).join(format!("_{filename}.scss")),
            dir.join(parent).join(format!("{filename}.scss")),
            dir.join(parent).join(format!("_{filename}.sass")),
            dir.join(parent).join(format!("{filename}.sass")),
            dir.join(parent).join(format!("_{filename}.css")),
            dir.join(parent).join(format!("{filename}.css")),
            dir.join(parent).join(format!("_{filename}.import.scss")),
            dir.join(parent).join(format!("{filename}.import.scss")),
            dir.join(url).join("_index.scss"),
            dir.join(url).join("index.scss"),
            dir.join(url).join("_index.sass"),
            dir.join(url).join("index.sass"),
        ];
        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }
        None
    }

    /// 加载文件模块——读取、词法分析、语法分析、求值，返回导出。
    pub(crate) fn load_module(
        path: &Path,
        config: &[(String, Value)],
        caller_env: &Env,
    ) -> Result<ModuleExports> {
        let span = tracing::info_span!("load_module", path = %path.display(), depth = caller_env.depth, n_config = config.len());
        let _enter = span.enter();
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
        let mut env = Env::default()
            .with_base_path(path.to_path_buf())
            .with_load_paths(caller_env.get_load_paths().to_vec());
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
    pub(crate) fn call_module_function(name: &str, args: &[Value], env: &Env) -> Result<Value> {
        let span = tracing::info_span!("call_module_function", name = name);
        let _enter = span.enter();
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
            "math.log" => "log",
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
            "string.split" => "str-split",
            "string.unique-id" => "unique-id",
            // sass:map
            "map.get" => "map-get",
            "map.merge" => "map-merge",
            "map.remove" => "map-remove",
            "map.keys" => "map-keys",
            "map.values" => "map-values",
            "map.has-key" => "map-has-key",
            "map.deep-remove" => "map-deep-remove",
            "map.deep-merge" => "map-deep-merge",
            "map.set" => "map-set",
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
            "list.slash" => "list-slash",
            // sass:color
            "color.adjust" => "adjust-color",
            "color.change" => "change-color",
            "color.scale" => "scale-color",
            "color.mix" => "mix",
            "color.invert" => "invert",
            "color.grayscale" => "grayscale",
            "color.complement" => "complement",
            "color.channel" => "color-channel",
            "color.hwb" => "hwb",
            "color.hsl" => "hsl",
            "color.hsla" => "hsla",
            "color.adjust-hue" => "adjust-hue",
            "color.saturate" => "saturate",
            "color.desaturate" => "desaturate",
            "color.transparentize" => "transparentize",
            "color.fade-out" => "fade-out",
            "color.opacify" => "opacify",
            "color.fade-in" => "fade-in",
            "color.alpha" => "alpha",
            "color.opacity" => "opacity",
            "color.red" => "red",
            "color.green" => "green",
            "color.blue" => "blue",
            "color.hue" => "hue",
            "color.saturation" => "saturation",
            "color.lightness" => "lightness",
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
}
