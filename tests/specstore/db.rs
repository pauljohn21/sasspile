//! SQLite 数据库连接 + schema 初始化 + 迁移。

use tracing::{info, info_span};

const SCHEMA_VERSION: u32 = 2;

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS spec_cases (
    case_id       TEXT PRIMARY KEY,
    hrx_file      TEXT NOT NULL,
    function      TEXT,
    dir           TEXT NOT NULL,
    input_scss    TEXT NOT NULL,
    expected_css  TEXT NOT NULL DEFAULT '',
    expect_error  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS case_files (
    case_id   TEXT NOT NULL REFERENCES spec_cases(case_id),
    file_path TEXT NOT NULL,
    content   TEXT NOT NULL,
    PRIMARY KEY (case_id, file_path)
);

CREATE TABLE IF NOT EXISTS snapshots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    commit_sha  TEXT,
    timestamp   INTEGER NOT NULL,
    total_pass  INTEGER NOT NULL DEFAULT 0,
    total_fail  INTEGER NOT NULL DEFAULT 0,
    total_skip  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS case_results (
    snapshot_id  INTEGER NOT NULL REFERENCES snapshots(id),
    case_id      TEXT NOT NULL REFERENCES spec_cases(case_id),
    status       TEXT NOT NULL CHECK(status IN ('PASS','FAIL','SKIP')),
    failure_type TEXT CHECK(failure_type IS NULL OR failure_type IN ('DIFF','ERR','ERR_EXP_OK')),
    actual_css   TEXT,
    error        TEXT,
    PRIMARY KEY (snapshot_id, case_id)
);

CREATE TABLE IF NOT EXISTS case_deltas (
    snapshot_id  INTEGER NOT NULL REFERENCES snapshots(id),
    case_id      TEXT NOT NULL REFERENCES spec_cases(case_id),
    prev_status  TEXT NOT NULL,
    curr_status  TEXT NOT NULL,
    PRIMARY KEY (snapshot_id, case_id)
);

CREATE INDEX IF NOT EXISTS idx_cases_function ON spec_cases(function);
CREATE INDEX IF NOT EXISTS idx_cases_dir ON spec_cases(dir);
CREATE INDEX IF NOT EXISTS idx_results_snapshot ON case_results(snapshot_id);
CREATE INDEX IF NOT EXISTS idx_results_status ON case_results(status);
CREATE INDEX IF NOT EXISTS idx_deltas_snapshot ON case_deltas(snapshot_id);
CREATE INDEX IF NOT EXISTS idx_snapshots_timestamp ON snapshots(timestamp);
CREATE INDEX IF NOT EXISTS idx_case_files_case ON case_files(case_id);

CREATE TABLE IF NOT EXISTS schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

/// 打开（或创建）spec-store 数据库，初始化 schema。
pub fn open_db(path: &std::path::Path) -> Result<rusqlite::Connection, String> {
    let span = info_span!("spec_store_open_db", path = %path.display());
    let _enter = span.enter();

    let conn = rusqlite::Connection::open(path).map_err(|e| format!("open db: {e}"))?;

    conn.pragma_update(None, "journal_mode", "wal")
        .map_err(|e| format!("set WAL: {e}"))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| format!("enable FK: {e}"))?;

    init_schema(&conn)?;
    info!("spec-store db initialized");

    Ok(conn)
}

fn init_schema(conn: &rusqlite::Connection) -> Result<(), String> {
    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| format!("init schema: {e}"))?;

    let current_version: u32 = conn
        .query_row(
            "SELECT value FROM schema_meta WHERE key = 'version'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    if current_version < SCHEMA_VERSION {
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta(key, value) VALUES('version', ?1)",
            [SCHEMA_VERSION.to_string()],
        )
        .map_err(|e| format!("write version: {e}"))?;
    }

    Ok(())
}
