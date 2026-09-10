//! Snapshot 创建 + delta 计算。

use rusqlite::Connection;
use tracing::{info, info_span};

use crate::specstore::runner::{run_case, CaseResult};

/// 创建新 snapshot，存储所有 case 结果，计算 deltas。
pub fn create_snapshot(
    conn: &mut Connection,
    commit_sha: Option<&str>,
    results: Vec<CaseResult>,
) -> Result<i64, String> {
    let span = info_span!("create_snapshot", n_results = results.len());
    let _enter = span.enter();

    let (mut pass, mut fail, mut skip) = (0u32, 0u32, 0u32);
    for r in &results {
        match r.status {
            crate::specstore::runner::CaseStatus::Pass => pass += 1,
            crate::specstore::runner::CaseStatus::Fail => fail += 1,
            crate::specstore::runner::CaseStatus::Skip => skip += 1,
        }
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let tx = conn.transaction().map_err(|e| format!("begin tx: {e}"))?;

    tx.execute(
        "INSERT INTO snapshots(commit_sha, timestamp, total_pass, total_fail, total_skip)
         VALUES(?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![commit_sha, timestamp, pass, fail, skip],
    )
    .map_err(|e| format!("insert snapshot: {e}"))?;

    let snapshot_id = tx.last_insert_rowid();

    let prev_snapshot: Option<i64> = tx
        .query_row(
            "SELECT id FROM snapshots WHERE id < ?1 ORDER BY id DESC LIMIT 1",
            [snapshot_id],
            |row| row.get(0),
        )
        .ok();

    for result in &results {
        tx.execute(
            "INSERT INTO case_results(snapshot_id, case_id, status, failure_type, actual_css, error)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                snapshot_id,
                result.case_id,
                result.status.as_str(),
                result.failure_type,
                result.actual_css,
                result.error,
            ],
        )
        .map_err(|e| format!("insert result {}: {e}", result.case_id))?;

        if let Some(prev_id) = prev_snapshot {
            let prev_status: Option<String> = tx
                .query_row(
                    "SELECT status FROM case_results WHERE snapshot_id = ?1 AND case_id = ?2",
                    rusqlite::params![prev_id, result.case_id.as_str()],
                    |row| row.get(0),
                )
                .ok();

            if let Some(prev) = prev_status
                && prev != result.status.as_str()
            {
                tx.execute(
                    "INSERT INTO case_deltas(snapshot_id, case_id, prev_status, curr_status)
                     VALUES(?1, ?2, ?3, ?4)",
                    rusqlite::params![snapshot_id, result.case_id, prev, result.status.as_str()],
                )
                .map_err(|e| format!("insert delta: {e}"))?;
            }
        }
    }

    tx.commit().map_err(|e| format!("commit: {e}"))?;
    info!(snapshot_id, pass, fail, skip, "snapshot created");
    Ok(snapshot_id)
}

/// 从 DB 加载所有 spec_cases（用于全量运行）。
pub fn load_all_cases(conn: &Connection) -> Vec<(String, String, String, bool)> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT case_id, input_scss, expected_css, expect_error FROM spec_cases"
    ) else {
        return Vec::new();
    };

    stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)? != 0,
        ))
    })
    .ok()
    .map(|it| it.flatten().collect())
    .unwrap_or_default()
}

/// 加载 case 的所有文件（编译上下文）。
pub fn load_case_files(conn: &Connection, case_id: &str) -> Vec<(String, String)> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT file_path, content FROM case_files WHERE case_id = ?1"
    ) else {
        return Vec::new();
    };

    stmt.query_map([case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })
    .ok()
    .map(|it| it.flatten().collect())
    .unwrap_or_default()
}
