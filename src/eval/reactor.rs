//! Reactor —— 编译管线类型状态机。
//!
//! 消费-返回 API 保证:
//! 1. 无隐式共享状态
//! 2. 单阶段可测试
//! 3. 每个管线阶段自动创建 OTel span 形成链式追踪
//!
//! # 管线流程
//!
//! ```text
//! Reactor::new(source).lex()?.parse()?.evaluate()?.serialize(style).finish()
//! ```

use crate::css::node::CssNode;
use crate::error::{Result, SassError};
use crate::eval::env::Env;
use crate::lex::token::Token;
use crate::parse::ast::Ast;
use crate::runtime::block_on;
use crate::OutputStyle;

use std::path::PathBuf;

pub use super::reactor_types::*;

// ─── Reactor 类型 ───

/// Reactor —— 编译管线类型状态机。
///
/// 通过泛型参数 `S` 编码管线阶段，保证编译顺序不可颠倒。
#[derive(Clone)]
pub struct Reactor<S = StateRaw> {
    /// 原始源码文本 (lex 后由 Some → None 释放)。
    text: String,
    /// 源文件路径 (用于 @use/@import)。
    base_path: Option<PathBuf>,
    /// 加载路径 (用于模块搜索)。
    load_paths: Vec<PathBuf>,

    /// 词法分析产物 (Token 序列)。
    tokens: Option<Vec<Token>>,
    /// 语法分析产物 (AST)。
    ast: Option<Ast>,
    /// 序列化后的 CSS 字符串。
    serialized: Option<String>,

    /// 已访问文件（循环检测）。
    imports_seen: Vec<PathBuf>,
    /// CSS 产物累积。
    pub css_nodes: Vec<CssNode>,

    /// OTel 追踪上下文。
    trace: ReactorTrace,

    /// 类型状态标记 (编译期检查用)。
    _state: std::marker::PhantomData<S>,
}

// ─── 私有辅助函数 ───

pub(crate) fn rand_id() -> u128 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::time::Instant::now().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    let h = hasher.finish();
    (u128::from(h)) << 32 | u128::from(h)
}

// ═══════════════════════════════════════════════════════════════════════════
// 状态转换实现
// ═══════════════════════════════════════════════════════════════════════════

// ─── StateRaw: 创建与 lex ───

impl Reactor<StateRaw> {
    /// 从源码字符串创建 Reactor (无文件路径)。
    ///
    /// # 示例
    ///
    /// ```
    /// use sasspile::eval::reactor::Reactor;
    ///
    /// let reactor = Reactor::new("a { color: red; }");
    /// ```
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            base_path: None,
            load_paths: vec![],
            tokens: None,
            ast: None,
            serialized: None,
            imports_seen: vec![],
            css_nodes: vec![],
            trace: ReactorTrace::new(),
            _state: std::marker::PhantomData,
        }
    }

    /// 从文件创建 Reactor —— 读取文件并携带路径信息。
    ///
    /// # Errors
    /// 如果文件不存在或读取失败，返回 IO 错误。
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let text = block_on(tokio::fs::read_to_string(path))?;
        Ok(Self {
            text,
            base_path: Some(path.clone()),
            load_paths: vec![],
            tokens: None,
            ast: None,
            serialized: None,
            imports_seen: vec![],
            css_nodes: vec![],
            trace: ReactorTrace::new(),
            _state: std::marker::PhantomData,
        })
    }

    /// 设置加载路径。
    pub fn with_load_paths(self, paths: Vec<PathBuf>) -> Self {
        Self {
            load_paths: paths,
            ..self
        }
    }

    /// 设置追踪上下文 (测试用, 注入特定 trace_id)。
    pub fn with_trace(self, trace: ReactorTrace) -> Self {
        Self { trace, ..self }
    }

    /// 词法分析 —— Raw → Lexed。
    ///
    /// 消费自身，返回 Token 序列或错误。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(stage = "lex", chars = self.text.len())))]
    pub fn lex(self) -> Result<Reactor<StateLexed>> {
        let start = std::time::Instant::now();

        use crate::lex::Lexer;

        let tokens: Vec<Token> = Lexer::new(&self.text)
            .filter(|t| !matches!(t.as_ref(), Ok(Token::Eof)))
            .collect::<Result<Vec<_>>>()?;

        #[cfg(feature = "tracing")]
        let n_tokens = tokens.len();

        #[cfg(feature = "tracing")]
        crate::__tracing::debug!(
            stage = "lex",
            elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX),
            n_tokens = n_tokens,
            "lex complete"
        );

        Ok(Reactor {
            text: String::new(),
            base_path: self.base_path,
            load_paths: self.load_paths,
            tokens: Some(tokens),
            ast: None,
            serialized: None,
            imports_seen: self.imports_seen,
            css_nodes: vec![],
            trace: self.trace.advance(CompileStage::Lex),
            _state: std::marker::PhantomData,
        })
    }
}

// ─── StateLexed: parse ───

impl Reactor<StateLexed> {
    /// 语法分析 —— Lexed → Parsed（统一 SCSS 管线）。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(stage = "parse", n_tokens = self.tokens.as_ref().map_or(0, Vec::len))))]
    pub fn parse(self) -> Result<Reactor<StateParsed>> {
        let start = std::time::Instant::now();

        let tokens = self.tokens.ok_or_else(|| {
            SassError::Internal("Reactor<StateLexed> without tokens — this is a bug.".into())
        })?;

        let ast = crate::parse::Parser::parse(&tokens)?;

        #[cfg(feature = "tracing")]
        crate::__tracing::debug!(
            stage = "parse",
            elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX),
            n_nodes = ast.nodes.len(),
            "parse complete"
        );

