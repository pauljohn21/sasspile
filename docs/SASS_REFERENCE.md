# Sass 官方文档参考 (AI-Usable Reference)

> 来源: https://sass-lang.com/documentation/  
> 版本: Dart Sass 1.102.0  
> 用途: 供 sasspile 编译器开发参考，所有内容直接来自 sass-lang.com 官方手册  
> 注意: 不参考 Dart Sass 源码实现，只参考官方文档 + sass-spec .hrx input/output + W3C CSS 规范

---

## 目录

1. [Syntax (语法)](#1-syntax-语法)
2. [Style Rules (样式规则)](#2-style-rules-样式规则)
3. [Variables (变量)](#3-variables-变量)
4. [Interpolation (插值)](#4-interpolation-插值)
5. [At-Rules (At 规则)](#5-at-rules-at-规则)
6. [Values (值类型)](#6-values-值类型)
7. [Operators (运算符)](#7-operators-运算符)
8. [Built-In Modules (内置模块)](#8-built-in-modules-内置模块)
9. [Special Functions (特殊函数)](#9-special-functions-特殊函数)
10. [Comments (注释)](#10-comments-注释)
11. [Breaking Changes (破坏性变更)](#11-breaking-changes-破坏性变更)
12. [Command Line (命令行)](#12-command-line-命令行)
13. [JavaScript API](#13-javascript-api)

---

## 1. Syntax (语法)

Sass 支持两种语法，每种都可以加载另一种。

### SCSS (`.scss`)

- CSS 的超集（几乎所有有效 CSS 也是有效 SCSS）
- 使用大括号 `{}` 和分号 `;`
- 最流行的语法

```scss
@mixin button-base() {
  @include typography(button);
  display: inline-flex;
  &:hover { cursor: pointer; }
}
```

### The Indented Syntax (`.sass`)

- Sass 的原始语法
- 使用缩进代替大括号和分号
- 支持与 SCSS 相同的功能

```sass
@mixin button-base()
  @include typography(button)
  display: inline-flex
  &:hover
    cursor: pointer
```

### 多行语句 (缩进语法, Dart Sass 1.84.0+)

缩进语法中，语句可以跨多行，只要换行出现在语句不能结束的位置（如括号内或 @-rule 关键字之间）：

```sass
.grid
  display: grid
  grid-template: (
    "header" min-content
    "main" 1fr
  )
```

---

## 2. Style Rules (样式规则)

样式规则是 Sass 的基础，与 CSS 一样：选择器 + 属性声明。

### Nesting (嵌套)

Sass 允许在一个样式规则内嵌套另一个样式规则，自动组合外层和内层选择器：

```scss
nav {
  ul {
    margin: 0;
    padding: 0;
  }
  li { display: inline-block; }
}
// 输出: nav ul { margin: 0; } nav li { display: inline-block; }
```

> ⚠️ 嵌套过深会增加 CSS 体积和浏览器渲染开销，保持选择器浅层。

### Selector Lists (选择器列表)

逗号分隔的每个复杂选择器独立嵌套，再合并回选择器列表：

```scss
.alert, .warning {
  ul, p { margin: 0; }
}
// 输出: .alert ul, .alert p, .warning ul, .warning p { margin: 0; }
```

### Selector Combinators (选择器组合器)

组合器 (`>`, `+`, `~`) 可放在外层选择器末尾、内层选择器开头，或两者之间：

```scss
ul > { li { ... } }       // → ul > li
h2 { + p { ... } }       // → h2 + p
p { ~ { span { ... } } } // → p ~ span
```

### Interpolation in Selectors (选择器中的插值)

使用 `#{}` 在选择器中注入表达式值：

```scss
@mixin define-emoji($name, $glyph) {
  span.emoji-#{$name} {
    content: $glyph;
  }
}
```

> 💡 Sass 在插值解析完成后再解析选择器，所以可以安全地用插值生成选择器的任何部分。

---

## 3. Variables (变量)

### 基本用法

变量名以 `$` 开头，赋值语法：`<variable>: <expression>`

```scss
$base-color: #c6538c;
$border-dark: rgba($base-color, 0.88);

.alert { border: 1px solid $border-dark; }
```

> Sass 变量与 CSS 变量不同：Sass 变量编译时被替换，CSS 变量保留在输出中。Sass 变量是 imperative 的（先使用后修改，早期使用不变），CSS 变量是 declarative 的。

> Sass 变量名中连字符 `-` 和下划线 `_` 等价：`$font-size` 和 `$font_size` 是同一个变量。

### Default Values (`!default`)

`!default` 标志：仅当变量未定义或值为 `null` 时才赋值。用于库的可配置性。

```scss
$black: #000 !default;
```

### Configuring Modules (配置模块)

通过 `@use` 的 `with` 语法配置 `!default` 变量：

```scss
@use 'library' with (
  $black: #222,
  $border-radius: 0.1rem
);
```

只有文件顶层的 `!default` 变量可以被配置。

### Built-in Variables (内置变量)

内置模块的变量不可修改（如 `math.$pi`）。

### Scope (作用域)

- 顶层变量是 **global** 的
- 块内（大括号/缩进）声明的变量是 **local** 的
- 局部变量可以 **shadowing** 同名全局变量
- `!global` 标志：强制赋值到全局作用域

```scss
$variable: first global value;
.content {
  $variable: second global value !global;
}
```

> ⚠️ Dart Sass 2.0.0+ 中 `!global` 只能用于已声明的变量，不能用于声明新变量。

### Flow Control Scope (流控制作用域)

流控制规则（`@if`, `@each`, `@for`, `@while`）中声明的变量不会遮蔽同级变量，而是直接赋值：

```scss
@if $dark-theme {
  $primary-color: darken($primary-color, 60%); // 修改外层变量
}
```

> 流控制中声明的**新**变量在外层不可用，只能赋值给已存在的变量。

### Advanced Variable Functions

- `meta.variable-exists($name)` — 当前作用域是否存在该变量
- `meta.global-variable-exists($name)` — 全局作用域是否存在该变量

---

## 4. Interpolation (插值)

插值 `#{}` 可在 Sass 样式表的几乎任何位置嵌入 SassScript 表达式的结果。

### 可用位置

- 样式规则中的选择器
- 声明中的属性名
- 自定义属性值
- CSS at-rules
- `@extend`
- 普通 CSS `@import`
- 引号字符串或非引号字符串
- 特殊函数
- 普通 CSS 函数名
- Loud comments

### 在 SassScript 中

插值可用于将 SassScript 注入非引号字符串。返回值始终是非引号字符串。

```scss
@mixin inline-animation($duration) {
  $name: inline-#{unique-id()};
  @keyframes #{$name} { @content; }
  animation-name: $name;
}
```

> ⚠️ 插值用于数字几乎总是坏主意——返回非引号字符串，无法继续数学运算。用单位算术代替，如 `$width * 1px` 而非 `#{$width}px`。

### Quoted Strings (引号字符串)

插值会去除引号字符串的引号：

```scss
.example { unquoted: #{"string"}; }
// 输出: .example { unquoted: string; }
```

> 如需去引号，更推荐用 `string.unquote()` 函数。

---

## 5. At-Rules (At 规则)

### 5.1 @use

> Dart Sass 1.23.0+

加载 mixins、functions、变量，合并 CSS。每个模块只加载一次。

**基本用法:**
```scss
@use 'foundation/code';
@use 'foundation/lists';
```

**命名空间:** 默认为 URL 最后一个组件。可用 `as` 自定义：
```scss
@use "src/corners" as c;
.button { @include c.rounded; padding: 5px + c.$radius; }
```

**无命名空间:** `as *`，不推荐用于第三方库。
```scss
@use "src/corners" as *;
```

**私有成员:** 以 `-` 或 `_` 开头的成员不可在外部访问：
```scss
$-radius: 3px; // 私有变量
```

**配置:**
```scss
@use 'library' with ($black: #222, $border-radius: 0.1rem);
```

**重新赋值变量:** 加载模块后可以重新赋值其变量：
```scss
library.$color: blue;
```

**文件查找规则:**
- 不需要写文件扩展名
- Partials 以 `_` 开头，编译时不单独编译
- `_index.scss` / `_index.sass` 是目录的默认入口
- 支持加载 `.css` 文件（但不支持 Sass 特殊功能）
- 支持 `pkg:` URL 方案

**与 @import 的区别:**
- `@use` 只在当前文件作用域内可见，不添加到全局
- `@use` 只加载一次
- `@use` 必须在文件开头，不能嵌套
- 每条 `@use` 只能有一个 URL
- `@use` 要求 URL 加引号（即使缩进语法）

### 5.2 @forward

加载模块并将其成员转发给下游用户。

```scss
@forward "src/list";
// styles.scss
@use "bootstrap"; // 获得 list 模块的成员
```

**添加前缀:**
```scss
@forward "src/list" as list-*;
// 下游可用 bootstrap.list-reset
```

**控制可见性:**
```scss
@forward "src/list" hide list-reset, $horizontal-list-gap;
@forward "src/list" show list-reset;
```

**配置转发:**
```scss
@forward 'library' with (
  $black: #222 !default,
  $border-radius: 0.1rem !default
);
```

### 5.3 @import (已废弃)

> Dart Sass 1.80.0+ 开始废弃 `@import`

Sass 扩展了 CSS `@import`，可导入 Sass/CSS 样式表。

**基本用法:**
```scss
@import 'foundation/code', 'foundation/lists';
```

**与模块系统集成:**
- 导入含 `@use` 的文件可获取该文件直接定义的所有成员（含私有）
- 导入含 `@forward` 的文件可获得转发成员
- 支持 import-only 文件：`<name>.import.scss`
- 可通过全局变量配置模块

**纯 CSS @import:** 以下条件编译为普通 CSS import：
- URL 以 `.css` 结尾
- URL 以 `http://` 或 `https://` 开头
- URL 写为 `url()`
- 带 media query

### 5.4 @mixin 和 @include

定义可复用样式块。

```scss
@mixin reset-list {
  margin: 0;
  padding: 0;
  list-style: none;
}
nav ul { @include horizontal-list; }
```

**参数:**
```scss
@mixin rtl($property, $ltr-value, $rtl-value) {
  #{$property}: $ltr-value;
  [dir=rtl] & { #{$property}: $rtl-value; }
}
@include rtl(float, left, right);
```

**可选参数 (默认值):**
```scss
@mixin replace-text($image, $x: 50%, $y: 50%) { ... }
```

**关键字参数:**
```scss
@include square(100px, $radius: 4px);
```

**任意参数 (`...`):**
```scss
@mixin order($height, $selectors...) {
  @for $i from 0 to length($selectors) {
    #{nth($selectors, $i + 1)} { position: absolute; height: $height; }
  }
}
```

**任意关键字参数:** 用 `meta.keywords($args)` 获取

**传递任意参数:**
```scss
@include order(150px, $form-selectors...);
```

**Content Blocks (`@content`):**
```scss
@mixin hover {
  &:not([disabled]):hover { @content; }
}
.button {
  @include hover { border-width: 2px; }
}
```

**向 Content Block 传参数 (Dart Sass 1.15.0+):**
```scss
@mixin media($types...) {
  @each $type in $types {
    @media #{$type} { @content($type); }
  }
}
@include media(screen, print) using ($type) {
  h1 { font-size: 40px; }
}
```

### 5.5 @function

定义自定义函数。

```scss
@function fibonacci($n) {
  $sequence: 0 1;
  @for $_ from 1 through $n {
    $new: nth($sequence, length($sequence)) + nth($sequence, length($sequence) - 1);
    $sequence: append($sequence, $new);
  }
  @return nth($sequence, length($sequence));
}
```

- 函数名不能以 `--` 开头
- 必须以 `@return` 结束
- 支持可选参数、关键字参数、任意参数（与 mixin 语法相同）
- 非 Sass 内建或用户定义的函数调用编译为普通 CSS 函数

### 5.6 @extend

让一个选择器继承另一个选择器的样式。

```scss
.error {
  border: 1px #f00;
  background-color: #fdd;
}
.error--serious {
  @extend .error;
  border-width: 3px;
}
// 输出: .error, .error--serious { border: 1px #f00; ... }
```

**How It Works:**
- 不是复制样式到当前规则，而是更新包含被扩展选择器的规则
- 智能统一：不生成不可能匹配的选择器（如 `#main#footer`）
- 知道一个选择器匹配另一个的所有情况并合并

**Placeholder Selectors (`%`):**
```scss
%strong-alert { font-weight: bold; }
// 只在被 @extend 时才出现在输出中
```

**Private Placeholders:** 以 `-` 或 `_` 开头，只能在定义文件内扩展

**Extension Scope:** 只影响上游模块（通过 `@use`/`@forward` 加载的）

**Optional Extends:** `@extend .foo !optional;` — 不存在时不报错

**限制:**
- 只能扩展简单选择器（单个选择器如 `.info` 或 `a`）
- 不能扩展 `.message.info`（应写 `@extend .message, .info`）
- 不能扩展 `.main .info`（应写 `@extend .info`）
- `@media` 内不能扩展 `@media` 外的选择器

### 5.7 Flow Control (流控制)

#### @if
```scss
@if $condition { ... }
@else if $other { ... }
@else { ... }
```

#### @each
```scss
@each $size in $sizes { ... }        // 遍历列表
@each $key, $value in $map { ... }   // 遍历 map
```

#### @for
```scss
@for $i from 1 through 3 { ... }  // 包含 3
@for $i from 1 to 3 { ... }       // 不包含 3
```

#### @while
```scss
$i: 6;
@while $i > 0 { $i: $i - 2; }
```

### 5.8 @error / @warn / @debug

- `@error <expression>` — 打印错误消息和堆栈跟踪，停止编译
- `@warn <expression>` — 打印警告，不停止编译
- `@debug <expression>` — 打印调试消息

### 5.9 @at-root

将样式放在 CSS 文档的根级别，忽略嵌套上下文。

---

## 6. Values (值类型)

Sass 支持以下值类型：

### 6.1 Numbers (数字)

两个组件：数字本身和单位。支持无单位、简单单位、复合单位。

```scss
@debug 100;    // 100
@debug 16px;   // 16px
@debug 5px * 2px; // 10px*px (square pixels)
```

- 支持科学计数法：`5.2e3` → `5200`
- 兼容单位自动转换：`1in + 6px` → `102px`
- 不兼容单位报错：`1in + 1s` → Error
- 精度：64位浮点数，小数点后最多 10 位有效数字

**Units (单位):**
- 乘法：单位相乘
- 除法：分子单位来自第一个数，分母单位来自第二个数
- 兼容单位自动约简：`math.div(96px, 1in)` → 无单位

### 6.2 Strings (字符串)

两种：引号字符串 (`"Helvetica Neue"`) 和非引号字符串/标识符 (`bold`)。

**Escapes (转义):**
- `\` 后接任何非 A-F/0-9 字符 → 该字符
- `\` 后接十六进制 Unicode 码点 → 对应字符

**String Indexes:** 从 1 开始，`-1` 指最后一个字符。

### 6.3 Colors (颜色)

支持多种颜色表示：
- Hex codes: `#f2ece4`, `#b37399aa` (带 alpha)
- CSS 颜色名: `midnightblue`, `transparent`
- 颜色函数: `rgb()`, `lab()`, `color()`, `hsl()`, `hwb()`

**Color Spaces (颜色空间, Dart Sass 1.79.0+):**

| Space | Syntax | Channels |
|-------|--------|----------|
| `rgb`* | `rgb(102 51 153)` `#663399` | red [0,255]; green [0,255]; blue [0,255] |
| `hsl`* | `hsl(270 50% 40%)` | hue [0,360]; saturation [0%,100%]; lightness [0%,100%] |
| `hwb`* | `hwb(270 20% 40%)` | hue [0,360]; whiteness [0%,100%]; blackness [0%,100%] |
| `srgb` | `color(srgb 0.4 0.2 0.6)` | red [0,1]; green [0,1]; blue [0,1] |
| `srgb-linear` | `color(srgb-linear ...)` | red [0,1]; green [0,1]; blue [0,1] |
| `display-p3` | `color(display-p3 ...)` | red [0,1]; green [0,1]; blue [0,1] |
| `display-p3-linear` | `color(display-p3-linear ...)` | red [0,1]; green [0,1]; blue [0,1] |
| `a98-rgb` | `color(a98-rgb ...)` | red [0,1]; green [0,1]; blue [0,1] |
| `prophoto-rgb` | `color(prophoto-rgb ...)` | red [0,1]; green [0,1]; blue [0,1] |
| `rec2020` | `color(rec2020 ...)` | red [0,1]; green [0,1]; blue [0,1] |
| `xyz`, `xyz-d65` | `color(xyz ...)` | x [0,1]; y [0,1]; z [0,1] |
| `xyz-d50` | `color(xyz-d50 ...)` | x [0,1]; y [0,1]; z [0,1] |
| `lab` | `lab(32.4% 38.4 -47.7)` | lightness [0%,100%]; a [-125,125]; b [-125,125] |
| `lch` | `lch(32.4% 61.2 308.9deg)` | lightness [0%,100%]; chroma [0,150]; hue [0°,360°] |
| `oklab` | `oklab(44% 0.088 -0.134)` | lightness [0%,100%]; a [-0.4,0.4]; b [-0.4,0.4] |
| `oklch` | `oklch(44% 0.16 303.4deg)` | lightness [0%,100%]; chroma [0,0.4]; hue [0°,360°] |

*标记为 legacy 颜色空间。

**Missing Channels (缺失通道):** 用 `none` 表示，如 `hsl(none 0% 50%)`。混合颜色时取另一颜色的值。

**Powerless Channels (无力通道):** 转换到新空间时，无力通道被替换为 `none`（除了转到 legacy 空间）。

**Legacy Color Spaces:** `rgb`, `hsl`, `hwb` 是 legacy 空间。旧函数操作任意 legacy 颜色，新函数需显式指定 `$space`。

### 6.4 Lists (列表)

值的序列，用逗号、空格或斜杠分隔。

- 不需要方括号（但可用 `[line1 line2]`）
- 用括号嵌套或消歧：`(1, 2), (3, 4)` 是包含两个列表的列表
- 单元素列表：`(value,)` 或 `[value]`
- 空列表：`()` 或 `[]`

**Slash-Separated Lists:** 只能用 `list.slash()` 创建（`/` 历史上用于除法）。

**Indexes:** 从 1 开始，`-1` 指最后一个元素。

**关键函数:**
- `list.nth($list, $n)` — 获取元素
- `list.append($list, $val)` — 添加元素（返回新列表）
- `list.index($list, $value)` — 查找元素索引，不存在返回 `null`
- `list.separator($list)` — 获取分隔符

**Immutability:** 列表不可变，所有列表函数返回新列表。

**Argument Lists:** mixin/function 的任意参数形成特殊列表，可用 `meta.keywords()` 获取关键字参数。

### 6.5 Maps (映射)

键值对集合，用括号包裹：`("key": value, "key2": value2)`。

- 键必须唯一，值可重复
- 键可以是任何 Sass 值，用 `==` 判断相同
- 空列表 `()` 等同于空 map

**关键函数:**
- `map.get($map, $key)` — 获取值，不存在返回 `null`
- `map.set($map, $key, $value)` — 设置值（返回新 map）
- `map.merge($map1, $map2)` — 合并两个 map
- `map.has-key($map, $key)` — 是否存在键

**Immutability:** Maps 不可变，所有 map 函数返回新 map。

### 6.6 Booleans (布尔值)

`true` 和 `false`。

- `and` — 两边为 true 返回 true
- `or` — 任一边为 true 返回 true
- `not` — 取反

**Truthiness:** `false` 和 `null` 是 falsey，其他所有值都是 truthy。

### 6.7 null

表示值的缺失。列表中的 `null` 在 CSS 输出中省略。属性值为 `null` 时该属性被省略。`null` 也是 falsey。

### 6.8 Function References (函数引用)

用 `meta.get-function("name")` 获取函数作为值，用 `meta.call($function, $args...)` 调用。

---

## 7. Operators (运算符)

| 运算符 | 用途 |
|--------|------|
| `==` `!=` | 相等判断 |
| `+` `-` `*` `/` `%` | 数学运算（含单位行为） |
| `<` `<=` `>` `>=` | 大小比较 |
| `and` `or` `not` | 布尔运算 |
| `+` `-` `/` | 字符串连接 |

> ⚠️ 颜色运算已被废弃，使用颜色函数代替。

### Order of Operations (优先级, 从高到低)

1. 一元运算符 `not`, `+`, `-`, `/`
2. `*`, `/`, `%`
3. `+`, `-`
4. `>`, `>=`, `<`, `<=`
5. `==`, `!=`
6. `and`
7. `or`
8. `=` (仅函数参数中可用)

### Parentheses (括号)

用括号显式控制运算顺序。括号可嵌套，最内层先计算。

### Single Equals (`=`)

仅函数参数中可用，创建以 `=` 分隔的非引号字符串。用于旧 IE 语法兼容：

```scss
.transparent-blue { filter: chroma(color=#0000ff); }
```

---

## 8. Built-In Modules (内置模块)

> Dart Sass 1.23.0+

用 `@use` 加载，函数通过命名空间调用。所有模块 URL 以 `sass:` 开头。

```scss
@use "sass:color";
@use "sass:math";

.button {
  color: $primary-color;
  border: 1px solid color.scale($primary-color, $lightness: 20%);
}
```

### 可用模块

| 模块 | 用途 |
|------|------|
| `sass:math` | 数学函数 (bounding, distance, exponential, trigonometric, unit, other) |
| `sass:string` | 字符串操作 (合并、搜索、拆分) |
| `sass:color` | 颜色操作 (生成新颜色、构建色彩主题) |
| `sass:list` | 列表访问和修改 |
| `sass:map` | Map 键值查找和操作 |
| `sass:selector` | 选择器引擎访问 |
| `sass:meta` | Sass 内部机制 |

### sass:math

**变量:**
- `math.$pi` — π 值
- `math.$e` — e 值

**Bounding Functions:**
- `math.ceil($number)` — 向上取整
- `math.floor($number)` — 向下取整
- `math.round($number)` — 四舍五入
- `math.abs($number)` — 绝对值

**Distance Functions:**
- `math.max($numbers...)` — 最大值
- `math.min($numbers...)` — 最小值
- `math.random()` / `math.random($limit)` — 随机数

**Exponential Functions:**
- `math.pow($base, $exponent)` — 幂运算
- `math.log($number, $base)` — 对数
- `math.sqrt($number)` — 平方根

**Trigonometric Functions:**
- `math.sin($angle)` — 正弦
- `math.cos($angle)` — 余弦
- `math.tan($angle)` — 正切
- `math.asin($number)` — 反正弦
- `math.acos($number)` — 反余弦
- `math.atan($number)` — 反正切
- `math.atan2($y, $x)` — 双参数反正切

**Unit Functions:**
- `math.comparable($number1, $number2)` — 单位是否兼容
- `math.is-unitless($number)` — 是否无单位
- `math.unit($number)` — 获取单位字符串

**Other Functions:**
- `math.div($number1, $number2)` — 除法（替代 `/`）
- `math.percentage($number)` — 转为百分比
- `math.clamp($min, $number, $max)` — 限制范围

### sass:string

- `string.quote($string)` — 加引号
- `string.unquote($string)` — 去引号
- `string.length($string)` — 字符串长度
- `string.slice($string, $start, $end)` — 截取子串
- `string.index($string, $substring)` — 查找子串位置
- `string.insert($string, $insert, $index)` — 插入字符串
- `string.split($string, $separator)` — 分割字符串
- `string.to-upper-case($string)` — 转大写
- `string.to-lower-case($string)` — 转小写

### sass:color

> Dart Sass 1.79.0+ 重构为 CSS Color 4 兼容函数

**现代函数:**
- `color.channel($color, $channel, $space)` — 获取通道值
- `color.mix($color1, $color2, $weight, $method)` — 混合颜色
- `color.scale($color, $kwargs, $space)` — 缩放通道
- `color.adjust($color, $kwargs, $space)` — 调整通道
- `color.change($color, $kwargs)` — 修改通道
- `color.to-space($color, $space)` — 转换颜色空间
- `color.is-legacy($color)` — 是否为 legacy 颜色
- `color.is-missing($color, $channel)` — 通道是否缺失
- `color.is-powerless($color, $channel, $space)` — 通道是否无力
- `color.same-color($color1, $color2)` — 颜色是否相同

**Legacy 函数 (已废弃):**
- `color.invert($color, $weight)` — 反色
- `color.adjust-hue($color, $degrees)` — 调整色相
- `color.lighten($color, $amount)` — 变亮
- `color.darken($color, $amount)` — 变暗
- `color.saturate($color, $amount)` — 增加饱和度
- `color.desaturate($color, $amount)` — 降低饱和度
- `color.grayscale($color)` — 灰度
- `color.complement($color)` — 互补色
- `color.alpha($color)` — 获取 alpha
- `color.hue($color)` — 获取色相
- `color.saturation($color)` — 获取饱和度
- `color.lightness($color)` — 获取亮度
- `color.red($color)` — 获取红色通道
- `color.green($color)` — 获取绿色通道
- `color.blue($color)` — 获取蓝色通道
- `color.ie-hex-str($color)` — IE hex 字符串

### sass:list

- `list.append($list, $val, $separator)` — 添加元素
- `list.index($list, $value)` — 查找索引
- `list.is-bracketed($list)` — 是否有方括号
- `list.length($list)` — 列表长度
- `list.nth($list, $n)` — 获取元素
- `list.separator($list)` — 获取分隔符 (space/comma/slash)
- `list.set-nth($list, $n, $value)` — 替换元素
- `list.slash($elements...)` — 创建斜杠分隔列表
- `list.zip($lists...)` — 合并列表

### sass:map

- `map.get($map, $key)` — 获取值
- `map.set($map, $key, $value)` — 设置值
- `map.has-key($map, $key)` — 是否有键
- `map.merge($map1, $map2)` — 合并
- `map.remove($map, $keys...)` — 移除键
- `map.keys($map)` — 所有键
- `map.values($map)` — 所有值
- `map.deep-merge($map1, $map2)` — 深度合并
- `map.deep-set($map, $key, $value)` — 深度设置
- `map.deep-get($map, $key...)` — 深度获取

### sass:selector

- `selector.is-superselector($super, $sub)` — 超选择器判断
- `selector.append($selectors...)` — 追加选择器
- `selector.extend($selector, $extendee, $extender)` — 扩展选择器
- `selector.nest($selectors...)` — 嵌套选择器
- `selector.parse($selector)` — 解析选择器
- `selector.replace($selector, $original, $replacement)` — 替换选择器
- `selector.unify($selector1, $selector2)` — 统一选择器
- `selector.simple-selectors($compound)` — 拆分简单选择器

### sass:meta

**Mixins:**
- `meta.load-css($url, $with)` — 加载 CSS 模块（替代嵌套 `@import`）

**Functions:**
- `meta.feature-exists($feature)` — 检查特性是否存在 (1.78.0+ 废弃)
- `meta.inspect($value)` — 检查值的字符串表示
- `meta.keywords($args)` — 获取参数列表中的关键字参数
- `meta.module-variables($module)` — 获取模块所有变量
- `meta.module-functions($module)` — 获取模块所有函数
- `meta.get-function($name, $css)` — 获取函数引用
- `meta.call($function, $args...)` — 调用函数引用
- `meta.get-mixin($name)` — 获取 mixin 引用
- `meta.apply($mixin, $args...)` — 调用 mixin 引用
- `meta.variable-exists($name)` — 变量是否在当前作用域存在
- `meta.global-variable-exists($name)` — 变量是否在全局存在
- `meta.mixin-exists($name)` — mixin 是否存在
- `meta.function-exists($name)` — 函数是否存在
- `meta.type-of($value)` — 获取值类型
- `meta.calc-args($calc)` — 获取 calc() 参数
- `meta.calc-name($calc)` — 获取 calc() 名称

---

## 9. Special Functions (特殊函数)

返回非引号字符串的特殊语法函数。

### if()

> Dart Sass 1.95.0+ 更新为 CSS if() 语法

CSS `if()` 函数 + Sass 扩展 `sass(...)` 条件：

```scss
$hungry: true;
@debug if(sass($hungry): breakfast burrito; else: cereal);
// → breakfast burrito

// 只评估匹配的分支
@debug if(sass(meta.variable-exists("thirsty")): thirsty; else: hungry);
// → hungry
```

纯 Sass 条件（只含 `sass(...)` 和 `else`）会被完全评估并返回对应值。无匹配返回 `null`。

### url()

特殊解析：可以是引号 URL 或非引号 URL。

```scss
url("#{$path}/font.woff2")  // 引号字符串
url($path + "/font.woff2")  // 算术表达式
url(#{$path}/font.woff2)    // 插值特殊函数
```

### element(), progid:...(), expression()

- `element()` — CSS spec 函数，ID 可能被解析为颜色
- `expression()` / `progid:...()` — 旧 IE 函数，使用非标准语法
- 这些函数中任何文本（含嵌套括号）都不被解析为 SassScript，但可用插值注入动态值

```scss
$logo-element: logo-bg;
.logo { background: element(##{$logo-element}); }
// 输出: .logo { background: element(#logo-bg); }
```

---

## 10. Comments (注释)

### SCSS

- **Silent comments** (`//`): 不输出到 CSS
- **Loud comments** (`/* */`): 输出到 CSS（压缩模式默认移除）
- `/*! ... */`: 即使压缩模式也保留
- Loud comments 可包含插值

```scss
// 不会出现在 CSS 中
/* 会出现在 CSS 中 */
/*! 即使压缩模式也会出现 */
/* 插值: 1 + 1 = #{1 + 1} */
```

### Indented Syntax

- `//` 后缩进内容也被注释
- `/*` 基于缩进确定范围，`*/` 可选
- 支持 `/*!` 和插值

### Documentation Comments (`///`)

三斜杠注释，用于文档化 mixin/function/variable/placeholder。SassDoc 工具解析。

```scss
/// Computes an exponent.
///
/// @param {number} $base - The number to multiply by itself.
/// @param {integer (unitless)} $exponent - The number of $base's to multiply.
/// @return {number} $base to the power of $exponent.
@function pow($base, $exponent) { ... }
```

---

## 11. Breaking Changes (破坏性变更)

Sass 在发布破坏性变更前会先产生 deprecation 警告。

### 近期和即将到来的变更

| 变更 | 起始版本 | 说明 |
|------|----------|------|
| Adjacent compound selectors | 1.100.0 | 相邻复合选择器 |
| Additional plain CSS function names | 1.98.0 | 额外的普通 CSS 函数名 |
| Legacy if() function | 1.95.0 | 旧 if() 函数 |
| Private variables in with | 1.92.0 | with 中的私有变量 |
| Misplaced rest arguments | 1.91.0 | 错位的 rest 参数 |
| type() function | 1.86.0 | type() 函数 |
| @import | 1.80.0 | @import 废弃 |
| Legacy JS API | 1.79.0 | 旧 JS API |
| Color functions | 1.79.0 | 颜色函数废弃（改用 CSS Color 4 兼容函数） |
| meta.feature-exists() | 1.78.0 | 废弃 |
| Mixing declarations with nested rules | 1.77.7 | 声明与嵌套规则混用行为变更 |
| Functions/Mixins beginning with -- | 1.76.0 | 废弃 |
| abs() with percentage | 1.65.0 | 传递百分比单位给 abs() 废弃 |
| Single !global or !default | 1.62.0 | 变量只能有一个 !global 或 !default |
| Strict unary operators | 1.55.0 | 严格一元运算符 |
| Media Queries Level 4 | 1.54.0 | Media 查询 Level 4 |
| / as list separator | 1.33.0 | / 从除法变为列表分隔符 |
| Stricter units | 1.32.0 | 函数对单位更严格 |
| @-moz-document | 1.7.2 | 解析 @-moz-document 特殊语法无效 |
| Compound selector extends | 1.0.0 | 复合选择器不能被扩展 |
| CSS custom property values | 1.0.0 | 自定义属性值语法变更 |

### Early Opt-In

Dart Sass 用户可用 `--fatal-deprecation` 选项提前将 deprecation 视为错误。

---

## 12. Command Line (命令行)

### Dart Sass CLI

```bash
sass input.scss output.css
sass --watch input.scss:output.css
sass --style=compressed input.scss output.css
sass --pkg-importer=node input.scss  # 启用 Node.js pkg: 导入器
sass --fatal-deprecation             # 将 deprecation 视为错误
```

---

## 13. JavaScript API

### 现代入口

- `compile(path, options)` / `compileAsync(path, options)` — 编译文件
- `compileString(string, options)` / `compileStringAsync(string, options)` — 编译字符串

```js
const sass = require('sass');
const result = sass.compile("style.scss");
console.log(result.css);

const compressed = sass.compile("style.scss", {style: "compressed"});
```

### 集成

- Webpack: `sass-loader`
- Gulp: `gulp-sass`
- Broccoli: `broccoli-sass-source-maps`
- Ember: `ember-cli-sass`
- Grunt: `grunt-sass`

### 旧 API (废弃)

- `renderSync(options)` — 同步编译
- `render(options, callback)` — 异步编译

### 性能

- `sass` 包：同步比异步快
- `sass-embedded` 包：通常更快，异步和并发编译更优
- `Compiler` / `AsyncCompiler` 类复用 Dart 进程，适合频繁编译

---

## 附录: 值类型快速参考

| 类型 | 字面量 | 示例 |
|------|--------|------|
| Number | `12`, `16px`, `5px * 2px` | `@debug 16px;` |
| String (quoted) | `"..."`, `'...'` | `@debug "Helvetica";` |
| String (unquoted) | identifier | `@debug bold;` |
| Color | hex, name, function | `@debug #f2ece4;` `@debug blue;` |
| List | space/comma/slash separated | `1.5em 1em 0 2em` |
| Map | `(key: value)` | `("bg": red, "fg": pink)` |
| Boolean | `true`, `false` | `@debug true;` |
| Null | `null` | `@debug null;` |
| Function | `get-function("name")` | `meta.get-function("fib")` |

## 附录: 运算符优先级

```
高  not, +, - (unary)
    * / %
    + -
    > >= < <=
    == !=
    and
    or
低  = (仅函数参数)
```

## 附录: 模块加载函数一览

### 全局函数（始终可用）

- `if($condition, $if-true, $if-false)` — 条件函数（旧语法）
- `rgb($red, $green, $blue, $alpha)` — RGB 颜色（旧）
- `rgba($red, $green, $blue, $alpha)` — RGBA 颜色（旧）
- `hsl($hue, $saturation, $lightness)` — HSL 颜色（旧）
- `hsla(...)` — HSLA 颜色（旧）
- `unquote($string)` — 去引号（旧，用 `string.unquote()` 代替）
- `quote($string)` — 加引号（旧，用 `string.quote()` 代替）
- `nth($list, $n)` — 列表元素（旧，用 `list.nth()` 代替）
- `length($list)` — 列表长度（旧，用 `list.length()` 代替）
- `append($list, $val)` — 列表添加（旧）
- `map-get($map, $key)` — Map 获取（旧）
- `map-merge($map1, $map2)` — Map 合并（旧）
- `map-keys($map)` — Map 键（旧）
- `map-values($map)` — Map 值（旧）
- `map-has-key($map, $key)` — Map 是否有键（旧）
- `lighten/darken/saturate/desaturate/adjust-hue/complement/invert/grayscale` — 颜色操作（旧）
- `red/green/blue/hue/saturation/lightness/alpha` — 颜色通道（旧）
- `adjust-color/change-color/scale-color` — 颜色调整（旧）
- `min/max(...)` — 最小/最大值（旧，用 `math.min/max()` 代替）
- `random()` — 随机数（旧，用 `math.random()` 代替）
- `unique-id()` — 唯一 ID
- `inspect($value)` — 检查值（旧，用 `meta.inspect()` 代替）
- `type-of($value)` — 类型（旧，用 `meta.type-of()` 代替）
- `unit($number)` — 单位（旧，用 `math.unit()` 代替）
- `unitless($number)` — 无单位（旧，用 `math.is-unitless()` 代替）
- `comparable($n1, $n2)` — 兼容单位（旧，用 `math.comparable()` 代替）
- `percentage($number)` — 百分比（旧，用 `math.percentage()` 代替）
- `str-length/str-insert/str-index/str-slice/to-upper-case/to-lower-case` — 字符串（旧）
- `selector-nest/selector-append/selector-extend/selector-replace/selector-unify/selector-parse` — 选择器（旧）
- `feature-exists($feature)` — 特性检查（废弃）
- `variable-exists/mixin-exists/function-exists/global-variable-exists` — 元检查（旧）
- `get-function($name)` — 函数引用（旧，用 `meta.get-function()` 代替）
- `call($function, $args...)` — 调用函数（旧，用 `meta.call()` 代替）
- `content-exists()` — content block 是否传入
- `keywords($args)` — 关键字参数（旧，用 `meta.keywords()` 代替）

> 全局函数中标注为"旧"的应优先使用对应模块函数。