// 综合示例
//
// 演示 sasspile 的完整功能：变量、嵌套、函数、媒体查询

use sasspile::compile_expanded;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scss = r#"
// ===== 配置变量 =====
$primary: #3498db;
$secondary: #2ecc71;
$danger: #e74c3c;
$text-light: #ffffff;
$text-dark: #333333;
$spacing-unit: 8px;
$border-radius: 4px;

// ===== 通用按钮样式 =====
.button {
    display: inline-block;
    padding: $spacing-unit * 2 $spacing-unit * 3;
    border: none;
    border-radius: $border-radius;
    font-size: 16px;
    cursor: pointer;
    text-decoration: none;
    transition: background 0.3s;

    // 主按钮
    &.primary {
        background: $primary;
        color: $text-light;

        &:hover {
            background: darken($primary, 10%);
        }

        &:active {
            background: darken($primary, 15%);
        }
    }

    // 次按钮
    &.secondary {
        background: $secondary;
        color: $text-light;

        &:hover {
            background: darken($secondary, 10%);
        }
    }

    // 危险按钮
    &.danger {
        background: $danger;
        color: $text-light;

        &:hover {
            background: darken($danger, 10%);
        }
    }

    // 禁用状态
    &.disabled {
        opacity: 0.5;
        cursor: not-allowed;
        pointer-events: none;
    }
}

// ===== 卡片组件 =====
.card {
    background: white;
    border-radius: $border-radius * 2;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    overflow: hidden;

    .card-header {
        padding: $spacing-unit * 3;
        border-bottom: 1px solid #eee;

        h2 {
            margin: 0;
            font-size: 24px;
            color: $text-dark;
        }
    }

    .card-body {
        padding: $spacing-unit * 3;

        p {
            margin: 0 0 $spacing-unit * 2 0;
            line-height: 1.6;
            color: $text-dark;

            &:last-child {
                margin-bottom: 0;
            }
        }
    }

    .card-footer {
        padding: $spacing-unit * 2 $spacing-unit * 3;
        background: #f9f9f9;
        border-top: 1px solid #eee;

        .actions {
            display: flex;
            justify-content: flex-end;
            gap: $spacing-unit * 2;
        }
    }
}

// ===== 响应式网格 =====
.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 $spacing-unit * 2;
}

.grid {
    display: grid;
    gap: $spacing-unit * 2;

    // 手机端：1 列
    @media (max-width: 576px) {
        grid-template-columns: 1fr;
    }

    // 平板：2 列
    @media (min-width: 577px) and (max-width: 992px) {
        grid-template-columns: repeat(2, 1fr);
    }

    // 桌面：3 列
    @media (min-width: 993px) {
        grid-template-columns: repeat(3, 1fr);
    }
}

// ===== 注释示例 =====
/* 这是一个多行注释
   会在生成的 CSS 中保留 */

// 这是一个单行注释
// 不会出现在生成的 CSS 中

.comment-demo {
    /* 保留的注释 */
    color: red;
}
"#;

    let css = compile_expanded(scss)?;
    println!("{}", css);

    Ok(())
}