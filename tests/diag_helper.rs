//! 诊断工具共享辅助函数（供多个诊断测试文件复用）。
//!
//! 每个 `tests/*.rs` 独立编译，通过 `#[path = "diag_helper.rs"] mod diag_helper;` 导入。

#![allow(dead_code)]

use std::path::{Path, PathBuf};

// ─── 导入依赖模块 ─────────────────────────────────────────────────────────

pub mod hrx_support;
use hrx_support::{HrxArchive, HrxEntry, Vfs, parse_hrx};
pub use hrx_support::{parse_hrx_to_cases, run_case};

pub mod spec_manifest;

pub mod common;
pub use common::diff_css;

// ─── 核心数据结构 ─────────────────────────────────────────────────────────

/// HRX 测试用例。
#[derive(Clone, Debug)]
pub struct HrxCase {
    pub files: Vec<(String, String)>,
    pub input_path: String,
    pub expected_output: String,
    pub expect_error: bool,
    pub name: String,
}

// ─── HRX 解析管线 ─────────────────────────────────────────────────────────

/// 解析 HRX entries 分组——按空路径分隔。
#[must_use]
pub fn group_entries(entries: &[HrxEntry]) -> Vec<Vec<&HrxEntry>> {
    entries.iter().fold(Vec::new(), |mut acc, entry| {
        if entry.path.is_empty() {
            acc.push(Vec::new());
        } else if let Some(last) = acc.last_mut() {
            last.push(entry);
        } else {
            acc.push(vec![entry]);
        }
        acc
    })
}

/// 从 HRX entries 分组构建文件列表（不含空组）。
#[must_use]
pub fn filter_groups(groups: Vec<Vec<&HrxEntry>>) -> Vec<Vec<HrxEntry>> {
    groups.into_iter()
        .filter(|g| !g.is_empty())
        .map(|g| g.into_iter().cloned().collect())
        .collect()
}

/// 解析 HRX 内容为测试用例列表。
#[must_use]
pub fn parse_cases(content: &str) -> Vec<HrxCase> {
    let Ok(archive) = parse_hrx(content) else { return Vec::new(); };
    filter_groups(group_entries(&archive.entries)).iter().flat_map(|group| {
        let group_archive = HrxArchive { entries: group.clone() };
        let vfs = Vfs::from_archive(&group_archive);
        let dirs = vfs.walk();
        let files: Vec<(String, String)> = dirs.iter().flat_map(|(dp, fs)| {
            fs.iter().map(move |(f, c)| {
                if dp == "." { (f.clone(), c.clone()) } else { (format!("{dp}/{f}"), c.clone()) }
            })
        }).filter(|(p, _)| {
            (p.ends_with(".scss") || p.ends_with(".css") || p.ends_with(".sass")) && !p.contains("/sass/")
        }).collect();
        dirs.iter().filter_map(move |(dp, fs)| {
            let (input_name, _) = fs.iter().find(|(f, _)| f == "input.scss")?;
            let input_path = if dp == "." { input_name.clone() } else { format!("{dp}/{input_name}") };
            let expected_output = fs.iter()
                .find(|(f, _)| f == "output.css")
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let expect_error = fs.iter().any(|(f, _)| f == "error");
            let name = if dp == "." { String::new() } else { dp.clone() };
            Some(HrxCase { files: files.clone(), input_path, expected_output, expect_error, name })
        }).collect::<Vec<_>>()
    }).collect()
}

// ─── 文件系统辅助 ─────────────────────────────────────────────────────────

/// 递归收集 HRX 文件（<50KB）。
pub fn collect_hrx(dir: &Path, files: &mut Vec<PathBuf>) {
    let _span = tracing::trace_span!("collect_hrx", dir = %dir.display()).entered();
    if let Ok(entries) = std::fs::read_dir(dir) {
        entries.flatten().for_each(|entry| {
            let p = entry.path();
            if p.is_dir() {
                collect_hrx(&p, files);
            } else if p.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&p)
                && meta.len() < 50_000 {
                files.push(p);
            }
        });
    }
}

