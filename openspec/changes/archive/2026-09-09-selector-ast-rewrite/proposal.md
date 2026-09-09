## Why

选择器模块当前通过率仅 45%（404/893），535 个 spec 失败，是 sass-spec 失败数第二大的热点区域。根因是数据模型三处缺陷：(1) `SimpleSelector::Type(String)` 将命名空间前缀与类型名混为不透明字符串，无法执行命名空间感知的统一；(2) 伪元素 `:before`/`::before` 二义性未归一化；(3) 伪类链式（`:c` + `:d` → `:c:d`）不支持。这些问题互相耦合——修复命名空间模型会暴露 extend NO-OP 检测的命名空间盲点，必须一次性全量重写而非渐进修补，否则每次数据模型变更都会引发新一轮回归。

## What Changes

- **数据模型**：新增 `Namespace` 枚举（`None`/`Empty`/`Any`/`Explicit(String)`），将 `SimpleSelector::Type(String)` 改造为 `Type { namespace: Namespace, name: String }`
- **解析器**：`take_type_with_ns` 输出结构化 `Namespace` + `name`，替代当前字符串拼接
- **统一算法**：`unify_compound` 实现命名空间三规则矩阵 + 伪类链式合并 + 伪元素归一化比较
- **超选择器判断**：`is_super_compound` 伪元素必须精确匹配（归一化后）
- **扩展算法**：`is_more_specific_than` 和 `compounds_conflict` 增加命名空间兼容性判断
- **序列化**：`Display for SimpleSelector::Type` 输出 `ns|name` 格式

**BREAKING**：无。`SimpleSelector` 是内部类型，不跨越 crate 边界。外部 API（`selector.unify()`/`selector.extend()` 等）签名不变。

## Capabilities

### New Capabilities

- `selector-namespace`: 命名空间感知的选择器解析、统一、扩展、超选择器判断
- `selector-pseudo-normalization`: 伪元素单/双冒号语法归一化 + 伪类链式合并

### Modified Capabilities

- `selector-unify`: 统一算法支持命名空间规则矩阵、伪类链式、伪元素归一化
- `selector-extend`: NO-OP 检测增加命名空间兼容性判断 + 伪元素冲突检测
- `selector-is-superselector`: 伪元素必须精确匹配（归一化后比较）

## Impact

**改动文件**（5 文件，~600~800 行 diff）：

- `src/css/selector_ast.rs` — `Namespace` 枚举 + `SimpleSelector::Type` 改造 + `Display`
- `src/css/selector_parser.rs` — `take_type_with_ns` 解析命名空间
- `src/css/selector_ops.rs` — `unify_compound`/`is_super_compound`/`extend_complex`/`compounds_conflict` 全量重写
- `src/css/selector_format.rs` — `fmt_to_vec_vec` 适配新 Type
- `src/eval/builtin/selector.rs` — 适配新 Type 定义

**测试影响**：`sass_spec_full` 中 535 个 selector case，预估 +250~350 passes（45% → 73~84%）。全局通过率 62% → 64~65%。

**无外部依赖**——选择器模块是封闭的，修改不波及 eval/css 其他部分。
