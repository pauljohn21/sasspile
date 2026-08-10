// 基础编译示例
//
// 演示如何使用 sasspile 将 SCSS 编译为 CSS

use sasspile::{OutputStyle, compile_compressed, compile_expanded};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scss = r#"
a {
    color: red;
    background: white;
}
"#;

    println!("=== 展开式输出 ===");
    let css_expanded = compile_expanded(scss)?;
    println!("{}", css_expanded);

    println!("\n=== 压缩式输出 ===");
    let css_compressed = compile_compressed(scss)?;
    println!("{}", css_compressed);

    println!("\n=== 使用 compile 函数 ===");
    let css = sasspile::compile(scss, OutputStyle::Expanded)?;
    println!("{}", css);

    Ok(())
}
