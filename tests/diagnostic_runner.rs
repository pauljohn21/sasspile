//! 统一诊断运行器——所有 sass-spec 诊断和统计测试的单一入口。
//!
//! 替代原来的 15 个独立诊断文件（cf_diag、cfs_diag、cfs_diag2、cfs_units、css_diag、diag_detail、diag_directives、diag_hsl2、diag_hsl3、expr_diag、extend_debug、failures_analyze、failures_inspect、trace_table、use_diag）。
//! 通过 `#[path = "diag_helper.rs"] mod diag_helper;` 共享辅助函数。
//!
//! 设计：每个测试文件 ≤ 500 行。过大的模块拆到 `diag_color.rs` / `diag_values.rs` 等。

#![allow(clippy::too_many_lines)]

#[path = "diag_helper.rs"]
mod diag_helper;

use diag_helper::prelude::*;
use std::path::{Path, PathBuf};

// ─── core_functions + directives 诊断 ─────────────────────────────────────

#[allow(clippy::too_many_lines)]
fn diag(subdir: &str, max_show: usize) {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let dir = spec_root.join(subdir);
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);
    let (err_types,shown) = files.iter().fold(
        (std::collections::HashMap::<String, usize>::new(), 0usize),
        |(mut err_map, mut shown), file| {
            if let Ok(content) = std::fs::read_to_string(file) {
                let stem = file.file_stem()
                    .expect("unexpected failure in test")
                    .to_string_lossy()
                    .to_string();
                let parent = file.parent().unwrap_or(Path::new("."));
                for case in &parse_cases(&content) {
                    if shown >= max_show { break; }
                    if case.expected_output.is_empty() && !case.expect_error { continue; }
                    let name = case.input_path.strip_suffix("input.scss")
                        .unwrap_or(&case.input_path)
                        .trim_end_matches('/')
                        .to_string();
                    match compile_case(case, &spec_root, parent, &stem) {
                        Ok(_actual) if case.expect_error => {
                            shown += 1;
                            *err_map.entry("expected_error_but_ok".to_string()).or_default() += 1;
                            tracing::warn!(test = %format!("{stem}/{name}"), "FAIL: expected_error_but_ok");
                        }
                        Ok(actual) if actual.trim() != case.expected_output.trim() => {
                            shown += 1;
                            let diff = diff_css(case.expected_output.trim(), actual.trim());
                            let key = diff.classify().to_string();
                            *err_map.entry(key).or_default() += 1;
                            tracing::warn!(test = %format!("{stem}/{name}"), kind = %diff.classify(), n_diffs = diff.lines.len(), "FAIL");
                        }
                        Err(err_str) if !case.expect_error => {
                            shown += 1;
                            let key = classify_error(&err_str);
                            *err_map.entry(key.to_string()).or_default() += 1;
                            tracing::warn!(test = %format!("{stem}/{name}"), kind = %key, error = %err_str, "ERROR");
                        }
                        _ => {}
                    }
                }
            }
            (err_map, shown)
        },
    );
        tracing::info!(subdir = %subdir, shown, "错误类型统计");
    err_types.iter().for_each(|(k, v)| { tracing::info!(error_type = %k, count = *v, "错误类型"); });
}

// ─── 测试函数：core_functions / directives 模块 ──────────────────────────

#[test] fn diag_list() { diag("core_functions/list", 15); }
#[test] fn diag_selector() { diag("core_functions/selector", 15); }
#[test] fn diag_math() { diag("core_functions/math", 15); }
#[test] fn diag_expressions() { diag("expressions", 15); }
#[test] fn diag_meta() { diag("core_functions/meta", 15); }
#[test] fn diag_import() { diag("directives/import", 50); }
#[test] fn diag_use() { diag("directives/use", 50); }
#[test] fn diag_css() { diag("css", 20); }
#[test] fn diag_non_conformant() { diag("non_conformant", 20); }
#[test] fn diag_function() { diag("directives/function", 15); }
#[test] fn diag_extend() { diag("directives/extend", 50); }
#[test] fn diag_forward() { diag("directives/forward", 50); }
#[test] fn diag_numbers() { diag("values/numbers", 20); }
#[test] fn diag_string() { diag("core_functions/string", 20); }
#[test] fn diag_map() { diag("core_functions/map", 15); }
#[test] fn diag_for() { diag("directives/for", 15); }
#[test] fn diag_each() { diag("directives/each", 15); }
#[test] fn diag_while() { diag("directives/while", 15); }
#[test] fn diag_media() { diag("directives/media", 15); }
#[test] fn diag_values_maps() { diag("values/maps", 10); }

#[test] fn stats_list() {
    let (pass, fail, total) = stats_subdir("core_functions/list");
    let pct = if total > 0 { pass * 100 / total } else { 0 };
    tracing::info!(pass, fail, total, pct, "stats_list");
}

#[test] fn stats_math() {
    let (pass, fail, total) = stats_subdir("core_functions/math");
    let pct = if total > 0 { pass * 100 / total } else { 0 };
    tracing::info!(pass, fail, total, pct, "stats_math");
}

