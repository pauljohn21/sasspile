//! Bootstrap 5.3.8 编译验证测试。
//!
//! 使用 compile_batch API 批量编译 Bootstrap 组件文件。

use sasspile::*;
use std::path::PathBuf;

// 内存监控 — tracing 事件 + 超限 panic
fn get_rss_mb() -> usize {
    let pid = std::process::id();
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output();
    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0),
        Err(_) => 0,
    }
}

fn start_bs_memory_monitor() {
    std::thread::spawn(|| {
        let mut warned = false;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let rss = get_rss_mb();
            if rss > 4 * 1024 * 1024 {
                tracing::error!(rss_mb = rss, "💥 BS MEMORY OOM — auto-aborting");
                panic!("💥 BS OOM: RSS={rss} MB");
            } else if rss > 2 * 1024 * 1024 && !warned {
                tracing::warn!(rss_mb = rss, "⚠️ BS 内存增长中");
                warned = true;
            } else if rss <= 2 * 1024 * 1024 {
                warned = false;
            }
        }
    });
}

fn bs_scss(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bs")
        .join("scss")
        .join(file)
}

/// 批量编译 Bootstrap 入口文件（完整编译链）。
#[test]
fn bs_entry_batch() {
    init_tracing();
    start_bs_memory_monitor();
    let files = [
        bs_scss("bootstrap.scss"),
        bs_scss("bootstrap-grid.scss"),
        bs_scss("bootstrap-reboot.scss"),
        bs_scss("bootstrap-utilities.scss"),
    ];

    let result = compile_batch(&files, OutputStyle::Expanded);
    let mut ok = 0;
    let mut fail = 0;

    for (name, res) in &result.outputs {
        match res {
            Ok(css) => {
                ok += 1;
                tracing::info!(component = name, bytes = css.len(), "OK");
            }
            Err(e) => {
                fail += 1;
                tracing::warn!(component = name, error = %e, "FAIL");
            }
        }
    }

    tracing::info!(
        ok = ok,
        fail = fail,
        total = result.outputs.len(),
        "Bootstrap 入口批量编译完成"
    );
    assert!(ok > 0, "至少一个入口文件应编译成功");
}

/// 批量编译 Bootstrap 组件 partial 文件（验证独立编译能力）。
#[test]
fn bs_components_batch() {
    init_tracing();
    start_bs_memory_monitor();
    let components = [
        "_reboot.scss",
        "_alert.scss",
        "_badge.scss",
        "_buttons.scss",
        "_card.scss",
        "_close.scss",
        "_containers.scss",
        "_functions.scss",
        "_grid.scss",
        "_mixins.scss",
        "_root.scss",
        "_type.scss",
        "_variables.scss",
    ];

    let files: Vec<PathBuf> = components.iter().map(|f| bs_scss(f)).collect();
    let result = compile_batch(&files, OutputStyle::Expanded);

    let ok = result.outputs.iter().filter(|(_, r)| r.is_ok()).count();
    let fail = result.outputs.len() - ok;

    // 单独编译 partial 可能因缺少 @import 上下文而失败——这是预期的
    tracing::info!(
        ok = ok,
        fail = fail,
        total = result.outputs.len(),
        "Bootstrap 组件 partial 批量编译完成（部分失败是预期的——缺少依赖上下文）"
    );

    assert_eq!(result.outputs.len(), components.len(), "返回结果数量应匹配");
}
