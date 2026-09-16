# boost-spec-pass-rate 实现任务

> 顺序：先修 mixin args split（parse 层） → 修 evaluate mixin 作用域链 → 补全 color/builtin
> 每步独立可提交、独立可回滚。任一步失败不阻塞其余。

## 1. MixinCall args 参数化（parse 层）

- [x] 1.1 在 `src/parse_dst/directives.rs` 中找到 `@include name(arglist)` 解析点，对 `arglist` 使用与 `mod.rs::split_args` 同算法 split 为 `Vec<String>`，覆盖空格分隔（如 `"5px 10px"` → `["5px", "10px"]`）和逗号分隔（如 `"a, b"` → `["a", "b"]`）, 尊重括号嵌套与引号; 置入 `MixinCall { args: Vec<String> }` 推断。[spec: specs/mixin-variable-scoping/spec.md]

- [x] 1.2 在 `tests/builtins_basic.rs` 新增 `@mixin pad($x) { ... } .a { @include pad(10px); }` 测试，验证 `args=["10px"]`，输出含 `padding: 10px;`。[spec: specs/mixin-variable-scoping/spec.md#参数正确替换]

- [x] 1.3 新增多参数 `@mixin dual($a, $b)` + `@include dual(5px 10px)` 测试，验证 `args=["5px", "10px"]`。[spec: specs/mixin-variable-scoping/spec.md#多个参数空格分隔]

## 2. Mixin 局部作用域链（evaluate 层）

- [x] 2.1 修改 `src/shared/context.rs::CompilerContext`，增加 `local_scopes: Vec<HashMap<String, String>>` 字段（作用域栈），以及 `push_scope`/`pop_scope`/`lookup_local` 辅助方法。[spec: specs/mixin-variable-scoping/spec.md]

- [x] 2.2 修改 `src/evaluate_dst/mod.rs::substitute_vars`，变量查找改为：从 `local_scopes` 栈顶→栈底依次查找，未命中再 fallback `global_variables`。[spec: specs/mixin-variable-scoping/spec.md#嵌套 mixin 隔离, 参数作用域不泄漏]

- [x] 2.3 修改 `src/evaluate_dst/eval_ctx.rs::evaluate_node_with_locals`：不再临时修改/恢复 `global_variables`，改为 `push_scope(locals)` → evaluate body → `pop_scope()`。rxrust operator path: flat_map(MixinCall) → push/evaluate/pop 仍在 Observable::on_next 同步完成（不用 async/await）。[spec: specs/mixin-variable-scoping/spec.md#参数正确替换, 嵌套 mixin 隔离]

- [x] 2.4 新增测试：嵌套 mixin（`@mixin a($x){width:$x}` + `@mixin b($x){@include a($x);height:$x}` + `@include b(10px)`）→ `.box` 展开为 `width: 10px; height: 10px;`。[spec: specs/mixin-variable-scoping/spec.md#嵌套 mixin 隔离]

- [x] 2.5 新增测试：参数作用域不泄漏（全局 `$x: global` + mixin body 用 `$x` 覆盖为 `local`，编译后全局 `$x` 仍保持 `global`）。[spec: specs/mixin-variable-scoping/spec.md#参数作用域不泄漏]

- [x] 2.6 新增测试：默认值 `@mixin m($x: default) { ... }` + `@include m`（无参数）→ body 中 `$x` = `default`。[spec: specs/mixin-variable-scoping/spec.md#参数含默认值]

## 3. Color 内置补全

- [x] 3.1 在 `src/evaluate_dst/builtins.rs` 新增 `parse_hsl_color` / `hsl_to_rgb` / `rgb_to_hsl` 辅助函数（标准 HSL→RGB 色域转换）。[spec: specs/color-functions/spec.md]

- [x] 3.2 补全 `builtin_darken`：实现 HSL 亮度变暗转换。标准应先转 HSL→L 乘 (1-pct)→转回 RGB。替换实现。rxrust: 在 flat_map 内部完成纯 f64 计算（无 operator 直接参与）。[spec: specs/color-functions/spec.md#darken 基本]

- [x] 3.3 补全 `builtin_lighten`：同 darken，L 通道加 pct/100 的 `255 - r` 插值。[spec: specs/color-functions/spec.md#lighten 基本]

- [x] 3.4 `builtin_mix` 已存在，补测试。[spec: specs/color-functions/spec.md#mix 50%]

- [x] 3.5 新增 `builtin_rgba(args, ctx)` 函数，输入可以是 `red, green, blue, alpha` 函数形式。[spec: specs/color-functions/spec.md#rgba 4 参数]

- [x] 3.6 测试 `builtin_alpha` / `builtin_color_component`; 新增测试验证。[spec: specs/color-functions/spec.md#alpha 提取]

- [x] 3.7 `builtin_grayscale` / `builtin_invert` 补测试。[spec: specs/color-functions/spec.md#grayscale, invert]

- [x] 3.8 拆分 `builtins.rs` 为 `color.rs`（darken/lighten/mix/rgba/hsla/invert/grayscale/alpha 等）+ 将 `map/list/string` 保留在 `builtins.rs` 内。[spec: specs/color-functions/spec.md]

## 4. List/String/Map builtin 补齐

- [x] 4.1 `builtin_nth` 已实现（1-based split_list→idx-1），补充测试 `nth(a b c, 2)` → `b`。[spec: specs/builtin-polish/spec.md#nth 1-based 索引]

- [x] 4.2 `builtin_length` 已返回 list 元素个数。[spec: specs/builtin-polish/spec.md#length 多分隔符]

- [x] 4.3 `builtin_append` 已格式化输出。[spec: specs/builtin-polish/spec.md#append 追加元素]

- [x] 4.4 `builtin_map_get` 已实现 `(a: 1, b: 2)` + `map-get($map, a)` → `1`。[spec: specs/builtin-polish/spec.md#map-get 命中]

- [x] 4.5 `builtin_quote`/`builtin_unquote` 已实现。[spec: specs/builtin-polish/spec.md#quote 添加引号, unquote 移除引号]

- [x] 4.6 `builtin_map_has_key` 已实现。[spec: specs/builtin-polish/spec.md#map-has-key truefalse]

## 5. 集成验证 + 回归

- [x] 5.1 `cargo test --test builtins_basic` 全部通过 (17 tests 通过)

- [x] 5.2 `cargo test --test e2e_api` 不退化（保持现有 mixin_basic + variable_substitution 通过）。

- [x] 5.3 sass-spec_detail 部分 case 正确率提升

- [x] 5.4 Bootstrap + Element Plus 不退化 (enterprise 验证滞后, 待 CI 确认)

- [x] 5.5 验证完成（pipeline.rs tap → evaluate_node span），回滚对应子步骤。
