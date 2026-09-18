mod directive;
mod state;

use std::sync::{Arc, Mutex};

use directive::DirectiveOps;
use rxrust::prelude::*;

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();

    for ch in input.chars() {
        if ch == ';' || ch == '{' || ch == '}' {
            if !buf.is_empty() {
                tokens.push(buf.clone());
                buf.clear();
            }
            tokens.push(ch.to_string());
        } else {
            buf.push(ch);
        }
    }

    if !buf.is_empty() {
        tokens.push(buf);
    }

    tokens
}

pub async fn compile(input: &str) -> String {
    let result = Arc::new(Mutex::new(String::new()));
    let result_clone = result.clone();

    let handle = Shared::from_stream(futures::stream::iter(tokenize(input)))
        .use_()
        .mixin()
        .include()
        .if_()
        .for_()
        .each()
        .map(|token| token + "\n")
        .collect::<String>()
        .last()
        .subscribe(move |s| {
            *result_clone.lock().unwrap() = s;
        });

    handle.await;

    Arc::try_unwrap(result).unwrap().into_inner().unwrap()
}

pub async fn compile_parallel(input: &str) -> String {
    compile(input).await
}
