//! 颜色相关诊断测试（从 diagnostic_runner.rs 分离）。
//!
//! 包含 change/scale/hsl/hwb 详细诊断 + HSL/HWB 统计 + values 错误模式。
//! 通过 `#[path = "diag_helper.rs"] mod diag_helper;` 共享辅助函数。

#[path = "diag_helper.rs"]
mod diag_helper;

use diag_helper::prelude::*;
use std::path::Path;

// ─── color change/scale 诊断 ──────────────────────────────────────────────

fn main_diag(subdir: &str, max: usize) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let dir = spec_root.join(subdir);
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);
    let mut shown = 0;
    for file in &files {
        let Ok(content) = std::fs::read_to_string(file) else { continue; };
        let stem = file.file_stem()
            .expect("unexpected failure in test")
            .to_string_lossy()
            .to_string();
        for case in &parse_cases(&content) {
            if shown >= max { break; }
            if case.expected_output.is_empty() && !case.expect_error { continue; }
            let name = case.input_path.strip_suffix("input.scss")
                .unwrap_or(&case.input_path)
                .trim_end_matches('/')
                .to_string();
            match compile_case(case, &spec_root, file.parent().unwrap_or(Path::new(".")), &stem) {
                Ok(actual) => {
                    let (a, e) = (actual.trim(), case.expected_output.trim());
                    if case.expect_error {
                        shown += 1;
                        tracing::error!("ERR_EXP_OK: {stem}/{name}\n  actual: {a}");
                    } else if a != e {
                        shown += 1;
                        tracing::error!("DIFF: {stem}/{name}\n  exp: {e}\n  got: {a}");
                    }
                }
                Err(err_str) => if !case.expect_error {
                    shown += 1;
                    tracing::error!("ERR: {stem}/{name}\n  {err_str}");
                }
            }
        }
        if shown >= max { break; }
    }
}

#[test] fn change_details() { main_diag("core_functions/color/change", 120); }
#[test] fn scale_details() { main_diag("core_functions/color/scale", 120); }
#[test] fn hsl_detail() { main_diag("core_functions/color/change", 250); }
#[test] fn hwb_details() { main_diag("core_functions/color/change/hwb", 25); }
#[test] fn hue_hsl_details() { main_diag("core_functions/color/change/hsl", 10); }

// ─── HSL 统计 ──────────────────────────────────────────────────────────────

fn count_dir(subdir: &str) -> (usize, usize) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let mut files = Vec::new();
    collect_hrx(&spec_root.join(subdir), &mut files);
    files.iter()
        .map(|file| process_cases_for_stats(file, &spec_root))
        .fold((0, 0), |(ap, af), (p, f, _)| (ap + p, af + f))
}

fn show_diffs(subdir: &str, max: usize) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let mut files = Vec::new();
    collect_hrx(&spec_root.join(subdir), &mut files);
    let mut n = 0;
    for file in &files {
        let Ok(content) = std::fs::read_to_string(file) else { continue; };
        let stem = file.file_stem()
            .expect("unexpected failure in test")
            .to_string_lossy()
            .to_string();
        for case in &parse_cases(&content) {
            if case.expected_output.is_empty() && n >= max { continue; }
            let nm = case.input_path.replace("input.scss", "");
            match compile_case(case, &spec_root, file.parent().unwrap_or(Path::new(".")), &stem) {
                Ok(r) if r.trim() == case.expected_output.trim() => {},
                Ok(r) => { n += 1; tracing::error!("[{stem}/{nm}]\nEXPECTED:\n{}\nGOT:\n{}\n", case.expected_output.trim(), r.trim()); }
                Err(e) => { n += 1; tracing::error!("[{stem}/{nm}] ERROR: {e}\n"); }
            }
        }
    }
}

#[test]
fn hsl_count() {
    let (_, fail_hsl) = count_dir("core_functions/color/hsl");
    let (_, fail_hsla) = count_dir("core_functions/color/hsla");
    tracing::error!("HSL_TOTAL: hsl fail={fail_hsl}, hsla fail={fail_hsla}");
    let _ = (fail_hsl, fail_hsla);
}

#[test]
fn hsl_diffs() { show_diffs("core_functions/color/hsl", 35); }

// ─── 颜色模块 diag_color / diag_values_colors 调用 ────────────────────────

#[test]
#[ignore = "颜色测试需手动 --ignored 触发"]
fn diag_color() {
    let _spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let _ = run_dir_diff("core_functions/color", 15);
}

#[test]
#[ignore = "颜色测试需手动 --ignored 触发"]
fn diag_values_colors() {
    let _spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let _ = run_dir_diff("values/colors", 10);
}

// ─── values 错误模式诊断（diag_output_mismatch） ───────────────────────────

type CaseTuple = (Vec<(String, String)>, String, String, bool);

fn parse_tuple(content: &str) -> Vec<CaseTuple> {
    let Ok(archive) = diag_helper::hrx_support::parse_hrx(content) else { return Vec::new(); };
    diag_helper::filter_groups(diag_helper::group_entries(&archive.entries)).iter().flat_map(|ge| {
        let ga = diag_helper::hrx_support::HrxArchive { entries: ge.clone() };
        let vfs = diag_helper::hrx_support::Vfs::from_archive(&ga);
        let dirs = vfs.walk();
        let af: Vec<(String, String)> = dirs.iter().flat_map(|(dp, files)| {
            files.iter().map(move |(f, c)| {
                if dp == "." { (f.clone(), c.clone()) } else { (format!("{dp}/{f}"), c.clone()) }
            })
        }).filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css")).collect();
        dirs.iter().filter_map(move |(dp, files)| {
            let (input_name, _) = files.iter().find(|(f, _)| f == "input.scss")?;
            let ip = if dp == "." { input_name.clone() } else { format!("{dp}/{input_name}") };
            let eo = files.iter().find(|(f, _)| f == "output.css").map(|(_, c)| c.clone()).unwrap_or_default();
            let ee = files.iter().any(|(f, _)| f == "error");
            Some((af.clone(), ip, eo, ee))
        }).collect::<Vec<_>>()
    }).collect()
}

