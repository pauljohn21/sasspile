//! 反应式管线测试 —— 使用和 rxrust 官方测试相同的模式
//!
//! 参考: rxrust-1.0.0-rc.5/tests/v1_integration.rs
//! 核心验证：pipeline::compile 是真正的 Observable，按 rxrust 方式消费。

use std::cell::RefCell;
use std::rc::Rc;
use rxrust::prelude::*;
use sasspile::pipeline::compile;

/// 收集 Observable<char> 为 String（trim 尾随 flush 空白）
fn collect_chars(obs: LocalBoxedObservable<'static, char, std::convert::Infallible>) -> String {
    let result = Rc::new(RefCell::new(Vec::new()));
    let result_clone = result.clone();
    obs.subscribe(move |ch| result_clone.borrow_mut().push(ch));
    let s: String = result.borrow().iter().collect();
    s.trim_end().to_string()
}

// ─── tokenize 阶段测试 ───────────────────────────────────────────────────────

#[test]
fn test_pipeline_tokenize_single_ident() {
    let output = collect_chars(compile("color"));
    assert_eq!(output, "color");
}

#[test]
fn test_pipeline_tokenize_braces_with_whitespace() {
    let output = collect_chars(compile("{ }"));
    assert_eq!(output, "{ }");
}

#[test]
fn test_pipeline_tokenize_keywords_true_false_null() {
    for (input, expected) in &[("true", "true"), ("false", "false"), ("null", "null")] {
        let output = collect_chars(compile(input));
        assert_eq!(output, *expected, "keyword {input} tokenize failed");
    }
}

#[test]
fn test_pipeline_tokenize_numbers() {
    let output = collect_chars(compile("42"));
    assert_eq!(output, "42");
}

#[test]
fn test_pipeline_tokenize_decimal_number() {
    let output = collect_chars(compile("3.14"));
    assert_eq!(output, "3.14");
}

#[test]
fn test_pipeline_tokenize_mixed_ident_and_number() {
    let output = collect_chars(compile("a { z"));
    assert_eq!(output, "a { z");
}

#[test]
fn test_pipeline_tokenize_string() {
    let output = collect_chars(compile("\"hello\""));
    assert!(output.contains("hello"), "string content preserved, got: {output}");
}

#[test]
fn test_pipeline_tokenize_empty_input() {
    let output = collect_chars(compile(""));
    assert!(output.is_empty(), "empty input should produce empty output, got: {output}");
}

// ─── 与旧 Lexer 的 token-by-token 对比 ───────────────────────────────────────

#[test]
fn test_pipeline_tokens_match_old_lexer() {
    // 收集 pipeline token 序列
    let pipeline_tokens = compile_tokens("a { color: red; }");

    // 收集旧 Lexer token 序列
    use sasspile::lex::Lexer;
    let old_tokens: Vec<String> = Lexer::new("a { color: red; }")
        .filter(|t| !matches!(t, Ok(sasspile::lex::token::Token::Whitespace)))
        .filter(|t| !matches!(t, Ok(sasspile::lex::token::Token::Eof)))
        .map(|t| t.unwrap().to_string())
        .collect();

    assert_eq!(pipeline_tokens, old_tokens, "token序列应与旧Lexer一致");
}

use sasspile::lex::stream::{feed, LexerState};

fn compile_tokens(input: &str) -> Vec<String> {
    use rxrust::prelude::*;
    // 复用 pipeline 内部的 feed 函数重建 tokenize，收集 token
    let mut chars: Vec<char> = input.chars().collect();
    chars.push(' ');

    let result = Rc::new(RefCell::new(Vec::new()));
    let result_clone = result.clone();

    Local::from_iter(chars)
        .scan(LexerState::default(), move |state, ch| {
            let (new_state, emitted) = feed(state.clone(), ch);
            for tok in &emitted {
                // 跳过 whitespace 和 eof
                if !matches!(tok, sasspile::lex::token::Token::Whitespace)
                    && !matches!(tok, sasspile::lex::token::Token::Eof)
                {
                    result_clone.borrow_mut().push(tok.to_string());
                }
            }
            new_state
        })
        .box_it()
        .subscribe(|_| {}); // 消费触发 scan

    // 副作用收集：scan 的 emit 类型限制使得纯 Observable 链不可行
    result.borrow().clone()
}

#[test]
fn test_pipeline_round_trip_identity() {
    // 多种输入的 round-trip 测试
    for input in &["color", "42", "3.14", "{ }", "a b c", "\"str\"", "true"] {
        let output = collect_chars(compile(input));
        assert_eq!(output, *input, "round-trip identity failed for: {input}");
    }
}
