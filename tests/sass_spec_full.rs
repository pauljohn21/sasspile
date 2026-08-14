//! sass-spec 全量统计——使用 manifest 跳过不支持的目录。
//!
//! manifest 在 `tests/spec_manifest.rs` 中定义 `SKIP_DIRS` 跳过列表。
//! 支持新功能后，从 `SKIP_DIRS` 移除对应条目即可。
//!
//! 流式处理：逐文件读取 → 解析 → 编译 → drop，内存 O(1)。
//! 使用 compile_batch 分块编译，每 chunk 后显式释放临时内存。

mod spec_manifest;

use spec_manifest::collect_hrx_files;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use tracing::{error, info, warn};

// ═══════════════════════════════════════════════════════════════
// 内存监控：后台线程每 2 秒检查 RSS，超过阈值自动 panic
// ═══════════════════════════════════════════════════════════════

/// 监控开关 — panic 后由 test harness 观察到并退出。
static MONITOR_STARTED: AtomicBool = AtomicBool::new(false);

/// 最近一次打印时的 case 计数，辅助定位爆内存位置。
static LAST_CASE_COUNT: AtomicU64 = AtomicU64::new(0);

/// 警告阈值（KB）— `ps -o rss=` 返回 KB。1GB。
/// SCSS 编译正常只需几十MB，超过 500MB 就是异常。
const RSS_WARN_KB: usize = 512 * 1024;
/// 致命阈值（KB）— 1GB。单次编译能吃 1G+ = 严重泄漏。
const RSS_FATAL_KB: usize = 1024 * 1024;

/// 启动内存监控线程。每 2 秒：
/// - 超 WARN → tracing::warn! 事件（可观测趋势）
/// - 超 FATAL → tracing::error! 事件 + panic（自动中止，不需要手动 kill）
fn start_memory_monitor() {
    if MONITOR_STARTED.swap(true, Ordering::SeqCst) {
        return; // 已启动
    }
    thread::spawn(|| {
        let pid = std::process::id();
        let mut warned = false;
        loop {
            thread::sleep(Duration::from_secs(2));
            let rss = get_rss_kb(pid);
            let cases = LAST_CASE_COUNT.load(Ordering::Relaxed);

            if rss > RSS_FATAL_KB {
                let rss_mb = rss / 1024;
                error!(
                    rss_mb = rss_mb,
                    cases = cases,
                    "💥 MEMORY OOM — aborting test"
                );
                panic!("💥 MEMORY OOM: RSS={rss_mb} MB, {cases} cases — auto-aborted");
            } else if rss > RSS_WARN_KB && !warned {
                warn!(
                    rss_mb = rss / 1024,
                    cases = cases,
                    "⚠️ 内存持续增长中 — 接近阈值，请留意是否组合爆炸"
                );
                warned = true;
            } else if rss <= RSS_WARN_KB {
                warned = false;
            }
        }
    });
}

/// 手动更新 case 计数（用于定位爆内存位置）。
fn record_case_count(count: usize) {
    LAST_CASE_COUNT.store(count as u64, Ordering::Relaxed);
}

/// 获取当前进程 RSS（KB）。macOS 用 `ps -o rss= -p PID`。
fn get_rss_kb(pid: u32) -> usize {
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output();
    match output {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            s.trim().parse::<usize>().unwrap_or(0)
        }
        Err(_) => 0,
    }
}

/// 每 N 个 case 发放一次诊断事件（含 RSS），定位内存增长来源。
const RSS_LOG_INTERVAL: usize = 5;

fn log_memory_per_case(case_idx: usize, file_hint: &str) {
    if case_idx % RSS_LOG_INTERVAL == 0 {
        let rss = get_rss_kb(std::process::id());
        info!(
            case_idx = case_idx,
            rss_mb = rss / 1024,
            file = file_hint,
            "📊 内存进度"
        );
    }
}


/// HRX 测试用例（借用引用，不克隆）。
struct HrxCase<'a> {
    files: &'a [(String, String)],
    input_path: &'a str,
    expected_output: &'a str,
    expect_error: bool,
}

