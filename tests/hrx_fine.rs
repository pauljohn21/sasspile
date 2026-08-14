//! 精细诊断：对 no_op.hrx 里的每个 input.scss 单独测内存。

use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
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
            .open("hrx_fine.log")
            .ok();
    }
    if let Some(f) = guard.as_mut() {
        let _ = writeln!(f, "{msg}");
        let _ = f.flush();
    }
    println!("{msg}");
}

fn get_rss_mb() -> usize {
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output();
    match output {
        Ok(out) => {
            String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse::<usize>()
                .unwrap_or(0)
                / 1024
        }
        Err(_) => 0,
    }
}

fn start_watchdog() {
    std::thread::spawn(|| {
        while !ABORT.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(500));
            let rss = get_rss_mb();
            if rss > 80 {
                let mut guard = LOG.lock().unwrap();
                if let Some(f) = guard.as_mut() {
                    use std::io::Write;
                    let _ = writeln!(f, "\n🛑 看护线程 abort: RSS={rss}MB");
                    let _ = f.flush();
                }
                std::process::abort();
            }
        }
    });
}

#[test]
fn diag_no_op_subtests() {
    sasspile::init_tracing();
    start_watchdog();

    let hrx_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../sass-spec-main/spec/core_functions/selector/extend/no_op.hrx");
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");

    let content = std::fs::read_to_string(&hrx_path).unwrap();
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

    let input_files: Vec<_> = files
        .iter()
        .filter(|(p, _)| p.ends_with("input.scss"))
        .collect();
    log(&format!("找到 {} 个子测试\n", input_files.len()));
    log(&format!("初始 RSS: {} MB\n", get_rss_mb()));

    for (i, (path, _)) in input_files.iter().enumerate() {
        let rss_before = get_rss_mb();
        log(&format!(
            "[{:>2}] COMPILING {:<50} | RSS={}MB",
            i + 1,
            path,
            rss_before
        ));

        let tmp = std::env::temp_dir().join(format!("sass-fine-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        for (fp, fc) in &files {
            let full = tmp.join(fp);
            if let Some(parent) = full.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&full, fc);
        }

        let input = tmp.join(path);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = sasspile::compile_file_with_load_paths(
                &input,
                sasspile::OutputStyle::Expanded,
                vec![spec_root.to_path_buf()],
            );
        }));

        let _ = std::fs::remove_dir_all(&tmp);
        let rss_after = get_rss_mb();
        let growth = rss_after.saturating_sub(rss_before);
        let status = if result.is_ok() { "OK" } else { "PANIC" };
        log(&format!(
            "     {} | {:>4} → {:>4} MB (+{})",
            status, rss_before, rss_after, growth
        ));

        if rss_after > 70 {
            ABORT.store(true, Ordering::Relaxed);
            break;
        }
    }

    ABORT.store(true, Ordering::Relaxed);
    log("\n完成！");
}
