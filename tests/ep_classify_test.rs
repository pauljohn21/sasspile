//! —— EP DIFF 细粒度分类 ——

use std::path::PathBuf;
use std::process::Command;

const EP_SRC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/src"
);

fn compile_dart_sass(path: &PathBuf, load_paths: &[PathBuf]) -> Result<String, String> {
    let mut cmd = Command::new("/opt/homebrew/bin/sass");
    cmd.arg("--style=expanded").arg("--no-source-map");
    for lp in load_paths { cmd.arg("--load-path").arg(lp); }
    cmd.arg(path);
    let output = cmd.output().map_err(|e| format!("执行 dart-sass 失败: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn normalize(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' { i += 2; break; }
                i += 1;
            }
            if !result.ends_with(' ') && !result.is_empty() { result.push(' '); }
            continue;
        }
        if c == b'"' || c == b'\'' {
            let quote = c; result.push(c as char); i += 1;
            while i < bytes.len() && bytes[i] != quote {
                if bytes[i] == b'\\' && i + 1 < bytes.len() { result.push(bytes[i] as char); i += 1; }
                result.push(bytes[i] as char); i += 1;
            }
            if i < bytes.len() { result.push(bytes[i] as char); i += 1; }
            continue;
        }
        if c.is_ascii_whitespace() {
            while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
            result.push(' '); continue;
        }
        result.push(c as char); i += 1;
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn test_ep_classify_diffs() {
    sasspile::init_tracing();
    let src_dir = PathBuf::from(EP_SRC);
    let load_paths = vec![src_dir.clone(), src_dir.join("mixins")];

    let mut entries: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("无法读取 src 目录")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "scss"))
        .collect();
    entries.sort_by_key(|e| e.path());

    // 细分分类
    let mut v_structural: Vec<(String, String, String)> = Vec::new();
    let mut v_cssvar: Vec<String> = Vec::new();
    let mut v_pseudo: Vec<String> = Vec::new();
    let mut v_empty_img: Vec<String> = Vec::new();
    let mut v_atroot_issue: Vec<String> = Vec::new();
    let mut v_minor_css: Vec<(String, f64)> = Vec::new();
    let mut identical = 0;

    for entry in &entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        let sp_css = match sasspile::compile_file(&path, sasspile::OutputStyle::Expanded) {
            Ok(css) => css,
            Err(e) => {
                tracing::warn!(file = %name, error = %e, "COMPILE FAIL");
                v_empty_img.push(format!("{name}(FAIL:{e})"));
                continue;
            }
        };

        let dart_css = match compile_dart_sass(&path, &load_paths) {
            Ok(css) => css,
            Err(_) => continue,
        };

        let sp = normalize(&sp_css);
        let dn = normalize(&dart_css);

        if sp == dn {
            identical += 1;
            continue;
        }

        // 找第一处差异
        let sp_chars: Vec<char> = sp.chars().collect();
        let dn_chars: Vec<char> = dn.chars().collect();
        let first_diff = sp_chars.iter().zip(dn_chars.iter())
            .enumerate().find(|(_, (a, b))| a != b).map(|(i, _)| i);

        let pos = first_diff.unwrap_or(0);
        let s_start = pos.saturating_sub(50);
        let s_end = (pos + 80).min(sp_chars.len());
        let d_end = (pos + 80).min(dn_chars.len());
        let sp_ctx: String = sp_chars[s_start..s_end].iter().collect();
        let dn_ctx: String = dn_chars[s_start..d_end].iter().collect();

        let max_len = sp_chars.len().max(dn_chars.len());
        let sim = if max_len > 0 {
            100.0 - (sp_chars.iter().zip(dn_chars.iter())
                .filter(|(a, b)| a != b).count().max(
                    sp_chars.len().abs_diff(dn_chars.len())
                ) as f64 / max_len as f64 * 100.0)
        } else { 100.0 };

        let sp_has_empty = sp.contains("  ") || sp.contains("{ }") || sp.contains("( )");
        let atroot_issue = sp.contains("dark__") || sp.contains("__arrow") || sp.contains(".is-dark__");
        let cssvar_issue = sp.contains("#79bbff") && dn.contains("rgb(47");
        let pseudo_issue = sp.contains(": before") || sp.contains(":after");

        let owned_name = name.clone();
        if atroot_issue || owned_name == "popper.scss" || owned_name == "index.scss" {
            v_atroot_issue.push(owned_name);
        } else if cssvar_issue && (owned_name.contains("base") || owned_name.contains("var") || owned_name.contains("index")) {
            v_cssvar.push(owned_name);
        } else if pseudo_issue || owned_name.contains("anchor") {
            v_pseudo.push(owned_name);
        } else if sp_has_empty || sp.len() < dn.len() / 3 {
            v_empty_img.push(owned_name);
        } else {
            v_minor_css.push((format!("{owned_name}({sim:.0}%)"), sim));
            v_structural.push((owned_name, sp_ctx, dn_ctx));
        }
    }

    let total = entries.len();
    let diff = total - identical;
    tracing::warn!(total, identical = identical, diff, "=== EP SUMMARY ===");
    tracing::warn!(
        count = v_atroot_issue.len(),
        files = v_atroot_issue.join(", "),
        "CAT1 @at-root/BEM & 展開"
    );
    tracing::warn!(
        count = v_cssvar.len(),
        files = v_cssvar.join(", "),
        "CAT2 CSS 變量顔色格式"
    );
    tracing::warn!(
        count = v_pseudo.len(),
        files = v_pseudo.join(", "),
        "CAT3 僞元素/anchor 空格"
    );
    tracing::warn!(
        count = v_empty_img.len(),
        files = v_empty_img.join(", "),
        "CAT4 EmptySelector/編譯失敗"
    );
    tracing::warn!(count = v_structural.len(), "CAT5 選擇器結構/排序差異:");
    for (info, _) in &v_minor_css {
        tracing::warn!(file = %info, "  diff");
    }

    for (name, sp_ctx, dn_ctx) in &v_structural {
        if v_structural.len() <= 40 {
            tracing::warn!(file = %name, sp_ctx = %sp_ctx, dart_ctx = %dn_ctx, "STRUCTURAL");
        }
    }
}
