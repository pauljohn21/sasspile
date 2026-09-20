//! —— EP dist 差异诊断（compressed vs compressed）——
//!
//! 用 sasspile compressed 输出与 EP 官方 dist 做对比，定位真实差异。

use std::path::PathBuf;

const EP_SRC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/src"
);
const EP_DIST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/dist"
);

fn scss_to_dist_name(scss_name: &str) -> String {
    let stem = scss_name.trim_end_matches(".scss");
    let no_el = ["index", "base", "display"];
    if no_el.contains(&stem) {
        format!("{stem}.css")
    } else {
        format!("el-{stem}.css")
    }
}

/// 结构规范化：去空白、统一属性分隔符（压缩后应该已经一致）
fn normalize(css: &str) -> String {
    css.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 提取属性集合（selector -> {props}），用于语义对比
fn extract_rules(css: &str) -> Vec<(String, Vec<String>)> {
    let mut rules = Vec::new();
    let normalized = normalize(css);

    // 简单解析：按 } 分割规则
    for block in normalized.split('}') {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }
        if let Some((sel, body)) = block.split_once('{') {
            let props: Vec<String> = body
                .split(';')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();
            rules.push((sel.trim().to_string(), props));
        }
    }
    rules
}

#[test]
fn test_ep_compressed_diff() {
    sasspile::init_tracing();
    let src_dir = PathBuf::from(EP_SRC);
    let dist_dir = PathBuf::from(EP_DIST);

    let mut entries: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("无法读取 src 目录")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "scss"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let total = entries.len();
    let mut identical = 0;
    let mut diff_files: Vec<(String, Vec<String>)> = vec![]; // (file, diff_details)

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let dist_name = scss_to_dist_name(&name);
        let dist_path = dist_dir.join(&dist_name);

        let sasspile_css = match sasspile::compile_file(&path, sasspile::OutputStyle::Compressed) {
            Ok(css) => css,
            Err(e) => {
                tracing::warn!(file = %name, error = %e, "✗ sasspile FAIL");
                continue;
            }
        };

        let dist_css = match std::fs::read_to_string(&dist_path) {
            Ok(css) => css,
            Err(_) => continue,
        };

        let sp_rules = extract_rules(&sasspile_css);
        let dist_rules = extract_rules(&dist_css);

        // 计算差异
        let sp_props_count: usize = sp_rules.iter().map(|(_, p)| p.len()).sum();
        let dist_props_count: usize = dist_rules.iter().map(|(_, p)| p.len()).sum();

        if sp_rules == dist_rules {
            identical += 1;
        } else {
            // 找出 sasspile 有但 dist 没有的 rule 或 prop
            let mut extra_rules: Vec<String> = vec![];
            let mut missing_rules: Vec<String> = vec![];

            // 简化对比：只检查 selector 是否存在 + prop 数量
            for (sel, props) in &sp_rules {
                match dist_rules.iter().find(|(s, _)| **s == *sel) {
                    Some((_, dp)) => {
                        let extra: Vec<_> = props.iter().filter(|p| !dp.contains(p)).collect();
                        if !extra.is_empty() {
                            extra_rules.push(format!("{}: +{:?}", sel, extra));
                        }
                    }
                    None => {
                        extra_rules.push(format!("{} ({} props, dist missing)", sel, props.len()));
                    }
                }
            }

            for (sel, props) in &dist_rules {
                if !sp_rules.iter().any(|(s, _)| *s == *sel) {
                    missing_rules.push(format!("{} ({} props)", sel, props.len()));
                }
            }

            if !extra_rules.is_empty() || !missing_rules.is_empty() {
                diff_files.push((
                    format!(
                        "{} [sp_rules={} sp_props={} | dist_rules={} dist_props={}]",
                        name,
                        sp_rules.len(),
                        sp_props_count,
                        dist_rules.len(),
                        dist_props_count
                    ),
                    [extra_rules, missing_rules].concat(),
                ));
            }
        }
    }

    tracing::info!(total = total, identical = identical, diff_files = diff_files.len(), "=== EP compressed diff 结果 ===");

    for (file, details) in diff_files.iter().take(20) {
        tracing::warn!(file = %file, "--- DIFF ---");
        for d in details.iter().take(5) {
            tracing::warn!("  {}", d);
        }
    }
}
