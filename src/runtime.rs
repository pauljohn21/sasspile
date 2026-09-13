//! Tokio runtime —— 对内异步的 IO 基础设施。
//!
//! ## 依赖
//!
//! `Cargo.toml` 中指定:
//! ```toml
//! tokio = { version = "1", features = ["rt-multi-thread", "fs"] }
//! ```
//! `rt-multi-thread` 提供多线程 runtime，`fs` 提供异步文件读取。
//! 不使用 `features = ["full"]` 避免用户引用歧义。

use std::future::Future;
use std::sync::OnceLock;

use tokio::runtime::Runtime;

/// 全局 tokio runtime —— 延迟初始化、线程安全。
///
/// 首次访问时创建一个多线程 runtime (worker_threads = CPU 核数)。
/// 所有 eval 层的异步 IO 共享此 runtime。
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// 获取全局 tokio runtime 引用。
///
/// ## Panics
///
/// panic 如果 runtime 初始化失败 (极少见, 意味着系统资源不足)。
pub fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        #[cfg(feature = "tracing")]
        tracing::debug!("initializing tokio runtime");

        tokio::runtime::Builder::new_multi_thread()
            .thread_name("sasspile-worker")
            .enable_all()
            .build()
            .expect("Failed to initialize tokio runtime")
    })
}

/// 在全局 runtime 上执行 async 计算并阻塞等待结果。
///
/// 用于同步 eval 管线中的异步模块加载:
/// ```ignore
/// let source = block_on(tokio::fs::read_to_string(path))?;
/// ```
#[inline]
pub fn block_on<F: Future>(future: F) -> F::Output {
    runtime().block_on(future)
}
