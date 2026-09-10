## ADDED Requirements

### Requirement: 函数级趋势查询
系统 SHALL 查询指定函数在不同时间点的通过率变化。

#### Scenario: 查询函数历史
- **WHEN** 运行 `spec_store trend --function math.sin`
- **THEN** 系统返回 math.sin 在所有 snapshot 中的 pass/fail 记录，按时间排序

#### Scenario: ASCII 趋势图
- **WHEN** 运行 `spec_store trend --function math.sin --chart`
- **THEN** 系统在终端输出 ASCII 折线图，显示通过率随时间变化

#### Scenario: 目录级趋势
- **WHEN** 运行 `spec_store trend --dir core_functions/math`
- **THEN** 系统聚合整个 math 子目录的通过率趋势

### Requirement: 时间范围过滤
系统 SHALL 支持按时间范围缩小查询。

#### Scenario: 最近 N 天
- **WHEN** 运行 `spec_store trend --function math.sin --since 30d`
- **THEN** 仅返回 30 天内的 snapshot 数据

#### Scenario: Commit 区间
- **WHEN** 运行 `spec_store trend --function math.sin --from <sha> --to <sha>`
- **THEN** 仅返回两个 commit 之间的 snapshot 数据
