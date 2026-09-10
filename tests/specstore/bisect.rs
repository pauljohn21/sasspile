//! 回归定位 + 两 commit 间 diff。

use rusqlite::Connection;
use std::collections::BTreeMap;
use tracing::info_span;

#[derive(Debug)]
pub struct DiffEntry {
    pub case_id: String,
    pub function: Option<String>,
    pub dir: String,
    pub prev_status: String,
    pub curr_status: String,
}

/// 对比两个 snapshot 的 case 状态变化。
pub fn diff_snapshots(conn: &mut Connection, snap1_id: i64, snap2_id: i64) -> Vec<DiffEntry> {
    let _span = info_span!("diff_snapshots", snap1_id, snap2_id);
    let _enter = _span.enter();

    let mut stmt = match conn.prepare(
        "SELECT r1.case_id, c.function, c.dir, r1.status, r2.status
         FROM case_results r1
         JOIN case_results r2 ON r1.case_id = r2.case_id
         JOIN spec_cases c ON r1.case_id = c.case_id
         WHERE r1.snapshot_id = ?1 AND r2.snapshot_id = ?2
           AND r1.status != r2.status
         ORDER BY c.dir, c.function"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([snap1_id, snap2_id], |row| {
        Ok(DiffEntry {
            case_id: row.get(0)?,
            function: row.get(1)?,
            dir: row.get(2)?,
            prev_status: row.get(3)?,
            curr_status: row.get(4)?,
        })
    })
    .ok()
    .map(|it| it.flatten().collect())
    .unwrap_or_default()
}

/// 获取 snapshot 的 commit_sha（短格式）。
pub fn snapshot_commit(conn: &mut Connection, snapshot_id: i64) -> Option<String> {
    conn.query_row(
        "SELECT commit_sha FROM snapshots WHERE id = ?1",
        [snapshot_id],
        |row| row.get(0),
    )
    .ok()
        .flatten()
}

/// 通过 commit_sha 前缀反查 snapshot_id。
pub fn find_snapshot_by_commit(conn: &rusqlite::Connection, commit_prefix: &str) -> Option<i64> {
    let pattern = format!("{commit_prefix}%");
    conn.query_row(
        "SELECT id FROM snapshots WHERE commit_sha LIKE ?1 ORDER BY timestamp DESC LIMIT 1",
        [pattern],
        |row| row.get(0),
    )
    .ok()
}

/// 二分搜索 first bad commit（按 snapshot_id 线性近似）。
pub fn bisect_function(
    conn: &mut Connection,
    function: &str,
    good_id: i64,
    bad_id: i64,
) -> Vec<(i64, String, u32, u32)> {
    let _span = info_span!("bisect_function", function, good_id, bad_id);
    let _enter = _span.enter();

    let mut stmt = match conn.prepare(
        "SELECT s.id, s.commit_sha,
                SUM(CASE WHEN r.status = 'PASS' THEN 1 ELSE 0 END),
                SUM(CASE WHEN r.status = 'FAIL' THEN 1 ELSE 0 END)
         FROM snapshots s
         JOIN case_results r ON r.snapshot_id = s.id
         JOIN spec_cases c ON r.case_id = c.case_id
         WHERE c.function = ?1 AND s.id BETWEEN ?2 AND ?3
         GROUP BY s.id
         ORDER BY s.id ASC"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([function, &good_id.to_string(), &bad_id.to_string()], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            row.get::<_, u32>(2)?,
            row.get::<_, u32>(3)?,
        ))
    })
    .ok()
    .map(|it| it.flatten().collect())
    .unwrap_or_default()
}

/// 格式化 diff 输出。
pub fn format_diff(entries: &[DiffEntry]) -> String {
    if entries.is_empty() {
        return "无变化\n".to_string();
    }

    let mut out = String::new();
    let mut by_dir: BTreeMap<&str, Vec<&DiffEntry>> = BTreeMap::new();
    for e in entries {
        by_dir.entry(&e.dir).or_default().push(e);
    }

    for (dir, items) in &by_dir {
        out.push_str(&format!("\n[{dir}] ({} 变化)\n", items.len()));
        // 只显示前 5 个 + 省略提示
        for item in items.iter().take(5) {
            out.push_str(&format!(
                "  {} → {}  {}\n",
                item.prev_status, item.curr_status, item.case_id
            ));
        }
        if items.len() > 5 {
            out.push_str(&format!("  ... +{} more\n", items.len() - 5));
        }
    }

    out
}
