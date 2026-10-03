//! Minimal dialog check with the strip function applied
#![allow(clippy::unwrap_used, clippy::uninlined_format_args, clippy::print_stdout, clippy::print_stderr)]

use std::path::PathBuf;

const EP_SRC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/src"
);
const EP_DIST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/dist"
);

fn minify(css: &str) -> String {
    use lightningcss::{
        printer::PrinterOptions,
        stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
    };
    let mut s = StyleSheet::parse(css, ParserOptions::default()).unwrap();
    s.minify(MinifyOptions::default()).unwrap();
    s.to_css(PrinterOptions { minify: true, ..Default::default() }).unwrap().code
}

// Inline copy of strip_empty_boundary_keyframes from ep_normalized_test.rs
fn strip_empty_boundary_keyframes(css: String) -> String {
    fn find_matching_close(s: &str, open_pos: usize) -> Option<usize> {
        let mut depth = 0;
        for (i, c) in s[open_pos..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 { return Some(open_pos + i); }
                }
                _ => {}
            }
        }
        None
    }
    fn remove_boundary_empty_steps(body: &str) -> Option<String> {
        let trimmed = body.trim();
        if trimmed.is_empty() { return None; }
        let steps = split_keyframe_steps(trimmed);
        if steps.is_empty() { return None; }
        let mut ts: Vec<&str> = steps.iter().copied().collect();
        while ts.first().is_some_and(|s| is_empty_step(s)) { ts.remove(0); }
        while ts.last().is_some_and(|s| is_empty_step(s)) { ts.pop(); }
        if ts.is_empty() { Some(String::new()) } else { Some(ts.join("")) }
    }
    fn split_keyframe_steps(body: &str) -> Vec<&str> {
        let mut result = Vec::new();
        let mut depth = 0;
        let mut step_start = 0;
        let bytes = body.as_bytes();
        for i in 0..bytes.len() {
            match bytes[i] {
                b'{' => {
                    if depth == 0 {
                        step_start = walk_back_to_step_start(body, i);
                    }
                    depth += 1;
                }
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        result.push(&body[step_start..=i]);
                        let mut next = i + 1;
                        while next < bytes.len() && bytes[next].is_ascii_whitespace() { next += 1; }
                        step_start = next;
                    }
                }
                _ => {}
            }
        }
        result
    }
    fn walk_back_to_step_start(body: &str, brace_pos: usize) -> usize {
        let bytes = body.as_bytes();
        let mut i = brace_pos;
        while i > 0 && bytes[i - 1].is_ascii_whitespace() { i -= 1; }
        while i > 0 && !bytes[i - 1].is_ascii_whitespace() {
            if bytes[i - 1] == b'}' { break; }
            i -= 1;
        }
        i
    }
    fn is_empty_step(step: &str) -> bool {
        let Some(open) = step.find('{') else { return false; };
        let Some(close) = step.rfind('}') else { return false; };
        if open >= close { return false; }
        step[open + 1..close].trim().is_empty()
    }
    
    let mut result = css;
    let mut search_start = 0;
    while let Some(kf_rel) = result[search_start..].find("@keyframes") {
        let kf_abs = search_start + kf_rel;
        let after_kw = kf_abs + "@keyframes".len();
        let Some(brace_rel) = result[after_kw..].find('{') else { break; };
        let brace_open = after_kw + brace_rel;
        let Some(brace_close) = find_matching_close(&result, brace_open) else { break; };
        let body = result[brace_open + 1..brace_close].to_string();
        let new_body_opt = remove_boundary_empty_steps(&body);
        match new_body_opt {
            Some(new_body) if new_body.len() != body.len() => {
                if new_body.is_empty() {
                    let mut del_start = kf_abs;
                    while del_start > 0 {
                        let b = result.as_bytes()[del_start - 1];
                        if b == b'\n' || b == b' ' { del_start -= 1; } else { break; }
                    }
                    let mut del_end = brace_close + 1;
                    while del_end < result.len() {
                        let b = result.as_bytes()[del_end];
                        if b == b'\n' || b == b' ' { del_end += 1; } else { break; }
                    }
                    result.replace_range(del_start..del_end, "");
                    search_start = del_start;
                } else {
                    result.replace_range(brace_open + 1..brace_close, &new_body);
                    let offset = new_body.len() as isize - body.len() as isize;
                    search_start = (brace_close as isize + offset) as usize + 1;
                }
            }
            _ => {
                search_start = brace_close + 1;
            }
        }
    }
    result
}

#[test]
fn test_dialog_with_strip() {
    sasspile::init_tracing();
    
    let src = PathBuf::from(EP_SRC).join("dialog.scss");
    let sp_expanded = sasspile::compile_file(&src, sasspile::OutputStyle::Expanded).unwrap();
    let sp_min = minify(&sp_expanded);
    let sp_stripped = strip_empty_boundary_keyframes(sp_min.clone());
    
    let dist = PathBuf::from(EP_DIST).join("el-dialog.css");
    let ep_raw = std::fs::read_to_string(&dist).unwrap();
    let ep_min = minify(&ep_raw);
    let ep_stripped = strip_empty_boundary_keyframes(ep_min.clone());
    
    eprintln!("SP: expanded.len={} minified.len={} stripped.len={}", sp_expanded.len(), sp_min.len(), sp_stripped.len());
    eprintln!("EP: raw.len={} minified.len={} stripped.len={}", ep_raw.len(), ep_min.len(), ep_stripped.len());
    eprintln!("SP has to{{}}: {} SP has 0%{{}}: {}, SP-stripped has to{{}}: {}", 
        sp_min.contains("to{}"), sp_min.contains("0%{}"), sp_stripped.contains("to{}"));
    
    if sp_stripped == ep_stripped {
        eprintln!("✓ EXACT MATCH after strip");
    } else {
        let diff_pos = sp_stripped.chars().zip(ep_stripped.chars())
            .position(|(a, b)| a != b).unwrap_or(sp_stripped.len().min(ep_stripped.len()));
        eprintln!("Diff after strip at pos {}", diff_pos);
        
        // Show diff segments
        let mut in_diff = false;
        let mut start = 0;
        let mut segments: Vec<(usize, usize, String, String)> = vec![];
        for (i, (a, b)) in sp_stripped.chars().zip(ep_stripped.chars()).enumerate() {
            if a != b {
                if !in_diff { start = i; in_diff = true; }
            } else if in_diff {
                let s: String = sp_stripped.chars().skip(start).take(i - start).collect();
                let e: String = ep_stripped.chars().skip(start).take(i - start).collect();
                segments.push((start, i, s, e));
                in_diff = false;
            }
        }
        
        eprintln!("{} diff segments after strip:", segments.len());
        for (s, e, sp_s, ep_s) in segments.iter().take(15) {
            eprintln!("  @{}-{}: SP=`{}` EP=`{}`", s, e, 
                sp_s.chars().take(60).collect::<String>(),
                ep_s.chars().take(60).collect::<String>());
        }
    }
}
