//! sasspile v2 CLI —— 纯 Rust 函数式 SCSS 编译器。

use std::io::{self, Read};

use sasspile::{OutputStyle, compile, compile_file, init_tracing};

fn main() {
    init_tracing();

    // 如果传入文件路径参数则用 compile_file，否则读 stdin
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let path = std::path::PathBuf::from(&args[1]);
        match compile_file(&path, OutputStyle::Expanded) {
            Ok(css) => print!("{css}"),
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::error!(error = %e, "编译错误");
                #[cfg(not(feature = "tracing"))]
                eprintln!("编译错误: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

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
