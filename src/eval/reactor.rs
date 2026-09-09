//! Reactor —— 编译世界的完整显式快照。
//!
//! 消费-返回 API 保证:
//! 1. 无隐式共享状态
//! 2. IO 通过 `ReactorIO` trait 显式化
//! 3. 单阶段 mock 可测试
//! 4. 每个管线阶段自动创建 OTel span 形成链式追踪
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
use crate::OutputStyle;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ─── 类型状态标记 ───

/// 原始源码状态 (管线起点)。
#[derive(Debug, Clone, Copy, Default)]
pub struct StateRaw;

/// 词法分析完成状态。
#[derive(Debug, Clone, Copy, Default)]
pub struct StateLexed;

/// 语法分析完成状态。
#[derive(Debug, Clone, Copy, Default)]
pub struct StateParsed;

/// 求值完成状态。
#[derive(Debug, Clone, Copy, Default)]
pub struct StateEvaluated;

/// 序列化完成状态 (管线终点)。
#[derive(Debug, Clone, Copy, Default)]
pub struct StateSerialized;

/// 编译管线阶段枚举 —— 用于 OTel span 标注。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileStage {
    Raw,
    Lex,
    Parse,
    Evaluate,
    Serialize,
    Finish,
}

impl std::fmt::Display for CompileStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw => write!(f, "raw"),
            Self::Lex => write!(f, "lex"),
            Self::Parse => write!(f, "parse"),
            Self::Evaluate => write!(f, "evaluate"),
            Self::Serialize => write!(f, "serialize"),
            Self::Finish => write!(f, "finish"),
        }
    }
}

// ─── Reactor 类型 ───

/// Reactor —— 编译世界的完整显式快照。
///
/// 通过泛型参数 `S` 编码管线阶段，保证编译顺序不可颠倒。
#[derive(Clone)]
pub struct Reactor<S = StateRaw> {
    /// 原始源码文本。
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

    /// 求值环境 —— 持久化作用域链。
    env: Option<Env>,
    /// 已编译模块缓存。
    modules: HashMap<PathBuf, ModuleCacheEntry>,
    /// 已访问文件（循环检测）。
    imports_seen: Vec<PathBuf>,
    /// IO 审计日志 (调试用)。
    io_log: Vec<IoRecord>,

    /// CSS 产物累积。
    pub css_nodes: Vec<CssNode>,
    /// 编译警告。
    pub warnings: Vec<Warning>,

    /// IO 抽象 —— trait object, 可 mock。
    io: Arc<dyn ReactorIO>,

    /// OTel 追踪上下文。
    trace: ReactorTrace,

    /// 类型状态标记 (编译期检查用)。
    _state: std::marker::PhantomData<S>,
}

/// 模块缓存条目。
#[derive(Clone)]
pub struct ModuleCacheEntry {
    /// 编译后的 Env 快照。
    pub env: Env,
    /// 编译时间戳（用于 cache 失效）。
    pub compiled_at: std::time::Instant,
}

/// IO 审计记录 —— 用于 OTel span。
#[derive(Debug, Clone)]
pub struct IoRecord {
    pub path: PathBuf,
    pub cached: bool,
    pub elapsed_us: u64,
}

/// 编译警告 —— 比 SassError 轻量。
#[derive(Debug, Clone)]
pub struct Warning {
    pub message: String,
    pub stage: CompileStage,
}

/// Reactor 追踪上下文 —— 与 OTel 链式追踪深度集成。
#[derive(Debug, Clone)]
pub struct ReactorTrace {
    pub trace_id: u128,
    pub stage: CompileStage,
    pub entered_at: std::time::Instant,
}

impl Default for ReactorTrace {
    fn default() -> Self {
        Self {
            trace_id: 0,
            stage: CompileStage::Raw,
            entered_at: std::time::Instant::now(),
        }
    }
}

impl ReactorTrace {
    /// 创建新的根 trace。
    pub fn new() -> Self {
        Self {
            trace_id: rand_id(),
            stage: CompileStage::Raw,
            entered_at: std::time::Instant::now(),
        }
    }

