//! sasspile v2 CLI —— 纯 Rust 函数式 SCSS 编译器。

use std::io::{self, Read};

use sasspile::{OutputStyle, compile, compile_file, init_tracing_otel};

fn main() {
    init_tracing_otel();

    // 检查命令行参数：如果提供了文件路径，从文件读取
    let args: Vec<String> = std::env::args().collect();
    let (input, is_file, file_path) = if args.len() > 1 {
        let path = &args[1];
        match std::fs::read_to_string(path) {
            Ok(content) => (content, true, Some(path.clone())),
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::error!(error = %e, "无法读取文件");
                #[cfg(not(feature = "tracing"))]
                eprintln!("无法读取文件: {e}");
                std::process::exit(1);
            }
        }
    } else {
        // 从 stdin 读取
        let mut input = String::new();
        if io::stdin().read_to_string(&mut input).is_err() {
            #[cfg(feature = "tracing")]
            tracing::error!("无法读取输入");
            #[cfg(not(feature = "tracing"))]
            eprintln!("无法读取输入");
            std::process::exit(1);
        }
        (input, false, None)
    };

    // 编译
    let style = OutputStyle::Expanded;
    let result = if is_file {
        let path = std::path::PathBuf::from(file_path.unwrap());
        compile_file(&path, style)
    } else {
        compile(&input, style)
    };

    match result {
        Ok(css) => {
            use std::io::Write;
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(css.as_bytes());
        }
        Err(e) => {
            #[cfg(feature = "tracing")]
            tracing::error!(error = %e, "编译错误");
            #[cfg(not(feature = "tracing"))]
            eprintln!("编译错误: {e}");
            std::process::exit(1);
        }
    }
}
