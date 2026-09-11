//! Reactor 管线类型定义 —— 状态标记、Trace、IO 抽象。

use crate::error::{Result, SassError};
use crate::eval::env::Env;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
            trace_id: crate::eval::reactor::rand_id(),
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
