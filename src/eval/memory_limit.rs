//! 编译器内存限制器 —— 嵌入管线，超限返回 Error，不爆进程。
//!
//! 设计理念："链式反应"——检测点在求值器内部，超限立刻返回 `Err`，
//! Rust 所有权系统自动释放当次编译的所有内存，后续编译不受影响。

use crate::error::{Result, SassError};

/// 默认内存限制：512MB。SCSS 编译正常 < 50MB，超 200MB 就是异常。
/// 可通过环境变量 `SASSPILE_MEMORY_LIMIT_MB` 调整。
pub const DEFAULT_LIMIT_MB: usize = 512;

/// 获取当前内存限制（MB），从环境变量读取。
pub fn memory_limit_mb() -> usize {
    match std::env::var("SASSPILE_MEMORY_LIMIT_MB") {
        Ok(val) => val.parse().unwrap_or(DEFAULT_LIMIT_MB),
        Err(_) => DEFAULT_LIMIT_MB,
    }
}

/// **浪费者**：检查内存使用是否超限。超限返回 `Err`（不是 panic）。
///
/// # 链式反应原理
/// 1. `eval_nodes` 每处理 N 个节点调用一次
/// 2. `apply_extends` 每次进入时调用一次
/// 3. 超限 → 返回 `SassError::Eval("内存超限")`
/// 4. 上层调用栈 unwind → 所有 `Value`/`CssNode`/`Env` 被 drop
/// 5. 内存立刻归还，下一次编译正常
pub fn check_memory_limit() -> Result<()> {
    let limit_mb = memory_limit_mb();
    let rss_mb = get_rss_mb();
    if rss_mb > limit_mb {
        Err(SassError::Eval(format!(
            "内存超限: RSS={rss_mb}MB > 限制={limit_mb}MB（可能是选择器组合爆炸或 @extend 嵌套过深，尝试设置 SASSPILE_MEMORY_LIMIT_MB=2048 提高限制）"
        )))
    } else {
        Ok(())
    }
}

/// 获取当前进程 RSS（macOS / Linux 通用）。
#[cfg(target_os = "macos")]
fn get_rss_mb() -> usize {
    let pid = std::process::id();
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output();
    match output {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            // ps 返回 KB，转 MB
            s.trim().parse::<usize>().unwrap_or(0) / 1024
        }
        Err(_) => 0,
    }
}

#[cfg(not(target_os = "macos"))]
fn get_rss_mb() -> usize {
    // Linux: 读取 /proc/self/status
    if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    // kB → MB
                    return parts[1].parse::<usize>().unwrap_or(0) / 1024;
                }
            }
        }
    }
    0
}

/// **订阅者**：在每次跨越阈值时发出 warn 事件。
/// 实际通过 `check_memory_limit()` 调用前触发，用于记录内存轨迹。
pub struct MemoryLimitSubscriber;

impl MemoryLimitSubscriber {
    /// 发出内存警告事件（如果超过阈值的 50%）。
    pub fn maybe_warn() {
        let rss_mb = get_rss_mb();
        let limit_mb = memory_limit_mb();
        if rss_mb > limit_mb / 2 {
            crate::__tracing::warn!(
                rss_mb = rss_mb,
                limit_mb = limit_mb,
                "⚠️ sasspile 内存接近限制，链式反应准备触发"
            );
        }
    }
}
