//! HRX 文件扫描 + 解析 → spec_cases 入库。

use crate::specstore::db::open_db;
use tracing::info_span;

/// HRX 文件中的一条条目（从 hrx_support.rs 提取的最小子集）。
#[derive(Debug, Clone)]
struct HrxEntry {
    path: String,
    body: String,
}

const FILE_HEADER: &str = "<===> ";
const DIR_SEPARATOR: &str = "<===>";

/// 解析 HRX 内容。
fn parse_hrx(content: &str) -> Result<Vec<HrxEntry>, String> {
    let mut entries = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line == DIR_SEPARATOR {
            if lines.peek().is_some_and(|next| next.starts_with("==")) {
                lines.next();
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix(FILE_HEADER) {
            let path = rest.trim().to_string();
            if path.is_empty() {
                continue;
            }
            let mut body_lines = Vec::new();
            while let Some(body_line) = lines.peek() {
                if body_line.starts_with(FILE_HEADER) || *body_line == DIR_SEPARATOR {
                    break;
                }
                body_lines.push(*body_line);
                lines.next();
            }
            entries.push(HrxEntry { path, body: body_lines.join("\n") });
        }
    }

    if entries.is_empty() {
        return Err("no entries found".to_string());
    }
    Ok(entries)
}

/// HRX case（解析结果）。
struct HrxCase {
    files: Vec<(String, String)>,
    input_path: String,
    expected_output: String,
    expect_error: bool,
}

/// 解析 HRX 内容为测试用例列表。
fn parse_hrx_to_cases(content: &str, hrx_rel_path: &str) -> Vec<HrxCase> {
    let Ok(entries) = parse_hrx(content) else {
        return Vec::new();
    };

    let prefix = hrx_rel_path.strip_suffix(".hrx").unwrap_or(hrx_rel_path);

    // 按目录分组
    let mut dirs: std::collections::BTreeMap<String, Vec<(String, String)>> = std::collections::BTreeMap::new();
    for entry in &entries {
        if entry.path.is_empty() {
            continue;
        }
        let parts: Vec<&str> = entry.path.split('/').collect();
        if parts.len() == 1 {
            dirs.entry(".".to_string()).or_default().push((entry.path.clone(), entry.body.clone()));
        } else {
            let dir = parts[..parts.len() - 1].join("/");
            let file = parts.last().expect("unexpected").to_string();
            dirs.entry(dir).or_default().push((file, entry.body.clone()));
        }
    }

    // 展平所有文件（带 HRX 目录前缀）
    let mut all_files: Vec<(String, String)> = dirs
        .iter()
        .flat_map(|(dir_path, files)| {
            let dp = dir_path.clone();
            files.iter().map(move |(f, c)| {
                let base = if dp == "." { f.clone() } else { format!("{dp}/{f}") };
                let prefixed = if prefix.is_empty() {
                    base
                } else {
                    format!("{prefix}/{base}")
                };
                (prefixed, c.clone())
            })
        })
        .filter(|(p, _)| p.ends_with(".scss") || p.ends_with(".css") || p.ends_with(".sass"))
        .collect();

    // 颜色目录注入 _utils.scss
    if prefix.starts_with("core_functions/color/") && !prefix.ends_with("utils") {
        let utils_path = "core_functions/color/_utils.scss";
        if !all_files.iter().any(|(p, _)| p == utils_path) {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let utils_file = format!("{manifest_dir}/sass-spec/spec/core_functions/color/_utils.scss");
            if let Ok(c) = std::fs::read_to_string(&utils_file) {
                all_files.push((utils_path.to_string(), c));
            }
        }
    }

    // list 目录注入 _utils.scss
    if prefix.starts_with("core_functions/list/") {
        let utils_path = "core_functions/list/_utils.scss";
        if !all_files.iter().any(|(p, _)| p == utils_path) {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let utils_file = format!("{manifest_dir}/sass-spec/spec/core_functions/list/_utils.scss");
            if let Ok(c) = std::fs::read_to_string(&utils_file) {
                all_files.push((utils_path.to_string(), c));
            }
        }
    }

    // 为每个含 input.scss/.sass 的目录构建 case
    let mut cases = Vec::new();
    for (dir_path, files) in &dirs {
        let Some((input_name, _)) = files.iter().find(|(f, _)| f == "input.scss" || f == "input.sass") else {
            continue;
        };
        let input_base = if dir_path == "." {
            input_name.clone()
        } else {
            format!("{dir_path}/{input_name}")
        };
        let input_path = if prefix.is_empty() {
            input_base
        } else {
            format!("{prefix}/{input_base}")
        };

        let expected_output = files
            .iter()
            .find(|(f, _)| f == "output.css")
            .map(|(_, c)| c.clone())
            .unwrap_or_default();
        let expect_error = files.iter().any(|(f, _)| f == "error");

        cases.push(HrxCase {
            files: all_files.clone(),
            input_path,
            expected_output,
            expect_error,
        });
    }
    cases
}

/// 扫描 spec 目录，将所有 HRX case 存入数据库。
pub fn index_all(spec_root: &std::path::Path, db_path: &std::path::Path) -> Result<usize, String> {
    let _span = info_span!("spec_store_index", spec_root = %spec_root.display());
    let _enter = _span.enter();

    let mut conn = open_db(db_path)?;
    let hrx_files = collect_hrx_files(spec_root);
    let mut total = 0usize;

    for hrx_path in &hrx_files {
        let rel_path = hrx_path
            .strip_prefix(spec_root)
            .unwrap_or(hrx_path)
            .to_string_lossy();
        total += index_one(hrx_path, &rel_path, &mut conn)?;
    }

    tracing::info!(total_cases = total, "indexing complete");
    Ok(total)
}

fn index_one(
    hrx_path: &std::path::Path,
    rel_path: &str,
    conn: &mut rusqlite::Connection,
) -> Result<usize, String> {
    let content = std::fs::read_to_string(hrx_path)
        .map_err(|e| format!("read {}: {e}", hrx_path.display()))?;

    let cases = parse_hrx_to_cases(&content, rel_path);
    let function = extract_function(rel_path);
    let dir = extract_dir(rel_path);

    let tx = conn.transaction().map_err(|e| format!("begin tx: {e}"))?;

    let mut count = 0usize;
    for case in &cases {
        if case.input_path.ends_with(".sass") {
            continue;
        }
        tx.execute(
            "INSERT OR REPLACE INTO spec_cases(case_id, hrx_file, function, dir, input_scss, expected_css, expect_error)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                case.input_path,
                rel_path,
                function,
                dir,
                get_input_content(&case.files, &case.input_path),
                case.expected_output,
                if case.expect_error { 1 } else { 0 },
            ],
        )
        .map_err(|e| format!("insert case {}: {e}", case.input_path))?;

        for (file_path, content) in &case.files {
            tx.execute(
                "INSERT OR REPLACE INTO case_files(case_id, file_path, content) VALUES(?1, ?2, ?3)",
                rusqlite::params![case.input_path, file_path, content],
            )
            .map_err(|e| format!("insert file {file_path}: {e}"))?;
        }
        count += 1;
    }

    tx.commit().map_err(|e| format!("commit: {e}"))?;
    Ok(count)
}

