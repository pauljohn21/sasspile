//! sasspile — 统一状态 + rxrust 响应式 SCSS 编译器
//!
//! 架构 (Shared 多线程上下文):
//!   CompileState (统一状态, Send + 'static)
//!     → Shared::from_stream(futures::stream::iter(tokens))
//!     → scan_map(dispatch_pass)  ← 按 phase 分发 struct/expand/resolve
//!     → flat_map(Shared::from_stream(futures::stream::iter(v)))
//!     → collect::<Vec<String>>()
//!     → last()
//!     → subscribe(消费结果到 mpsc::channel)
//!     → TaskHandle
//!
//! 驱动方式:
//!   tokio::task::block_in_place + tokio::runtime::Handle::current()
//!   SharedScheduler 内部使用 tokio runtime, 由 block_in_place 驱动.
//!
//! scan_map 框架线程化状态，零 clone，零 Arc<Mutex>。

mod directive;

pub use directive::compile_pipeline;

/// 编译 SCSS 源码为 CSS（响应式多线程管线）
pub fn compile(input: &str) -> String {
    compile_pipeline(input)
}
