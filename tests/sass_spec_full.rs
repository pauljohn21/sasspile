//! sass-spec 全量统计——支持文件路径解析（@import/@use/@forward）。

use std::path::Path;
use tracing::info;

/// 递归遍历目录，收集所有 HRX 文件路径。
fn collect_hrx_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_hrx_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() < 100_000 {
                        files.push(path);
                    }
                }
            }
        }
    }
}

/// HRX 测试用例——包含所有文件和期望输出。
struct HrxCase {
    /// (相对路径, 内容) — 所有 .scss/.sass 文件
    files: Vec<(String, String)>,
    /// input.scss 的路径（相对于 HRX 根）
    input_path: String,
    /// 期望输出
    expected_output: String,
    /// 是否期望错误
    expect_error: bool,
}

/// 解析 HRX——提取所有文件和测试用例。
fn parse_hrx(content: &str) -> Vec<HrxCase> {
    // 第一步：收集所有 (path, content) 对
    let mut files: Vec<(String, String)> = Vec::new();
    let mut current_path = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with("<===>") {
            if !current_path.is_empty() {
                files.push((current_path.clone(), current_content));
            }
            current_path = line.trim_start_matches("<===>").trim().to_string();
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if !current_path.is_empty() {
        files.push((current_path, current_content));
    }

    // 第二步：为每个 input.scss 构建测试用例
    let mut cases = Vec::new();
    for (path, _input) in &files {
        if path.ends_with("input.scss") {
            let base = path.strip_suffix("input.scss").unwrap_or(path).to_string();
            let output_path = format!("{base}output.css");
            let error_path = format!("{base}error");

            let expected_output = files.iter()
                .find(|(p, _)| p == &output_path)
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let expect_error = files.iter().any(|(p, _)| p == &error_path);

            // 收集所有 .scss/.css 文件（排除 sass/ 变体）
            let case_files: Vec<(String, String)> = files.iter()
                .filter(|(p, _)| {
                    (p.ends_with(".scss") || p.ends_with(".css"))
                    && !p.contains("/sass/")
                })
                .map(|(p, c)| (p.clone(), c.clone()))
                .collect();

            cases.push(HrxCase {
                files: case_files,
                input_path: path.clone(),
                expected_output,
                expect_error,
            });
        }
    }
    cases
}

/// 运行单个测试用例——写入临时目录并用 compile_file 编译。
fn run_case(case: &HrxCase) -> bool {
    if case.expected_output.is_empty() && !case.expect_error {
        return true; // 跳过
    }

    // 输入大小限制
    let total_size: usize = case.files.iter().map(|(_, c)| c.len()).sum();
    if total_size > 50000 {
        return false;
    }

    // 创建临时目录
    let tmp_dir = std::env::temp_dir().join(format!("sass-spec-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();

    // 写入所有文件
    for (path, content) in &case.files {
        let file_path = tmp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).ok();
    }

    // 编译 input.scss
    let input_file = tmp_dir.join(&case.input_path);
    let result = sasspile::compile_file(&input_file, sasspile::OutputStyle::Expanded);

    // 清理
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

#[test]
fn test_import_use_forward() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
    for subdir in &["directives/import", "directives/use", "directives/forward"] {
        let dir = spec_root.join(subdir);
        if !dir.exists() { continue; }
        let mut files = Vec::new();
        collect_hrx_files(&dir, &mut files);
        let (mut pass, mut fail, mut skip, mut cases) = (0, 0, 0, 0);
        for file in &files {
            if let Ok(content) = std::fs::read_to_string(file) {
                for case in &parse_hrx(&content) {
                    cases += 1;
                    if case.expected_output.is_empty() && !case.expect_error { skip += 1; continue; }
                    if run_case(case) { pass += 1; } else { fail += 1; }
                }
            }
        }
        let evaluated = cases - skip;
        let pct = if evaluated > 0 { pass * 100 / evaluated } else { 0 };
        info!(subdir = subdir, pass = pass, fail = fail, skip = skip, total = cases, pct = pct, "sass-spec 子目录");
    }
}

#[test]
fn test_sass_spec_full_stats() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");

    let subdirs = [
        "variables",
        "values/numbers",
        "values/colors",
        "values/lists",
        "values/maps",
        "css",
        "operators",
        "expressions",
        "directives/if",
        "directives/for",
        "directives/each",
        "directives/while",
        "directives/mixin",
        "directives/function",
        "directives/use",
        "directives/forward",
        "directives/import",
        "directives/extend",
        "directives/at_root",
        "directives/media",
        "directives/supports",
        "core_functions/math",
        "core_functions/string",
        "core_functions/list",
        "core_functions/map",
        "core_functions/color",
        "core_functions/meta",
        "core_functions/selector",
        "parser",
        "non_conformant",
        "libsass",
        "libsass-closed-issues",
        "callable",
    ];

    let mut total_pass = 0usize;
    let mut total_fail = 0usize;
    let mut total_skip = 0usize;
    let mut total_cases = 0usize;

    for subdir in &subdirs {
        let dir = spec_root.join(subdir);
        if !dir.exists() {
            continue;
        }

        let mut files = Vec::new();
        collect_hrx_files(&dir, &mut files);

        let mut pass = 0usize;
        let mut fail = 0usize;
        let mut skip = 0usize;
        let mut cases = 0usize;

        for file in &files {
            if let Ok(content) = std::fs::read_to_string(file) {
                for case in &parse_hrx(&content) {
                    cases += 1;
                    if case.expected_output.is_empty() && !case.expect_error {
                        skip += 1;
                        continue;
                    }
                    if run_case(case) {
                        pass += 1;
                    } else {
                        fail += 1;
                    }
                }
            }
        }

        total_pass += pass;
        total_fail += fail;
        total_skip += skip;
        total_cases += cases;
        let evaluated = cases - skip;
        let pct = if evaluated > 0 { pass * 100 / evaluated } else { 0 };
        info!(subdir = subdir, pass = pass, fail = fail, skip = skip, total = cases, pct = pct, "sass-spec 子目录");
    }

    let evaluated = total_cases - total_skip;
    let overall_pct = if evaluated > 0 { total_pass * 100 / evaluated } else { 0 };
    info!(
        pass = total_pass,
        fail = total_fail,
        skip = total_skip,
        total = total_cases,
        evaluated = evaluated,
        pct = overall_pct,
        "sass-spec 全量统计"
    );
}
