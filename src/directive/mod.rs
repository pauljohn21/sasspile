//! 指令管线模块 — 统一状态 + scan_map 响应式架构
//!
//! 设计:
//!   1. CompileState: 唯一状态载体（变量/mixin/模块/展开缓冲/阶段）
//!   2. pipeline.rs: scan_map 主循环，消费 &mut CompileState + token → Vec<String>
//!   3. dispatch_pass 按 state.phase 分发到 struct/expand/resolve
//!
//! 对比旧设计:
//!   旧 = 5 个独立 Observer（EachOp/ForOp/IfOp/MixinOp/UseOp）× 5 个分散 State 枚举
//!   新 = 1 个 CompileState + 1 个 scan_map + dispatch_pass 分发

pub mod state;
pub mod pipeline;

pub use self::state::{CompileState, MixinDef, Module, Scope, Collecting, Phase};
pub use self::pipeline::compile_pipeline;
