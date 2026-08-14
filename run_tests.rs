//! 测试隔离运行器——每个 test 单独进程，死机只影响一个。
//!
//! 使用方法：
//!   rust-script run_tests.rs
//!
//! 原理：
//!   macOS + Rust 默认分配器不归还内存 → RSS 虚高 → 死机。
//!   每个 test 跑完进程退出，OS 立刻回收全部内存，下一个 test 干干净净。

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn main() {
    println!("📋 获取测试列表...");
    let output = Command::new("cargo")
        .args(["test", "--test", "sass_spec_full", "--", "--list"])
        .output()
        .expect("无法运行 cargo test --list");

    let list_output = String::from_utf8_lossy(&output.stdout);
    let test_names: Vec<&str> = list_output
        .lines()
        .filter(|l| l.contains(": test"))
        .map(|l| l.trim().trim_end_matches(": test"))
        .collect();

    println!("找到 {} 个测试\n", test_names.len());

    let mut results: Vec<(&str, bool, Duration)> = Vec::new();
    let timeout = Duration::from_secs(30); // 每个测试 30 秒超时

    for (i, name) in test_names.iter().enumerate() {
        let start = Instant::now();
        print!("[{}/{}] {} ... ", i + 1, test_names.len(), name);

        let output = Command::new("cargo")
            .args(["test", "--test", "sass_spec_full", name, "--", "--nocapture"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        let mut child = match output {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ 启动失败: {e}");
                results.push((*name, false, start.elapsed()));
                continue;
            }
        };

        // 等待完成或超时
        let wait_result = wait_with_timeout(&mut child, timeout);
        let elapsed = start.elapsed();

        let (status, detail) = match wait_result {
            WaitResult::Completed(Ok(status)) => {
                if status.success() {
                    (true, format!("✅ {:.1}s", elapsed.as_secs_f64()))
                } else {
                    (false, format!("❌ FAILED {:.1}s", elapsed.as_secs_f64()))
                }
            }
            WaitResult::Completed(Err(e)) => (false, format!("❌ 错误: {e}")),
            WaitResult::Timeout => {
                let _ = child.kill();
                // 进程被 kill，内存立刻被 OS 回收
                (false, "⏰ 超时 (内存可能爆炸)".to_string())
            }
        };

        println!("{detail}");
        results.push((*name, status, elapsed));
    }

    // 汇总
    println!("\n═══════════════════════════════════════");
    let passed = results.iter().filter(|(_, ok, _)| *ok).count();
    let total = results.len();
    println!("总计: {}/{} 通过", passed, total);

    if passed < total {
        println!("\n❌ 失败的测试:");
        for (name, ok, _) in &results {
            if !ok {
                println!("  - {name}");
            }
        }
    }
}

enum WaitResult {
    Completed(std::io::Result<std::process::ExitStatus>),
    Timeout,
}

fn wait_with_timeout(child: &mut std::process::Child, timeout: Duration) -> WaitResult {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return WaitResult::Completed(Ok(status)),
            Ok(None) => {
                if start.elapsed() > timeout {
                    return WaitResult::Timeout;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return WaitResult::Completed(Err(e)),
        }
    }
}