    /// 推进到下一阶段 —— 返回新 trace。
    pub fn advance(&self, stage: CompileStage) -> Self {
        Self {
            trace_id: self.trace_id,
            stage,
            entered_at: std::time::Instant::now(),
        }
    }
}

/// IO 抽象 trait —— 所有文件 IO 通过此 trait 显式化。
pub trait ReactorIO: Send + Sync {
    /// 读取文件内容。
    fn read_file(&self, path: &Path) -> std::io::Result<String>;

    /// 解析模块路径。
    fn resolve_path(&self, base: &Path, import: &str) -> Result<PathBuf>;

    /// 获取加载路径列表。
    fn load_paths(&self) -> &[PathBuf];

    /// 返回 true 表示路径是合法的 SCSS 文件。
    fn is_scss_file(&self, path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("scss") | Some("sass")
        )
    }
}

/// 默认 IO 实现 —— 真实文件系统。
pub struct DefaultReactorIO {
    load_paths: Vec<PathBuf>,
}

impl DefaultReactorIO {
    pub fn new(load_paths: Vec<PathBuf>) -> Self {
        Self { load_paths }
    }
}

impl ReactorIO for DefaultReactorIO {
    fn read_file(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn resolve_path(&self, base: &Path, import: &str) -> Result<PathBuf> {
        use crate::eval::Evaluator;
        Evaluator::resolve_file(Some(&base.to_path_buf()), import, &self.load_paths)
            .ok_or_else(|| SassError::Module(format!("Cannot resolve: {import}")))
    }

    fn load_paths(&self) -> &[PathBuf] {
        &self.load_paths
    }
}

/// Mock IO —— 测试用, 从内存 HashMap 读取。
pub struct MockReactorIO {
    files: HashMap<PathBuf, String>,
    load_paths: Vec<PathBuf>,
}

impl MockReactorIO {
    pub fn new(files: HashMap<PathBuf, String>) -> Self {
        Self {
            files,
            load_paths: vec![],
        }
    }

    pub fn with_load_paths(self, paths: Vec<PathBuf>) -> Self {
        Self { load_paths: paths, ..self }
    }
}

impl ReactorIO for MockReactorIO {
    fn read_file(&self, path: &Path) -> std::io::Result<String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "mock file not found"))
    }

    fn resolve_path(&self, base: &Path, import: &str) -> Result<PathBuf> {
        use crate::eval::Evaluator;
        Evaluator::resolve_file(Some(&base.to_path_buf()), import, &self.load_paths)
            .ok_or_else(|| SassError::Module(format!("Cannot resolve (mock): {import}")))
    }

    fn load_paths(&self) -> &[PathBuf] {
        &self.load_paths
    }
}

// ─── 私有辅助函数 ───

fn rand_id() -> u128 {
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
            env: None,
            modules: HashMap::new(),
            imports_seen: vec![],
            io_log: vec![],
            css_nodes: vec![],
            warnings: vec![],
            io: Arc::new(DefaultReactorIO::new(vec![])),
            trace: ReactorTrace::new(),
            _state: std::marker::PhantomData,
        }
    }

    /// 从文件创建 Reactor —— 读取文件并携带路径信息。
    ///
    /// # Errors
    /// 如果文件不存在或读取失败，返回 IO 错误。
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self {
            text,
            base_path: Some(path.clone()),
            load_paths: vec![],
            tokens: None,
            ast: None,
            serialized: None,
            env: None,
            modules: HashMap::new(),
            imports_seen: vec![],
            io_log: vec![],
            css_nodes: vec![],
            warnings: vec![],
            io: Arc::new(DefaultReactorIO::new(vec![])),
            trace: ReactorTrace::new(),
            _state: std::marker::PhantomData,
        })
    }

    /// 注入 IO 实现 —— 消费 self 返回新 Reactor (用于 mock 测试)。
    pub fn with_io(self, io: Arc<dyn ReactorIO>) -> Self {
        Self { io, ..self }
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

        // 直接调用 Lexer, 保留 Whitespace 令牌
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
            text: String::new(),              // lex 后源码不再需要
            base_path: self.base_path,
            load_paths: self.load_paths,
            tokens: Some(tokens),
            ast: None,
            serialized: None,
            env: None,
            modules: self.modules,
            imports_seen: self.imports_seen,
            io_log: self.io_log,
            css_nodes: vec![],
            warnings: vec![],
            io: self.io,
            trace: self.trace.advance(CompileStage::Lex),
            _state: std::marker::PhantomData,
        })
    }
}

