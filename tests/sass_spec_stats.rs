//! sass-spec 统计报告生成工具
//! 运行: cargo test --test sass_spec_stats -- --nocapture

use std::fs;
use std::path::PathBuf;

const LOG_PATH: &str = "/tmp/sass-spec-full.log";
const OUTPUT_PATH: &str = "tests/sass-spec-stats.md";

#[test]
fn generate_sass_spec_stats() {
    let raw = fs::read_to_string(LOG_PATH).unwrap_or_default();
    if raw.is_empty() {
        panic!("日志不存在: {LOG_PATH}");
    }

    // Strip ANSI escape codes
    let log = strip_ansi(&raw);

    let mut sections: Vec<Section> = Vec::new();
    let mut total: Option<Total> = None;

    for line in log.lines() {
        if line.contains("sass-spec 全量统计") && line.contains("evaluated=") {
            total = parse_total(line);
        } else if line.contains("sass-spec 目录") && line.contains("pct=") && !line.contains("parse_hrx") {
            if let Some(s) = parse_dir(line) {
                sections.push(s);
            }
        }
    }

    sections.sort_by_key(|s| s.pct);

    let mut md = String::from("# sass-spec 统计报告\n\n");
    md.push_str("**生成时间**: 自动由 `cargo test --test sass_spec_stats` 生成\n\n");

    if let Some(t) = &total {
        md.push_str("## 总计\n\n");
        md.push_str("| PASS | FAIL | SKIP | TOTAL | 通过率 |\n");
        md.push_str("|------|------|------|-------|--------|\n");
        md.push_str(&format!("| {} | {} | {} | {} | {}% |\n\n", t.pass, t.fail, t.skip, t.total, t.pct));
    }

    md.push_str("## 各目录详情\n\n");
    md.push_str("| 目录 | 通过 | 失败 | 跳过 | 总计 | 通过率 |\n");
    md.push_str("|------|------|------|------|------|--------|\n");
    for s in &sections {
        md.push_str(&format!("| {} | {} | {} | {} | {} | {}% |\n", s.dir, s.pass, s.fail, s.skip, s.total, s.pct));
    }

    fs::write(OUTPUT_PATH, &md).expect("write failed");
    tracing::info!(sections = sections.len(), "md generated");
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            while let Some(c) = chars.next() {
                if c.is_ascii_alphabetic() { break; }
            }
        } else {
            result.push(c);
        }
    }
    result
}

struct Section { dir: String, pass: u32, fail: u32, skip: u32, total: u32, pct: u32 }
#[derive(Debug)]
struct Total { pass: u32, fail: u32, skip: u32, total: u32, pct: u32 }

fn parse_total(l: &str) -> Option<Total> {
    Some(Total { pass: num_after(l, "pass=")?, fail: num_after(l, "fail=")?, skip: num_after(l, "skip=")?, total: num_after(l, "total=")?, pct: num_after(l, "pct=")? })
}

fn parse_dir(l: &str) -> Option<Section> {
    Some(Section { dir: between_quotes(l, "dir=\"")?, pass: num_after(l, "pass=")?, fail: num_after(l, "fail=")?, skip: num_after(l, "skip=")?, total: num_after(l, "total=")?, pct: num_after(l, "pct=")? })
}

fn between_quotes(s: &str, pat: &str) -> Option<String> {
    let i = s.find(pat)? + pat.len();
    let j = s[i..].find('"')?;
    Some(s[i..i+j].to_string())
}

fn num_after(s: &str, key: &str) -> Option<u32> {
    let i = s.find(key)? + key.len();
    let j = s[i..].find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len()-i);
    s[i..i+j].parse().ok()
}
