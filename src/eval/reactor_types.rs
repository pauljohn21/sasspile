//! Reactor 管线类型定义 —— 状态标记、Trace。

use std::time::Instant;

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

// ─── ReactorTrace ───

/// Reactor 追踪上下文 —— 与 OTel 链式追踪深度集成。
#[derive(Debug, Clone)]
pub struct ReactorTrace {
    pub trace_id: u128,
    pub stage: CompileStage,
    /// 管线入口时间 —— 全程保留, 用于计算总耗时。
    pub entered_at: Instant,
}

impl Default for ReactorTrace {
    fn default() -> Self {
        Self {
            trace_id: 0,
            stage: CompileStage::Raw,
            entered_at: Instant::now(),
        }
    }
}

impl ReactorTrace {
    /// 创建新的根 trace。
    pub fn new() -> Self {
        Self {
            trace_id: crate::eval::reactor::rand_id(),
            stage: CompileStage::Raw,
            entered_at: Instant::now(),
        }
    }

    /// 推进到下一阶段 —— 返回新 trace。
    ///
    /// 保留原始 `entered_at` 以准确测量管线总耗时。
    pub fn advance(&self, stage: CompileStage) -> Self {
        Self {
            trace_id: self.trace_id,
            stage,
            entered_at: self.entered_at,
        }
    }
}
