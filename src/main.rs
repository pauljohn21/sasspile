//! sasspile v2 CLI —— 纯 Rust 函数式 SCSS 编译器。

use std::io::{self, Read};

use sasspile::{OutputStyle, compile};

fn main() {
    // 读取 stdin
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("错误: 无法读取输入");
        std::process::exit(1);
    }

    // 编译
    let style = OutputStyle::Expanded;
    match compile(&input, style) {
        Ok(css) => print!("{css}"),
        Err(e) => {
            eprintln!("编译错误: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_compile() {
        let input = "a { color: red; }";
        let css = compile(input, OutputStyle::Expanded).unwrap();
        assert!(css.contains("color: red"));
    }
}