/// 解析 HRX 并直接运行——O(1) 额外内存。
/// 每个 case 编译后立即释放，不累积 case 向量。
fn parse_and_run_hrx(
    content: &str,
    load_paths: &[PathBuf],
    pass: &mut usize,
    fail: &mut usize,
    skip: &mut usize,
    cases: &mut usize,
) {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut current_path = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        // 跳过纯 = 分隔线（如 ========================================）
        if line.trim().chars().all(|c| c == '=')
            || line.trim().is_empty()
        {
            continue;
        }
        if line.starts_with("<===>") {
            if !current_path.is_empty() {
                files.push((current_path.clone(), current_content));
                current_content = String::new(); // 复用分配
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

    // 直接运行每个 case，不 clone
    for (path, _) in &files {
        if !path.ends_with("input.scss") {
            continue;
        }
        log_memory_per_case(*cases, path);
        let base = path.strip_suffix("input.scss").unwrap_or(path);
        let output_path = format!("{base}output.css");
        let error_path = format!("{base}error");

        let expected_output = files
            .iter()
            .find(|(p, _)| p == &output_path)
            .map(|(_, c)| c.as_str())
            .unwrap_or("");
        let expect_error = files.iter().any(|(p, _)| p == &error_path);

        *cases += 1;
        if expected_output.is_empty() && !expect_error {
            *skip += 1;
            continue;
        }

        let case = HrxCase {
            files: &files,
            input_path: path,
            expected_output,
            expect_error,
        };
        if run_case(&case, load_paths) {
            *pass += 1;
        } else {
            *fail += 1;
        }

        // 同步断点：每个 case 编译完立刻查 RSS —— 比后台线程更早发现问题
        let rss = get_rss_kb(std::process::id());
        if rss > RSS_FATAL_KB {
            error!(rss_mb = rss / 1024, case = path, "💥 CASE 触发内存爆炸");
            panic!("💥 内存爆炸在 case '{}': RSS={}MB", path, rss / 1024);
        } else if rss > RSS_WARN_KB {
            warn!(rss_mb = rss / 1024, case = path, "⚠️ 该 case 内存异常");
        }
    }
}

/// 运行单个测试用例——写入临时目录并用 compile_file_with_load_paths 编译。
fn run_case(case: &HrxCase, load_paths: &[PathBuf]) -> bool {
    if case.expected_output.is_empty() && !case.expect_error {
        return true;
    }

    let total_size: usize = case.files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 20_000 {
        return false;
    }

    let tmp_dir = std::env::temp_dir().join(format!("sass-spec-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();

    for (path, content) in case.files.iter() {
        let file_path = tmp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).ok();
    }

    let input_file = tmp_dir.join(&case.input_path);
    let result = sasspile::compile_file_with_load_paths(
        &input_file,
        sasspile::OutputStyle::Expanded,
        load_paths.to_vec(),
    );
    let _ = std::fs::remove_dir_all(&tmp_dir);

    if case.expect_error {
        result.is_err()
    } else {
        match result {
            Ok(actual) => actual.trim() == case.expected_output.trim(),
            Err(_) => false,
        }
    }
}

/// 流式处理单个 HRX 文件——边解析边跑，O(1) 额外内存。
fn process_hrx_file(
    file_path: &Path,
    spec_root: &Path,
    pass: &mut usize,
    fail: &mut usize,
    skip: &mut usize,
    cases: &mut usize,
) {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    // 关键：parse_and_run_hrx 不返回 case Vec，直接在函数内跑掉并释放
    parse_and_run_hrx(
        &content,
        &[spec_root.to_path_buf()],
        pass,
        fail,
        skip,
        cases,
    );
    // content 在这里 drop，所有 case 编译完即释放
}

/// 按 spec 一级目录运行并统计（流式处理，O(1) 内存）。
fn run_spec_dir(spec_root: &Path, dir_name: &str) -> (usize, usize, usize, usize) {
    let dir = spec_root.join(dir_name);
    if !dir.exists() {
        return (0, 0, 0, 0);
    }

    let (files, skipped) = collect_hrx_files(&dir, spec_root);

    let (mut pass, mut fail, mut skip, mut cases) = (0, 0, 0, 0);
    for file in &files {
        process_hrx_file(file, spec_root, &mut pass, &mut fail, &mut skip, &mut cases);
        record_case_count(cases);
        // 每个文件处理完，相关内存已释放
    }

    let evaluated = cases - skip;
    let pct = pass * 100 / evaluated.max(1);
    info!(
        dir = dir_name,
        pass = pass,
        fail = fail,
        skip = skip,
        skipped_dirs = skipped,
        total = cases,
        pct = pct,
        "sass-spec 目录"
    );
    (pass, fail, skip, cases)
}

#[test]
fn test_import_use_forward() {
    sasspile::init_tracing();
    start_memory_monitor();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
    for subdir in &["directives/import", "directives/use", "directives/forward"] {
        let (pass, fail, skip, cases) = run_spec_dir(&spec_root, subdir);
        let evaluated = cases - skip;
        let pct = pass * 100 / evaluated.max(1);
        info!(
            subdir = subdir,
            pass = pass,
            fail = fail,
            skip = skip,
            total = cases,
            pct = pct,
            "import/use/forward"
        );
    }
}

/// 处理单个目录——每个 HRX 文件编译后显式 drop。
fn run_spec_dir_chunked(spec_root: &Path, dir_name: &str) -> (usize, usize, usize, usize) {
    let dir = spec_root.join(dir_name);
    if !dir.exists() {
        return (0, 0, 0, 0);
    }

    let (files, skipped) = collect_hrx_files(&dir, spec_root);

    let (mut pass, mut fail, mut skip, mut cases) = (0, 0, 0, 0);
    for file in &files {
        process_hrx_file(file, spec_root, &mut pass, &mut fail, &mut skip, &mut cases);
        record_case_count(cases);
    }
    // 释放文件列表
    drop(files);

    let evaluated = cases - skip;
    let pct = pass * 100 / evaluated.max(1);
    info!(
        dir = dir_name,
        pass = pass,
        fail = fail,
        skip = skip,
        skipped_dirs = skipped,
        total = cases,
        pct = pct,
        "sass-spec 目录"
    );
    (pass, fail, skip, cases)
}

/// 运行指定目录列表并汇总统计。
fn run_dirs(spec_root: &Path, dirs: &[&str]) -> (usize, usize, usize, usize) {
    let (mut total_pass, mut total_fail, mut total_skip, mut total_cases) = (0, 0, 0, 0);
    for dir in dirs {
        let (pass, fail, skip, cases) = run_spec_dir_chunked(spec_root, dir);
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
        total_cases += cases;
    }
    (total_pass, total_fail, total_skip, total_cases)
}

/// 运行单个 spec 目录，独立测试防止内存累积 OOM。
fn run_one_spec_dir(dir_name: &str) {
    sasspile::init_tracing();
    start_memory_monitor();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
    let (pass, fail, skip, cases) = run_spec_dir_chunked(&spec_root, dir_name);
    let evaluated = cases - skip;
    let pct = pass * 100 / evaluated.max(1);
    info!(
        dir = dir_name,
        pass = pass,
        fail = fail,
        skip = skip,
        total = cases,
        evaluated = evaluated,
        pct = pct,
        "sass-spec 目录"
    );
}

#[test]
fn test_variables() { run_one_spec_dir("variables"); }
#[test]
fn test_values() { run_one_spec_dir("values"); }
#[test]
fn test_css() { run_one_spec_dir("css"); }
#[test]
fn test_operators() { run_one_spec_dir("operators"); }
#[test]
fn test_expressions() { run_one_spec_dir("expressions"); }
#[test]
fn test_directives() { run_one_spec_dir("directives"); }

// core_functions 子目录——独立测试（selector 拆开，防 OOM）
#[test]
fn test_core_list() { run_one_spec_dir("core_functions/list"); }
#[test]
fn test_core_map() { run_one_spec_dir("core_functions/map"); }
#[test]
fn test_core_math() { run_one_spec_dir("core_functions/math"); }
#[test]
fn test_core_meta() { run_one_spec_dir("core_functions/meta"); }
#[test]
fn test_core_string() { run_one_spec_dir("core_functions/string"); }
#[test]
fn test_core_selector_append() { run_one_spec_dir("core_functions/selector/append.hrx"); }
#[test]
fn test_core_selector_nest() { run_one_spec_dir("core_functions/selector/nest"); }
#[test]
fn test_core_selector_parse() { run_one_spec_dir("core_functions/selector/parse"); }
#[test]
fn test_core_selector_replace() { run_one_spec_dir("core_functions/selector/replace.hrx"); }
#[test]
fn test_core_selector_extend() { run_one_spec_dir("core_functions/selector/extend"); }
#[test]
fn test_core_selector_unify() { run_one_spec_dir("core_functions/selector/unify"); }
#[test]
fn test_core_selector_is_superselector() { run_one_spec_dir("core_functions/selector/is_superselector"); }

// parser + callable
#[test]
fn test_parser() { run_one_spec_dir("parser"); }
#[test]
fn test_callable() { run_one_spec_dir("callable"); }
