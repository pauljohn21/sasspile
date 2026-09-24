//! sasspile — Flux 思维 + rxrust 算子组合的 SCSS 编译器
//!
//! 架构 (Flux → rxrust 转译):
//!   groupBy(classify) → scan_map(accumulate_block) → flat_map(process_block)
//!
//! 数据流:
//!   Shared::subject<String>                                  // Flux Sinks.Many
//!     → scan_map(BlockAccumulator, accumulate_block)         // Flux bufferUntil/groupBy
//!     → flat_map(Shared::from_iter<Vec<DirectiveBlock>>)     // Flux groupBy → inner
//!     → scan_map(CompileState, expand_block)                 // Flux scanWith (各 block 独立)
//!     → flat_map(Shared::from_iter<Vec<String>>)
//!     → scan_map(CssBuilder, feed)                           // Flux scanWith (AST 构建)
//!     → flat_map(Shared::from_iter<Vec<CssNode>>)
//!     → collect::<Vec<CssNode>>().last()                     // Flux collectList
//!     → map(merge_media_nodes)
//!     → map(render_node)                                     // 借引用, 零 clone
//!     → collect::<Vec<String>>().last()
//!     → subscribe(move |v| tx.send(v.join("\n")))            // move 转移终态
//!
//! 驱动: push all lines → complete() → terminal 同步执行完毕 → rx.recv()

pub mod directive;
pub mod css;
pub mod eval;

pub use directive::compile_pipeline;

/// 编译 SCSS 源码为 CSS (响应式多线程管线)
pub fn compile(input: &str) -> String {
    compile_pipeline(input)
}