fn get_input_content(files: &[(String, String)], input_path: &str) -> String {
    files
        .iter()
        .find(|(p, _)| p == input_path)
        .map(|(_, c)| c.clone())
        .unwrap_or_default()
}

fn extract_function(rel_path: &str) -> Option<String> {
    let stem = rel_path.strip_suffix(".hrx").unwrap_or(rel_path);
    let parts: Vec<&str> = stem.split('/').collect();
    if parts.len() == 3 && parts[0] == "core_functions" {
        return Some(format!("{}.{}", parts[1], parts[2]));
    }
    if parts.len() > 3 && parts[0] == "core_functions" {
        return Some(format!("{}.{}", parts[1], parts[2]));
    }
    None
}

fn extract_dir(rel_path: &str) -> String {
    let stem = rel_path.strip_suffix(".hrx").unwrap_or(rel_path);
    let parts: Vec<&str> = stem.split('/').collect();
    if parts.is_empty() {
        return String::new();
    }
    parts[..parts.len().min(2)].join("/")
}

fn collect_hrx_files(spec_root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let skip = ["libsass", "libsass-closed-issues", "libsass-todo-issues", "libsass-todo-tests", "non_conformant"];
    let mut files = Vec::new();
    collect_recursive(spec_root, spec_root, &mut files, &skip);
    files
}

fn collect_recursive(dir: &std::path::Path, spec_root: &std::path::Path, files: &mut Vec<std::path::PathBuf>, skip: &[&str]) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let rel = path.strip_prefix(spec_root).unwrap_or(&path);
            let first = rel.components().next().map(|c| c.as_os_str().to_string_lossy().to_string()).unwrap_or_default();
            if skip.contains(&first.as_str()) {
                continue;
            }
            collect_recursive(&path, spec_root, files, skip);
        } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
            if let Ok(meta) = std::fs::metadata(&path)
                && meta.len() < 100_000
            {
                files.push(path);
            }
        }
    }
}
