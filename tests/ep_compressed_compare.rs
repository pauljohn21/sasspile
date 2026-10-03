//! 压缩格式直接对比——验证 lightningcss 规范化前后对比。
#![allow(clippy::unwrap_used, clippy::uninlined_format_args, clippy::print_stdout, clippy::print_stderr)]

#[test]
#[ignore = "Diagnostic tool, run manually"]
fn diag_normalized_compare() {
    use lightningcss::{
        printer::PrinterOptions,
        stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    };

    fn normalize(css: &str) -> String {
        let ss = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let mut s = ss;
        s.minify(MinifyOptions::default()).unwrap();
        let result = s.to_css(PrinterOptions { minify: true, ..Default::default() }).unwrap();
        result.code
    }

    let src = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/src/dialog.scss"
    );
    let dist_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/element-plus/packages/theme-chalk/dist/el-dialog.css"
    );

    let sp_expanded = sasspile::compile_file(
        &std::path::PathBuf::from(src),
        sasspile::OutputStyle::Expanded,
    )
    .unwrap();
    let dist = std::fs::read_to_string(dist_path).unwrap();

    let sp_norm = normalize(&sp_expanded);
    let dist_norm = normalize(&dist);

    eprintln!("NORMALIZED LENGTHS: SP={} EP={}", sp_norm.len(), dist_norm.len());

    // 写文件
    std::fs::write("/tmp/epnorm_sp.txt", &sp_norm).unwrap();
    std::fs::write("/tmp/epnorm_ep.txt", &dist_norm).unwrap();

    // 找到 diff
    let pos = sp_norm.chars().zip(dist_norm.chars()).position(|(a, b)| a != b);
    if let Some(p) = pos {
        let sp_ctx: String = sp_norm.chars().skip(p.saturating_sub(30)).take(100).collect();
        let dist_ctx: String = dist_norm.chars().skip(p.saturating_sub(30)).take(100).collect();
        eprintln!("NORM FIRST DIFF @{} SP=`{}`", p, sp_ctx);
        eprintln!("NORM FIRST DIFF @{} EP=`{}`", p, dist_ctx);
    } else {
        eprintln!("NORM IDENTICAL: positions match (len diff={})", sp_norm.len() as isize - dist_norm.len() as isize);
    }

    // 找所有 diff 段
    let sp_c: Vec<char> = sp_norm.chars().collect();
    let dist_c: Vec<char> = dist_norm.chars().collect();
    let max = sp_c.len().max(dist_c.len());
    let mut diff_count = 0;
    let mut in_diff = false;
    for i in 0..max {
        let a = sp_c.get(i).copied().unwrap_or('\0');
        let b = dist_c.get(i).copied().unwrap_or('\0');
        if a != b && !in_diff {
            in_diff = true;
        } else if a == b && in_diff {
            in_diff = false;
            diff_count += 1;
        }
    }
    if in_diff {
        diff_count += 1;
    }
    eprintln!("NORM TOTAL DIFF SEGMENTS: {}", diff_count);
}
