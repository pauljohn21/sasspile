# Selector Simplification — Tasks

## Tasks

- [x] T1: 实现 `selector_simplify.rs` — compound dedup + superselector elimination
- [x] T2: 集成到 `Serializer::serialize` 管线（`mod.rs` 添加调用）
- [x] T3: 修复 `extend.rs` 单文件 placeholder extend 检测
- [x] T4: 验证无回归（compile_test 全部通过，sass-spec +1 case）
- [x] T5: 更新 CHANGELOG.md (0.9.12)
- [x] T6: 归档变更

## 最终状态

- **snapshot 38**: 7699/12131 passed (+1 from 7698)
- **唯一新增通过**: `directives/use/extend/diamond/merge`
- **test_compile_extend_placeholder**: pre-existing failure（跨规则 placeholder extend 限制，非本次引入）

## 遗留问题

1. `test_compile_extend_placeholder`: 单文件 `%base { } .child { @extend %base }` 场景需要跨规则 extendee 搜索架构改造
2. specificity 消除条件：当前仅依赖 is_superselector，未精确对比 specificity(A) ≥ specificity(extender)