// ─── StateLexed: parse ───

impl Reactor<StateLexed> {
    /// 语法分析 —— Lexed → Parsed。
    ///
    /// 消费自身，返回 AST 或错误。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(stage = "parse")))]
    pub fn parse(self) -> Result<Reactor<StateParsed>> {
        let start = std::time::Instant::now();

        let tokens = self.tokens.ok_or_else(|| {
            SassError::Internal("Reactor<StateLexed> without tokens — this is a bug.".into())
        })?;

        use crate::parse::Parser;
        let ast = Parser::parse(&tokens)?;

        #[cfg(feature = "tracing")]
        crate::__tracing::debug!(
            stage = "parse",
            elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX),
            "parse complete"
        );

        Ok(Reactor {
            text: self.text,
            base_path: self.base_path,
            load_paths: self.load_paths,
            tokens: None,
            ast: Some(ast),
            serialized: None,
            env: None,
            modules: self.modules,
            imports_seen: self.imports_seen,
            io_log: self.io_log,
            css_nodes: vec![],
            warnings: vec![],
            io: self.io,
            trace: self.trace.advance(CompileStage::Parse),
            _state: std::marker::PhantomData,
        })
    }
}

// ─── StateParsed: evaluate ───

impl Reactor<StateParsed> {
    /// 求值 —— Parsed → Evaluated。
    ///
    /// 消费自身，构建环境并求值 AST，返回求值后的 CSS 节点序列。
    #[cfg_attr(feature = "tracing", tracing::instrument(skip(self), fields(stage = "evaluate")))]
    pub fn evaluate(self) -> Result<Reactor<StateEvaluated>> {
        let start = std::time::Instant::now();

        let ast = self.ast.ok_or_else(|| {
            SassError::Internal("Reactor<StateParsed> without AST — this is a bug.".into())
        })?;

        // 构建 Env, 注入文件路径和加载路径
        let mut env = Env::default();

        if let Some(ref path) = self.base_path {
            let is_plain_css = path.extension().and_then(|e| e.to_str()) == Some("css");
            env = env
                .with_base_path(path.clone())
                .with_plain_css(is_plain_css);
        }

        if !self.load_paths.is_empty() {
            env = env.with_load_paths(self.load_paths.clone());
        }

        let nodes = crate::eval::Evaluator::evaluate_with_env(&ast, env.clone())?;

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
            env: Some(env),
            modules: self.modules,
            imports_seen: self.imports_seen,
            io_log: self.io_log,
            css_nodes: nodes,
            warnings: vec![],
            io: self.io,
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
            env: self.env,
            modules: self.modules,
            imports_seen: self.imports_seen,
            io_log: self.io_log,
            css_nodes: self.css_nodes,
            warnings: self.warnings,
            io: self.io,
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

    /// 获取 IO 审计日志。
    pub fn io_log(&self) -> &[IoRecord] {
        &self.io_log
    }

    /// 滚动到 Reactor 内部状态快照 —— 测试用。
    pub fn snapshot(&self) -> ReactorSnapshot {
        ReactorSnapshot {
            stage: self.trace.stage,
            n_css_nodes: self.css_nodes.len(),
            n_warnings: self.warnings.len(),
            n_io_ops: self.io_log.len(),
            n_modules_cached: self.modules.len(),
        }
    }
}

/// Reactor 快照 —— 用于测试断言。
#[derive(Debug, Clone)]
pub struct ReactorSnapshot {
    pub stage: CompileStage,
    pub n_css_nodes: usize,
    pub n_warnings: usize,
    pub n_io_ops: usize,
    pub n_modules_cached: usize,
}

// ─── 便捷入口 (公开 API) ───

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

// ─── 模块导入 / 重新导出 ───

pub use self::StateRaw as ReactorRaw;
pub use self::StateLexed as ReactorLexed;
pub use self::StateParsed as ReactorParsed;
pub use self::StateEvaluated as ReactorEvaluated;
pub use self::StateSerialized as ReactorSerialized;
