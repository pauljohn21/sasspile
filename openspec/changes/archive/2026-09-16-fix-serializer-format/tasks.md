## 1. 核心修复 — CssNode::render 多行格式

- [x] 1.1 修改 `src/ast.rs` `CssNode::render()` Rule 分支，输出 `selector {\n  decl;\n}\n` 格式，每个 decl 前 2 空格缩进

## 2. 冒烟测试

- [x] 2.1 创建 `tests/serialize_format.rs` 验证：single-decl Rule、multi-decl Rule、empty Rule、top-level Declaration、Comment

## 3. 回归检查

- [x] 3.1 跑全量 `cargo test`，检查所有已有 tests/ 下是否因 format 改变而 fail；若有直接断言旧格式的测试，改用 normalize 比较或更新期望值

## 4. 验证 pass rate 增量

- [x] 4.1 执行 `cargo test --test sass_spec_detail -- --nocapture`，确认 `core_functions` 从 0/7793 显著提升
