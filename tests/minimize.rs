//! sass-spec 最小化工具——delta debugging 找到最小复现用例。
//!
//! HRX 解析使用 `hrx_auditor` crate（VFS + parser），正确支持 `===` 多层嵌套。
//!
//! 用法：
//! ```bash
//! # 看最小化摘要
//! RUST_LOG="minimize=info" cargo test --test minimize minimize_color_error -- --nocapture
//!
//! # 看每次移除尝试
//! RUST_LOG="minimize=debug" cargo test --test minimize minimize_color_error -- --nocapture
//! ```

mod common;

use hrx_auditor::parser::{parse_hrx as hrx_parse, HrxArchive, HrxEntry};
use hrx_auditor::vfs::Vfs;
use sasspile::lex::Lexer;
use sasspile::lex::token::Token;
use sasspile::parse::Parser;
use sasspile::parse::ast::Node;
use std::path::Path;

/// 失败判定——决定最小化后是否"仍然失败"。
enum FailOracle<'a> {
    /// 错误模式：编译仍然报错。
    Error,
    /// 输出保持模式：输出与原始（错误）输出相同。
    #[allow(dead_code)]
    OutputPreserve { original_output: &'a str },
}

impl<'a> FailOracle<'a> {
    fn still_fails(&self, input: &str) -> bool {
        match self {
            FailOracle::Error => match sasspile::compile_expanded(input) {
                Ok(css) => {
                    tracing::debug!(
                        target: "minimize",
                        input_len = input.len(),
                        output_len = css.len(),
                        "compiled OK, revert removal"
                    );
                    false
                }
                Err(e) => {
                    tracing::info!(
                        target: "minimize",
                        error = %e,
                        input_len = input.len(),
                        "still errors, keep removal"
                    );
                    true
                }
            },
            FailOracle::OutputPreserve { original_output } => {
                match sasspile::compile_expanded(input) {
                    Ok(css) => {
                        let same = css.trim() == original_output.trim();
                        tracing::info!(
                            target: "minimize",
                            output_unchanged = same,
                            input_len = input.len(),
                            "output comparison"
                        );
                        same
                    }
                    Err(_) => {
                        tracing::warn!(
                            target: "minimize",
                            "compilation failed during output-preserve"
                        );
                        false
                    }
                }
            }
        }
    }
}

/// 解析 SCSS 为 AST 节点列表。
fn parse_to_nodes(input: &str) -> Vec<Node> {
    let tokens: Vec<Token> = Lexer::new(input)
        .filter(|t| !matches!(t.as_ref(), Ok(Token::Whitespace) | Ok(Token::Eof)))
        .collect::<sasspile::error::Result<Vec<_>>>()
        .unwrap_or_default();
    Parser::parse(&tokens)
        .map(|ast| ast.nodes)
        .unwrap_or_default()
}

/// 最小化 SCSS 输入——delta debugging。
fn minimize(input: &str, oracle: &FailOracle) -> String {
    let mut nodes = parse_to_nodes(input);
    let original_n = nodes.len();

    tracing::info!(
        target: "minimize",
        original_nodes = original_n,
        input_len = input.len(),
        "minimization started"
    );

    let mut changed = true;
    let mut round = 0;

    while changed && nodes.len() > 1 {
        changed = false;
        round += 1;

        tracing::info!(
            target: "minimize",
            round = round,
            n_nodes = nodes.len(),
            "new round"
        );

        let mut i = 0;
        while i < nodes.len() {
            let removed = nodes.remove(i);
            let remaining: String = nodes
                .iter()
                .map(|n| n.to_scss(0))
                .collect::<Vec<_>>()
                .join("\n");

            tracing::debug!(
                target: "minimize",
                round = round,
                removed_node = ?std::mem::discriminant(&removed),
                remaining_nodes = nodes.len(),
                "trying removal"
            );

            if oracle.still_fails(&remaining) {
                changed = true;
                tracing::info!(
                    target: "minimize",
                    round = round,
                    index = i,
                    remaining_nodes = nodes.len(),
                    "removed node, still fails"
                );
            } else {
                nodes.insert(i, removed);
                tracing::debug!(
                    target: "minimize",
                    round = round,
                    index = i,
                    "reverted removal"
                );
            }
            i += 1;
        }
    }

    let result: String = nodes
        .iter()
        .map(|n| n.to_scss(0))
        .collect::<Vec<_>>()
        .join("\n");

    tracing::info!(
        target: "minimize",
        original_nodes = original_n,
        final_nodes = nodes.len(),
        original_len = input.len(),
        final_len = result.len(),
        rounds = round,
        "minimization complete"
    );

    result
}

