//! # sasspile
//!
//! 纯 Rust 函数式 SCSS 编译器，从零实现。
//!
//! ## 架构
//!
//! ```text
//! Source → Lexer → Parser → Evaluator → Serializer → CSS
//!          (lex/)   (parse/)  (eval/)     (css/)
//! ```
//!
//! 每个阶段通过类型状态机（Type-State Pattern）确保编译时类型安全：
//! 必须先词法分析后语法分析，先语法分析后求值，先求值后序列化。
//!
//! ## 快速开始
//!
//! ```rust
//! use sasspile::compile_expanded;
//!
//! let css = compile_expanded("a { color: red; }").unwrap();
//! assert_eq!(css, "a {\n  color: red;\n}\n");
//! ```
//!
//! ## 变量与嵌套
//!
//! ```rust
//! use sasspile::compile_expanded;
//!
//! let scss = r#"
//!     $primary: #3498db;
//!     .btn {
//!         background: $primary;
//!         &:hover { background: darken($primary, 10%); }
//!     }
//! "#;
//! let css = compile_expanded(scss).unwrap();
//! assert!(css.contains("#3498db"));
//! assert!(css.contains(".btn:hover"));
//! ```
//!
//! ## 压缩输出
//!
//! ```rust
//! use sasspile::compile_compressed;
//!
//! let css = compile_compressed("a { color: red; }").unwrap();
//! assert_eq!(css, "a{color:red;}");
//! ```
//!
//! ## 文件编译
//!
//! ```rust,no_run
//! use sasspile::{compile_file, OutputStyle};
//! use std::path::PathBuf;
//!
//! let css = compile_file(&PathBuf::from("input.scss"), OutputStyle::Expanded).unwrap();
//! println!("{}", css);
//! ```
//!
//! ## 支持的 SCSS 特性
//!
//! - 变量（`$var`）、`!default`、`!important`
//! - 嵌套规则、父选择器引用（`&`）
//! - Mixin（`@mixin`/`@include`/`@content`）
//! - 用户函数（`@function`/`@return`）
//! - 控制流（`@if`/`@for`/`@each`/`@while`）
//! - 模块系统（`@use`/`@forward`/`@import`）
//! - 继承（`@extend`）
//! - 内建函数：颜色、字符串、列表、Map、数学、选择器
//!
//! ## 兼容性
//!
//! - Bootstrap 5.3.8：全量编译通过 ✅
//! - Element Plus：121/121 (100%) 全量通过 ✅
//! - sass-spec：1843/5069 (36%)

pub mod css;
pub mod error;
pub mod eval;
pub mod lex;
pub mod parse;
pub mod stage;

pub use error::{Result, SassError, Span};
pub use eval::Evaluator;
pub use lex::Lexer;
pub use parse::{Parser, ast::Ast};
pub use stage::source::Source;

use std::path::PathBuf;

/// 初始化 tracing 日志——用 `RUST_LOG` 环境变量控制级别和 target。
///
/// # Target 过滤
///
/// ```bash
/// # 只看颜色相关 events
/// RUST_LOG="sasspile::color=debug" cargo test -- --nocapture
///
/// # 组合多个 target
/// RUST_LOG="sasspile::color=trace,sasspile::extend=info" cargo test -- --nocapture
/// ```
pub fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let _ = fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_ansi(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .try_init();
}

/// 输出风格。
///
/// 控制编译器生成的 CSS 格式。
///
/// # 示例
///
/// ```rust
/// use sasspile::{compile, OutputStyle};
///
/// let expanded = compile("a { color: red; }", OutputStyle::Expanded).unwrap();
/// assert_eq!(expanded, "a {\n  color: red;\n}\n");
///
/// let compressed = compile("a { color: red; }", OutputStyle::Compressed).unwrap();
/// assert_eq!(compressed, "a{color:red;}");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStyle {
    /// 展开式——带缩进和换行。
    Expanded,
    /// 压缩式——无空白。
    Compressed,
}

