//! sass-spec 最小化工具——delta debugging 找到最小复现用例。
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

/// HRX 解析——提取所有 (name, input, expected) 三元组。
fn parse_hrx(content: &str) -> Vec<(String, String, String)> {
    let mut files: Vec<(String, String)> = Vec::new();
    let mut path = String::new();
    let mut content_buf = String::new();
    for line in content.lines() {
        if line.starts_with("<===>") {
            if !path.is_empty() {
                files.push((path.clone(), content_buf));
            }
            path = line.trim_start_matches("<===>").trim().to_string();
            content_buf = String::new();
        } else {
            content_buf.push_str(line);
            content_buf.push('\n');
        }
    }
    if !path.is_empty() {
        files.push((path, content_buf));
    }
    let mut cases = Vec::new();
    for (p, input) in &files {
        if p.ends_with("input.scss") {
            let base = p.strip_suffix("input.scss").unwrap_or(p).to_string();
            let out_path = format!("{base}output.css");
            let err_path = format!("{base}error");
            let output = files
                .iter()
                .find(|(pp, _)| pp == &out_path)
                .map(|(_, c)| c.clone())
                .unwrap_or_default();
            let has_error = files.iter().any(|(pp, _)| pp == &err_path);
            if !has_error && !output.is_empty() {
                cases.push((
                    base.trim_end_matches('/').to_string(),
                    input.clone(),
                    output,
                ));
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
            } else if path.extension().and_then(|s| s.to_str()) == Some("hrx") {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() < 50_000 {
                        files.push(path);
                    }
                }
            }
        }
    }
}

#[test]
fn minimize_color_error() {
    sasspile::init_tracing();
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
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
    let spec_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../sass-spec-main/spec");
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
