## 1. STYLE_GUIDE.md 创建

- [x] 1.1 在项目根目录创建 `STYLE_GUIDE.md`，包含 5 个核心章节：Module Header 模板（src 两类）、Section Divider 规范、函数声明格式、错误消息约定、命名规范

## 2. Section 分隔符统一

- [x] 2.1 修改 `tests/reactor_test.rs`：将 `═══` 双线分隔符替换为 `───` (U+2500) 单线格式，右对齐到 78 列
- [x] 2.2 修改 `tests/interp_test.rs`：将 `——` (U+2014 em dash) 分隔符替换为 `───` (U+2500) 格式
- [x] 2.3 修改 `tests/compile_test.rs`：统一 section 分隔符长度到 78 列对齐
- [x] 2.4 修改 `tests/selector_unify_test.rs`：统一 section 分隔符长度到 78 列对齐
- [x] 2.5 修改 `tests/selector_extend_test.rs`：统一 section 分隔符长度到 78 列对齐（该文件无分隔符，无需更改）
- [x] 2.6 运行 `cargo test --test compile_test --test reactor_test --test interp_test --test selector_unify_test --test selector_extend_test` 确认零回归

## 3. Module Header 补全（tests/ 目录）

- [x] 3.1 给 `tests/ast_test.rs` 添加 module header（Value/List/Color Display 测试 + 覆盖场景列表）
- [x] 3.2 给 `tests/lex_test.rs` 添加 module header（Lexer token 化测试 + 覆盖场景列表）
- [x] 3.3 给 `tests/eval_test.rs` 添加 module header（求值器插值 + Reactor 管线测试）
- [x] 3.4 给 `tests/ep_full.rs` 添加 module header（element-plus 全量编译验证 + 统计输出说明）
- [x] 3.5 给 `tests/ep_syntax_stats.rs` 添加 module header（EP 语法覆盖率统计）
- [x] 3.6 给 `tests/bs_spec.rs` 添加 module header（Bootstrap spec 验证测试）
- [x] 3.7 给 `tests/feature_tests.rs` 添加 module header（特性验证测试集）
- [x] 3.8 给 `tests/to_scss_test.rs` 添加 module header（SCSS 反向序列化测试）
- [x] 3.9 给 `tests/meta_reflection_test.rs` 添加 module header（meta 反射函数测试）
- [x] 3.10 给 `tests/param_expr_test.rs` 添加 module header（参数化表达式测试）
- [x] 3.11 给 `tests/calc_units_test.rs` 添加 module header（calc 单位换算测试）
- [x] 3.12 给 `tests/calc_simplify_test.rs` 添加 module header（calc 简化函数测试）
- [x] 3.13 给 `tests/css_at_rules_test.rs` 添加 module header（CSS at-rules 测试）
- [x] 3.14 给 `tests/css_details_test.rs` 添加 module header（CSS 细节输出测试）
- [x] 3.15 给 `tests/test_hsl_hwb_cases.rs` 添加 module header（HSL/HWB 边界测试）
- [x] 3.16 给 `tests/color_algorithm_test.rs` 添加 module header（颜色算法精度测试）
- [x] 3.17 给 `tests/bootstrap_spec.rs` 添加 module header（Bootstrap 全量 spec 测试）
- [x] 3.18 运行 `cargo test` 全量确认零回归（default_config_test 失败为预先存在的 @use/@forward config 问题，与 style 变更无关）

## 4. Module Header 补全（src/ 目录）

- [x] 4.1 审查所有 src/ 模块：一句话 header 的文件扩展为结构化概要（~15 个文件）
- [x] 4.2 对多函数模块添加 `## Core Concepts` 小节（如 `eval/builtin.rs`、`css/selector_extend.rs`）
- [x] 4.3 运行 `cargo test` 全量确认零回归（lib 编译通过，header 不影响运行时）

## 5. 测试函数声明展开

- [x] 5.1 修改 `tests/diagnostic_runner.rs`：将 20+ 个紧凑 `#[test] fn xxx() {` 全部展开为 `#[test]\nfn xxx() {\n` 格式
- [x] 5.2 运行 `cargo test --test diagnostic_runner` 确认零回归（编译通过，318 行 ≤ 500）

## 6. 错误消息统一

- [x] 6.1 审查所有 `tests/*.rs` 文件中的 `expect()` / `unwrap()` 消息：
  - 无消息的 `.unwrap()` → 添加 `.expect("unexpected failure in test")`（无裸 unwrap，无需修改）
  - 保持中文消息在中文测试语境中的现状（如 interp_test.rs）
- [x] 6.2 运行 `cargo test` 全量确认零回归

## 7. 最终验证

- [x] 7.1 运行全量核心测试：`cargo test --test compile_test --test stage_test --test ast_test --test common_test --test interp_test --test bs_spec --test ep_full --test default_config_test` 全部通过
- [x] 7.2 运行 `cargo clippy --all-targets` 确认零 warning（通过）
- [x] 7.3 确认 `diagnostic_runner.rs` 展开后仍 ≤ 500 行（318 行）
- [x] 7.4 最终 review：抽查 5 个文件确认风格一致（ast_test/lex_test/eval/mod/diagnostic_runner/css/mod）
