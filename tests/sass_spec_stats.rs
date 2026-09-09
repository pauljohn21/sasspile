//! sass-spec 统计报告生成 + 基线对比工具。
//!
//! 用法：
//!   基线模式:  BASELINE=1 cargo test --test sass_spec_stats -- --nocapture
//!   对比模式:  cargo test --test sass_spec_stats -- --nocapture
//!
//! 对比模式会加载 tests/sass-spec-baseline.json 作为基线，生成退化/进步报告。

use std::collections::HashMap;
use std::fs;

const LOG_PATH: &str = "/tmp/sass-spec-full.log";
const OUTPUT_PATH: &str = "tests/sass-spec-stats.md";
const BASELINE_PATH: &str = "tests/sass-spec-baseline.json";

#[test]
fn generate_sass_spec_stats() {
    let raw = fs::read_to_string(LOG_PATH).unwrap_or_default();
    if raw.is_empty() {
        panic!("日志不存在: {LOG_PATH}");
    }

    let log = strip_ansi(&raw);
    let sections = parse_all_sections(&log);
    let total = parse_total(&log);

    // 基线模式：保存当前结果
    if std::env::var("BASELINE").is_ok() {
        save_baseline(&sections, &total);
        return;
    }

    // 对比模式：加载基线并生成对比报告
    let baseline = load_baseline();
    generate_report(&sections, &total, baseline.as_ref());
}

// ─── 数据结构 ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DirStats {
    dir: String,
    pass: u32,
    fail: u32,
    skip: u32,
    total: u32,
    pct: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Baseline {
    total_pass: u32,
    total_fail: u32,
    total_skip: u32,
    total_cases: u32,
    dirs: HashMap<String, DirStats>,
}

// ─── 解析 ──────────────────────────────────────────────────────────────

fn parse_all_sections(log: &str) -> Vec<DirStats> {
    let mut sections: Vec<DirStats> = Vec::new();
    for line in log.lines() {
        if line.contains("sass-spec 目录") && line.contains("pct=") && !line.contains("parse_hrx") {
            if let Some(s) = parse_dir(line) {
                sections.push(s);
            }
        }
    }
    sections.sort_by(|a, b| a.pct.cmp(&b.pct));
    sections
}

fn parse_total(log: &str) -> Option<DirStats> {
    for line in log.lines() {
        if line.contains("sass-spec 全量统计") && line.contains("evaluated=") {
            return Some(DirStats {
                dir: "TOTAL".to_string(),
                pass: num_after(line, "pass=")?,
                fail: num_after(line, "fail=")?,
                skip: num_after(line, "skip=")?,
                total: num_after(line, "total=")?,
                pct: num_after(line, "pct=")?,
            });
        }
    }
    None
}

fn parse_dir(l: &str) -> Option<DirStats> {
    Some(DirStats {
        dir: between_quotes(l, "dir=\"")?,
        pass: num_after(l, "pass=")?,
        fail: num_after(l, "fail=")?,
        skip: num_after(l, "skip=")?,
        total: num_after(l, "total=")?,
        pct: num_after(l, "pct=")?,
    })
}

fn between_quotes(s: &str, pat: &str) -> Option<String> {
    let i = s.find(pat)? + pat.len();
    let j = s[i..].find('"')?;
    Some(s[i..i + j].to_string())
}

fn num_after(s: &str, key: &str) -> Option<u32> {
    let i = s.find(key)? + key.len();
    let j = s[i..].find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len() - i);
    s[i..i + j].parse().ok()
}

// ─── 基线管理 ──────────────────────────────────────────────────────────

fn save_baseline(sections: &[DirStats], total: &Option<DirStats>) {
    let mut dirs = HashMap::new();
    for s in sections {
        dirs.insert(s.dir.clone(), s.clone());
    }
    let baseline = Baseline {
        total_pass: total.as_ref().map_or(0, |t| t.pass),
        total_fail: total.as_ref().map_or(0, |t| t.fail),
        total_skip: total.as_ref().map_or(0, |t| t.skip),
        total_cases: total.as_ref().map_or(0, |t| t.total),
        dirs,
    };
    let json = serde_json::to_string_pretty(&baseline).expect("serialize failed");
    fs::write(BASELINE_PATH, json).expect("write baseline failed");
    tracing::info!(path = BASELINE_PATH, "baseline saved");
}

fn load_baseline() -> Option<Baseline> {
    let content = fs::read_to_string(BASELINE_PATH).ok()?;
    serde_json::from_str(&content).ok()
}

// ─── 报告生成 ──────────────────────────────────────────────────────────