        Ok(Reactor {
            text: self.text,
            base_path: self.base_path,
            load_paths: self.load_paths,
            tokens: None,
            ast: Some(ast),
            serialized: None,
            imports_seen: self.imports_seen,
            css_nodes: vec![],
            trace: self.trace.advance(CompileStage::Parse),
            _state: std::marker::PhantomData,
        })
    }
}

// ─── StateParsed: evaluate ───

impl Reactor<StateParsed> {
    /// 求值 —— Parsed → Evaluated（统一 SCSS 求值器）。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(stage = "evaluate", n_nodes = self.ast.as_ref().map_or(0, |a| a.nodes.len()))))]
    pub fn evaluate(self) -> Result<Reactor<StateEvaluated>> {
        let start = std::time::Instant::now();

        let ast = self.ast.ok_or_else(|| {
            SassError::Internal("Reactor<StateParsed> without AST — this is a bug.".into())
        })?;

        let mut env = Env::default();
        if let Some(ref path) = self.base_path {
            env = env.with_base_path(path.clone());
        }
        if !self.load_paths.is_empty() {
            env = env.with_load_paths(self.load_paths.clone());
        }

        let nodes = crate::eval::Evaluator::evaluate_with_env(&ast, env)?;

        #[cfg(feature = "tracing")]
        let n_nodes = nodes.len();

        #[cfg(feature = "tracing")]
        crate::__tracing::debug!(
            stage = "evaluate",
            elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX),
            n_nodes = n_nodes,
            "evaluate complete"
        );

        Ok(Reactor {
            text: self.text,
            base_path: self.base_path,
            load_paths: self.load_paths,
            tokens: None,
            ast: None,
            serialized: None,
            imports_seen: self.imports_seen,
            css_nodes: nodes,
            trace: self.trace.advance(CompileStage::Evaluate),
            _state: std::marker::PhantomData,
        })
    }
}

// ─── StateEvaluated: serialize ───

impl Reactor<StateEvaluated> {
    /// 序列化 —— Evaluated → Serialized。
    ///
    /// 将 CssNode 树序列化为 CSS 字符串。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(stage = "serialize", n_nodes = self.css_nodes.len())))]
    pub fn serialize(self, style: OutputStyle) -> Reactor<StateSerialized> {
        let start = std::time::Instant::now();

        let css = crate::css::Serializer::serialize(&self.css_nodes, style);

        #[cfg(feature = "tracing")]
        let css_len = css.len();

        #[cfg(feature = "tracing")]
        crate::__tracing::debug!(
            stage = "serialize",
            elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX),
            css_len = css_len,
            "serialize complete"
        );

        Reactor {
            text: self.text,
            base_path: self.base_path,
            load_paths: self.load_paths,
            tokens: None,
            ast: None,
            serialized: Some(css),
            imports_seen: self.imports_seen,
            css_nodes: self.css_nodes,
            trace: self.trace.advance(CompileStage::Serialize),
            _state: std::marker::PhantomData,
        }
    }
}

// ─── StateSerialized: finish ───

impl Reactor<StateSerialized> {
    /// 提取最终 CSS 字符串 —— Serialized → String (管线终点)。
    ///
    /// 消费自身, 返回编译产物。
    pub fn finish(self) -> Result<String> {
        let start = std::time::Instant::now();

        let css = self.serialized.ok_or_else(|| {
            SassError::Internal("Reactor<StateSerialized> without CSS — this is a bug.".into())
        })?;

        #[cfg(feature = "tracing")]
        crate::__tracing::debug!(
            stage = "finish",
            elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX),
            total_elapsed_us = u64::try_from(self.trace.entered_at.elapsed().as_micros()).unwrap_or(u64::MAX),
            "compile complete"
        );

        Ok(css)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 共享方法 (所有状态可用)
// ═══════════════════════════════════════════════════════════════════════════

impl<S> Reactor<S> {
    /// 获取当前编译阶段。
    pub fn stage(&self) -> CompileStage {
        self.trace.stage
    }

    /// 获取 trace 信息 (用于 OTel 测试断言)。
    pub fn trace(&self) -> &ReactorTrace {
        &self.trace
    }

    /// 返回 Reactor 状态快照 —— 测试用。
    pub fn snapshot(&self) -> ReactorSnapshot {
        ReactorSnapshot {
            stage: self.trace.stage,
            n_css_nodes: self.css_nodes.len(),
        }
    }
}

/// Reactor 快照 —— 用于测试断言。
#[derive(Debug, Clone)]
pub struct ReactorSnapshot {
    pub stage: CompileStage,
    pub n_css_nodes: usize,
}

// ─── 便捷入口 (内部 API, async) ───

/// 通过 Reactor 管线编译 SCSS 源码为 CSS 字符串。
///
/// 公开 API 入口，功能同 `compile()`。
pub fn compile(input: &str, style: OutputStyle) -> Result<String> {
    Reactor::new(input.to_string())
        .lex()?
        .parse()?
        .evaluate()?
        .serialize(style)
        .finish()
}

/// 通过 Reactor 管线编译 SCSS 文件为 CSS 字符串。
pub fn compile_file(path: &PathBuf, style: OutputStyle) -> Result<String> {
    Reactor::from_file(path)?
        .lex()?
        .parse()?
        .evaluate()?
        .serialize(style)
        .finish()
}

/// 通过 Reactor 管线编译 SCSS 文件 (带加载路径) 为 CSS 字符串。
pub fn compile_file_with_load_paths(
    path: &PathBuf,
    style: OutputStyle,
    load_paths: Vec<PathBuf>,
) -> Result<String> {
    Reactor::from_file(path)?
        .with_load_paths(load_paths)
        .lex()?
        .parse()?
        .evaluate()?
        .serialize(style)
        .finish()
}
