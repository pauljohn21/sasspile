//! sasspile_tracker — external control library for sass-spec + enterprise pass-rate.
//!
//! 两种模式:
//! 1. `snapshot` — sass-spec HRX 用例 (锦上添花,非阻塞)
//! 2. `enterprise` — Bootstrap + Element Plus 端到端编译 ⭐ BLOCKING (P0 验收)
//!
//! 所有输出使用 tracing 宏 (服从项目规则: src/ 禁止 println!/eprintln!).
//!
//! Commands:
//! - `snapshot`       — sass-spec HRX full pass-rate run.
//! - `enterprise`     — Bootstrap + Element Plus entry-point compile (P0 gate).
//! - `diff <a> <b>`   — Compare two snapshot files, show improvements/regressions.
//! - `history`        — List all snapshots sorted by timestamp.
//! - `trend`          — Summary ASCII line-chart for pass rate over time.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use sasspile_rx::{compile, compile_file};

// ---------------------------------------------------------------------------
// Enterprise (P0 gate) — Bootstrap + Element Plus
// ---------------------------------------------------------------------------

const ENTERPRISE_TARGETS: &[(&str, &str, &str)] = &[
    ("Bootstrap 5", "bootstrap/scss/bootstrap.scss", "@import-global"),
    ("Element Plus", "element-plus/packages/theme-chalk/src/index.scss", "@use-as-star-global"),
];

/// Result of a single enterprise target compilation attempt.
#[derive(Debug, Clone)]
struct EnterpriseTarget {
    name: String,
    entry: String,
    paradigm: String,
    exists: bool,
    compiled: bool,
    output_len: usize,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct EnterpriseSnapshot {
    timestamp_ms: u64,
    passed: usize,
    total: usize,
    targets: Vec<EnterpriseTarget>,
}

impl EnterpriseSnapshot {
    fn pass_rate(&self) -> f64 {
        if self.total == 0 { 0.0 } else { (self.passed as f64 / self.total as f64) * 100.0 }
    }
}

fn run_enterprise() -> EnterpriseSnapshot {
    let span = tracing::info_span!("sasspile_tracker.enterprise", targets = ENTERPRISE_TARGETS.len());
    let _enter = span.enter();

    let mut targets: Vec<EnterpriseTarget> = Vec::with_capacity(ENTERPRISE_TARGETS.len());
    let mut passed = 0usize;

    for (name, entry, paradigm) in ENTERPRISE_TARGETS {
        let t_span = tracing::info_span!("enterprise.target", name, entry, paradigm);
        let _tenter = t_span.enter();

        let path = Path::new(entry);
        let exists = path.exists();
        let mut et = EnterpriseTarget {
            name: (*name).to_string(),
            entry: (*entry).to_string(),
            paradigm: (*paradigm).to_string(),
            exists,
            compiled: false,
            output_len: 0,
            error: None,
        };

        if !exists {
            warn!("entry file not found, skipping");
            et.error = Some("entry file not found".to_string());
            targets.push(et);
            continue;
        }

        // Enterprise compile: pass the entry file path so @import/@use can
        // resolve against the parent directory.
        let result = std::panic::catch_unwind(|| compile_file(path));

        match result {
            Ok(Ok(css)) => {
                let len = css.len();
                let has_err_banner = css.contains("COMPILE ERROR:");
                let len_threshold = 64usize;
                let has_body = css.contains('{') && css.contains('}');
                if has_err_banner {
                    error!(output_len = len, "enterprise entry emitted COMPILE ERROR banner");
                    et.error = Some("COMPILE ERROR banner in output".to_string());
                } else if len < len_threshold || !has_body {
                    warn!(
                        output_len = len,
                        has_body,
                        "enterprise entry produced insufficient output — directive expansion likely not implemented yet"
                    );
                    et.error = Some(format!(
                        "insufficient output (len={len}, has_body={has_body}) — @import/@use not yet expanded"
                    ));
                } else {
                    et.compiled = true;
                    et.output_len = len;
                    passed += 1;
                    info!(output_len = len, "enterprise entry compiled OK");
                }
            }
            Ok(Err(e)) => {
                warn!(error = ?e, "enterprise entry returned Err");
                et.error = Some(format!("{e:?}"));
            }
            Err(_panic) => {
                error!("enterprise entry panicked in compile_file()");
                et.error = Some("panic in compile_file()".to_string());
            }
        }
        targets.push(et);
    }

    EnterpriseSnapshot {
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        passed,
        total: ENTERPRISE_TARGETS.len(),
        targets,
    }
}

fn serialize_enterprise_snapshot(snap: &EnterpriseSnapshot) -> String {
    let mut out = String::with_capacity(512);
    out.push_str("{\n");
    out.push_str(&format!("  \"timestamp_ms\": {},\n", snap.timestamp_ms));
    out.push_str(&format!("  \"type\": \"enterprise\",\n"));
    out.push_str(&format!("  \"passed\": {},\n", snap.passed));
    out.push_str(&format!("  \"total\": {},\n", snap.total));
    out.push_str(&format!("  \"pass_rate\": {:.2},\n", snap.pass_rate()));
    out.push_str("  \"targets\": [\n");
    for (i, t) in snap.targets.iter().enumerate() {
        let comma = if i + 1 < snap.targets.len() { "," } else { "" };
        let err_json = match &t.error {
            Some(e) => {
                let escaped = e.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"{escaped}\"")
            }
            None => "null".to_string(),
        };
        out.push_str(&format!(
            "    {{ \"name\": \"{}\", \"entry\": \"{}\", \"paradigm\": \"{}\", \"compiled\": {}, \"output_len\": {}, \"error\": {err_json} }}{comma}\n",
            t.name, t.entry, t.paradigm, t.compiled, t.output_len,
        ));
    }
    out.push_str("  ]\n}\n");
    out
}

