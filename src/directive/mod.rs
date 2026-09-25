//! 指令管线模块 — rxrust 算子组合 (Flux → rxrust 转译)
//!
//! 架构: scan_map(分块状态机) → flat_map(各 block 独立 scan_map) → merge
//!   - state.rs:   CompileState / Scope / MixinDef / Module / Phase / 分块状态
//!   - parse.rs:   纯解析辅助 (签名解析 / 变量解析 / mixin 展开)
//!   - blocks.rs:  DirectiveBlock enum + parse_blocks 状态机
//!   - ops.rs:     独立 block 展开 (flat_map reducer)
//!   - eval.rs:    TokenKind 分类 (纯函数)
//!   - pipeline.rs: 管线入口 — 算子链组合

pub mod state;
pub mod parse;
pub mod blocks;
pub mod ops;
pub mod extend_ops;
pub mod while_ops;
pub mod eval;
pub mod pipeline;
pub mod module_system;
pub mod member_parse;

pub use self::pipeline::compile_pipeline;
pub use self::module_system::process_module_imports;
