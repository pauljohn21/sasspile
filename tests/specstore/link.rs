//! CodeGraph 调用链 ↔ spec cases 桥接。

use rusqlite::Connection;
use tracing::info_span;

#[derive(Debug)]
pub struct LinkResult {
    pub function: String,
    pub total_cases: u32,
    pub pass_cases: u32,
    pub fail_cases: u32,
    pub failing_ids: Vec<String>,
    pub callers: Vec<String>,
}

/// 查询函数的 spec case 详情 + CodeGraph 调用链。
pub fn link_function(conn: &mut Connection, function: &str) -> Option<LinkResult> {
    let _span = info_span!("link_function", function);
    let _enter = _span.enter();

    let mut stmt = conn.prepare(
        "SELECT case_id, status FROM case_results r
         JOIN spec_cases c ON r.case_id = c.case_id
         WHERE c.function = ?1 AND r.snapshot_id = (
             SELECT MAX(id) FROM snapshots
         )"
    ).ok()?;

    let rows: Vec<_> = stmt
        .query_map([function], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .ok()?
        .flatten()
        .collect();

    if rows.is_empty() {
        return None;
    }

    let total = rows.len() as u32;
    let pass = rows.iter().filter(|(_, s)| s == "PASS").count() as u32;
    let fail = rows.iter().filter(|(_, s)| s == "FAIL").count() as u32;
    let failing: Vec<_> = rows
        .iter()
        .filter(|(_, s)| s == "FAIL")
        .map(|(id, _)| id.clone())
        .collect();

    // 调用 CodeGraph callers
    let callers = call_codegraph(function);

    Some(LinkResult {
        function: function.to_string(),
        total_cases: total,
        pass_cases: pass,
        fail_cases: fail,
        failing_ids: failing,
        callers,
    })
}

fn call_codegraph(function: &str) -> Vec<String> {
    // 尝试调用 codegraph CLI (可能不在 PATH 中)
    let Ok(output) = std::process::Command::new("codegraph")
        .args(["callers", function])
        .output()
    else {
        return vec!["(codegraph 不可用)".to_string()];
    };

    if !output.status.success() {
        return vec![format!("(codegraph 调用失败: {})", output.status)];
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .take(10)
        .map(|s| s.to_string())
        .collect()
}

/// 格式化输出桥接视图。
pub fn format_link(result: &LinkResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "=== Spec Coverage: {} ===\n函数: {}\nCases: {} total, {} pass, {} fail\n\n",
        result.function, result.function, result.total_cases, result.pass_cases, result.fail_cases
    ));

    if !result.failing_ids.is_empty() {
        out.push_str("Failing cases:\n");
        for id in &result.failing_ids {
            out.push_str(&format!("  - {id}\n"));
        }
        out.push('\n');
    }

    out.push_str("=== CodeGraph Callers ===\n");
    for caller in &result.callers {
        out.push_str(&format!("  → {caller}\n"));
    }

    out
}
