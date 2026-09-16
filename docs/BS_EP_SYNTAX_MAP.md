# Bootstrap + Element Plus SCSS 语法特性清单

> 扫描日期: 2026-09-15
> 扫描范围: `bootstrap/scss/` + `element-plus/packages/theme-chalk/src/`
> 用途: 对照 sass-spec,找出缺失语义,按优先级补齐

---

## 1. @-rules (已用)

| @-rule | sass-spec 对应目录 | 状态 |
|---|---|---|
| `@import` | directives/import | ✅ 已实现 |
| `@use` | directives/use | ⚠️ 已实现(无命名空间,`as *` 直接注入) |
| `@forward` | directives/forward | ⚠️ passthrough(无真正模块转发语义) |
| `@mixin` / `@include` | directives/mixin | ✅ 已实现 |
| `@function` / `@return` | directives/function | ⚠️ 需验证 |
| `@if` / `@else` | directives/if | ✅ 已实现 |
| `@for` | directives/for | ✅ 已实现 |
| `@each` | directives/each | ❌ 未实现 (Boot Phase 4) |
| `@while` | directives/while | ❌ 未实现 (Boot Phase 4) |
| `@media` | directives/media | ⚠️ 需嵌套 selector 支持 |
| `@supports` | directives/supports | ❌ 未实现 |
| `@at-root` | directives/at_root | ❌ 未实现 |
| `@extend` | directives/extend | ❌ 未实现 (Boot Phase 6) |
| `@content` | directives/mixin | ✅ 已实现 |
| `@warn` | directives/warn | ❌ 未实现 |
| `@error` | directives/error | ❌ 未实现 |
| `@debug` | directives/debug | ❌ 未实现 |
| `@keyframes` | css/keyframes | ⚠️ 需测试 |

---

## 2. 内置函数 (已用)

### 2.1 Map 函数
- `map-get` ❌ 未实现 (Boot Phase 3)
- `map-merge` ❌ 未实现
- `map-has-key` ❌ 未实现
- `map-keys` ❌ 未实现
- `map-values` ❌ 未实现

### 2.2 List 函数
- `length` ❌ 未实现
- `nth` ❌ 未实现
- `join` ❌ 未实现
- `append` ❌ 未实现
- `zip` ❌ 未实现
- `index` ❌ 未实现

### 2.3 String 函数
- `unquote` ⚠️ 需验证 (Element Plus 大量使用)
- `quote` ⚠️ 需验证
- `str-length` ❌ 未实现
- `str-index` ❌ 未实现
- `str-slice` ❌ 未实现

### 2.4 Color 函数
- `darken` ❌ 未实现 (Boot Phase 3)
- `lighten` ❌ 未实现 (Boot Phase 3)
- `mix` ❌ 未实现 (Boot Phase 3)
- `rgba` / 颜色组件 `red/green/blue/alpha` ❌ 未实现
- `inver` / `grayscale` / `adjust-hue` ❌ 未实现
- `saturate` / `desaturate` ❌ 未实现
- `opacify` / `transparentize` ❌ 未实现
- `ie-hex-str` ❌ 未实现
- `color-mix` ❌ 未实现

### 2.5 Math 函数
- `abs` ❌ 未实现
- `ceil` / `floor` / `round` ❌ 未实现
- `min` / `max` ❌ 未实现
- `random` ❌ 未实现
- `percentage` ❌ 未实现
- `unit` / `unitless` ❌ 未实现
- `comparable` ❌ 未实现

### 2.6 Meta / Type 函数
- `type-of` ⚠️ 需验证 (Element Plus 用)
- `variable-exists` ⚠️ 需验证 (Element Plus 用)
- `inspect` ⚠️ 需验证
- `get-function` ⚠️ 需验证

### 2.7 Selector 函数
- `selector-nest` ❌ 未实现
- `selector-append` ❌ 未实现
- `selector-extend` ❌ 未实现
- `selector-replace` ❌ 未实现
- `selector-parse` ❌ 未实现
- `is-superselector` ❌ 未实现
- `simple-selectors` ❌ 未实现
- `selector-unify` ❌ 未实现

### 2.8 私有/项目自定义函数
- `getCssVar` ❌ 未实现 (Element Plus CSS 变量封装)
- `getCssVarName` ❌ 未实现
- `v-bind` ❌ 未实现 (Element Plus Vue 集成)

---

## 3. 运算符

| 类别 | 运算符 | 状态 |
|---|---|---|
| 算术 | `+` `-` `*` `/` `%` | ⚠️ `+ -` 已实现, `*` `/` `%` 未实现 (Boot Phase 7) |
| 比较 | `==` `!=` `<` `>` `<=` `>=` | ❌ 未实现 (Boot Phase 7) |
| 逻辑 | `and` `or` `not` | ❌ 未实现 |
| 字符串拼接 | `+` (混合类型) | ⚠️ 部分 |
| 标志 | `!global` `!default` `!optional` | ⚠️ `!global` 需验证 |

---

## 4. 高级语义

| 特性 | 状态 | 说明 |
|---|---|---|
| Placeholder selector `%name` | ❌ 未实现 | Element Plus 大量使用 |
| 嵌套 selector (descendant) | ⚠️ 基础支持 | `@media` 嵌套待补 |
| 变量 interpolation `#{}` | ⚠️ 基础支持 | 属性值 OK,selector 待验证 |
| 父 selector `&` | ⚠️ 基础支持 | `&:hover` OK, `&.class` 待验证 |
| 多行注释 `/* */` | ✅ | 已支持 |
| 单行注释 `//` | ✅ | 刚修复 |

---

## 5. 实现优先级 (对齐 Boot Phase)

### Phase 3 (内建函数) — 阻塞 EP 编译
1. `map-get` / `map-has-key` / `map-merge`
2. `length` / `nth`
3. `darken` / `lighten` / `mix`
4. `if` (Sass `if()` 函数, 非指令)
5. `unquote` / `quote` (Element Plus 依赖)

### Phase 4 (循环)
6. `@each`
7. `@while`

### Phase 5 (嵌套)
8. `@media` 嵌套 selector
9. `@supports` 嵌套

### Phase 6 (扩展)
10. `@extend` + Placeholder

### Phase 7 (运算符)
11. 完整算术 `*` `/` `%`
12. 比较运算符 `==` `!=` `<` `>` `<=` `>=`
13. 逻辑运算符 `and` `or` `not`

### Phase 8 (补充)
14. `@at-root`
15. `@warn` / `@error` / `@debug`
16. 剩余 string/math/meta 函数

---

## 6. 开发策略

**每个特性按以下步骤推进:**

```bash
# 1. 定位 sass-spec 用例
cd sass/spec && grep -l 'lighten' */**/*.hrx 2>/dev/null | head -3

# 2. 写单测 + 跑 (利用 test/sass_spec_detail.rs 的单测模式)
cargo test --test sass_spec_detail <name> -- --nocapture

# 3. 实现语义

# 4. 验证 spec case 通过

# 5. commit (等用户确认推送)
```

**禁止项:**
- ❌ 在 `src/` 写 `#[test]`
- ❌ 用 `println!` / `eprintln!`
- ❌ 直接跑企业库 (浪费时间)
- ✅ 用 `tracing` span + `tap` 观测
- ✅ 单测在 `tests/` 目录
- ✅ 通过 sass-spec HRX 观测覆盖率变化
