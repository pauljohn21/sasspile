//! HRX 诊断——写文件模式，进程死也能保留进度。
//! 看护线程: RSS 超限立刻 abort，避免系统死机。

mod spec_manifest;

use spec_manifest::collect_hrx_files;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

static LOG: Mutex<Option<std::fs::File>> = Mutex::new(None);
static ABORT: AtomicBool = AtomicBool::new(false);

fn log(msg: &str) {
    use std::io::Write;
    let mut guard = LOG.lock().unwrap();
    if guard.is_none() {
        *guard = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("hrx_diag.log")
            .ok();
    }
    if let Some(f) = guard.as_mut() {
        let _ = writeln!(f, "{}", msg);
        let _ = f.flush();
    }
    drop(guard);
    println!("{}", msg);
}

fn get_rss_mb() -> usize {
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output();
    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().parse::<usize>().unwrap_or(0) / 1024,
        Err(_) => 0,
    }
}

/// 启动看护线程：RSS 超 80MB 立刻 abort。
fn start_watchdog() {
    std::thread::spawn(|| {
        while !ABORT.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(500));
            let rss = get_rss_mb();
            if rss > 80 {
                let _ = std::io::Write::write_all(
                    &mut std::io::stdout(),
                    format!("\n🛑 看护线程 abort: RSS={rss}MB > 80MB\n").as_bytes(),
                );
                // 立刻写日志文件
                let mut guard = LOG.lock().unwrap();
                if let Some(f) = guard.as_mut() {
                    use std::io::Write;
                    let _ = writeln!(f, "\n🛑 看护线程 abort: RSS={rss}MB > 80MB");
                    let _ = f.flush();
                }
                std::process::abort();
            }
        }
    });
}

#[test]
fn diag_memory_per_file() {
    sasspile::init_tracing();
    start_watchdog();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");

    let (all_files, skipped) = collect_hrx_files(&spec_root, &spec_root);
    log(&format!("找到 {} 个 HRX 文件 (跳过 {skipped})", all_files.len()));

    let initial_rss = get_rss_mb();
    log(&format!("初始 RSS: {initial_rss} MB\n"));

    for (i, file) in all_files.iter().enumerate() {
        let file_str = file.to_string_lossy().to_string();
        let rss_before = get_rss_mb();
        let rel = file.strip_prefix(&spec_root).unwrap_or(file);
        log(&format!(
            "[{:>4}] COMPILING {:>55} | RSS={}MB",
            i + 1,
            rel.display(),
            rss_before
        ));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            compile_hrx_file(&file_str, &spec_root);
        }));

        let rss_after = get_rss_mb();
        let growth = if rss_after > rss_before { rss_after - rss_before } else { 0 };

        let rel = file.strip_prefix(&spec_root).unwrap_or(file);
        let status = if result.is_ok() { "OK  " } else { "PANIC" };
        log(&format!(
            "[{:<4}] {} {:<60} | {:>4} → {:>4} MB (+{})",
            i + 1,
            status,
            rel.display(),
            rss_before,
            rss_after,
            growth
        ));

        if rss_after > 70 {
            log(&format!("🛑 超过 70MB，停止于文件 {}", i + 1));
            ABORT.store(true, Ordering::Relaxed);
            break;
        }
    }

    ABORT.store(true, Ordering::Relaxed);
    log("\n完成！");
}

fn compile_hrx_file(file_path: &str, spec_root: &Path) {
    let path = Path::new(file_path);
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut current_path = String::new();
    let mut current_content = String::new();
    let mut files: Vec<(String, String)> = Vec::new();

    for line in content.lines() {
        if line.trim().chars().all(|c| c == '=') || line.trim().is_empty() {
            continue;
        }
        if line.starts_with("<===>") {
            if !current_path.is_empty() {
                files.push((current_path.clone(), current_content.clone()));
                current_content.clear();
            }
            current_path = line.trim_start_matches("<===>").trim().to_string();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if !current_path.is_empty() {
        files.push((current_path, current_content));
    }

    for (p, _) in &files {
        if !p.ends_with("input.scss") {
            continue;
        }
        let output_key = p.replace("input.scss", "output.css");
        let error_key = p.replace("input.scss", "error");

        let expected = files
            .iter()
            .find(|(k, _)| k == &output_key)
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        let expect_error = files.iter().any(|(k, _)| k == &error_key);

        if expected.is_empty() && !expect_error {
            continue;
        }

        let tmp = std::env::temp_dir().join(format!("sass-diag-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);

        for (fp, fc) in &files {
            let full = tmp.join(fp);
            if let Some(parent) = full.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&full, fc);
        }

        let input = tmp.join(p);
        let _ = sasspile::compile_file_with_load_paths(
            &input,
            sasspile::OutputStyle::Expanded,
            vec![spec_root.to_path_buf()],
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
