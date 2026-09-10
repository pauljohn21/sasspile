//! 函数级历史趋势查询。

use rusqlite::Connection;
use tracing::info_span;

#[derive(Debug)]
pub struct TrendPoint {
    pub snapshot_id: i64,
    pub commit_sha: Option<String>,
    pub timestamp: i64,
    pub pass: u32,
    pub fail: u32,
    pub total: u32,
    pub pct: u32,
}

/// 查询指定函数的历史趋势。
pub fn trend_by_function(conn: &mut Connection, function: &str) -> Vec<TrendPoint> {
    let _span = info_span!("trend_by_function", function);
    let _enter = _span.enter();

    let mut stmt = match conn.prepare(
        "SELECT s.id, s.commit_sha, s.timestamp,
                SUM(CASE WHEN r.status = 'PASS' THEN 1 ELSE 0 END),
                SUM(CASE WHEN r.status = 'FAIL' THEN 1 ELSE 0 END),
                COUNT(*)
         FROM snapshots s
         JOIN case_results r ON r.snapshot_id = s.id
         JOIN spec_cases c ON r.case_id = c.case_id
         WHERE c.function = ?1
         GROUP BY s.id
         ORDER BY s.timestamp ASC"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([function], |row| {
        let pass: u32 = row.get(3)?;
        let total: u32 = row.get(5)?;
        Ok(TrendPoint {
            snapshot_id: row.get(0)?,
            commit_sha: row.get(1)?,
            timestamp: row.get(2)?,
            pass,
            fail: row.get(4)?,
            total,
            pct: if total > 0 { pass * 100 / total } else { 0 },
        })
    })
    .ok()
    .map(|it| it.flatten().collect())
    .unwrap_or_default()
}

/// 查询指定目录的历史趋势。
pub fn trend_by_dir(conn: &mut Connection, dir_prefix: &str) -> Vec<TrendPoint> {
    let _span = info_span!("trend_by_dir", dir_prefix);
    let _enter = _span.enter();

    let pattern = format!("{dir_prefix}%");
    let mut stmt = match conn.prepare(
        "SELECT s.id, s.commit_sha, s.timestamp,
                SUM(CASE WHEN r.status = 'PASS' THEN 1 ELSE 0 END),
                SUM(CASE WHEN r.status = 'FAIL' THEN 1 ELSE 0 END),
                COUNT(*)
         FROM snapshots s
         JOIN case_results r ON r.snapshot_id = s.id
         JOIN spec_cases c ON r.case_id = c.case_id
         WHERE c.dir LIKE ?1
         GROUP BY s.id
         ORDER BY s.timestamp ASC"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([pattern], |row| {
        let pass: u32 = row.get(3)?;
        let total: u32 = row.get(5)?;
        Ok(TrendPoint {
            snapshot_id: row.get(0)?,
            commit_sha: row.get(1)?,
            timestamp: row.get(2)?,
            pass,
            fail: row.get(4)?,
            total,
            pct: if total > 0 { pass * 100 / total } else { 0 },
        })
    })
    .ok()
    .map(|it| it.flatten().collect())
    .unwrap_or_default()
}

/// ASCII 折线图输出。
pub fn format_chart(points: &[TrendPoint], label: &str) -> String {
    if points.is_empty() {
        return format!("{label}: 无数据\n");
    }

    let mut out = format!("{label} 趋势\n");
    let width = 40;
    let height = 10;

    for row in (0..height).rev() {
        let threshold = row * 100 / height;
        out.push_str(&format!("{threshold:>3}% │"));
        for p in points {
            let pos = p.pct as usize * width / 100;
            let threshold_pos = threshold as usize * width / 100;
            if pos >= threshold_pos {
                out.push('█');
            } else {
                out.push(' ');
            }
        }
        out.push('\n');
    }
    out.push_str("     └");
    for _ in points {
        out.push('─');
    }
    out.push('\n');

    // 数据摘要
    if let (Some(first), Some(last)) = (points.first(), points.last()) {
        out.push_str(&format!(
            "  首次: {}% → 最近: {}%\n",
            first.pct, last.pct
        ));
    }

    out
}
