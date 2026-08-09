// 内建函数示例
//
// 演示 sasspile 支持的各类内建函数

use sasspile::compile_expanded;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scss = r#"
// ===== 颜色函数 =====
$base-color: #3498db;

.box-1 {
    background: rgba(52, 152, 219, 0.8);
    border: 1px solid darken($base-color, 20%);
}

.box-2 {
    background: lighten($base-color, 10%);
}

.box-3 {
    background: mix($base-color, #e74c3c, 50%);
}

.box-4 {
    background: invert($base-color);
}

.box-5 {
    background: grayscale($base-color);
}

// ===== 数学函数 =====
.container {
    width: 100% - 20px;
    padding: math.max(10px, 15px);
    font-size: math.round(16.6px);
}

// ===== 字符串函数 =====
.test-string {
    content: str-length("hello world");
    content: str-index("hello", "e");
    content: to-upper-case("hello");
    content: to-lower-case("WORLD");
}

// ===== 列表函数 =====
$colors: (#f00, #0f0, #00f);

.palette {
    background: list.nth($colors, 2);
}

// ===== 媒体查询 =====
.responsive {
    width: 100%;

    @media (max-width: 768px) {
        width: 100%;
        font-size: 14px;
    }

    @media (min-width: 769px) and (max-width: 1200px) {
        width: 80%;
        margin: 0 auto;
    }
}
"#;

    let css = compile_expanded(scss)?;
    println!("{}", css);

    Ok(())
}