//! EP SCSS 语法特性统计。运行: cargo test --test ep_syntax_stats -- --nocapture

use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_ep_syntax_stats() {
    let dir = PathBuf::from("/Users/pauljohn/rust/element-plus-dev/packages/theme-chalk/src");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("无法读取目录")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "scss"))
        .collect();
    files.sort_by_key(|e| e.path());

    let mut stats: HashMap<String, usize> = HashMap::new();
    let mut total_files = 0;
    let mut total_lines = 0;

    for entry in &files {
        let content = std::fs::read_to_string(entry.path()).expect("读取失败");
        total_lines += content.lines().count();
        total_files += 1;

        for line in content.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with("//") { continue; }

            if t.starts_with("@use") { inc(&mut stats, "@use"); }
            if t.starts_with("@import") { inc(&mut stats, "@import"); }
            if t.starts_with("@forward") { inc(&mut stats, "@forward"); }
            if t.starts_with("@include") { inc(&mut stats, "@include"); }
            if t.starts_with("@mixin") { inc(&mut stats, "@mixin"); }
            if t.starts_with("@function") { inc(&mut stats, "@function"); }
            if t.starts_with("@if") { inc(&mut stats, "@if"); }
            if t.starts_with("@else if") { inc(&mut stats, "@else if"); }
            if t.starts_with("@else") && !t.starts_with("@else if") { inc(&mut stats, "@else"); }
            if t.starts_with("@for") { inc(&mut stats, "@for"); }
            if t.starts_with("@each") { inc(&mut stats, "@each"); }
            if t.starts_with("@while") { inc(&mut stats, "@while"); }
            if t.starts_with("@content") { inc(&mut stats, "@content"); }
            if t.starts_with("@extend") { inc(&mut stats, "@extend"); }
            if t.starts_with("@at-root") { inc(&mut stats, "@at-root"); }
            if t.starts_with("@media") { inc(&mut stats, "@media"); }
            if t.starts_with("@supports") { inc(&mut stats, "@supports"); }
            if t.starts_with("@keyframes") { inc(&mut stats, "@keyframes"); }
            if t.starts_with("@font-face") { inc(&mut stats, "@font-face"); }
            if t.starts_with("@charset") { inc(&mut stats, "@charset"); }
            if t.starts_with("@namespace") { inc(&mut stats, "@namespace"); }
            if t.starts_with("@warn") { inc(&mut stats, "@warn"); }
            if t.starts_with("@error") { inc(&mut stats, "@error"); }
            if t.starts_with("@debug") { inc(&mut stats, "@debug"); }
            if t.starts_with("@return") { inc(&mut stats, "@return"); }
            if t.starts_with("@layer") { inc(&mut stats, "@layer"); }
            if t.starts_with("@container") { inc(&mut stats, "@container"); }

            if t.starts_with('$') && t.contains(':') { inc(&mut stats, "变量定义 $var"); }
            if t.contains("#{") { inc(&mut stats, "插值 #{}"); }
            if t.contains("&.") || t.contains("&,") || t.contains("&:") || t.starts_with('&') {
                inc(&mut stats, "父选择器 &");
            }
            if t.contains("!default") { inc(&mut stats, "!default"); }
            if t.contains("!global") { inc(&mut stats, "!global"); }
            if t.contains("!important") { inc(&mut stats, "!important"); }
            if t.contains("...") && t.contains(')') { inc(&mut stats, "参数展开 ..."); }
            if t.contains("map-") { inc(&mut stats, "map-* 函数"); }
            if has_any(t, &["join(", "append(", "nth(", "length(", "set-nth(", "zip("]) {
                inc(&mut stats, "list 函数");
            }
            if has_any(t, &["rgba(", "rgb(", "hsl(", "hwb("]) { inc(&mut stats, "颜色构造"); }
            if has_any(t, &["darken(", "lighten(", "transparentize(", "opacify(",
                "fade-out(", "fade-in(", "mix(", "saturate(", "desaturate(",
                "adjust-hue(", "invert(", "grayscale(", "complement("]) {
                inc(&mut stats, "颜色操作");
            }
            if t.contains("math.") { inc(&mut stats, "math 模块"); }
            if t.contains("list.") { inc(&mut stats, "list 模块"); }
            if t.contains("meta.") { inc(&mut stats, "meta 模块"); }
            if t.contains("string.") { inc(&mut stats, "string 模块"); }
            if t.contains("selector.") { inc(&mut stats, "selector 模块"); }
            if has_any(t, &["type-of(", "unit(", "unitless(", "comparable(",
                "feature-exists(", "variable-exists(", "function-exists(",
                "mixin-exists(", "call(", "content-exists(", "get-function(",
                "inspect(", "global-variable-exists("]) {
                inc(&mut stats, "meta 内省");
            }
            if t.contains("if(") { inc(&mut stats, "if()"); }
            if t.contains("unquote(") { inc(&mut stats, "unquote()"); }
            if t.contains("quote(") { inc(&mut stats, "quote()"); }
            if t.contains("calc(") { inc(&mut stats, "calc()"); }
            if t.contains("var(--") { inc(&mut stats, "var()"); }
            if has_any(t, &[":not(", ":is(", ":where(", ":has("]) { inc(&mut stats, "复杂伪类"); }
            if has_any(t, &["nth-child(", "nth-of-type(", "nth-last"]) { inc(&mut stats, "nth-*"); }
            if t.contains("::") { inc(&mut stats, "伪元素 ::"); }
            if has_any(t, &[":hover", ":focus", ":active", ":disabled", ":checked"]) {
                inc(&mut stats, "状态伪类");
            }
            if t.contains(":root") { inc(&mut stats, ":root"); }
            if t.contains("&--") || t.contains("&__") { inc(&mut stats, "BEM 命名"); }
        }
    }

    let mut sorted: Vec<_> = stats.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let mut md = String::new();
    md.push_str("# EP (element-plus) SCSS 语法特性统计\n\n");
    md.push_str("**数据源**: `packages/theme-chalk/src/`  \n");
    md.push_str(&format!("**总文件数**: {total_files}  \n"));
    md.push_str(&format!("**总行数**: {total_lines}  \n\n"));
    md.push_str("---\n\n");
    md.push_str("## 按频次排序\n\n");
    md.push_str("| 频次 | 语法特性 |\n");
    md.push_str("|------|----------|\n");
    for (k, v) in &sorted {
        md.push_str(&format!("| {v} | {k} |\n"));
    }

    md.push_str("\n---\n\n");
    md.push_str("## 分类汇总\n\n");
    add_table(&mut md, "模块系统", &["@use", "@import", "@forward"], &sorted);
    add_table(&mut md, "混入与函数", &["@mixin", "@function", "@include", "@content", "@return"], &sorted);
    add_table(&mut md, "流程控制", &["@if", "@else if", "@else", "@for", "@each", "@while"], &sorted);
    add_table(&mut md, "CSS 指令", &["@media", "@supports", "@keyframes", "@font-face", "@at-root", "@layer", "@container", "@charset", "@namespace"], &sorted);
    add_table(&mut md, "变量与插值", &["变量定义 $var", "插值 #{}", "!default", "!global"], &sorted);
    add_table(&mut md, "Sass 模块", &["math 模块", "list 模块", "map-* 函数", "meta 模块", "string 模块", "selector 模块", "meta 内省"], &sorted);
    add_table(&mut md, "颜色", &["颜色构造", "颜色操作"], &sorted);
    add_table(&mut md, "CSS 功能", &["calc()", "var()", "父选择器 &", "BEM 命名"], &sorted);
    add_table(&mut md, "选择器", &["复杂伪类", "伪元素 ::", "nth-*", "状态伪类", ":root"], &sorted);
    add_table(&mut md, "其他", &["@extend", "@warn", "@error", "@debug", "!important", "参数展开 ...", "if()", "unquote()", "quote()", "list 函数"], &sorted);

    // 写文件
    std::fs::write("/Users/pauljohn/rust/sasspile/tests/ep_syntax_stats.md", &md)
        .expect("写入失败");

    tracing::info!("已生成 tests/ep_syntax_stats.md");
}

fn inc(map: &mut HashMap<String, usize>, key: &str) {
    *map.entry(key.to_string()).or_insert(0) += 1;
}

fn has_any(line: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| line.contains(p))
}

fn add_table(md: &mut String, title: &str, keys: &[&str], sorted: &[(String, usize)]) {
    let items: Vec<_> = sorted.iter().filter(|(k, _)| keys.contains(&k.as_str())).collect();
    if items.is_empty() { return; }
    md.push_str(&format!("### {title}\n\n"));
    md.push_str("| 频次 | 语法特性 |\n");
    md.push_str("|------|----------|\n");
    for (k, v) in items {
        md.push_str(&format!("| {v} | {k} |\n"));
    }
    md.push('\n');
}