// ─── CSS 目录诊断 ──────────────────────────────────────────────────────────

#[test]
fn css_fail_details() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let css_dir = spec_root.join("css");
    let (files, _) = diag_helper::spec_manifest::collect_hrx_files(&css_dir, &spec_root);
    let (fail_count, shown) = files.iter().fold((0, 0), |(fc, sh), file| {
        let Ok(content) = std::fs::read_to_string(file) else { return (fc, sh); };
        parse_cases(&content).iter().fold((fc, sh), |(fail_count, shown), case| {
            if let Some(diff) = run_case_generic(case, std::slice::from_ref(&spec_root), "css-diag") {
                let new_shown = if shown < 200 { tracing::info!("\n{diff}"); shown + 1 } else { shown };
                (fail_count + 1, new_shown)
            } else { (fail_count, shown) }
        })
    });
    tracing::info!(total_fails = fail_count, shown = shown, "css fail summary");
}

// ─── expressions 目录诊断 ──────────────────────────────────────────────────

#[test]
fn expr_fail_details() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let expr_dir = spec_root.join("expressions");
    let (files, _) = diag_helper::spec_manifest::collect_hrx_files(&expr_dir, &spec_root);
    let (fail_count, shown) = files.iter().fold((0, 0), |(fc, sh), file| {
        let Ok(content) = std::fs::read_to_string(file) else { return (fc, sh); };
        parse_cases(&content).iter().fold((fc, sh), |(fail_count, shown), case| {
            if let Some(diff) = run_case_generic(case, std::slice::from_ref(&spec_root), "expr-diag") {
                let new_shown = if shown < 40 { tracing::info!("\n{diff}"); shown + 1 } else { shown };
                (fail_count + 1, new_shown)
            } else { (fail_count, shown) }
        })
    });
    tracing::info!(total_fails = fail_count, shown = shown, "expressions fail summary");
}

// ─── directives 诊断（forward/extend） ─────────────────────────────────────

#[test]
fn diag_forward_extend() {
    sasspile::init_tracing();
    let spec_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let directives_dir = spec_root.join("directives");
    let hrx_files = ["forward/error/extend.hrx", "forward/extend.hrx"];
    for hrx_rel in hrx_files {
        let hrx_path = directives_dir.join(hrx_rel);
        let Ok(content) = std::fs::read_to_string(&hrx_path) else { continue };
        let archive = diag_helper::hrx_support::parse_hrx(&content).expect("parse hrx failed");
        let (pass, fail) = diag_helper::filter_groups(diag_helper::group_entries(&archive.entries))
            .iter()
            .flat_map(|ge| {
                let ga = diag_helper::hrx_support::HrxArchive { entries: ge.clone() };
                let vfs = diag_helper::hrx_support::Vfs::from_archive(&ga);
                let dirs = vfs.walk();
                let _af: Vec<(String, String)> = dirs.iter().flat_map(|(dp, fs)| {
                    fs.iter().map(move |(f, c)| {
                        if dp == "." { (f.clone(), c.clone()) } else { (format!("{dp}/{f}"), c.clone()) }
                    })
                }).filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css")).collect();
                dirs.iter().filter_map(move |(dp, fs)| {
                    let (input_name, _) = fs.iter().find(|(f, _)| f == "input.scss")?;
                    let expected_output = fs.iter().find(|(f, _)| f == "output.css").map(|(_, c)| c.clone());
                    let expect_error = fs.iter().any(|(f, _)| f == "error");
                    let name = if dp == "." { String::new() } else { dp.clone() };
                    let input_path = if dp == "." { input_name.clone() } else { format!("{dp}/{input_name}") };
                    let result = sasspile::compile_file_with_load_paths(
                        &std::path::PathBuf::from(&input_path),
                        sasspile::OutputStyle::Expanded,
                        vec![],
                    ).map_err(|e| format!("{e}"));
                    Some((name, result, expected_output, expect_error))
                }).collect::<Vec<_>>()
            })
            .fold((0, 0), |(pass, fail), (name, result, expected, expect_error)| {
                let ok = if expect_error { result.is_err() }
                    else { result.is_ok() && expected.is_some()
                        && result.as_ref().expect("unexpected failure in test").trim()
                            == expected.as_ref().expect("unexpected failure in test").trim()
                    };
                if ok { (pass + 1, fail) } else {
                    match (result, expect_error) {
                        (Ok(_css), true) => tracing::warn!(name = %name, "EXPECTED ERROR but got OK"),
                        (Err(e), true) => tracing::warn!(name = %name, error = %e, "wrong error"),
                        (Ok(_css), false) => tracing::warn!(name = %name, expected = ?expected, "CONTENT DIFF"),
                        (Err(e), false) => tracing::warn!(name = %name, error = %e, "UNEXPECTED ERROR"),
                    }
                    (pass, fail + 1)
                }
            });
        tracing::info!(hrx = %hrx_rel, pass, fail, "forward/extend summary");
    }
}