fn classify_error_str(msg: &str) -> String {
    if msg.contains("UndefinedFunction") { "undef_function".to_string() }
    else if msg.contains("求值错误") { format!("eval: {}", msg.split(':').nth(1).unwrap_or("").trim().chars().take(100).collect::<String>()) }
    else if msg.contains("解析错误") || msg.contains("ParseError") { "parse_error".to_string() }
    else { format!("other: {}", msg.chars().take(100).collect::<String>()) }
}

fn run_case_tuple(case: &CaseTuple, load_paths: &[std::path::PathBuf]) -> Result<String, String> {
    let (files, input_path, _, expect_error) = case;
    if files.iter().map(|(_, c)| c.len()).sum::<usize>() > 50_000 { return Err("too_large".to_string()); }
    let tmp = std::env::temp_dir().join(format!("sass-diag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();
    for (p, c) in files {
        let fp = tmp.join(p);
        if let Some(parent) = fp.parent() { std::fs::create_dir_all(parent).ok(); }
        std::fs::write(&fp, c).ok();
    }
    let r = sasspile::compile_file_with_load_paths(&tmp.join(input_path), sasspile::OutputStyle::Expanded, load_paths.to_vec());
    let _ = std::fs::remove_dir_all(&tmp);
    if *expect_error {
        match r { Ok(_) => Err("expected_error_but_got_ok".to_string()), Err(_) => Ok(String::new()) }
    } else {
        match r {
            Ok(a) if a.trim() == case.2.trim() => Ok(String::new()),
            Ok(a) => {
                let (a, e) = (a.trim(), case.2.trim());
                let ds = a.chars().zip(e.chars()).position(|(a, e)| a != e);
                let ctx = match ds {
                    Some(pos) => {
                        let ac: String = a.chars().skip(pos.saturating_sub(20)).take(60).collect();
                        let ec: String = e.chars().skip(pos.saturating_sub(20)).take(60).collect();
                        format!("actual_near=|{ac}| expected_near=|{ec}|")
                    }
                    None if a.len() < e.len() => format!("actual_shorter, al={} el={}", a.len(), e.len()),
                    _ => format!("expected_shorter, al={} el={}", a.len(), e.len()),
                };
                Err(format!("output_mismatch: {ctx}"))
            }
            Err(e) => Err(classify_error_str(&format!("{e}"))),
        }
    }
}

fn process_dir(dir_name: &str, spec_root: &Path) -> Vec<(String, String)> {
    let dir = spec_root.join(dir_name);
    if !dir.exists() { return Vec::new(); }
    let spec_root_buf = spec_root.to_path_buf();
    let (files, _) = diag_helper::spec_manifest::collect_hrx_files(&dir, &spec_root_buf);
    let mut all_errors = Vec::new();
    for file in &files {
        let Ok(content) = std::fs::read_to_string(file) else { continue; };
        let cases = parse_tuple(&content);
        for case in &cases {
            let (_, _, expected, expect_error) = case;
            if expected.is_empty() && !expect_error { continue; }
            match run_case_tuple(case, std::slice::from_ref(&spec_root_buf)) {
                Err(err) => all_errors.push((file.as_path().to_string_lossy().to_string(), err)),
                Ok(_) => {}
            }
        }
    }
    all_errors
}

#[test]
fn diag_output_mismatch() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let dirs = ["core_functions", "values", "css", "directives", "expressions"];
    let all_errors: Vec<(String, String)> = dirs.iter().flat_map(|d| process_dir(d, &spec_root)).collect();
    let error_counts: std::collections::BTreeMap<String, usize> = all_errors.iter().fold(
        std::collections::BTreeMap::new(),
        |mut acc, (_, err)| { *acc.entry(err.clone()).or_insert(0) += 1; acc },
    );
    let is_target = |e: &str| {
        e.starts_with("output_mismatch") || e.starts_with("eval: hsl") || e.starts_with("eval: hwb")
            || e.starts_with("eval: rgba") || e.starts_with("eval: alpha")
    };
    let patterns: std::collections::BTreeMap<String, Vec<(String, String)>> = all_errors.iter()
        .filter(|(_, e)| is_target(e))
        .fold(std::collections::BTreeMap::new(), |mut acc, (f, err)| {
            acc.entry(err.clone()).or_default().push((f.clone(), String::new()));
            acc
        });
    let mut sorted_errors: Vec<_> = error_counts.iter().collect();
    sorted_errors.sort_by(|a, b| b.1.cmp(a.1));
    tracing::info!("=== Top 30 错误类型 ===");
    sorted_errors.iter().take(30).enumerate().for_each(|(i, (err, count))| {
        tracing::info!(rank = i + 1, error = err.as_str(), count, "错误");
    });
    let mut sorted_patterns: Vec<_> = patterns.iter().collect();
    sorted_patterns.sort_by_key(|b| std::cmp::Reverse(b.1.len()));
    tracing::info!("=== Top 15 失败模式详情 ===");
    sorted_patterns.iter().take(15).enumerate().for_each(|(i, (pattern, files))| {
        tracing::info!(rank = i + 1, pattern = pattern.as_str(), count = files.len(), "模式");
        files.iter().take(2).for_each(|(f, _)| {
            tracing::info!(file = f.as_str(), "  详情");
        });
    });
}
