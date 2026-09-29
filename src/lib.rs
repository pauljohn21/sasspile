//! sasspile — Rust ownership 三态 (move/&/&mut) 驱动的 SCSS 编译器
//!
//! 架构 (rxrust 算子链组合):
//!   Shared::subject<String> (入口)
//!     → scan_map(BlockAccumulator, accumulate_block)         // &mut 就地累积 block
//!     → flat_map(Shared::from_iter<Vec<DirectiveBlock>>)     // Vec → 单元素流
//!     → scan_map(CompileState, expand_block)                 // &mut 就地展开各 block
//!     → flat_map(Shared::from_iter<Vec<String>>)
//!     → scan_map(CssBuilder, feed)                           // &mut 就地构建 AST
//!     → flat_map(Shared::from_iter<Vec<CssNode>>)
//!     → collect::<Vec<CssNode>>().last()                     // 汇聚, complete 时发射
//!     → map(merge_media_nodes)
//!     → map(render_node)                                     // &借用, 零 clone
//!     → collect::<Vec<String>>().last()
//!     → subscribe(move |v| tx.send(v.join("\n")))            // move 转移终态
//!
//! 驱动: push all lines → complete() → terminal 同步执行完毕 → rx.recv()
//!
//! 所有权三态:
//!   - move: subscribe 闭包将 tx 所有权移入, 此后外部不可用
//!   - &: render_node(&node) 借引用产生 String, 零 clone
//!   - &mut Acc: scan_map 唯一持有状态, 就地修改, 零外部共享可变

pub mod directive;
pub mod css;
pub mod eval;

use std::collections::HashMap;
pub use directive::compile_pipeline;

/// 编译 SCSS 源码为 CSS (响应式多线程管线)
pub fn compile(input: &str) -> String {
    compile_pipeline(input)
}

/// 多文件编译 — @use / @forward 模块系统
///
/// `files`: 辅助文件映射 (如 `_other.scss` → 内容), 不含主输入
/// @use "other" 解析规则: 先精确匹配 key, 再尝试 "_" + key, 再尝试 key + ".scss"
pub fn compile_with_files(input: &str, files: &HashMap<String, String>) -> String {
    crate::directive::pipeline::compile_pipeline_with_files(input, files)
}
