//! 共享测试工具——CSS diff 和辅助函数。
//!
//! 被 `cf_diag.rs` 和 `minimize.rs` 引用。

#![allow(dead_code)]

/// CSS diff 行类型。
#[derive(Debug, Clone, PartialEq)]
pub enum DiffLine {
    Changed {
        line: usize,
        expected: String,
        actual: String,
    },
    ExtraExpected {
        line: usize,
        content: String,
    },
    ExtraActual {
        line: usize,
        content: String,
    },
}

/// CSS diff 结果。
#[derive(Debug, Clone, Default)]
pub struct DiffResult {
    pub lines: Vec<DiffLine>,
}

impl DiffResult {
    /// 分类错误模式——用于统计。
    pub fn classify(&self) -> &'static str {
        if self.lines.is_empty() {
            return "identical";
        }
        let has_missing = self
            .lines
            .iter()
            .any(|l| matches!(l, DiffLine::ExtraExpected { .. }));
        let has_extra = self
            .lines
            .iter()
            .any(|l| matches!(l, DiffLine::ExtraActual { .. }));
        if has_missing && !has_extra {
            "missing_output"
        } else if has_extra && !has_missing {
            "extra_output"
        } else {
            "content_diff"
        }
    }

    /// 格式化为终端可读文本。
    pub fn format_terminal(&self) -> String {
        self.lines
            .iter()
            .map(|l| match l {
                DiffLine::Changed {
                    line,
                    expected,
                    actual,
                } => format!("  L{line}: exp='{expected}' act='{actual}'"),
                DiffLine::ExtraExpected { line, content } => {
                    format!("  L{line}: exp='{content}' act=(missing)")
                }
                DiffLine::ExtraActual { line, content } => {
                    format!("  L{line}: exp=(missing) act='{content}'")
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// CSS 逐行 diff——返回结构化差异结果。
///
/// 发出 tracing events：
/// - `cssdiff` info: diff 检测摘要
/// - `cssdiff` debug: 每行差异详情
pub fn diff_css(expected: &str, actual: &str) -> DiffResult {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    tracing::info!(
        target: "cssdiff",
        n_expected = expected_lines.len(),
        n_actual = actual_lines.len(),
        "diff detected"
    );

    let mut result = DiffResult::default();
    let max = expected_lines.len().max(actual_lines.len());

    for i in 0..max {
        let e = expected_lines.get(i).copied();
        let a = actual_lines.get(i).copied();
        match (e, a) {
            (Some(e), Some(a)) if e == a => {}
            (Some(e), Some(a)) => {
                tracing::debug!(
                    target: "cssdiff",
                    line = i + 1,
                    expected = e,
                    actual = a,
                    "line diff"
                );
                result.lines.push(DiffLine::Changed {
                    line: i + 1,
                    expected: e.to_string(),
                    actual: a.to_string(),
                });
            }
            (Some(e), None) => {
                tracing::debug!(
                    target: "cssdiff",
                    line = i + 1,
                    expected = e,
                    actual = "(missing)",
                    "line missing in actual"
                );
                result.lines.push(DiffLine::ExtraExpected {
                    line: i + 1,
                    content: e.to_string(),
                });
            }
            (None, Some(a)) => {
                tracing::debug!(
                    target: "cssdiff",
                    line = i + 1,
                    expected = "(missing)",
                    actual = a,
                    "extra line in actual"
                );
                result.lines.push(DiffLine::ExtraActual {
                    line: i + 1,
                    content: a.to_string(),
                });
            }
            (None, None) => break,
        }
    }

    result
}
