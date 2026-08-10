//! sass-spec 全量统计——用和 sass_spec.rs 相同的 HRX 解析逻辑。

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

/// HRX 测试用例。
struct HrxCase {
    input: String,
    expected_output: String,
    expect_error: bool,
}

/// 解析 HRX——和 sass_spec.rs 完全相同的逻辑。
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

    // 第二步：为每个 input.scss 找对应的 output.css / error
    let mut cases = Vec::new();
    for (path, input) in &files {
        if path.ends_with("input.scss") {
            let base = path.strip_suffix("input.scss").unwrap_or(path).to_string();
            let output_path = format!("{base}output.css");
            let error_path = format!("{base}error");

            let expected_output = files.iter()
                .find(|(p, _)| p == &output_path)
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let expect_error = files.iter().any(|(p, _)| p == &error_path);

            cases.push(HrxCase {
                input: input.clone(),
                expected_output,
                expect_error,
            });
        }
    }
    cases
}

/// 运行单个测试用例。
fn run_case(case: &HrxCase) -> bool {
    if case.input.len() > 10000 {
        return false;
    }

    let result = sasspile::compile_expanded(&case.input);

    if case.expect_error {
        result.is_err()
    } else if case.expected_output.is_empty() {
        // 无期望输出——跳过
        true // 不计入失败
    } else {
        match result {
            Ok(actual) => actual.trim() == case.expected_output.trim(),
            Err(_) => false,
        }
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
