# Selector Simplification — Proposal

## 动机

`sass-spec` 基线 7698/12131 (63.4%) 中，唯一一个 `directives/use/extend/diamond/merge` 因选择器简化缺失而失败。

该 case 测试菱形依赖（diamond dependency）中的 @extend 合并行为——两个子模块分别扩展同一个 placeholder，最终产物应消除冗余选择器。

## 目标

- 实现 compound 内 simple 去重（`.a.a` → `.a`）
- 实现 selector list 内 superselector 消除
- 不引入任何测试回退
- sass-spec 通过率 7698 → 7699 (+1)

## 非目标

- 跨文件 placeholder extend 完整支持（需架构改造，另行处理）
- specificity 精确计算（用现有的 `is_superselector` 近似）

## 影响范围

- 仅 CSS 序列化阶段（不影响 eval、parse、lex）
- 仅影响含重复 compound 的极端场景（大多数 selector 无变化）

## 计划

1. 实现 `selector_simplify.rs` 模块
2. 添加到 `Serializer::serialize` 管线
3. 修复 `extend.rs` 中的单文件 placeholder 检测
4. 跑全量验证
5. 更新 CHANGELOG 和归档
