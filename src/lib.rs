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
//! - sass-spec：2603/4848 (53%)

/// 内部 tracing 桥接模块——当 `tracing` feature 启用时重导出 tracing crate，
/// 关闭时提供 no-op 宏替代。
///
/// src/ 代码统一使用 `use crate::__tracing::*` 而非 `use tracing::*`，
/// 使 tracing 成为可选依赖。
/// `#[instrument]` 属性用 `#[cfg_attr(feature = "tracing", tracing::instrument)]` 处理。
#[cfg(feature = "tracing")]
pub(crate) mod __tracing {
    pub use tracing::{
        debug, debug_span, error, info, info_span, trace, warn,
    };
}

#[cfg(not(feature = "tracing"))]
pub(crate) mod __tracing {
    /// No-op span 类型——使 `span.enter()` 有效。
    pub struct DummySpan;
    impl DummySpan {
        pub fn enter(&self) -> DummyGuard { DummyGuard }
    }
    pub struct DummyGuard;

    /// No-op span 宏——返回 DummySpan 实例。
    #[macro_export]
    macro_rules! __noop_span {
        ($($args:tt)*) => { $crate::__tracing::DummySpan };
    }

    pub use crate::__noop_span as debug_span;
    pub use crate::__noop_span as info_span;
    pub use crate::__noop_span as trace_span;

    /// No-op 日志宏——编译时完全消除。
    #[macro_export]
    macro_rules! __noop_log {
        ($($args:tt)*) => {};
    }

    pub use crate::__noop_log as debug;
    pub use crate::__noop_log as error;
    pub use crate::__noop_log as info;
    pub use crate::__noop_log as trace;
    pub use crate::__noop_log as warn;
}

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
/// 当 `tracing` feature 未启用时，此函数为 no-op。
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
#[cfg(feature = "tracing")]
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

/// No-op `init_tracing`——当 `tracing` feature 未启用时。
#[cfg(not(feature = "tracing"))]
pub fn init_tracing() {}

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