/// 递归收集 HRX 文件（自定义大小限制）。
pub fn collect_hrx_with_limit(dir: &Path, files: &mut Vec<PathBuf>, size_limit: u64) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        entries.flatten().for_each(|entry| {
            let p = entry.path();
            if p.is_dir() {
                collect_hrx_with_limit(&p, files, size_limit);
            } else if p.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&p)
                && meta.len() < size_limit {
                files.push(p);
            }
        });
    }
}

// ─── 编译执行器 ───────────────────────────────────────────────────────────

/// 编译单个测试用例。
pub fn compile_case(case: &HrxCase, spec_root: &Path, hrx_dir: &Path, hrx_stem: &str) -> Result<String, String> {
    let _span = tracing::trace_span!("compile_case", input = %case.input_path).entered();
    let tmp = std::env::temp_dir().join(format!("sass-diag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|e| format!("{e}"))?;
    for (path, content) in &case.files {
        let target = if path.starts_with(&format!("{hrx_stem}/")) {
            tmp.join(path)
        } else {
            tmp.join(hrx_stem).join(path)
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&target, content).map_err(|e| format!("{e}"))?;
    }
    if let Ok(entries) = std::fs::read_dir(hrx_dir) {
        entries.flatten().for_each(|entry| {
            let p = entry.path();
            if (p.extension().and_then(|s| s.to_str()) == Some("scss")
                || p.extension().and_then(|s| s.to_str()) == Some("css"))
                && let Ok(content) = std::fs::read_to_string(&p)
            {
                let fname = p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                let _ = std::fs::write(tmp.join(&fname), content);
            }
        });
    }
    let input = if case.input_path.starts_with(&format!("{hrx_stem}/")) {
        tmp.join(&case.input_path)
    } else {
        tmp.join(hrx_stem).join(&case.input_path)
    };
    let result = sasspile::compile_file_with_load_paths(&input, sasspile::OutputStyle::Expanded, vec![spec_root.to_path_buf()]);
    let _ = std::fs::remove_dir_all(&tmp);
    result.map_err(|e| format!("{e}"))
}

/// 通用测试执行器——写入临时目录后编译比对。
#[must_use]
pub fn run_case_generic(case: &HrxCase, load_paths: &[PathBuf], prefix: &str) -> Option<String> {
    if case.expected_output.is_empty() && !case.expect_error { return None; }
    if case.files.iter().map(|(_, c)| c.len()).sum::<usize>() > 50_000 { return Some("TOO_LARGE".to_string()); }
    let tmp = std::env::temp_dir().join(format!("{prefix}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();
    for (path, content) in &case.files {
        let fp = tmp.join(path);
        if let Some(parent) = fp.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&fp, content).ok();
    }
    let input = tmp.join(&case.input_path);
    let result = sasspile::compile_file_with_load_paths(&input, sasspile::OutputStyle::Expanded, load_paths.to_vec());
    let _ = std::fs::remove_dir_all(&tmp);
    match result {
        Ok(actual) => if actual.trim() == case.expected_output.trim() { None }
            else { Some(format!("--- FAIL: {} ---\nEXPECTED:\n{}\nACTUAL:\n{}\n", case.name, case.expected_output.trim(), actual.trim())) }
        Err(e) => if case.expect_error { None }
            else { Some(format!("--- FAIL: {} ---\nEXPECTED:\n{}\nERROR:\n{}\n", case.name, case.expected_output.trim(), e)) }
    }
}

// ─── 统计聚合 ─────────────────────────────────────────────────────────────

/// 处理单个文件内的所有 case（面向 stats 统计）。
#[must_use]
pub fn process_cases_for_stats(file: &Path, spec_root: &Path) -> (usize, usize, usize) {
    let Ok(content) = std::fs::read_to_string(file) else { return (0, 0, 0) };
    let stem = file.file_stem()
        .expect("unexpected failure: missing file stem")
        .to_string_lossy()
        .to_string();
    let parent = file.parent().unwrap_or(Path::new("."));
    parse_cases(&content).iter().fold((0, 0, 0), |(pass, fail, total), case| {
        if case.expected_output.is_empty() && !case.expect_error { return (pass, fail, total); }
        match compile_case(case, spec_root, parent, &stem) {
            Ok(_actual) if case.expect_error => (pass, fail + 1, total + 1),
            Ok(actual) if actual.trim() == case.expected_output.trim() => (pass + 1, fail, total + 1),
            Ok(_) => (pass, fail + 1, total + 1),
            Err(_) if case.expect_error => (pass + 1, fail, total + 1),
            Err(_) => (pass, fail + 1, total + 1),
        }
    })
}

/// 统计单个目录的 pass/fail。
#[must_use]
pub fn stats_subdir(subdir: &str) -> (usize, usize, usize) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let dir = spec_root.join(subdir);
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);
    let (pass, fail, total) = files.iter()
        .map(|f| process_cases_for_stats(f, &spec_root))
        .fold((0, 0, 0), |(ap, af, at), (p, f, t)| (ap + p, af + f, at + t));
    (pass, fail, total)
}

// ─── 分类辅助 ─────────────────────────────────────────────────────────────

/// 分类错误消息为可读标签。
#[must_use]
pub fn classify_error(msg: &str) -> &'static str {
    if msg.contains("Undefined") || msg.contains("undefined") { "undefined" }
    else if msg.contains("Parse error") || msg.contains("Syntax") { "syntax" }
    else if msg.contains("Eval") || msg.contains("type") { "eval" }
    else if msg.contains("Module") || msg.contains("Cannot") { "module" }
    else { "other_err" }
}

