# 修复 @each 无法迭代 ArgList

## Why（为什么）

element-plus 全量测试 119/121（98.3%），仅差 2 个文件。

`@each $item in $list` 在 `$list` 为 `ArgList` 类型时无法正确迭代，
导致 `joinVarName($args)` 失败，影响所有使用 `getCssVarName` 的 component SCSS。

## What Changes（变更内容）

- **BREAKING**: 否
- **影响范围**: `eval_each` 的行为变更 — `ArgList` 现在与 `List` 一致被迭代

## 受影响用户

- 使用 rest-param (`$args...`) 并在 `@each` 中迭代的 SCSS 代码
- 所有 element-plus 编译（修复后 +2 files）
