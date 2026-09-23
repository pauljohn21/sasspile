//! CSS 模块 — CssNode AST + CssBuilder + 渲染器
//!
//! 模块结构:
//!   - node.rs: CssNode 枚举 (Rule/Declaration/AtRoot/AtRule/Comment)
//!   - builder.rs: CssBuilder — scan_map reducer 构建 AST

pub mod node;
pub mod builder;

pub use node::{CssNode, render_node};
pub use builder::CssBuilder;