/// 计算单个文件的 diff 诊断（返回失败数量 + 展示用字符串）。
pub fn file_diff_report(
    file: &Path,
    spec_root: &Path,
    max_show: usize,
    shown: &mut usize,
) -> usize {
    let Ok(content) = std::fs::read_to_string(file) else { return 0 };
    let stem = file.file_stem()
        .expect("unexpected failure: missing file stem")
        .to_string_lossy()
        .to_string();
    parse_cases(&content).iter().fold(0, |fail_count, case| {
        if case.expected_output.is_empty() && !case.expect_error { return fail_count; }
        let name = case.input_path.strip_suffix("input.scss")
            .unwrap_or(&case.input_path)
            .trim_end_matches('/')
            .to_string();
        match compile_case(case, spec_root, file.parent().unwrap_or(Path::new(".")), &stem) {
            Ok(_actual) if case.expect_error => {
                if *shown < max_show { tracing::error!("{stem}/{name}: ERR_EXP_OK"); *shown += 1; }
                fail_count + 1
            }
            Ok(actual) if actual.trim() != case.expected_output.trim() => {
                if *shown < max_show {
                    tracing::error!("{stem}/{name}:\n  exp: {}\n  got: {}", case.expected_output.trim(), actual.trim());
                    *shown += 1;
                }
                fail_count + 1
            }
            Err(err) if !case.expect_error => {
                if *shown < max_show { tracing::error!("{stem}/{name}: {err}"); *shown += 1; }
                fail_count + 1
            }
            _ => fail_count
        }
    })
}

/// 收集目录文件并运行 diff 诊断。
#[must_use]
pub fn run_dir_diff(subdir: &str, max_show: usize) -> (usize, usize) {
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let mut files = Vec::new();
    collect_hrx(&spec_root.join(subdir), &mut files);
    let mut shown = 0;
    let fail = files.iter()
        .map(|f| file_diff_report(f, &spec_root, max_show, &mut shown))
        .sum::<usize>();
    let total = files.iter()
        .map(|f| {
            let Ok(content) = std::fs::read_to_string(f) else { return 0; };
            parse_cases(&content).iter()
                .filter(|c| !c.expected_output.is_empty() || c.expect_error)
                .count()
        })
        .sum::<usize>();
    (fail, total)
}

/// 文件名简化的 import macro：每个测试文件按需 use 所需函数。
#[allow(unused_imports)]
pub mod prelude {
    pub use super::{
        classify_error, collect_hrx, collect_hrx_with_limit, compile_case,
        diff_css, parse_cases, parse_hrx_to_cases, process_cases_for_stats,
        run_case, run_case_generic, run_dir_diff, stats_subdir,
    };
    
    
    
}
