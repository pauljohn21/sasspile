## Context

sasspile 求值器当前不检测顶层 CSS 声明。`Env` 有 `current_selector: Option<String>` 字段，`eval_rule` 通过 `env.with_selector()` 设置它，但 `Node::Decl` 分支从不检查它。这导致 `@import` 导入的文件中出现的裸声明（如 `a: b;`）和 `@include` 在顶层调用 mixin 产生的 CSS 声明不会被报错。

两个失败的 spec 测试组：
1. `_upstream.scss` 含 `a: b;`（裸声明）→ 预期 error `expected "{"`
2. `_upstream.scss` 含 `@mixin a { b: c }` + `@include a;`（顶层 include 产生声明）→ 预期 error `Declarations may only be used within style rules.`

## Goals / Non-Goals

**Goals:**
- 在求值器层面检测两种顶层声明场景并报错
- 通过 `directives/import/error/top_level_declaration` 的 2 个 spec 测试
- 不影响 plain CSS 模式下的合法顶层声明
- 不影响 `@at-root` 等合法顶层 CSS 输出

**Non-Goals:**
- 不改变解析器层面行为（`in_body` 标志仅用于 @forward 上下文验证）
- 不修改 `top_level_parent.hrx`（`& {a: b}` 在顶层——这是合法的，因为它产生 `CssNode::Rule` 而非裸 `Declaration`）

## Decisions

### 决策 1：在求值器层面检测，而非解析器层面

**选择**：求值器层面（`eval_node` 的 `Node::Decl` 分支）

**理由**：
- 解析器的 `in_body` 标志目前仅用于 @forward 上下文验证，扩展它会增加复杂度
- 求值器已有 `env.current_selector` 可用于判断上下文
- `@include` 产生的声明是在 mixin 执行时动态产生的，解析器无法静态检测
- sass-spec 的错误消息指向具体行号，求值器层面有足够信息

**替代方案**：在解析器 `parse_node` 中检查 `!self.in_body` 并报错 → 拒绝，因为无法检测 @include 动态产生的声明

### 决策 2：裸声明检测放在 `Node::Decl` 分支

**选择**：在 `eval_node` 的 `Node::Decl` 分支中，在 `plain_css` 检查之后、值求值之前，检查 `env.current_selector.is_none()`

**理由**：
- `Node::Decl` 是声明被求值的唯一入口
- `env.current_selector` 在 `eval_rule` 调用 `env.with_selector()` 时设置，在顶层时为 `None`
- plain CSS 模式下顶层声明合法，需跳过

**错误消息**：`expected "{".`（匹配 sass-spec）

### 决策 3：@include 顶层声明检测放在 `eval_include` 返回后

**选择**：在 `eval_include` 返回的 CSS 节点列表中检查是否包含 `CssNode::Declaration`，且 `env.current_selector.is_none()`

**理由**：
- mixin 执行产生 CSS 节点列表，其中可能包含 Declaration
- 需要在调用上下文（而非 mixin 内部）检测，因为 mixin 在规则体内调用时产生 Declaration 是合法的
- `exec_mixin` 内部 `eval_nodes` 的 env 设置了 mixin body 的局部 selector，不能用于判断调用上下文

**错误消息**：`Declarations may only be used within style rules.`（匹配 sass-spec）

### 决策 4：`Node::Decl` 检查中排除 null 值声明

**选择**：先求值，如果是 `Value::Null` 则跳过（不报错也不输出）

**理由**：sass-spec 中 `$var: null;` 在顶层是合法的（变量赋值不是 CSS 声明），且 null 声明不产生 CSS 输出。但注意 `Node::Decl` 是 CSS 声明 `property: value`，不是变量赋值 `Node::Variable`。对于 `a: null;` 这样的声明，在顶层是否报错需要看 spec——目前两个测试组不涉及 null，所以先求值后检查，null 值不报错。

## Risks / Trade-offs

- **[风险] 误报合法顶层声明** → 通过检查 `env.current_selector.is_none() && !env.plain_css` 缩小范围；`@at-root` 产生 `CssNode::AtRoot` 而非 `Declaration`，不受影响
- **[风险] @include 嵌套调用中的误报** → 检测点在 `eval_include` 返回后，仅检查调用上下文的 `current_selector`，不检查 mixin 内部层级
- **[风险] 错误消息精确性** → sass-spec 的错误消息包含行号和文件路径信息，当前实现可能无法完全匹配格式，但不影响通过测试（测试只比较 error 是否存在，不比较 error 内容）