fn write_enterprise_snapshot(snap: &EnterpriseSnapshot) -> std::io::Result<PathBuf> {
    let dir = snapshot_dir();
    fs::create_dir_all(&dir)?;
    let filename = format!("enterprise_{:013}.json", snap.timestamp_ms);
    let path = dir.join(&filename);
    let payload = serialize_enterprise_snapshot(snap);
    fs::write(&path, payload)?;
    Ok(path)
}

fn cmd_enterprise() {
    let snap = run_enterprise();
    let path = match write_enterprise_snapshot(&snap) {
        Ok(p) => p,
        Err(e) => {
            error!("error writing enterprise snapshot: {e}");
            return;
        }
    };

    for t in &snap.targets {
        let status = if t.compiled { "✓" } else { "✗" };
        info!(
            target = %t.name,
            entry = %t.entry,
            paradigm = %t.paradigm,
            compiled = t.compiled,
            output_len = t.output_len,
            error = ?t.error,
            status,
            "enterprise entry result"
        );
    }
    info!(
        passed = snap.passed,
        total = snap.total,
        pass_rate = format!("{:.2}%", snap.pass_rate()),
        snapshot_path = ?path,
        "P0 enterprise gate written"
    );

    if snap.passed == snap.total {
        info!("� ENTERPRISE GATE PASSED — Bootstrap + Element Plus 100%");
    } else {
        warn!(
            pending = snap.total - snap.passed,
            "ENTERPRISE GATE NOT PASSED — fix before any sass-spec work"
        );
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const SNAPSHOT_DIR: &str = ".codegraph/snapshots";
const SPEC_ROOT: &str = "sass-spec/spec";

const EXCLUDED_DIRS: &[&str] = &[
    "libsass",
    "libsass-closed-issues",
    "libsass-todo-issues",
    "libsass-todo-tests",
    "non_conformant",
];

// ---------------------------------------------------------------------------
// Snapshot data structures
// ---------------------------------------------------------------------------

mod snap {
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct DirRecord {
        pub passed: usize,
        pub total: usize,
    }

    #[derive(Debug, Clone)]
    pub struct Snapshot {
        pub timestamp_ms: u64,
        pub total_passed: usize,
        pub total_cases: usize,
        pub per_dir: HashMap<String, DirRecord>,
    }

    impl Snapshot {
        /// Percentage (0.0–100.0).
        pub fn pass_rate(&self) -> f64 {
            if self.total_cases == 0 {
                0.0
            } else {
                (self.total_passed as f64 / self.total_cases as f64) * 100.0
            }
        }
    }

    /// Minimal JSON serializer — no external crate, simple format.
    pub fn serialize(snap: &Snapshot) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str("{\n");
        out.push_str(&format!("  \"timestamp_ms\": {},\n", snap.timestamp_ms));
        out.push_str(&format!("  \"total_passed\": {},\n", snap.total_passed));
        out.push_str(&format!("  \"total_cases\": {},\n", snap.total_cases));

        out.push_str("  \"per_dir\": {\n");
        let mut dirs: Vec<_> = snap.per_dir.iter().collect();
        dirs.sort_by_key(|(k, _)| *k);
        let n = dirs.len();
        for (i, (dir, rec)) in dirs.into_iter().enumerate() {
            let comma = if i + 1 < n { "," } else { "" };
            out.push_str(&format!(
                "    \"{dir}\": {{ \"passed\": {}, \"total\": {} }}{comma}\n",
                rec.passed, rec.total
            ));
        }
        out.push_str("  }\n}\n");
        out
    }

    pub fn deserialize(text: &str) -> Option<Snapshot> {
        let timestamp_ms = json_u64(text, "\"timestamp_ms\"")?;
        let total_passed = json_usize(text, "\"total_passed\"")?;
        let total_cases = json_usize(text, "\"total_cases\"")?;

        let mut per_dir = HashMap::new();
        if let Some(obj_start) = text.find("\"per_dir\"") {
            let after = &text[obj_start..];
            if let Some(open) = after.find('{') {
                let body = &after[open + 1..];
                let mut depth = 1;
                let mut end = 0;
                for (idx, ch) in body.char_indices() {
                    match ch {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                end = idx;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let inner = &body[..end];

                for entry in split_top_level_entries(inner) {
                    if let Some(colon) = entry.find(':') {
                        let key_raw = entry[..colon].trim();
                        let val_raw = entry[colon + 1..].trim();
                        let key = key_raw.trim_matches('"').to_string();
                        let passed = json_usize(val_raw, "\"passed\"").unwrap_or(0);
                        let total = json_usize(val_raw, "\"total\"").unwrap_or(0);
                        if !key.is_empty() && (passed + total) > 0 {
                            per_dir.insert(key, DirRecord { passed, total });
                        }
                    }
                }
            }
        }

        Some(Snapshot {
            timestamp_ms,
            total_passed,
            total_cases,
            per_dir,
        })
    }

    fn split_top_level_entries(s: &str) -> Vec<&str> {
        let mut entries = Vec::new();
        let mut depth = 0;
        let mut start = 0;
        for (idx, ch) in s.char_indices() {
            match ch {
                '{' | '[' => depth += 1,
                '}' | ']' => depth -= 1,
                ',' if depth == 0 => {
                    entries.push(&s[start..idx]);
                    start = idx + 1;
                }
                _ => {}
            }
        }
        let tail = &s[start..];
        if !tail.trim().is_empty() {
            entries.push(tail);
        }
        entries
    }

    fn json_u64<'a>(text: &'a str, key: &str) -> Option<u64> {
        let pos = text.find(key)?;
        let after = &text[pos + key.len()..];
        let colon = after.find(':')?;
        let rest = &after[colon + 1..];
        let digits: String = rest
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        digits.parse().ok()
    }

    fn json_usize<'a>(text: &'a str, key: &str) -> Option<usize> {
        json_u64(text, key).map(|v| v as usize)
    }
}

use snap::{DirRecord, Snapshot};

// ---------------------------------------------------------------------------
// HRX parsing & file collection
// ---------------------------------------------------------------------------

fn parse_hrx(content: &str) -> Vec<(String, String)> {
    let separator = "<===> ";
    let mut cases = Vec::new();
    let mut current_input: Option<String> = None;
    let mut current_output: Option<String> = None;
    let mut in_input = false;
    let mut in_output = false;
    let mut input_buf = String::new();
    let mut output_buf = String::new();

    for line in content.lines() {
        if line.starts_with(separator) {
            let path = line[separator.len()..].trim();

            if in_input {
                current_input = Some(input_buf.trim().to_string());
                input_buf.clear();
                in_input = false;
            } else if in_output {
                current_output = Some(output_buf.trim().to_string());
                output_buf.clear();
                in_output = false;
                if let (Some(i), Some(o)) = (current_input.take(), current_output.take()) {
                    cases.push((i, o));
                }
            }

            if path.ends_with("input.scss") || path.ends_with("input.sass") {
                in_input = true;
            } else if path.ends_with("output.css") {
                in_output = true;
            }
        } else if in_input {
            input_buf.push_str(line);
            input_buf.push('\n');
        } else if in_output {
            output_buf.push_str(line);
            output_buf.push('\n');
        }
    }

    if in_input {
        current_input = Some(input_buf.trim().to_string());
    } else if in_output {
        current_output = Some(output_buf.trim().to_string());
    }
    if let (Some(i), Some(o)) = (current_input, current_output) {
        if !i.is_empty() {
            cases.push((i, o));
        }
    }

    cases
}

fn collect_hrx_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(read_dir) = fs::read_dir(dir) else { return };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !EXCLUDED_DIRS.contains(&name) {
                    collect_hrx_files(&path, files);
                }
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("hrx") {
            files.push(path);
        }
    }
}