/// 编译 SCSS 源码为 CSS 字符串。
///
/// # 参数
///
/// - `input`: SCSS 源码字符串
/// - `style`: 输出风格（展开式或压缩式）
///
/// # 示例
///
/// ```rust
/// use sasspile::{compile, OutputStyle};
///
/// let css = compile("a { color: red; }", OutputStyle::Expanded).unwrap();
/// assert!(css.contains("color: red"));
/// ```
///
/// # 错误
///
/// 返回 [`SassError`] 如果输入包含语法错误或求值错误。
pub fn compile(input: &str, style: OutputStyle) -> Result<String> {
    let source = Source::new(input.to_string());
    let lexed = source.lex()?;
    let parsed = lexed.parse()?;
    let evaluated = parsed.evaluate()?;
    let serialized = evaluated.serialize(style);
    Ok(serialized.into_string())
}

/// 编译 SCSS 为展开式 CSS。
///
/// 等价于 `compile(input, OutputStyle::Expanded)`。
///
/// # 示例
///
/// ```rust
/// use sasspile::compile_expanded;
///
/// let css = compile_expanded("a { color: red; }").unwrap();
/// assert_eq!(css, "a {\n  color: red;\n}\n");
/// ```
///
/// # 错误
///
/// 返回 [`SassError`] 如果输入包含语法错误或求值错误。
pub fn compile_expanded(input: &str) -> Result<String> {
    compile(input, OutputStyle::Expanded)
}

/// 编译 SCSS 为压缩式 CSS。
///
/// 等价于 `compile(input, OutputStyle::Compressed)`。
///
/// # 示例
///
/// ```rust
/// use sasspile::compile_compressed;
///
/// let css = compile_compressed("a { color: red; }").unwrap();
/// assert_eq!(css, "a{color:red;}");
/// ```
///
/// # 错误
///
/// 返回 [`SassError`] 如果输入包含语法错误或求值错误。
pub fn compile_compressed(input: &str) -> Result<String> {
    compile(input, OutputStyle::Compressed)
}

/// 编译 SCSS 文件为 CSS 字符串。
///
/// 从文件读取 SCSS 源码，编译为 CSS。`@use`/`@import` 的解析基于文件所在目录。
///
/// # 参数
///
/// - `path`: SCSS 文件路径
/// - `style`: 输出风格
///
/// # 示例
///
/// ```rust,no_run
/// use sasspile::{compile_file, OutputStyle};
/// use std::path::PathBuf;
///
/// let css = compile_file(&PathBuf::from("style.scss"), OutputStyle::Expanded).unwrap();
/// println!("{}", css);
/// ```
///
/// # 错误
///
/// 返回 [`SassError`] 如果文件不存在或编译失败。
pub fn compile_file(path: &PathBuf, style: OutputStyle) -> Result<String> {
    let input = std::fs::read_to_string(path)?;
    let source = Source::new(input);
    let lexed = source.lex()?;
    let parsed = lexed.parse()?;
    use crate::eval::Evaluator;
    let nodes = Evaluator::evaluate_with_path(&parsed.ast, path.clone())?;
    let serialized = crate::css::Serializer::serialize(&nodes, style);
    Ok(serialized)
}

