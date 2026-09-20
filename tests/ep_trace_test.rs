//! EP @function 分派诊断测试
//!
//! 编译一个简单 EP scss 文件（仅含 `@use` 和函数调用），输出完整 trace。
//! 用于定位 `@function` 调用链 miss 的根因。
//!
//! 运行前置：EP submodule 已初始化
//!
//! 运行：RUST_LOG=trace cargo test --test ep_trace_test -- --nocapture 2>&1 > /tmp/ep_trace.log

use std::path::PathBuf;

const EP_SRC: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/element-plus/packages/theme-chalk/src"
);

/// 编译一个极简 test scss，不依赖外部文件
#[test]
fn test_trace_func_dispatch() {
    sasspile::init_tracing();

    // 内联测试：定义 + 调用 @function
    let inline_test = r#"
@function test-double($x) {
    @return $x * 2;
}
@function test-join($a, $b) {
    @return $a + "-" + $b;
}
.result {
    width: test-double(5);
    name: test-join("foo", "bar");
}
"#;

    tracing::info!("=== 内联 @function 测试 ===");
    let result =
        sasspile::compile(inline_test, sasspile::OutputStyle::Expanded);
    match &result {
        Ok(css) => tracing::info!(output = %css, "内联编译成功"),
        Err(e) => tracing::error!(error = %e, "内联编译失败"),
    }

    // 内联带 list 参数
    let list_test = r#"
@function test-first($list) {
    @return nth($list, 1);
}
.res {
    val: test-first((a, b, c));
}
"#;

    tracing::info!("=== List 参数 @function 测试 ===");
    let result =
        sasspile::compile(list_test, sasspile::OutputStyle::Expanded);
    match &result {
        Ok(css) => tracing::info!(output = %css, "List 编译成功"),
        Err(e) => tracing::error!(error = %e, "List 编译失败"),
    }

    // EP 真实文件 - button.scss
    tracing::info!("=== EP button.scss 测试 ===");
    let button_path = PathBuf::from(EP_SRC).join("button.scss");
    if button_path.exists() {
        let load_paths = vec![
            PathBuf::from(EP_SRC),
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/element-plus/packages/theme-chalk/node_modules"
            )),
        ];
        let result = sasspile::compile_file_with_load_paths(
            &button_path,
            sasspile::OutputStyle::Expanded,
            load_paths,
        );
        match &result {
            Ok(css) => {
                // 检查输出是否包含未解析的函数调用
                let unresolved: Vec<&str> = css
                    .lines()
                    .filter(|l| {
                        (l.contains("joinVarName(")
                            || l.contains("getCssVar(")
                            || l.contains("bem(")
                            || l.contains("selectorParse("))
                            && !l.contains("///")
                    })
                    .collect();

                if unresolved.is_empty() {
                    tracing::info!("EP button.scss 输出无未解析函数调用");
                } else {
                    tracing::warn!(
                        count = unresolved.len(),
                        lines = ?unresolved.iter().take(5).collect::<Vec<_>>(),
                        "发现未解析函数调用！"
                    );
                }
                tracing::info!(total_len = css.len(), "EP button.scss 编译完成");
            }
            Err(e) => tracing::error!(error = %e, "EP button.scss 编译失败"),
        }
    } else {
        tracing::warn!(path = ?button_path, "button.scss 不存在");
    }

    // EP aside.scss — 最简单的组件，用于隔离问题
    tracing::info!("=== EP aside.scss 测试 ===");
    let aside_path = PathBuf::from(EP_SRC).join("aside.scss");
    if aside_path.exists() {
        let load_paths = vec![PathBuf::from(EP_SRC)];
        let result = sasspile::compile_file_with_load_paths(
            &aside_path,
            sasspile::OutputStyle::Expanded,
            load_paths,
        );
        match &result {
            Ok(css) => {
                let unresolved: Vec<&str> = css
                    .lines()
                    .filter(|l| {
                        (l.contains("joinVarName(")
                            || l.contains("getCssVar(")
                            || l.contains("bem("))
                            && !l.contains("///")
                    })
                    .collect();
                if !unresolved.is_empty() {
                    tracing::warn!(
                        count = unresolved.len(),
                        "aside.scss 也有未解析函数！"
                    );
                }
                tracing::info!(total_len = css.len(), output_preview = %css.chars().take(500).collect::<String>(), "aside.scss 编译完成");
            }
            Err(e) => tracing::error!(error = %e, "aside.scss 编译失败"),
        }
    }
}