/// HRX 解析——按 `===` 分组，提取所有 (name, input, expected) 三元组。
fn parse_hrx(content: &str) -> Vec<(String, String, String)> {
    let archive = match hrx_parse(content) {
        Ok(a) => a,
        Err(_) => return Vec::new(),
    };

    let groups: Vec<Vec<HrxEntry>> = {
        let mut groups: Vec<Vec<HrxEntry>> = Vec::new();
        let mut current: Vec<HrxEntry> = Vec::new();
        for entry in archive.entries {
            if entry.path.is_empty() {
                if !current.is_empty() {
                    groups.push(std::mem::take(&mut current));
                }
            } else {
                current.push(entry);
            }
        }
        if !current.is_empty() {
            groups.push(current);
        }
        groups
    };

    let mut cases = Vec::new();
    for group_entries in &groups {
        let group_archive = HrxArchive {
            entries: group_entries.clone(),
        };
        let vfs = Vfs::from_archive(&group_archive);
        let dirs = vfs.walk();

        for (dir_path, files) in &dirs {
            let input_file = files.iter().find(|(f, _)| f == "input.scss");
            if input_file.is_none() {
                continue;
            }
            let (_, input_content) = input_file.unwrap();
            let output = files
                .iter()
                .find(|(f, _)| f == "output.css")
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let has_error = files.iter().any(|(f, _)| f == "error");
            if !has_error && !output.is_empty() {
                let name = if dir_path == "." {
                    String::new()
                } else {
                    dir_path.clone()
                };
                cases.push((name, input_content.clone(), output));
            }
        }
    }
    cases
}

fn collect_hrx(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_hrx(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx")
                && let Ok(meta) = std::fs::metadata(&path)
                    && meta.len() < 50_000 {
                        files.push(path);
                    }
        }
    }
}

#[test]
fn minimize_color_error() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let dir = spec_root.join("core_functions/color");
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for (name, input, _expected) in &parse_hrx(&content) {
                if sasspile::compile_expanded(input).is_err() {
                    let minimized = minimize(input, &FailOracle::Error);
                    tracing::info!(test = %format!("{stem}/{name}"), "=== 最小化结果 ===");
                    tracing::info!(original_bytes = input.len(), input = %input, "原始");
                    tracing::info!(minimized_bytes = minimized.len(), minimized = %minimized, "最小化");
                    return; // 只处理第一个错误用例
                }
            }
        }
    }
    tracing::info!("no error cases found in core_functions/color");
}

#[test]
fn minimize_extend_error() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("sass-spec/spec");
    let dir = spec_root.join("directives/extend");
    let mut files = Vec::new();
    collect_hrx(&dir, &mut files);

    for file in &files {
        if let Ok(content) = std::fs::read_to_string(file) {
            let stem = file.file_stem().unwrap().to_string_lossy().to_string();
            for (name, input, _expected) in &parse_hrx(&content) {
                if sasspile::compile_expanded(input).is_err() {
                    let minimized = minimize(input, &FailOracle::Error);
                    tracing::info!(test = %format!("{stem}/{name}"), "=== 最小化结果 ===");
                    tracing::info!(original_bytes = input.len(), input = %input, "原始");
                    tracing::info!(minimized_bytes = minimized.len(), minimized = %minimized, "最小化");
                    return;
                }
            }
        }
    }
    tracing::info!("no error cases found in directives/extend");
}
