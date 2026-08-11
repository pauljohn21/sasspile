//! sasspile v2 CLI —— 纯 Rust 函数式 SCSS 编译器。

use std::io::{self, Read};

use sasspile::{OutputStyle, compile, init_tracing};

fn main() {
    init_tracing();

    // 读取 stdin
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        #[cfg(feature = "tracing")]
        tracing::error!("无法读取输入");
        #[cfg(not(feature = "tracing"))]
        eprintln!("无法读取输入");
        std::process::exit(1);
    }

    // 编译
    let style = OutputStyle::Expanded;
    match compile(&input, style) {
        Ok(css) => print!("{css}"),
        Err(e) => {
            #[cfg(feature = "tracing")]
            tracing::error!(error = %e, "编译错误");
            #[cfg(not(feature = "tracing"))]
            eprintln!("编译错误: {e}");
            std::process::exit(1);
        }
    }
}
