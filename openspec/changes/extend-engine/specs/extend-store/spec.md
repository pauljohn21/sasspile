## ADDED Requirements

### Requirement: ExtensionStore 集中管理 @extend 请求

系统 SHALL 引入 `ExtensionStore` 结构体，集中收集、索引和查询所有 `@extend` 请求，替代当前散落的 `Vec<ExtendEntry>`。

#### Scenario: @extend 请求被收集到 ExtensionStore

- **WHEN** eval 层遇到 `@extend .foo` 或 `@extend %placeholder` 指令
- **THEN** 系统 MUST 将此请求添加到 `ExtensionStore` 的 `extensions` map 中
- **AND** key 为 extendee 选择器的规范形式，value 为 `Extension` 结构体列表
- **AND** 每条 `Extension` MUST 携带 extender `SelectorList`、extendee `SelectorList`、`module_id` 和 `optional` 标志

#### Scenario: eval_use_rule 传递 ExtensionStore 引用

- **WHEN** eval 层执行 `@use "module"` 指令
- **THEN** 系统 MUST 将当前 `ExtensionStore` 的可变引用传递给子模块的 `eval_stmts`
- **AND** 子模块内部的 `@extend` 请求 MUST 被收集到同一个 `ExtensionStore` 中
- **AND** 系统 MUST NOT 传递 `&mut Vec::new()` 丢弃子模块的 extends

#### Scenario: serialize 层不做选择器匹配

- **WHEN** serialize 层序列化 `CssTree`
- **THEN** 系统 MUST 在 eval 层完成所有 extend 应用
- **AND** `CssTree.extends` MUST 为空（extends 已应用）
- **AND** serialize 层 MUST NOT 包含 `apply_extends` 或 `apply_extends_to_rule` 函数

#### Scenario: 按 extendee 查询 extender

- **WHEN** 系统对一条 CSS 规则的选择器应用 extends
- **THEN** `ExtensionStore` MUST 能按 extendee 选择器字符串查找所有匹配的 `Extension`
- **AND** 查询结果 MUST 包含传递性产生的 extensions（见 extend-transitivity）
