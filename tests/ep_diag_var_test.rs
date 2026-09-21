//! CSS 变量颜色格式诊断

use std::path::PathBuf;

const EP_SRC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/src"
);

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
fn test_diag_var_color_format() {
    sasspile::init_tracing();
    let src = PathBuf::from(EP_SRC);

    for fname in ["var.scss", "base.scss"] {
        let path = src.join(fname);
        let sp = normalize(&sasspile::compile_file(&path, sasspile::OutputStyle::Expanded).unwrap());
        let output = std::process::Command::new("/opt/homebrew/bin/sass")
            .arg("--style=expanded").arg("--no-source-map")
            .arg(&path)
            .output()
            .expect("dart-sass");
        let dn = normalize(&String::from_utf8_lossy(&output.stdout).to_string());

        // 比较所有 CSS 变量值
        let sp_vars: Vec<String> = sp.split("--el-").filter(|s| s.contains(':')).map(|s| {
            let val = s.split(':').nth(1).unwrap_or("").trim().trim_end_matches(';').trim();
            format!("--el-{}", s.split(':').next().unwrap_or("").trim())
        }).collect();

        // 检查不同点
        let mut diffs = Vec::new();
        for (sp_line, dn_line) in sp.lines().zip(dn.lines()) {
            if sp_line != dn_line {
                diffs.push((sp_line.to_string(), dn_line.to_string()));
                if diffs.len() >= 3 { break; }
            }
        }

        tracing::warn!(file = fname, total_diffs = diffs.len(), "CSS VAR DIAG:");
        for (sp_ctx, dart_ctx) in &diffs {
            tracing::warn!(sasspile = %sp_ctx, dart = %dart_ctx, "  diff line");
        }
    }

    // 找第一处颜色相关的差异
    let path = src.join("var.scss");
    let sp = sasspile::compile_file(&path, sasspile::OutputStyle::Expanded).unwrap();
    let dn = String::from_utf8_lossy(
        &std::process::Command::new("/opt/homebrew/bin/sass")
            .arg("--style=expanded").arg("--no-source-map")
            .arg(&path)
            .output().unwrap().stdout
    ).to_string();

    let sp_chars: Vec<char> = sp.chars().collect();
    let dn_chars: Vec<char> = dn.chars().collect();
    for (i, (a, b)) in sp_chars.iter().zip(dn_chars.iter()).enumerate() {
        if a != b {
            let start = i.saturating_sub(60);
            let end = (i + 120).min(sp_chars.len());
            let s: String = sp_chars[start..end].iter().collect();
            let d_end = (i + 120).min(dn_chars.len());
            let d: String = dn_chars[start..d_end].iter().collect();
            tracing::warn!(pos = i, sasspile = %s, dart = %d, "FIRST COLOR DIFF");
            break;
        }
    }
}
