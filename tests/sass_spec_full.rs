//! sass-spec 全量统计——使用内联 hrx_support 模块，不依赖 hrx-auditor crate。
//!
//! manifest 在 `tests/spec_manifest.rs` 中定义 `SKIP_DIRS` 跳过列表。
//! HRX 解析：所有条目写入同一临时目录，路径加 HRX 名作前缀。
//! 这使 `@use 'callable/arguments/mixin/utils'` 等跨组引用能正确解析。

mod hrx_support;
mod spec_manifest;

use hrx_support::{parse_hrx_to_cases, run_case};
use spec_manifest::SKIP_DIRS;
use std::path::{Path, PathBuf};
use tracing::{info, info_span};

// ─── 文件收集 ─────────────────────────────────────────────────────────────

/// 递归收集目录下所有 .hrx 文件。
fn collect_hrx_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(dir, &mut files);
    files
}

fn collect_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_recursive(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() < 100_000
            {
                files.push(path);
            }
        }
    }
}

/// 使用 manifest 跳过列表收集 HRX 文件。
fn collect_hrx_files_with_manifest(
    dir: &Path,
    spec_root: &Path,
) -> (Vec<PathBuf>, usize) {
    let all = collect_hrx_files(dir);
    let mut kept = Vec::new();
    let mut skipped = 0;
    for path in all {
        let rel = path
            .strip_prefix(spec_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        if SKIP_DIRS
            .iter()
            .any(|skip| rel.starts_with(skip) || rel == *skip)
        {
            skipped += 1;
            continue;
        }
        kept.push(path);
    }
    (kept, skipped)
}

// ─── 统计运行 ─────────────────────────────────────────────────────────────

/// 按 spec 一级目录运行并统计。
fn run_spec_dir(spec_root: &Path, dir_name: &str) -> (usize, usize, usize, usize) {
    let span = info_span!("run_spec_dir", dir = %dir_name);
    let _enter = span.enter();

    let dir = spec_root.join(dir_name);
    if !dir.exists() {
        return (0, 0, 0, 0);
    }

    let (files, skipped) = collect_hrx_files_with_manifest(&dir, spec_root);

    let (mut pass, mut fail, mut skip, mut cases) = (0, 0, 0, 0);
    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let rel_path = file
                .strip_prefix(spec_root)
                .unwrap_or(file)
                .to_string_lossy()
                .to_string();
            for case in &parse_hrx_to_cases(&content, &rel_path) {
                cases += 1;
                if case.expected_output.is_empty() && !case.expect_error {
                    skip += 1;
                    continue;
                }
                if run_case(case) {
                    pass += 1;
                } else {
                    fail += 1;
                    if std::env::var("SHOW_FAILS").is_ok() {
                        info!(case = %case.input_path, hrx = %rel_path, "FAIL");
                    }
                }
            }
        }
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

// ─── 测试入口 ─────────────────────────────────────────────────────────────

#[test]
fn test_import_use_forward() {
    sasspile::init_tracing_otel();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
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

#[test]
fn test_directives_subdirs() {
    sasspile::init_tracing_otel();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");

    let subdirs = [
        "directives/at_root",
        "directives/extend",
        "directives/for",
        "directives/forward",
        "directives/function",
        "directives/if",
        "directives/import",
        "directives/mixin",
        "directives/use",
    ];

    let (mut tp, mut tf, mut ts, mut tc) = (0, 0, 0, 0);
    for sub in &subdirs {
        let (p, f, s, c) = run_spec_dir(&spec_root, sub);
        let eval = c - s;
        let pct = p * 100 / eval.max(1);
        info!(sub, pass = p, fail = f, skip = s, total = c, evaluated = eval, pct = pct, "子目录");
        tp += p;
        tf += f;
        ts += s;
        tc += c;
    }

    // top-level hrx files
    for hrx in &["debug", "each", "error", "return", "warn", "while"] {
        let dir = spec_root.join(format!("directives/{hrx}.hrx"));
        if dir.exists() {
            if let Ok(content) = std::fs::read_to_string(&dir) {
                let hrx_rel = format!("directives/{hrx}.hrx");
                let (mut hp, mut hf, mut hs, mut hc) = (0, 0, 0, 0);
                for case in &parse_hrx_to_cases(&content, &hrx_rel) {
                    hc += 1;
                    if case.expected_output.is_empty() && !case.expect_error {
                        hs += 1;
                        continue;
                    }
                    if run_case(case) {
                        hp += 1;
                    } else {
                        hf += 1;
                    }
                }
                let heval = hc - hs;
                let hpct = hp * 100 / heval.max(1);
                info!(hrx, pass = hp, fail = hf, skip = hs, total = hc, evaluated = heval, pct = hpct, "hrx文件");
                tp += hp;
                tf += hf;
                ts += hs;
                tc += hc;
            }
        }
    }

    let evaluated = tc - ts;
    let pct = tp * 100 / evaluated.max(1);
    info!(
        pass = tp,
        fail = tf,
        skip = ts,
        total = tc,
        evaluated = evaluated,
        pct = pct,
        "directives 子目录汇总"
    );
}

#[test]
fn test_sass_spec_full_stats() {
    sasspile::init_tracing_otel();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");

    let dirs = [
        "variables",
        "values",
        "css",
        "operators",
        "expressions",
        "directives",
        "core_functions",
        "parser",
        "callable",
    ];

    let (mut total_pass, mut total_fail, mut total_skip, mut total_cases) = (0, 0, 0, 0);

    for dir in &dirs {
        let (pass, fail, skip, cases) = run_spec_dir(&spec_root, dir);
        let eval = cases - skip;
        let pct = pass * 100 / eval.max(1);
        info!(dir, pass, fail, skip, total = cases, evaluated = eval, pct, "sass-spec 目录");
        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
        total_cases += cases;
    }

    let evaluated = total_cases - total_skip;
    let overall_pct = total_pass * 100 / evaluated.max(1);
    info!(
        pass = total_pass,
        fail = total_fail,
        skip = total_skip,
        total = total_cases,
        evaluated = evaluated,
        pct = overall_pct,
        "sass-spec 全量统计（已跳过不支持的目录）"
    );
}

/// 颜色子目录统计——已跳过（颜色测试需手动 --ignored 触发）。
#[test]
#[ignore]
fn test_core_functions_subdirs() {
    sasspile::init_tracing_otel();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");

    let subdirs = [
        "color/to_space",
        "color/to_gamut",
        "color/adjust",
        "color/change",
        "color/scale",
        "color/channel",
        "color/mix",
        "color/hsl",
        "color/hwb",
        "color/rgb",
        "color/invert",
        "color/is_powerless",
        "color/lab",
        "color/lch",
        "color/oklab",
        "color/oklch",
        "color/color",
    ];

    let (mut tp, mut tf, mut ts, mut tc) = (0, 0, 0, 0);
    for sub in &subdirs {
        let full = format!("core_functions/{}", sub);
        let (p, f, s, c) = run_spec_dir(&spec_root, &full);
        let eval = c - s;
        let pct = p * 100 / eval.max(1);
        info!(sub, pass = p, fail = f, skip = s, total = c, evaluated = eval, pct, "cf子目录");
        tp += p;
        tf += f;
        ts += s;
        tc += c;
    }

    let evaluated = tc - ts;
    let pct = tp * 100 / evaluated.max(1);
    info!(pass = tp, fail = tf, skip = ts, total = tc, evaluated, pct, "cf子目录汇总");
}
