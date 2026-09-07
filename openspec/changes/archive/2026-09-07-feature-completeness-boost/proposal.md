## Why

sasspile 的 sass-spec 通过率目前为 54.3%（6427/11824），还有 5397 个失败 case 分布在多个功能领域。经过对 trace 日志的系统性分析，失败集中在：颜色函数（3194 fail）、values 解析（644 fail）、CSS 规范用法（374 fail）、模块系统高级特性（57 fail）、指令边界 case（45 fail）等。核心测试虽 100% 通过，但功能完整性距离"可用的 SCSS 编译器"目标仍有显著差距。本变更方案按失败量和影响从重到轻，分六个阶段系统性补齐所有功能缺失。

## What Changes

按优先级从重到轻分阶段推进：

- **Phase 0**: 合并已修复的 module-dispatch bug（fix-module-dispatch），立即见效 +30~80 case
- **Phase 1**: 补齐 Core Functions 非颜色模块——meta 反射函数（get-function, get-mixin, module-variables, module-functions, module-mixins, load-css, apply, keywords 等）、list/map/string/selector/math 边界 case
- **Phase 2**: 颜色函数深度修复——hsl/hwb/lab/lch/oklab/oklch/mix/invert/change/scale 等低通过率子模块，目标将从 17%~31% 提升至 50%~60%+
- **Phase 3**: Values + CSS spec 合规——values 解析序列化边界、CSS 原生函数/变量高级用法，目标从 46%~52% 提升至 70%+
- **Phase 4**: 模块系统高级特性——@use 命名空间冲突处理、@forward show/hide/prefix/config 边界、@import 歧义检测
- **Phase 5**: Directives + Operators + 错误消息——@function 边界、operators 类型转换、@extend 边缘 case、错误消息格式对标

预估总体目标：从 6427/11824 (54.3%) → 7500+/11824 (63%+)，核心测试维持 202/202 全通过。

## Capabilities

### New Capabilities

- `meta-reflection`: meta 反射内建函数族——get-function, get-mixin, module-variables, module-functions, module-mixins, load-css, apply, keywords, feature-exists, content-exists
- `module-advanced`: @use/@forward 高级特性——命名空间冲突处理、show/hide member filter、prefix 拼接、with() config 动态加载、@import 歧义检测增强
- `color-edge-cases`: CSS Color Level 4 边界行为——hsl/hwb 序列化精度、gamut mapping 边界、mix 算法精度、color() 函数现代空间解析

### Modified Capabilities

- `values-parsing`: values 模块需完善 CSS 数字/字符串/列表/map 解析、插值边界、特殊值（null、true、false、空列表）序列化表达式
- `css-compat`: CSS 原生函数和 @规则的行为兼容性——@supports 查询语法、@media 嵌套、自定义属性（var/--*）高级用法、CSS 函数（calc/min/max/clamp 嵌套）
- `directives-robust`: @for/@each/@while/@if 控制流的边界行为——空列表、单值、步长兼容性、变量作用域边界
- `operators-semantics`: 运算符类型转换和运算符优先级——字符串拼接、数值比较、布尔逻辑、null 传播
- `error-format`: 错误消息格式需对标 sass-spec 规范——文件位置、错误类型描述、建议修复

## Impact

- **受影响代码**：`src/eval/builtin/` 全域、`src/eval/meta_ops.rs`、`src/eval/module.rs`、`src/eval/forward.rs`、`src/eval/import.rs`、`src/parse/at_rules_modules.rs`、`src/eval/value/`、`src/css/` 等
- **测试预期**：核心测试 202/202 维持不回归；sass-spec 全量 +1000~1500 case
- **风险**：中低——所有改动遵循现有架构模式（函数式 + move 语义），仅扩展行为不改变 API
- **依赖**：无新增依赖，纯 Rust 标准库 + 现有 crate
- **参考**：已归档变更 `fix-module-dispatch`（Phase 0 可直接复用）