fn normalize_css(css: &str) -> String {
    css.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Snapshot pipeline
// ---------------------------------------------------------------------------

fn run_snapshot() -> Snapshot {
    let root_span = tracing::info_span!(
        "sasspile_tracker.snapshot",
        spec_root = SPEC_ROOT,
        excluded_dirs = ?EXCLUDED_DIRS
    );
    let _enter = root_span.enter();

    let root = Path::new(SPEC_ROOT);
    let mut hrx_files = Vec::new();
    if root.exists() {
        collect_hrx_files(root, &mut hrx_files);
    }
    info!(files = hrx_files.len(), "sass-spec HRX collected");

    let mut per_dir: HashMap<String, DirRecord> = HashMap::new();
    let mut total_passed = 0usize;
    let mut total_cases = 0usize;

    for (idx, hrx_path) in hrx_files.iter().enumerate() {
        let file_span = tracing::debug_span!("hrx.file", idx, path = ?hrx_path);
        let _fenter = file_span.enter();

        let Ok(content) = fs::read_to_string(hrx_path) else {
            warn!("cannot read, skipping");
            continue;
        };
        let cases = parse_hrx(&content);
        if cases.is_empty() {
            continue;
        }

        let rel = hrx_path.strip_prefix(root).unwrap_or(hrx_path);
        let top_dir: String = rel
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut dir_passed = 0usize;
        for (input, expected) in &cases {
            let result = std::panic::catch_unwind(|| compile(input));
            if let Ok(Ok(css)) = result {
                if normalize_css(&css) == normalize_css(expected) {
                    dir_passed += 1;
                }
            }
        }

        total_passed += dir_passed;
        total_cases += cases.len();

        let dir_c = top_dir.clone();
        let entry = per_dir
            .entry(top_dir)
            .or_insert(DirRecord { passed: 0, total: 0 });
        entry.passed += dir_passed;
        entry.total += cases.len();

        if (idx + 1) % 50 == 0 || idx + 1 == hrx_files.len() {
            info!(
                progress = format!("{}/{}", idx + 1, hrx_files.len()),
                dir = dir_c,
                passed = dir_passed,
                "snapshot progress"
            );
        }
    }

    Snapshot {
        timestamp_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        total_passed,
        total_cases,
        per_dir,
    }
}

// ---------------------------------------------------------------------------
// Snapshot file I/O
// ---------------------------------------------------------------------------

fn snapshot_dir() -> PathBuf {
    PathBuf::from(SNAPSHOT_DIR)
}

fn write_snapshot(snap: &Snapshot) -> std::io::Result<PathBuf> {
    let dir = snapshot_dir();
    fs::create_dir_all(&dir)?;
    let filename = format!("snap_{:013}.json", snap.timestamp_ms);
    let path = dir.join(&filename);
    fs::write(&path, snap::serialize(snap))?;
    Ok(path)
}

fn list_snapshots() -> Vec<PathBuf> {
    let dir = snapshot_dir();
    let Ok(read_dir) = fs::read_dir(&dir) else { return Vec::new() };
    let mut files: Vec<_> = read_dir
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    files
}

fn load_snapshot(path: &Path) -> Option<Snapshot> {
    fs::read_to_string(path).ok().and_then(|t| snap::deserialize(&t))
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

fn cmd_snapshot() {
    let snap = run_snapshot();
    let path = match write_snapshot(&snap) {
        Ok(p) => p,
        Err(e) => {
            error!("error writing snapshot: {e}");
            return;
        }
    };

    info!(
        total_passed = snap.total_passed,
        total_cases = snap.total_cases,
        pass_rate = format!("{:.2}%", snap.pass_rate()),
        snapshot_path = ?path,
        "snapshot written"
    );
}

fn cmd_history() {
    let span = tracing::info_span!("sasspile_tracker.history");
    let _enter = span.enter();

    let files = list_snapshots();
    if files.is_empty() {
        warn!("no snapshots under {SNAPSHOT_DIR}");
        return;
    }

    info!("{:-<72}", "");
    info!(
        "  {:>12}  {:>10}  {:>10}  {:>8}  {}",
        "TIMESTAMP", "PASSED", "TOTAL", "RATE", "FILE"
    );
    info!("{:-<72}", "");

    for f in &files {
        let Some(snap) = load_snapshot(f) else { continue };
        let ts = format_ts(snap.timestamp_ms);
        let file_name = f.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        info!(
            ts,
            total_passed = snap.total_passed,
            total_cases = snap.total_cases,
            pass_rate = format!("{:.2}%", snap.pass_rate()),
            file_name,
            "history row"
        );
    }
}

fn cmd_diff(a_path: &str, b_path: &str) -> std::io::Result<()> {
    let span = tracing::info_span!("sasspile_tracker.diff", a = a_path, b = b_path);
    let _enter = span.enter();

    let a = load_snapshot(Path::new(a_path));
    let b = load_snapshot(Path::new(b_path));
    let (Some(a), Some(b)) = (a, b) else {
        warn!("cannot load one or both snapshots");
        return Ok(());
    };

    info!("A={} B={}", format_ts(a.timestamp_ms), format_ts(b.timestamp_ms));
    info!(
        a_passed = a.total_passed,
        a_total = a.total_cases,
        a_rate = format!("{:.2}%", a.pass_rate()),
        b_passed = b.total_passed,
        b_total = b.total_cases,
        b_rate = format!("{:.2}%", b.pass_rate()),
        delta = format!("{:.2}%", b.pass_rate() - a.pass_rate()),
        "overall diff"
    );

    if b.total_passed >= a.total_passed {
        info!(delta = b.total_passed - a.total_passed, "improvement");
    } else {
        warn!(delta = a.total_passed - b.total_passed, "regression");
    }

    let mut all_dirs: Vec<String> = a.per_dir.keys().cloned().collect();
    for k in b.per_dir.keys() {
        if !all_dirs.contains(k) {
            all_dirs.push(k.clone());
        }
    }
    all_dirs.sort();

    info!("{:-<72}", "");
    info!("  {:<20} {:>12} {:>12} {:>10}", "DIR", "A (rate%)", "B (rate%)", "Δ");
    info!("{:-<72}", "");

    for dir in all_dirs {
        let a_rec = a.per_dir.get(&dir);
        let b_rec = b.per_dir.get(&dir);
        let a_rate = a_rec.map(|r| rate(r)).unwrap_or(-1.0);
        let b_rate = b_rate_of(b_rec);
        let delta = b_rate - a_rate;

        let a_str = a_rec
            .map(|r| format!("{:>4}/{:<4} {:>5.1}%", r.passed, r.total, rate(r)))
            .unwrap_or_else(|| "  —".to_string());
        let b_str = format!(
            "{:>4}/{:<4} {:>5.1}%",
            b_rec.map(|r| r.passed).unwrap_or(0),
            b_rec.map(|r| r.total).unwrap_or(0),
            b_rate
        );
        let delta_str = if delta.abs() < 0.05 {
            "  =".to_string()
        } else if delta > 0.0 {
            format!("{:>+6.1}%", delta)
        } else {
            format!("{:>+6.1}%", delta)
        };
        info!(dir, a_str, b_str, delta_str, "dir diff");
    }

    Ok(())
}

fn rate(rec: &DirRecord) -> f64 {
    if rec.total == 0 {
        0.0
    } else {
        (rec.passed as f64 / rec.total as f64) * 100.0
    }
}

fn b_rate_of(rec: Option<&DirRecord>) -> f64 {
    rec.map(rate).unwrap_or(0.0)
}

fn cmd_trend() {
    let span = tracing::info_span!("sasspile_tracker.trend");
    let _enter = span.enter();

    let files = list_snapshots();
    if files.len() < 2 {
        warn!("need ≥2 snapshots for a trend; run 'snapshot' first");
        return;
    }
    let snaps: Vec<Snapshot> = files.iter().filter_map(|f| load_snapshot(f)).collect();

    info!("{:-<72}", "");
    info!("sass-spec pass rate trend ({} data points)", snaps.len());
    info!("{:-<72}", "");

    let chart_w = 48_usize;

    for snap in &snaps {
        let r = snap.pass_rate();
        let filled = ((r / 100.0) * chart_w as f64).min(chart_w as f64) as usize;
        let ts = format_ts(snap.timestamp_ms);
        info!(
            ts,
            pass_rate = format!("{r:.2}%"),
            bar_len = filled,
            total_passed = snap.total_passed,
            total_cases = snap.total_cases,
            "trend point"
        );
    }

    let latest = snaps.last().unwrap();
    let mut dirs: Vec<_> = latest.per_dir.iter().collect();
    dirs.sort_by(|a, b| {
        let ra = a.1.passed as f64 / (a.1.total as f64 + 0.001);
        let rb = b.1.passed as f64 / (b.1.total as f64 + 0.001);
        ra.partial_cmp(&rb).unwrap_or(Ordering::Equal)
    });

    info!("Top-5 improved directories (latest):");
    for (dir, rec) in dirs.iter().take(5) {
        let r = rate(rec);
        info!(
            dir,
            passed = rec.passed,
            total = rec.total,
            pass_rate = format!("{r:.2}%"),
            "top-5"
        );
    }
}

// ---------------------------------------------------------------------------
// Utilities — small timestamp → YYYY-MM-DD HH:MM (no chrono dependency)
// ---------------------------------------------------------------------------

fn format_ts(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let days_since_epoch = secs / 86400;
    let secs_in_day = secs % 86400;
    let hour = secs_in_day / 3600;
    let min = (secs_in_day % 3600) / 60;

    let mut y = 1970i64;
    let mut remaining_days = days_since_epoch;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }
    let (month, day) = day_of_year_to_month_day(remaining_days as u32, is_leap(y));

    format!("{y:04}-{month:02}-{day:02} {hour:02}:{min:02}")
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn day_of_year_to_month_day(day_of_year: u32, leap: bool) -> (u32, u32) {
    let month_lengths = if leap {
        [31u32, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut day = day_of_year;
    for (i, len) in month_lengths.iter().enumerate() {
        if day < *len {
            return ((i as u32) + 1, day + 1);
        }
        day -= len;
    }
    (12, 31)
}

// ---------------------------------------------------------------------------
// Entry
// ---------------------------------------------------------------------------

fn main() {
    // Required before any tracing macros fire.
    let subscriber = FmtSubscriber::builder()
        .with_target(false)
        .with_thread_ids(false)
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("--help");

    let _root = tracing::info_span!("sasspile_tracker", command = cmd).entered();

    match cmd {
        "snapshot" => cmd_snapshot(),
        "enterprise" => cmd_enterprise(),
        "history" => cmd_history(),
        "trend" => cmd_trend(),
        "diff" => {
            if args.len() < 4 {
                error!("usage: sasspile_tracker diff <snap_a.json> <snap_b.json>");
                return;
            }
            let _ = cmd_diff(&args[2], &args[3]);
        }
        "--help" | "-h" | _ => {
            info!(
                "sasspile_tracker — external control library for sass-spec + enterprise\n\
                 \n\
                 USAGE:\n\
                 \x20   sasspile_tracker snapshot     # sass-spec HRX 基线 (P2)\n\
                 \x20   sasspile_tracker enterprise   # Bootstrap + Element Plus 100% (P0)\n\
                 \x20   sasspile_tracker history\n\
                 \x20   sasspile_tracker trend\n\
                 \x20   sasspile_tracker diff <a.json> <b.json>"
            );
        }
    }
}
