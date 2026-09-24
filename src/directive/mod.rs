//! 指令管线模块 — 统一状态 + scan_map 响应式架构
//!
//! 模块划分:
//!   - state.rs:   CompileState / Scope / MixinDef / Module / Collecting / Phase
//!   - parse.rs:   纯解析辅助 (签名解析 / 变量解析 / mixin 展开)
//!   - eval.rs:    scan_map reducer (dispatch_pass / finalize_collecting)
//!   - pipeline.rs: 管线入口 (compile_pipeline / merge_media_nodes)

pub mod state;
pub mod parse;
pub mod eval;
pub mod pipeline;

pub use self::pipeline::compile_pipeline;
