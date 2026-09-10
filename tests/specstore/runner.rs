//! 编译 case 并记录结果。

use tracing::{info_span, info};

/// 单个 case 的编译结果。
#[derive(Debug)]
pub struct CaseResult {
    pub case_id: String,
    pub status: CaseStatus,
    pub failure_type: Option<String>,
    pub actual_css: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStatus {
    Pass,
    Fail,
    Skip,
}

impl CaseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
        }
    }
}

/// 编译单个 case 并返回结果（不写入 DB）。
pub fn run_case(
    input_path: &str,
    expected_css: &str,
    expect_error: bool,
    files: &[(String, String)],
) -> CaseResult {
    let span = info_span!("run_case", input = %input_path);
    let _enter = span.enter();

    if expected_css.is_empty() && !expect_error {
        return CaseResult {
            case_id: input_path.to_string(),
            status: CaseStatus::Skip,
            failure_type: None,
            actual_css: None,
            error: None,
        };
    }

    let tmp_dir = std::env::temp_dir().join(format!(
        "ss-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0u128, |d| d.as_nanos())
    ));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).ok();

    for (path, content) in files {
        let file_path = tmp_dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&file_path, content).ok();
    }

    let input_file = tmp_dir.join(input_path);
    let result = sasspile::compile_file_with_load_paths(
        &input_file,
        sasspile::OutputStyle::Expanded,
        vec![tmp_dir.clone()],
    );
    let _ = std::fs::remove_dir_all(&tmp_dir);

    match result {
        Ok(actual) => {
            if actual.trim() == expected_css.trim() {
                CaseResult {
                    case_id: input_path.to_string(),
                    status: CaseStatus::Pass,
                    failure_type: None,
                    actual_css: None,
                    error: None,
                }
            } else {
                info!(input = %input_path, "DIFF detected");
                CaseResult {
                    case_id: input_path.to_string(),
                    status: CaseStatus::Fail,
                    failure_type: Some("DIFF".to_string()),
                    actual_css: Some(actual.trim().to_string()),
                    error: None,
                }
            }
        }
        Err(err) => {
            if expect_error {
                CaseResult {
                    case_id: input_path.to_string(),
                    status: CaseStatus::Pass,
                    failure_type: None,
                    actual_css: None,
                    error: None,
                }
            } else {
                info!(input = %input_path, error = %err, "ERR");
                CaseResult {
                    case_id: input_path.to_string(),
                    status: CaseStatus::Fail,
                    failure_type: Some("ERR".to_string()),
                    actual_css: None,
                    error: Some(err.to_string()),
                }
            }
        }
    }
}

/// 是否期望编译失败（通过 HRX 中是否有 `error` 文件判断）。
pub fn is_expect_error(files: &[(String, String)]) -> bool {
    files.iter().any(|(p, _)| p.ends_with("/error") || p == "error")
}
