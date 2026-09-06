## ADDED Requirements

### Requirement: src 目录单文件上限 500 行
所有 `src/**/*.rs` 文件的行数 SHALL ≤ 500 行（不含空行和纯注释行）。超限文件 MUST 在 CI 测试中明确报告超限行数。

#### Scenario: 正常运行时通过
- **WHEN** 所有 src/**/*.rs 文件都 ≤500 行
- **THEN** `cargo test --test file_size_check check_file_size_limits` SHALL 通过

#### Scenario: 文件超限时测试失败
- **WHEN** foo.rs 有 636 行
- **THEN** 测试 SHALL 失败并输出：`FAIL: foo.rs: 636 行，超出 136 行（上限 500）`

#### Scenario: 文件等于 500 行
- **WHEN** bar.rs 恰好 500 行
- **THEN** 测试 SHALL 通过（边界值包含）

### Requirement: 超限文件按功能组拆分
4 个超限文件 SHALL 按 logical concern 拆分为子模块，每个子模块 ≤300 行。

#### Scenario: display.rs 拆分
- **WHEN** 拆分 `parse/ast/display.rs`
- **THEN** SHALL 生成 `parse/ast/color_display.rs` 承载 Color/ColorSpace 序列化逻辑
- **AND** 原 `display.rs` SHALL ≤ 250 行

#### Scenario: color.rs 拆分
- **WHEN** 拆分 `eval/builtin/color.rs`
- **THEN** SHALL 生成 `color_channels.rs`、`color_adjust.rs`、`color_hsl_hwb.rs` 三个子模块
- **AND** 原 `color.rs` SHALL ≤ 200 行（仅保留入口路由）

#### Scenario: css/mod.rs 拆分
- **WHEN** 拆分 `css/mod.rs`
- **THEN** SHALL 生成 `css/merge.rs` 承载 `merge_at_rules` 折叠逻辑
- **AND** `mod.rs` SHALL ≤ 400 行

### Requirement: 公开 API 不变
文件拆分 SHALL 保持所有现有 `pub` 函数路径不变（通过内部 `pub use` 重新导出）。

#### Scenario: 现有 callers 无需修改
- **WHEN** 拆分完成
- **THEN** 现有使用 `use sasspile::Serializer` / `use sasspile::CssNode` 的代码 SHALL 继续编译
- **AND** 现有 `use crate::eval::builtin::color::rgba` 的内部引用 SHALL 继续工作
