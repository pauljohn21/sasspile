// 嵌套规则示例
//
// 演示 SCSS 选择器嵌套功能

use sasspile::compile_expanded;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scss = r#"
.navbar {
    background: #333;
    padding: 10px 0;

    // 嵌套子元素
    .logo {
        font-size: 24px;
        color: white;
    }

    .menu {
        display: flex;
        gap: 20px;

        li {
            list-style: none;

            a {
                color: white;
                text-decoration: none;

                // 嵌套伪类
                &:hover {
                    color: #fff;
                }
            }
        }
    }
}

// 伪类和伪元素
.button {
    background: #3498db;
    color: white;

    &:hover {
        background: darken(#3498db, 10%);
    }

    &:active {
        transform: scale(0.95);
    }

    &::before {
        content: "→";
        margin-right: 5px;
    }
}
"#;

    let css = compile_expanded(scss)?;
    println!("{}", css);

    Ok(())
}
