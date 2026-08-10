// 变量使用示例
//
// 演示 SCSS 变量定义和使用

use sasspile::compile_expanded;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scss = r#"
// 定义颜色变量
$primary: #3498db;
$secondary: #2ecc71;
$text-color: #333;
$font-size: 16px;

$spacing-unit: 8px;

.button {
    background: $primary;
    color: white;
    padding: $spacing-unit * 2;
    font-size: $font-size;

    &:hover {
        background: darken($primary, 10%);
    }
}

.card {
    border: 1px solid $secondary;
    padding: $spacing-unit * 3;
    color: $text-color;
}
"#;

    let css = compile_expanded(scss)?;
    println!("{}", css);

    Ok(())
}
