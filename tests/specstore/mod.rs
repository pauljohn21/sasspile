//! spec-store: sass-spec 数据管理工具（模块根）。

pub mod bisect;
pub mod db;
pub mod hrx_loader;
pub mod link;
pub mod runner;
pub mod snapshot;
pub mod stats;
pub mod trend;

use tracing::info;

const DB_PATH: &str = "tests/spec-store.db";
const SPEC_ROOT: &str = "sass-spec/spec";

/// 根据 SPEC_STORE_CMD 环境变量分派子命令。
pub fn run() {
    sasspile::init_tracing();

    let cmd = std::env::var("SPEC_STORE_CMD").unwrap_or_else(|_| "stats".to_string());
    let db_path = std::path::Path::new(DB_PATH);
    let spec_root = std::path::Path::new(SPEC_ROOT);
    let mut conn = db::open_db(db_path).expect("open db");

    match cmd.as_str() {
        "run" => cmd_run(&mut conn, spec_root, db_path),
        "index" => cmd_index(spec_root, db_path),
        "stats" => cmd_stats(&mut conn),
        "trend" => cmd_trend(&mut conn),
        "link" => cmd_link(&mut conn),
        "bisect" => cmd_bisect(&mut conn),
        "diff" => cmd_diff(&mut conn),
        "snapshot" => cmd_snapshot(&mut conn),
        _ => tracing::error!(cmd, "unknown command. Use: run|index|stats|trend|link|bisect|diff|snapshot"),
    }
}

fn cmd_index(spec_root: &std::path::Path, db_path: &std::path::Path) {
    info!("=== spec_store: index ===");
    let total = hrx_loader::index_all(spec_root, db_path).expect("index failed");
    tracing::info!(total, "HRX indexing complete");
}

fn cmd_run(conn: &mut rusqlite::Connection, spec_root: &std::path::Path, db_path: &std::path::Path) {
    info!("=== spec_store: run ===");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM spec_cases", [], |r| r.get(0))
        .unwrap_or(0);
    if count == 0 {
        hrx_loader::index_all(spec_root, db_path).expect("index failed");
    }

    let cases = snapshot::load_all_cases(conn);
    tracing::info!(n_cases = cases.len(), "running all cases...");

    let mut results = Vec::new();
    for (case_id, _input, expected, expect_error) in &cases {
        let files = snapshot::load_case_files(conn, case_id);
        let result = runner::run_case(case_id, expected, *expect_error, &files);
        results.push(result);
    }

    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    let snap_id = snapshot::create_snapshot(conn, commit.as_deref(), results)
        .expect("create snapshot");

    let (pass, fail, skip) = count_results(conn, snap_id);
    tracing::info!(snapshot_id = snap_id, pass, fail, skip, "run + snapshot complete");
}

fn count_results(conn: &rusqlite::Connection, snap_id: i64) -> (u32, u32, u32) {
    let Ok(mut stmt) = conn.prepare(
        "SELECT status, COUNT(*) FROM case_results WHERE snapshot_id = ?1 GROUP BY status"
    ) else { return (0, 0, 0) };

    let rows: Vec<_> = stmt
        .query_map([snap_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))
        .expect("query")
        .flatten()
        .collect();
    let mut pass = 0;
    let mut fail = 0;
    let mut skip = 0;
    for (status, count) in rows {
        match status.as_str() {
            "PASS" => pass = count,
            "FAIL" => fail = count,
            "SKIP" => skip = count,
            _ => {}
        }
    }
    (pass, fail, skip)
}

fn cmd_stats(conn: &mut rusqlite::Connection) {
    info!("=== spec_store: stats ===");
    let (stats, _id) = stats::latest_stats(conn);
    let md = stats::format_md(&stats, None);
    info!("{md}");
}

fn cmd_trend(conn: &mut rusqlite::Connection) {
    let function = std::env::var("FN").unwrap_or_else(|_| "math.sin".to_string());
    info!("=== spec_store: trend --function {function} ===");

    let trend = if function.contains('.') {
        trend::trend_by_function(conn, &function)
    } else {
        trend::trend_by_dir(conn, &function)
    };

    let chart = trend::format_chart(&trend, &function);
    info!("{chart}");
}

fn cmd_link(conn: &mut rusqlite::Connection) {
    let function = std::env::var("FN").unwrap_or_else(|_| "math.sin".to_string());
    info!("=== spec_store: link --function {function} ===");

    match link::link_function(conn, &function) {
        Some(result) => {
            let output = link::format_link(&result);
            info!("{output}");
        }
        None => tracing::warn!(function, "no cases found"),
    }
}

fn cmd_bisect(conn: &mut rusqlite::Connection) {
    let function = std::env::var("FN").unwrap_or_else(|_| "math.sin".to_string());
    let good_id: i64 = std::env::var("GOOD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let bad_id: i64 = std::env::var("BAD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            conn.query_row("SELECT MAX(id) FROM snapshots", [], |r| r.get::<_, i64>(0))
                .unwrap_or(1)
        });

    info!("=== spec_store: bisect --function {function} good={good_id} bad={bad_id} ===");

    let points = bisect::bisect_function(conn, &function, good_id, bad_id);
    for (id, commit, pass, fail) in &points {
        let pct = if *pass + *fail > 0 { *pass * 100 / (*pass + *fail) } else { 0 };
        tracing::info!(snapshot_id = id, commit, pass, fail, pct, "bisect point");
    }
}

fn cmd_diff(conn: &mut rusqlite::Connection) {
    let from_id: i64 = std::env::var("FROM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let to_id: i64 = std::env::var("TO")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    info!("=== spec_store: diff {from_id} → {to_id} ===");

    let entries = bisect::diff_snapshots(conn, from_id, to_id);
    let output = bisect::format_diff(&entries);
    info!("{output}");
}

fn cmd_snapshot(conn: &mut rusqlite::Connection) {
    info!("=== spec_store: snapshot ===");
    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    let snap_id = snapshot::create_snapshot(conn, commit.as_deref(), Vec::new())
        .expect("create snapshot");
    tracing::info!(snapshot_id = snap_id, "empty snapshot created");
}
