//! 检查特定目录下失败 case 的详情。

use std::collections::BTreeMap;
use std::fs;

#[test]
fn inspect_selector_failures() {
    sasspile::init_tracing();
    let content = fs::read_to_string("tests/sass-spec-failures.json").expect("读取失败");
    let d: serde_json::Value = serde_json::from_str(&content).expect("JSON 解析失败");
    let failures = d["failures"].as_array().expect("failures not array");

    // core_functions/selector 的失败
    let selector_fails: Vec<&serde_json::Value> = failures
        .iter()
        .filter(|f| {
            f["dir"].as_str() == Some("core_functions")
                && f["id"].as_str().map_or(false, |id| id.starts_with("core_functions/selector/"))
        })
        .collect();

    tracing::info!("core_functions/selector 失败数: {}", selector_fails.len());

    for f in selector_fails.iter().take(20) {
        let id = f["id"].as_str().unwrap_or("?");
        let ty = f["type"].as_str().unwrap_or("?");
        let exp = f["expected"].as_str().map_or("", |s| &s[..s.len().min(80)]);
        let act = f["actual"].as_str().map_or("", |s| &s[..s.len().min(80)]);
        let err = f["error"].as_str().map_or("", |s| s);
        tracing::info!("  id={id} type={ty}");
        tracing::info!("    expected={exp}");
        tracing::info!("    actual  ={act}");
        if !err.is_empty() {
            tracing::info!("    error   ={err}");
        }
    }

    // 所有包含 selector 的失败，按顶层目录分组
    let all_selector: Vec<&serde_json::Value> = failures
        .iter()
        .filter(|f| f["id"].as_str().map_or(false, |id| id.contains("selector") || id.contains("extend") || id.contains("nest")))
        .collect();

    let mut dir_counts: BTreeMap<String, u64> = BTreeMap::new();
    for f in &all_selector {
        let dir = f["id"].as_str().unwrap_or("").split('/').next().unwrap_or("?").to_string();
        *dir_counts.entry(dir).or_default() += 1;
    }
    tracing::info!("所有含 selector/extend/nest 的失败: {}", all_selector.len());
    for (dir, c) in &dir_counts {
        tracing::info!("  {dir}: {c}");
    }
}
