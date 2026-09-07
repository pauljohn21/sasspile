//! HSL 失败 diff — 40 samples。
mod hrx_support;
use hrx_support::{HrxArchive, parse_hrx as hrx_parse};
use std::path::{Path, PathBuf};

struct HrxCase { files: Vec<(String, String)>, input_path: String, expected_output: String }

fn parse_hrx<'a>(content: &'a str) -> Vec<HrxCase> {
    let Ok(archive) = hrx_parse(content) else { return Vec::new(); };
    let groups: Vec<Vec<hrx_support::HrxEntry>> = { let mut g = Vec::new(); let mut c = Vec::new();
        for e in archive.entries { if e.path.is_empty() { if !c.is_empty() { g.push(std::mem::take(&mut c)); } } else { c.push(e); } }
        if !c.is_empty() { g.push(c); } g
    };
    let mut cases = Vec::new();
    for ge in &groups {
        let ga = HrxArchive { entries: ge.clone() };
        let vfs = hrx_support::Vfs::from_archive(&ga);
        let dirs = vfs.walk();
        let af: Vec<(String, String)> = dirs.iter().flat_map(|(dp, files)| files.iter().map(move |(f, c)|
            if dp == "." { (f.clone(), c.clone()) } else { (format!("{dp}/{f}"), c.clone()) }
        )).filter(|(p, _)| (p.ends_with(".scss") || p.ends_with(".css")) && !p.contains("/sass/")).collect();
        for (dp, files) in &dirs {
            let Some((in_, _)) = files.iter().find(|(f, _)| f == "input.scss") else { continue; };
            let ip = if dp == "." { in_.clone() } else { format!("{dp}/{in_}") };
            let eo = files.iter().find(|(f, _)| f == "output.css").map(|(_, c)| c.clone()).unwrap_or_default();
            cases.push(HrxCase { files: af.clone(), input_path: ip, expected_output: eo });
        }
    }
    cases
}

fn collect_hrx(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) { for e in entries.flatten() {
        let p = e.path(); if p.is_dir() { collect_hrx(&p, files); }
        else if p.extension().and_then(|s| s.to_str()) == Some("hrx") { files.push(p); }
    }}
}

fn cc(case: &HrxCase, sr: &Path, st: &str) -> Result<String, String> {
    let tmp = std::env::temp_dir().join(format!("dhsl3-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok(); std::fs::create_dir_all(tmp.join(st)).ok();
    for (p, c) in &case.files { let t = if p.starts_with(&format!("{st}/")) { tmp.join(p) } else { tmp.join(st).join(p) };
        if let Some(pa) = t.parent() { std::fs::create_dir_all(pa).ok(); } std::fs::write(&t, c).ok(); }
    let i = if case.input_path.starts_with(&format!("{st}/")) { tmp.join(&case.input_path) } else { tmp.join(st).join(&case.input_path) };
    let r = sasspile::compile_file_with_load_paths(&i, sasspile::OutputStyle::Expanded, vec![sr.to_path_buf()]);
    let _ = std::fs::remove_dir_all(&tmp); r.map_err(|e| format!("{e}"))
}

#[test]
fn hsl_diffs() {
    let sr = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let mut files = Vec::new(); collect_hrx(&sr.join("core_functions/color/hsl"), &mut files);
    let mut n = 0;
    for f in &files { if let Ok(content) = std::fs::read_to_string(f) {
        let st = f.file_stem().unwrap().to_string_lossy().to_string();
        for case in &parse_hrx(&content) {
            if case.expected_output.is_empty() { continue; }
            if n >= 35 { break; }
            let nm = case.input_path.replace("input.scss", "");
            match cc(case, &sr, &st) {
                Ok(r) if r.trim() == case.expected_output.trim() => {},
                Ok(r) => { n += 1; eprintln!("[{st}/{nm}]\nEXPECTED:\n{}\nGOT:\n{}\n", case.expected_output.trim(), r.trim()); },
                Err(e) => { n += 1; eprintln!("[{st}/{nm}] ERROR: {e}\n"); },
            }
        }
    }}
}
