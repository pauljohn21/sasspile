## Why

sasspile 的 sass-spec 全量 38,393 个 case 中，2535 个失败（93.4% 通过）。失败集中在两大瓶颈：

1. **选择器代数运算**（~560 fail）— `selector-unify`/`is-superselector`/`selector-extend`/`@extend` 全部基于 `String::contains`/`String::replace`，选择器不是字符串而是代数结构
2. **calc() 表达式简化**（~583 fail）— 当前 `simplify_calc` 做字符串模式匹配，不支持单位兼容性检查（如 `deg` vs `rad` 转换）、运算符优先级、CSS 函数内嵌、错误检测等

两大瓶颈合计 ~1143 个失败 = scss 失败的 50%。修复后整体通过率预期从 93.4% → 96.4%。

## What Changes

### 选择器 AST

- 新增 `Selector` AST 类型层级：`Selector` → `ComplexSelector` → `CompoundSelector` → `SimpleSelector`
- 新增选择器解析器 + 序列化器
- 实现 `unify_compound` / `unify_complex` / `unify`：复合选择器交集运算
- 实现 `is_super_compound` / `is_super_complex` / `is_superselector`：超选择器判断
- 重写 `selector-unify` / `selector-extend` / `selector-replace` / `is-superselector`
- 重写 `@extend` 指令（`eval/extend.rs`）

### calc() 简化增强

- 新增 CSS 单位兼容性表（`deg`/`rad`/`turn`、`px`/`cm`/`in`、`s`/`ms`/`Hz`/`kHz` 等）
- 实现 calc 表达式 AST 解析器（支持运算符优先级、嵌套括号、CSS 函数）
- 实现单位兼容性检查 + 自动转换（`deg + rad` → 换算后简化）
- 实现不兼容单位错误检测（`deg + s` → `Error: incompatible`）
- 实现运算符优先级正确简化（`a * b + c` ≠ `a * (b + c)`）
- 支持嵌套 calc/min/max/clamp 递归简化
- 支持 `calc-size()`、`round()`、`rem()`、`mod()`、`abs()`、`sign()`、`exp()`、`pow()`、`sqrt()`、`log()`、`sin()`/`cos()`/`tan()`/`asin()`/`acos()`/`atan()`/`atan2()`/`hypot()` 等 CSS 数学函数
- 保持 `CssNode::Rule.selector` 字段为 `String`，不改变管线数据流

## Capabilities

### New Capabilities
- `selector-ast`: 选择器 AST 类型定义、解析器、序列化器、unify/is_superselector/extend/replace 算法
- `calc-simplification`: calc() 表达式 AST 解析、单位兼容性检查、运算符优先级、CSS 数学函数简化

### Modified Capabilities
（无现有 spec 需要修改）

## Impact

- **新增文件**：
  - `src/css/selector_ast.rs`（选择器 AST 类型 + parser + display）
  - `src/css/selector_ops.rs`（unify + is_superselector + extend + replace 算法）
  - `src/eval/value/calc_ast.rs`（calc 表达式 AST + parser）
  - `src/eval/value/calc_units.rs`（单位兼容性表 + 转换）
- **修改文件**：
  - `src/eval/builtin/selector.rs` — 重写 unify/extend/replace/is-superselector
  - `src/eval/extend.rs` — 重写 `@extend` 后处理
  - `src/eval/value/calc.rs` — 重写 simplify_calc 使用新 AST
  - `src/css/mod.rs` — 引用新模块
  - `src/eval/value/mod.rs` — 引用 calc_ast 模块
- **CssNode 不变**：`Rule.selector` 保持 `String`
- **sass-spec 预期提升**：
  - 选择器：unify +183、extend +159、is_superselector +98、directives/extend +16
  - calc：calc/operator +24、calc/no_operator +23、calc/constant +16、calc/simplify +9、calc/error/known_incompatible +218、calc/error 其他 +53、其余 calculation 子函数 +240
- **核心测试无回归**：202/202 保持