fn generate_report(current: &[DirStats], total: &Option<DirStats>, baseline: Option<&Baseline>) {
    let mut md = String::from("# sass-spec 统计报告\n\n");
    md.push_str("**生成时间**: 自动由 `cargo test --test sass_spec_stats` 生成\n\n");

    // 总计对比
    if let Some(t) = total {
        md.push_str("## 总计\n\n");
        md.push_str("| 指标 | 当前 |");
        if baseline.is_some() { md.push_str(" 基线 | 变化 |"); }
        md.push_str("\n|------|------|");
        if baseline.is_some() { md.push_str("------|------|"); }
        md.push_str("\n");

        if let Some(bl) = baseline {
            let diff = t.pass as i64 - bl.total_pass as i64;
            let diff_str = format!("{:+}", diff);
            md.push_str(&format!(
                "| PASS | {} | {} | {} |\n",
                t.pass, bl.total_pass, diff_str
            ));
            md.push_str(&format!("| FAIL | {} | {} | {:+} |\n", t.fail, bl.total_fail, t.fail as i64 - bl.total_fail as i64));
            md.push_str(&format!("| SKIP | {} | {} | {:+} |\n", t.skip, bl.total_skip, t.skip as i64 - bl.total_skip as i64));
            md.push_str(&format!("| 通过率 | {}% | {}% | {:+}pp |\n\n", t.pct, bl.total_pass * 100 / bl.total_cases.max(1), t.pct as i64 - (bl.total_pass * 100 / bl.total_cases.max(1)) as i64));
        } else {
            md.push_str(&format!("| PASS | {} |\n", t.pass));
            md.push_str(&format!("| FAIL | {} |\n", t.fail));
            md.push_str(&format!("| SKIP | {} |\n", t.skip));
            md.push_str(&format!("| 通过率 | {}% |\n\n", t.pct));
        }
    }

    // 目录对比
    md.push_str("## 各目录详情\n\n");
    md.push_str("| 目录 | 通过 | 失败 | 跳过 | 总计 | 通过率 |");
    if baseline.is_some() { md.push_str(" 变化 |"); }
    md.push_str("\n|------|------|------|------|------|--------|");
    if baseline.is_some() { md.push_str("------|"); }
    md.push_str("\n");

    if let Some(ref bl) = baseline {
        for s in current {
            let bl_pass = bl.dirs.get(&s.dir).map_or(0, |b| b.pass);
            let diff = s.pass as i64 - bl_pass as i64;
            let diff_str = if diff != 0 { format!("{:+}", diff) } else { "-".to_string() };
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {}% | {} |\n",
                s.dir, s.pass, s.fail, s.skip, s.total, s.pct, diff_str
            ));
        }
    } else {
        for s in current {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {}% |\n",
                s.dir, s.pass, s.fail, s.skip, s.total, s.pct
            ));
        }
    }

    // 退化详情（仅对比模式）
    if let Some(ref bl) = baseline {
        let regressions: Vec<_> = current.iter().filter_map(|s| {
            let bl_pass = bl.dirs.get(&s.dir).map_or(0, |b| b.pass);
            let diff = s.pass as i64 - bl_pass as i64;
            (diff < 0).then_some((s.dir.clone(), bl_pass, s.pass, diff))
        }).collect();

        if !regressions.is_empty() {
            md.push_str("\n## ⚠️ 退化目录\n\n");
            md.push_str("| 目录 | 基线通过 | 当前通过 | 变化 |\n");
            md.push_str("|------|----------|----------|------|\n");
            for (dir, bl_pass, cur_pass, diff) in &regressions {
                md.push_str(&format!("| {} | {} | {} | {:+} |\n", dir, bl_pass, cur_pass, diff));
            }
        }

        let improvements: Vec<_> = current.iter().filter_map(|s| {
            let bl_pass = bl.dirs.get(&s.dir).map_or(0, |b| b.pass);
            let diff = s.pass as i64 - bl_pass as i64;
            (diff > 0).then_some((s.dir.clone(), bl_pass, s.pass, diff))
        }).collect();

        if !improvements.is_empty() {
            md.push_str("\n## ✅ 进步目录\n\n");
            md.push_str("| 目录 | 基线通过 | 当前通过 | 变化 |\n");
            md.push_str("|------|----------|----------|------|\n");
            for (dir, bl_pass, cur_pass, diff) in &improvements {
                md.push_str(&format!("| {} | {} | {} | +{} |\n", dir, bl_pass, cur_pass, diff));
            }
        }
    }

    fs::write(OUTPUT_PATH, &md).expect("write failed");
    tracing::info!(path = OUTPUT_PATH, "report generated");
}

// ─── ANSI 清理 ─────────────────────────────────────────────────────────

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