/// 编译 SCSS 文件为 CSS 字符串（带加载路径）。
///
/// 加载路径用于解析 `@use`/`@import` 中无法从当前文件目录找到的模块。
///
/// # 参数
///
/// - `path`: SCSS 文件路径
/// - `style`: 输出风格
/// - `load_paths`: 额外的模块搜索路径
///
/// # 示例
///
/// ```rust,no_run
/// use sasspile::{compile_file_with_load_paths, OutputStyle};
/// use std::path::PathBuf;
///
/// let css = compile_file_with_load_paths(
///     &PathBuf::from("main.scss"),
///     OutputStyle::Expanded,
///     vec![PathBuf::from("./node_modules")],
/// ).unwrap();
/// println!("{}", css);
/// ```
///
/// # 错误
///
/// 返回 [`SassError`] 如果文件不存在或编译失败。
pub fn compile_file_with_load_paths(
    path: &PathBuf,
    style: OutputStyle,
    load_paths: Vec<PathBuf>,
) -> Result<String> {
    let input = std::fs::read_to_string(path)?;
    let source = Source::new(input);
    let lexed = source.lex()?;
    let parsed = lexed.parse()?;
    use crate::eval::Evaluator;
    let nodes = Evaluator::evaluate_with_path_and_load_paths(&parsed.ast, path.clone(), load_paths)?;
    let serialized = crate::css::Serializer::serialize(&nodes, style);
    Ok(serialized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple() {
        let css = compile_expanded("a { color: red; }").unwrap();
        assert_eq!(css, "a {\n  color: red;\n}\n");
    }

    #[test]
    fn test_compile_variable() {
        let css = compile_expanded("$w: 10px; a { width: $w; }").unwrap();
        assert!(css.contains("width: 10px"));
    }

    #[test]
    fn test_compile_nested() {
        let css = compile_expanded(".outer { color: red; .inner { color: blue; } }").unwrap();
        assert!(css.contains(".outer"));
        assert!(css.contains(".outer .inner"));
    }

    #[test]
    fn test_compile_amp() {
        let css = compile_expanded(".btn { &:hover { color: red; } }").unwrap();
        assert!(css.contains(".btn:hover"));
    }

    #[test]
    fn test_compile_if() {
        let css = compile_expanded("@if true { a { color: red; } }").unwrap();
        assert!(css.contains("color: red"));
    }

    #[test]
    fn test_compile_for() {
        let css = compile_expanded("@for $i from 1 through 3 { .col-#{$i} { width: $i * 100%; } }")
            .unwrap();
        assert!(css.contains("col-1"));
    }

    #[test]
    fn test_compile_mixin() {
        let css = compile_expanded("@mixin bold { font-weight: bold; } .title { @include bold; }")
            .unwrap();
        assert!(css.contains("font-weight: bold"));
    }

    #[test]
    fn test_compile_content() {
        let css = compile_expanded(
            "@mixin wrapper { .inner { @content; } } @include wrapper { color: red; }",
        )
        .unwrap();
        assert!(css.contains(".inner"));
        assert!(css.contains("color: red"));
    }

    #[test]
    fn test_compile_each_map() {
        let css =
            compile_expanded("@each $key, $val in (a: 1, b: 2) { .#{$key} { width: $val; } }")
                .unwrap();
        assert!(css.contains(".a"));
        assert!(css.contains("width: 1"));
    }

    #[test]
    fn test_compile_math_round() {
        let css = compile_expanded("@use 'sass:math' as math; a { w: math.round(3.7); }").unwrap();
        assert!(css.contains("w: 4"));
    }

    #[test]
    fn test_compile_string_slice() {
        let css =
            compile_expanded("@use 'sass:string' as string; a { s: string.slice('hello', 2, 4); }")
                .unwrap();
        assert!(css.contains("ell"));
    }

    #[test]
    fn test_compile_map_get() {
        let css = compile_expanded("@use 'sass:map' as map; $m: (a: 1); a { v: map.get($m, a); }")
            .unwrap();
        assert!(css.contains("v: 1"));
    }

    #[test]
    fn test_compile_at_root() {
        let css = compile_expanded(".parent { @at-root { .child { color: red; } } }").unwrap();
        assert!(css.contains(".child"));
        assert!(!css.contains(".parent .child"));
    }

    #[test]
    fn test_compile_user_function() {
        let css =
            compile_expanded("@function double($x) { @return $x * 2; } a { w: double(5px); }")
                .unwrap();
        assert!(css.contains("w: 10px"));
    }

    #[test]
    fn test_compile_use_file() {
        
        let dir = std::env::temp_dir().join("sasspile_test_use");
        std::fs::create_dir_all(&dir).unwrap();
        // 创建 _config.scss
        std::fs::write(dir.join("_config.scss"), "$primary: #ff0000;\n").unwrap();
        // 创建 main.scss
        let main = dir.join("main.scss");
        std::fs::write(&main, "@use 'config';\na { color: config.$primary; }\n").unwrap();
        let css = compile_file(&main, OutputStyle::Expanded).unwrap();
assert!(
    css.contains("red") || css.contains("#ff0000"),
    "应该包含 config.$primary 的值: {css}"
);
        // 清理
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_compile_use_star() {
        
        let dir = std::env::temp_dir().join("sasspile_test_star");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("_vars.scss"), "$w: 100px;\n").unwrap();
        let main = dir.join("main.scss");
        std::fs::write(&main, "@use 'vars' as *;\na { width: $w; }\n").unwrap();
        let css = compile_file(&main, OutputStyle::Expanded).unwrap();
        assert!(css.contains("100px"), "应该包含 $w 的值: {css}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_compile_extend() {
        let css =
            compile_expanded(".btn { color: blue; } .large { @extend .btn; font-size: 20px; }")
                .unwrap();
        assert!(css.contains(".btn"), "应该包含 .btn: {css}");
        assert!(css.contains(".large"), "应该包含 .large: {css}");
        assert!(css.contains("color: blue"), "应该包含 color: blue: {css}");
        assert!(css.contains("font-size: 20px"), "应该包含 font-size: {css}");
    }

    #[test]
    fn test_compile_extend_placeholder() {
        let css = compile_expanded("%base { color: red; } .child { @extend %base; }").unwrap();
        assert!(!css.contains("%base"), "占位符不应出现: {css}");
        assert!(css.contains(".child"), "应该包含 .child: {css}");
        assert!(css.contains("color: red"), "应该包含 color: red: {css}");
    }

    #[test]
    fn test_compile_hsl() {
        let css = compile_expanded("a { color: hsl(120, 50%, 50%); }").unwrap();
        assert!(css.contains("#"), "应该包含 hex 颜色: {css}");
    }

    #[test]
    fn test_compile_append() {
        let css = compile_expanded("$l: append((1 2), 3); a { v: $l; }").unwrap();
        assert!(css.contains("1"), "应该包含 1: {css}");
        assert!(css.contains("3"), "应该包含 3: {css}");
    }

    #[test]
    fn test_compile_clamp() {
        let css = compile_expanded("a { w: clamp(1, 5, 10); }").unwrap();
        assert!(css.contains("5"), "应该包含 5: {css}");
    }

    #[test]
    fn test_debug_interleaved() {
        init_tracing();
        let css = compile_expanded(".a { b: c; .d { e: f; } }").unwrap();
        tracing::info!("DEBUG OUTPUT:\n{}", css);
        assert!(css.contains(".a .d"), "Missing .a .d: {css}");
    }

    #[test]
    fn test_debug_atfoo() {
        init_tracing();
        let css = compile_expanded("@foo {}").unwrap();
        tracing::info!("@foo OUTPUT: [{}]", css);
        assert!(!css.is_empty(), "Output should not be empty");
        assert!(css.contains("@foo"), "Should contain @foo: [{css}]");
    }

    #[test]
    fn test_debug_minus() {
        init_tracing();
        let result = compile_expanded("a {b: c - d}");
        match &result {
            Ok(css) => tracing::info!("MINUS OUTPUT: [{}]", css),
            Err(e) => tracing::error!("MINUS ERROR: {}", e),
        }
    }

    #[test]
    fn test_debug_bs_close() {
        init_tracing();
        let input = "$m: (c: d);\na {b: map-remove($m, x)}";
        let result = compile_expanded(input);
        match &result {
            Ok(css) => tracing::info!(css = css.as_str(), "MAP OUTPUT"),
            Err(e) => tracing::error!(error = %e, "MAP ERROR"),
        }
    }

    #[test]
    fn test_init_tracing_shows_target() {
        init_tracing();
        tracing::info!(target: "test_target_check", "tracing target visibility test");
    }

    #[test]
    fn test_compile_load_path() {
        let spec_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../sass-spec-main/spec");
        let utils_path = spec_root.join("core_functions/list/_utils.scss");
        if !utils_path.exists() {
            eprintln!("Skipping load path test: _utils.scss not found");
            return;
        }
        let tmp = std::env::temp_dir().join("sasspile_test_loadpath");
        std::fs::create_dir_all(&tmp).unwrap();
        let input = tmp.join("input.scss");
        std::fs::write(&input, "@use \"core_functions/list/utils\";\na {b: utils.real-separator(())}\n").unwrap();
        let result = compile_file_with_load_paths(&input, OutputStyle::Expanded, vec![spec_root]);
        std::fs::remove_dir_all(&tmp).ok();
        match result {
            Ok(css) => {
                assert!(css.contains("undecided"), "should contain undecided: {css}");
            }
            Err(e) => panic!("load path test failed: {e}"),
        }
    }
}
