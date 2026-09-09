//! 分析 sass-spec-failures.json 的失败分布。
//!
//! 用法：
//!   cargo test --test failures_analyze -- --nocapture

use std::collections::BTreeMap;
use std::fs;

#[test]
fn analyze_failures() {
    sasspile::init_tracing();
    let content = fs::read_to_string("tests/sass-spec-failures.json")
        .expect("读取 sass-spec-failures.json 失败");
    let d: serde_json::Value =
        serde_json::from_str(&content).expect("JSON 解析失败");

    let m = &d["metadata"];
    let total = m["total_cases"].as_u64().expect("missing total") as u64;
    let pass = m["pass"].as_u64().expect("missing pass") as u64;
    let fail = m["fail"].as_u64().expect("missing fail") as u64;
    let skip = m["skip"].as_u64().expect("missing skip") as u64;
    let pct = pass * 100 / total;

    tracing::info!("总数={total} 通过={pass} 失败={fail} 跳过={skip} 通过率={pct}%");

    // 按类型
    tracing::info!("按类型: {:?}", d["by_type"]);

    // 按目录（失败数降序）
    let mut dir_totals: Vec<(String, u64)> = d["by_dir"]
        .as_object()
        .expect("by_dir not object")
        .iter()
        .map(|(k, a)| {
            let t = a["diff"].as_u64().unwrap_or(0)
                + a["err"].as_u64().unwrap_or(0)
                + a["err_exp_ok"].as_u64().unwrap_or(0);
            (k.clone(), t)
        })
        .collect();
    dir_totals.sort_by(|a, b| b.1.cmp(&a.1));

    tracing::info!("=== 按目录分布 ===");
    for (name, t) in &dir_totals {
        tracing::info!("  {name:40} 失败={t}");
    }

    // core_functions 子目录细分
    let cf_failures: Vec<&serde_json::Value> = d["failures"]
        .as_array()
        .expect("failures not array")
        .iter()
        .filter(|f| f["dir"].as_str() == Some("core_functions"))
        .collect();

    let mut subdirs: BTreeMap<String, u64> = BTreeMap::new();
    for f in &cf_failures {
        let id = f["id"].as_str().unwrap_or("");
        let sub = id.split('/').nth(1).unwrap_or("other").to_string();
        *subdirs.entry(sub).or_default() += 1;
    }

    let mut sub_vec: Vec<_> = subdirs.into_iter().collect();
    sub_vec.sort_by(|a, b| b.1.cmp(&a.1));

    tracing::info!("=== core_functions 子目录细分 ===");
    for (name, t) in &sub_vec {
        tracing::info!("  {name:30} 失败={t}");
    }

    // ERR_EXP_OK 分布
    let err_exp_ok: Vec<&serde_json::Value> = d["failures"]
        .as_array()
        .expect("failures not array")
        .iter()
        .filter(|f| f["type"].as_str() == Some("ERR_EXP_OK"))
        .collect();

    let mut err_ok_dirs: BTreeMap<String, u64> = BTreeMap::new();
    for f in &err_exp_ok {
        let dir = f["dir"].as_str().unwrap_or("unknown").to_string();
        *err_ok_dirs.entry(dir).or_default() += 1;
    }

    tracing::info!("=== ERR_EXP_OK 分布（共 {} 个）===", err_exp_ok.len());
    for (dir, t) in err_ok_dirs.iter().rev() {
        tracing::info!("  {dir:20} {t}");
    }
}
