## ADDED Requirements

### Requirement: EP 模式下声明排序
系统 SHALL 在 EP-style 输出中按照 EP dist 文件中的属性出现顺序输出声明。

#### Scenario: Vendor prefix 属性位置
- **WHEN** EP dist 将 `-webkit-appearance: none` 放在 `vertical-align: middle` 之后
- **THEN** sasspile 输出保持相同相对位置

#### Scenario: CSS custom property 值格式
- **WHEN** 输出 `--el-color-primary: #409eff` 类变量
- **THEN** 值格式与 EP dist 一致（空格、大小写）

### Scenario: 空白字符规范化
- **WHEN** EP dist 在属性值前使用单空格
- **THEN** sasspile 统一使用单空格分隔

### Requirement: 不破坏 sass-spec 格式
系统 SHALL 不在 sass-spec format 测试中引入差异。格式化调整仅限 EP-style 模式。

#### Scenario: sass-spec expanded mode
- **WHEN** 输出 expanded 格式且非 EP 路径
- **THEN** 保持现有格式化行为（不应用 EP-specific reordering）