// ─── use 子目录统计（parse_hrx_to_cases + run_case from hrx_support）────────

#[test]
fn test_use_top_level_hrx() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let use_dir = spec_root.join("directives/use");
    if !use_dir.exists() { tracing::info!("directives/use 目录不存在，跳过"); return; }
    let mut hrx_files: Vec<PathBuf> = std::fs::read_dir(&use_dir).ok()
        .map(|entries| entries.flatten()
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("hrx"))
            .filter(|e| std::fs::metadata(e.path()).map(|m| m.len() < 100_000).unwrap_or(false))
            .map(|e| e.path())
            .collect())
        .unwrap_or_default();
    hrx_files.sort();
    let show_fails = std::env::var("SHOW_FAILS").is_ok();
    let results: Vec<(String, usize, usize, usize, usize)> = hrx_files.iter().filter_map(|f| {
        let hrx_name = f.file_name().and_then(|s| s.to_str())?.to_string();
        let content = std::fs::read_to_string(f).ok()?;
        let rel_path = f.strip_prefix(&spec_root).unwrap_or(f).to_string_lossy().to_string();
            let (hp, hs, hc) = parse_hrx_to_cases(&content, &rel_path).iter().fold((0, 0, 0), |(p, s, t), case| {
                if case.expected_output.is_empty() && !case.expect_error { return (p, s + 1, t + 1); }
                let t = t + 1;
                if run_case(case) { (p + 1, s, t) } else {
                    if show_fails { tracing::info!(hrx = %hrx_name, case = %case.input_path, "FAIL"); }
                    (p, s, t)
                }
            });
            Some((hrx_name, hp, hc - hs - hp, hs, hc))
    }).collect();
    results.iter().for_each(|(name, hp, hf, hs, hc)| {
        let heval = hc - hs;
        let hpct = *hp * 100 / heval.max(1);
        tracing::info!(hrx = %name, pass = *hp, fail = *hf, skip = *hs, total = *hc, evaluated = heval, pct = hpct, "顶层HRX");
    });
    let (tp, tf, ts, tc) = results.iter().fold((0, 0, 0, 0), |(ap, af, afs, at), (_, p, f, s, t)| (ap + p, af + f, afs + s, at + t));
    let evaluated = tc - ts;
    let pct = tp * 100 / evaluated.max(1);
    tracing::info!(pass = tp, fail = tf, skip = ts, total = tc, evaluated, pct, "顶层HRX汇总");
}

// ─── use 子目录统计（100KB 限制） ──────────────────────────────────────────

#[test]
fn test_use_subdirs() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let use_dir = spec_root.join("directives/use");
    if !use_dir.exists() { tracing::info!("directives/use 目录不存在，跳过"); return; }
    let mut subdirs: Vec<String> = std::fs::read_dir(&use_dir).ok()
        .map(|entries| entries.flatten()
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.path().file_name().and_then(|s| s.to_str()).map(String::from))
            .collect())
        .unwrap_or_default();
    subdirs.sort();
    let show_fails = std::env::var("SHOW_FAILS").is_ok();
    let all_results: Vec<(String, usize, usize, usize, usize)> = subdirs.iter().map(|subdir| {
        let dir = use_dir.join(subdir);
        let mut files = Vec::new();
        collect_hrx_with_limit(&dir, &mut files, 100_000);
        let (pass, fail, skip, _failed_cases) = files.iter().flat_map(|file| {
            let Ok(content) = std::fs::read_to_string(file) else { return Vec::new(); };
            let rel_path = file.strip_prefix(&spec_root).unwrap_or(file).to_string_lossy().to_string();
            parse_hrx_to_cases(&content, &rel_path).iter().map(move |case| {
                if case.expected_output.is_empty() && !case.expect_error {
                    (0, 0, 1, String::new())
                } else if run_case(case) {
                    (1, 0, 0, String::new())
                } else {
                    (0, 1, 0, case.input_path.clone())
                }
            }).collect::<Vec<_>>()
        }).fold((0, 0, 0, Vec::new()), |(ap, af, ask, mut ac), (p, f, s, cp)| {
            if !cp.is_empty() { ac.push(cp); }
            (ap + p, af + f, ask + s, ac)
        });
        let total = pass + fail + skip;
        let evaluated = total - skip;
        let pct = pass * 100 / evaluated.max(1);
        tracing::info!(subdir = %subdir, pass, fail, skip, total, evaluated, pct, "子目录统计");
        (subdir.clone(), pass, fail, skip, total)
    }).collect();
    let (tp, tf, ts, tc) = all_results.iter().fold((0, 0, 0, 0), |(ap, af, fs, ft), (_, p, f, s, _)| {
        (ap + p, af + f, fs + s, ft + s)
    });
    let evaluated = tc - ts;
    let pct = tp * 100 / evaluated.max(1);
    tracing::info!(pass = tp, fail = tf, skip = ts, total = tc, evaluated, pct, "directives/use 汇总");
    if show_fails {
        // 失败项已在各子目录 span 中输出
    }
}
