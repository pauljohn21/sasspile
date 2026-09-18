## Why

`sasspile-rx` 当前通过率 0.3%(27/10274)是因为评估层(`eval_node_vec`)是 identity 函数。但企业级验收不应先对齐 dart-sass,而应直接对齐企业应用的**真实编译产物**——浏览器跑过的 CSS 才是最终契约。

本 change 建立"**编译产物驱动开发**"(Compile-Product-Driven Development, CDDD):
1. 基线应用:**Element Plus**(`@use`/`@forward` 现代语义,更简洁)
2. 第二目标:**Bootstrap**(`@import` 老式全局共享,作用域混乱度高)

## What Changes

### 核心哲学
- **对照契约**:`element-plus/packages/theme-chalk/src/index.scss` 输入 vs 企业真实输出比对
- **不照搬 dart-sass 内部模型**:所有权策略由实现者自由选择,只保证输出一致
- **跳过 sass-spec 优先**:spec 用例仅作辅助覆盖,不作为守门 gate

### 改造点
- **`src/pipeline.rs`**: 不变(已有单一 chain + 末尾 box_it)
- **`src/parse_dst/mod.rs`**: 增 `@use` / `@forward` / `@include` / `@mixin` / `@each` / `@for` / `@if` 状态机分支,产出 `Node::Use/Forward/MixinDef/MixinCall/If/For`
- **`src/evaluate_dst/mod.rs`**: 增 `eval_scope_forwarding` —— scan_map 链内 scope 状态转移,无全局堆
- **`src/shared/mod.rs`**: define `Scope { vars: HashMap, mixins: HashMap, forwarded: HashSet }` 结构( scan_map 的 Acc,非 Rc<RefCell>)
- **`tests/enterprise_element_plus.rs`**: fixture —— 编译 `index.scss` 所有 @use 链, ignore-accepted until P0 ready
- **`tests/enterprise_bootstrap.rs`**: fixture —— 编译 `bootstrap.scss` 完整链(Phase 2)

### 移除依赖点
- 删除 `openspec/changes/pass-core-functions`(spec-first 路径被本 change 替代)

## Capabilities

### New Capabilities
- `use-forward-resolution`: `@use "./foo" as *` 等价于全局注入所有 public symbols;`@forward "./bar"` 让下游可访问上游;`with ($var: value)` 参数覆盖
- `scope-global-semantics`: `$var !global` 在 `@use` 链内穿透作用域边界 —— 编译产物体现为 CSS 变量值唯一确定
- `each-for-control`: `@each $x in list` / `@for $i from 1 through N` / `@while cond` 展开循环体,产出多个 Rule/Declaration
- `mixin-include-apply`: `@mixin name($args){...}` 定义 block;`@include name($args)` 在 reducer 内展开;可选默认值 `$x: null`
- `function-call-eval`: `@function name($args){ @return expr; }` 自定义函数调用,在 reducer 内纯函数求值
- `variable-declaration-default`: `$var: value !default;` —— 只在未定义时赋值

### Modified Capabilities
(无现有 capability 需要修改)

## Impact

- **受影响文件**:`src/parse_dst/mod.rs`(增语言特性分支),`src/evaluate_dst/mod.rs`(增 scope 状态转移),`src/shared/mod.rs`(新增 Scope struct),`tests/enterprise_*.rs`(新增 fixture)
- **输入契约锚点**:
  - Element Plus:`element-plus/packages/theme-chalk/src/index.scss`(100+ 组件通过 @use 注册)
  - Bootstrap:`bootstrap/scss/bootstrap.scss`(@import global chain)
- **对照真值**:企业级的 CSS 产物(运行时生成,不作为 repo 内 submodule)
- **风险**:中。Element Plus 的 `@use` 语义比 Bootstrap 的 `@import` 更干净(无全局污染),先通过可控
