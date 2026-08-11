//! sasspile v2 CLI —— 纯 Rust 函数式 SCSS 编译器。
//!
//! 使用 tracing 进行问题追踪，不使用 eprintln!。

use std::io::{self, Read};

use sasspile::{OutputStyle, compile, init_tracing};

fn main() {
    init_tracing();

    // 读取 stdin
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        tracing::error!("无法读取输入");
        std::process::exit(1);
    }

    // 编译
    let style = OutputStyle::Expanded;
    match compile(&input, style) {
        Ok(css) => print!("{css}"),
        Err(e) => {
            tracing::error!(error = %e, "编译错误");
            std::process::exit(1);
        }
    }
}
