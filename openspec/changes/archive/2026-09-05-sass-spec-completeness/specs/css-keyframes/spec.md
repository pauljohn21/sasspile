## ADDED Requirements

### Requirement: @keyframes 解析
MUST 解析 `@keyframes name { ... }` 规则，支持关键帧选择器 `0%`、`50%`、`100%`、`from`、`to`。

#### Scenario: 基本 keyframes
- **WHEN** 输入包含 `@keyframes fade { 0% { opacity: 0 } 100% { opacity: 1 } }`
- **THEN** 输出保留完整 keyframes 结构（可选择压缩格式）

#### Scenario: from/to 关键字
- **WHEN** 输入使用 `@keyframes fade { from { opacity: 0 } to { opacity: 1 } }`
- **THEN** 正确解析 from/to 为 0%/100%

### Requirement: Vendor Prefix keyframes
MUST 支持 `-webkit-keyframes`、`-moz-keyframes`、`-o-keyframes`、`-ms-keyframes`。

#### Scenario: webkit 前缀
- **WHEN** 输入包含 `@-webkit-keyframes slide { ... }`
- **THEN** 正确解析并保留前缀

### Requirement: @keyframes 提升
MUST 将 @keyframes 提升到文档根级别（不从属于父选择器）。

#### Scenario: 嵌套 keyframes
- **WHEN** keyframes 定义在 `.class { @keyframes x { ... } }` 内部
- **THEN** 输出时 keyframes 在根级别
