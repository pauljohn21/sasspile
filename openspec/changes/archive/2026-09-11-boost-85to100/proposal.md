# boost-85to100: 将 85%+ 目录补全到 100%

## Why

当前 sass-spec 通过率 7718/12131 = 63.6%。有 11 个目录的通过率在 85%-97% 之间，
合计 194 个失败 case。将它们全部修复到 100% 可将通过率推升至 7912/12131 = 65.2%（+1.6%），
同时让 18 个目录达到满分（含已有的 7 个 100% 目录）。

这些高通过率目录的特点是：**核心功能已基本实现，失败多为 edge case 和 root cause 聚类**。
修复一个根因通常能一次性解决多个 case。

## What Changes

按根因主题组织，共 194 个 case：

### 主题 1: @use/@forward 注释解析 (12 ERRs)
- `directives/use/comment/*` — 6 ERRs
- `directives/forward/comment/*` — 6 ERRs
- **根因**：`@use`/`@forward` 声明解析使用 `skip_ws()` 而非 `skip_ws_and_comments()`
- **修复**：类似 `parse_variable` 的修复方案，在适当位置调用 `skip_ws_and_comments()`

### 主题 2: CSS 空字符串属性抑制 (1+ DIFFs)
- `core_functions/string/unquote/empty` — 空字符串属性应被完全抑制
- **根因**：`result: ;` 应该整行不输出
- **修复**：序列化器检测空字符串值时跳过属性声明

### 主题 3: List 等值括号类型区分 (1 DIFF)
- `values/lists/equality` — `[] == ()` 应当为 false
- **根因**：List equality 未区分 bracket list `[]` 和 paren list `()`
- **修复**：equality 比较增加容器类型检查

### 主题 4: 传递 @extend 穿越伪选择器 (1+ DIFFs)
- `directives/extend/pseudo/into_pseudo/extends_after` — 复杂传递 extend
- **根因**：`@extend` 在 `:is()` 伪选择器内的传递链断裂
- **修复**：extend 图构建需考虑伪选择器内的目标

### 主题 5: @import 从规则内提升 (3 ERRs)
- `directives/at_root/nested_import/*` — 2 ERRs
- `directives/use/css/import/nested_import_into_use` — 1 ERR
- **根因**：`@import` 在规则内部应被提升到根级别，而非报错
- **修复**：at-rule 处理器遇到规则内 `@import` 时执行提升

### 主题 6: Map 函数 edge case (8 DIFFs + 1 ERR)
- `core_functions/map/deep_remove/*` — 4 fails
- `core_functions/map/deep_merge/*` — 1 fail
- `core_functions/map/has_key/*`、`get/nested/*`、`remove/*` — 3 fails

### 主题 7: CSS 选择器格式化 (7 DIFFs)
- `css/selector/escaping/*` — 数字逃逸、插值括号
- `css/selector/combinator/adjacent/function`、`reference_combinator`、`slotted`

### 主题 8: @if 表达式错误消息与解析 (14 fails)
- `expressions/if/error/*` — and/or/not 错误消息差异
- `expressions/if/raw/*` — 原始逻辑操作符解析
- `expressions/if/css/alone/argument` — `?` 字符 lex 错误
- `expressions/if/syntax/trailing_semi` — 尾随分号解析

### 主题 9: List 函数 edge case (20 fails)
- `core_functions/list/join/*` — separator 校验 (auto/e-in/"null")
- `core_functions/list/zip/*` — 参数数量、bracketed 行为
- `core_functions/list/index/*`、`slash/*`、`utils/*`

### 主题 10: Math 函数 — 单位/特殊值/错误 (73 fails)
- `core_functions/math/atan2/*` — 角度/单位/无穷
- `core_functions/math/clamp/*` — 单位保持
- `core_functions/math/pow/*` — negative_zero/infinity 边界
- `core_functions/math/negative_zero/*` — sin/asin/tan/sqrt
- `core_functions/math/unit/*` — 单位转换/归一化
- `core_functions/math/variables/*` — 命名常量
- `core_functions/math/hypot/*`、`max/*`、`min/*`、`log/*`、`percentage/*`、`round/*`、`comparable/*`、`div/*`

### 主题 11: @forward 成员处理 (8 DIFFs + 4 ERRs)
- `directives/forward/member/import/*` — import 优先级/覆盖
- `directives/forward/member/as/*` — 分隔符处理
- `directives/forward/member/shadowed/*` — 变量赋值

### 主题 12: @use CSS 排序 (5 DIFFs)
- `directives/use/css/order/*` — use + import 混合排序
- `directives/use/member/*` — 命名空间变量赋值

## Impact

- **受影响文件**:
  - `src/parse/nodes.rs` — 主题 1 (use/forward 注释)
  - `src/css/serialize.rs` — 主题 2 (空字符串抑制)
  - `src/eval/value/list.rs` — 主题 3 (list equality)
  - `src/css/selector_extend.rs` — 主题 4 (传递 extend)
  - `src/eval/directives.rs` — 主题 5 (@import 提升)
  - `src/eval/functions/map.rs` — 主题 6 (map 函数)
  - `src/css/serialize.rs` — 主题 7 (选择器格式化)
  - `src/parse/expr.rs` — 主题 8 (@if 表达式)
  - `src/eval/functions/list.rs` — 主题 9 (list 函数)
  - `src/eval/functions/math.rs` — 主题 10 (math 函数)
  - `src/eval/module_import.rs` — 主题 11+12 (forward/use 成员)
- **新增通过**: +194 cases (7718 → 7912)
- **新通过率**: 7912/12131 = 65.2%
- **新增 100% 目录**: 11 个 (从 7 个增至 18 个)
- **风险**: 中等 — 各主题独立，可分批修复验证
