//! 目录统计报告。

use rusqlite::Connection;
use std::collections::BTreeMap;
use tracing::info_span;

#[derive(Debug)]
pub struct DirStat {
    pub dir: String,
    pub pass: u32,
    pub fail: u32,
    pub skip: u32,
    pub total: u32,
    pub pct: u32,
}

/// 从指定 snapshot 生成目录统计。
pub fn stats_by_dir(conn: &mut Connection, snapshot_id: i64) -> Vec<DirStat> {
    let _span = info_span!("stats_by_dir", snapshot_id);
    let _enter = _span.enter();

    let mut stmt = match conn.prepare(
        "SELECT c.dir,
                SUM(CASE WHEN r.status = 'PASS' THEN 1 ELSE 0 END),
                SUM(CASE WHEN r.status = 'FAIL' THEN 1 ELSE 0 END),
                SUM(CASE WHEN r.status = 'SKIP' THEN 1 ELSE 0 END),
                COUNT(*)
         FROM case_results r
         JOIN spec_cases c ON r.case_id = c.case_id
         WHERE r.snapshot_id = ?1
         GROUP BY c.dir
         ORDER BY (SUM(CASE WHEN r.status = 'PASS' THEN 1 ELSE 0 END) * 100 / COUNT(*)) DESC"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt
        .query_map([snapshot_id], |row| {
            let pass: u32 = row.get(1)?;
            let total: u32 = row.get(4)?;
            Ok(DirStat {
                dir: row.get(0)?,
                pass,
                fail: row.get(2)?,
                skip: row.get(3)?,
                total,
                pct: if total > 0 { pass * 100 / total } else { 0 },
            })
        })
        .ok();

    rows.map(|it| it.flatten().collect()).unwrap_or_default()
}

/// 获取最新 snapshot 的统计。
pub fn latest_stats(conn: &mut Connection) -> (Vec<DirStat>, Option<i64>) {
    let latest_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM snapshots ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    match latest_id {
        Some(id) => (stats_by_dir(conn, id), Some(id)),
        None => (Vec::new(), None),
    }
}

/// 生成 Markdown 报告。
pub fn format_md(stats: &[DirStat], baseline: Option<&[DirStat]>) -> String {
    let mut md = String::from("# sass-spec 统计报告\n\n");

    let total_pass: u32 = stats.iter().map(|s| s.pass).sum();
    let total_fail: u32 = stats.iter().map(|s| s.fail).sum();
    let total_skip: u32 = stats.iter().map(|s| s.skip).sum();
    let total: u32 = total_pass + total_fail + total_skip;
    let pct = if total > 0 { total_pass * 100 / total } else { 0 };

    md.push_str("## 总计\n\n");
    md.push_str(&format!("| 指标 | 当前 |\n|------|------|\n"));
    md.push_str(&format!("| PASS | {total_pass} |\n"));
    md.push_str(&format!("| FAIL | {total_fail} |\n"));
    md.push_str(&format!("| SKIP | {total_skip} |\n"));
    md.push_str(&format!("| 通过率 | {pct}% |\n\n"));

    md.push_str("## 各目录详情\n\n");
    md.push_str("| 目录 | 通过 | 失败 | 跳过 | 总计 | 通过率 |\n");
    md.push_str("|------|------|------|------|------|--------|\n");

    for s in stats {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {}% |\n",
            s.dir, s.pass, s.fail, s.skip, s.total, s.pct
        ));
    }

    // 退化详情
    if let Some(bl) = baseline {
        let bl_map: BTreeMap<&str, u32> = bl.iter().map(|s| (s.dir.as_str(), s.pass)).collect();
        let regressions: Vec<_> = stats
            .iter()
            .filter_map(|s| {
                let bl_pass = bl_map.get(s.dir.as_str()).copied().unwrap_or(0);
                (s.pass < bl_pass).then_some((&s.dir, bl_pass, s.pass, s.pass as i64 - bl_pass as i64))
            })
            .collect();

        if !regressions.is_empty() {
            md.push_str("\n## ⚠️ 退化目录\n\n");
            md.push_str("| 目录 | 基线通过 | 当前通过 | 变化 |\n");
            md.push_str("|------|----------|----------|------|\n");
            for (dir, bl_pass, cur_pass, diff) in &regressions {
                md.push_str(&format!("| {dir} | {bl_pass} | {cur_pass} | {diff:+} |\n"));
            }
        }
    }

    md
}
