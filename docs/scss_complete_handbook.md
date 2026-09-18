# SCSS 完整开发手册

> 基于 Sass 官方文档编写的完整参考手册,用于辅助 sasspile-rx 编译器开发。

---

## 目录

1. [基础语法](#1-基础语法)
2. [变量 Variables](#2-变量-variables)
3. [插值 Interpolation](#3-插值-interpolation)
4. [嵌套 Nesting](#4-嵌套-nesting)
5. [@use 模块系统](#5-use-模块系统)
6. [@forward 转发](#6-forward-转发)
7. [@import 导入(遗留)](#7-import-导入遗留)
8. [@mixin 和 @include](#8-mixin-和-include)
9. [@function 和 @return](#9-function-和-return)
10. [流程控制 Flow Control](#10-流程控制-flow-control)
11. [值类型 Values](#11-值类型-values)
12. [运算符 Operators](#12-运算符-operators)
13. [父选择器 &](#13-父选择器-)
14. [占位符选择器 %](#14-占位符选择器-)
15. [内置模块 Built-In Modules](#15-内置模块-built-in-modules)
16. [所有 At-Rules 速查](#16-所有-at-rules-速查)
17. [编译输出规则](#17-编译输出规则)

---

## 1. 基础语法

### 1.1 两种语法
- **SCSS (Sassy CSS)**: 扩展的 CSS 语法,文件扩展名 `.scss`,完全兼容 CSS
- **Indented Syntax (Sass)**: 缩进语法,文件扩展名 `.sass`,不使用大括号和分号

### 1.2 语句类型
- **Style Rules**: 样式规则,如 `selector { property: value; }`
- **At-Rules**: 指令规则,如 `@use`, `@mixin`, `@if`
- **Property Declarations**: 属性声明,如 `width: 100px`
- **Variable Declarations**: 变量声明,如 `$color: red`
- **Comments**: 注释,`// 单行` 或 `/* 多行 */`

### 1.3 顶层语句
- `@use` 必须在除 `@forward` 之前的任何规则之前
- 可以在 `@use` 之前声明变量用于配置模块
- `@import` 已被废弃,推荐使用 `@use`

---

## 2. 变量 Variables

### 2.1 声明
```scss
$primary-color: #036;
$font-stack: "Helvetica Neue", Helvetica, Arial;
$base-padding: 1em !default;  // 可被覆盖
```

### 2.2 使用
```scss
.sidebar {
  width: $base-padding * 2;
  color: $primary-color;
}
```

### 2.3 作用域
- 局部变量: 在声明它的块内有效
- `!global`: 在局部作用域声明全局变量
- `!default`: 仅在变量未赋值时生效(用于模块配置)

### 2.4 变量类型
变量可以持有任何值类型:数字、字符串、颜色、列表、映射、布尔值、null

---

## 3. 插值 Interpolation

### 3.1 语法: `#{}`
```scss
@mixin define-border($side) {
  border-#{$side}: 1px solid;
}
// 生成 border-left: 1px solid;

$name: ".icon-#{$size}";  // 选择器插值
```

### 3.2 使用场景
- 选择器名
- 属性名
- 属性值
- 注释
- 值列表

### 3.3 在 SassScript 中
插值会计算表达式,返回未引号的字符串

---

## 4. 嵌套 Nesting

### 4.1 选择器嵌套
```scss
nav {
  ul {
    margin: 0;
    li { display: inline-block; }
  }
}
```
编译为:
```css
nav ul { margin: 0; }
nav ul li { display: inline-block; }
```

### 4.2 属性嵌套
```scss
.foo {
  border: {
    style: solid;
    width: 1px;
    color: #ccc;
  }
}
```
编译为:
```css
.foo {
  border-style: solid;
  border-width: 1px;
  border-color: #ccc;
}
```

### 4.3 父选择器 &
```scss
a {
  color: blue;
  &:hover { color: red; }
  body.dark & { color: white; }
}
```

---

## 5. @use 模块系统

### 5.1 基本用法
```scss
// 加载模块
@use 'foundation/code';
@use 'foundation/lists';

// 命名空间访问
.button {
  @include corners.rounded;
  padding: 5px + corners.$radius;
}
```

### 5.2 默认命名空间
默认命名空间是模块 URL 的最后一个组件(不含扩展名):
```scss
@use "src/corners";  // 命名空间为 corners
```

### 5.3 自定义命名空间
```scss
@use "src/corners" as c;    // 自定义为 c
@use "src/corners" as *;    // 无命名空间(不推荐用于外部库)
```

### 5.4 私有成员
以 `-` 或 `_` 开头的成员是私有的,外部不可见:
```scss
// _corners.scss
$-radius: 3px;  // 私有变量
```

### 5.5 配置
```scss
// _library.scss
$black: #000 !default;
$border-radius: 0.25rem !default;

// style.scss
@use 'library' with (
  $black: #222,
  $border-radius: 0.1rem
);
```

### 5.6 配置规则
- 模块保持相同配置,即使多次加载
- `@use ... with` 只能在模块首次加载时使用一次
- 配置可用于创建主题

### 5.7 加载路径
- Load Paths: 通过 `--load-path` 指定
- Partials: 以 `_` 开头的文件(如 `_colors.scss`)
- Index Files: 目录下的 `_index.scss` 或 `_index.sass`
- pkg: URLs: 通过 `pkg:` 协议加载

### 5.8 与 @import 的区别
- `@use` 只导入一次,无论使用多少次
- `@use` 创建命名空间,避免命名冲突
- `@use` 的成员只在加载它的样式表中可见
- `@use` 必须在除 `@forward` 之前的任何规则之前

---

## 6. @forward 转发

### 6.1 基本用法
```scss
// _list.scss
@forward 'background-color';
@forward 'border-color';
@forward 'color';
```

### 6.2 作用
- 将多个模块的成员转发到一个共享文件
- 用于创建库的入口点
- 可以转发整个模块或选择性转发

---

## 7. @import 导入(遗留)

### 7.1 基本用法
```scss
@import 'foundation';
@import 'foundation/code', 'foundation/lists';
```

### 7.2 与 @use 的区别
- `@import` 全局导入,无命名空间
- `@import` 多次导入会重复包含
- `@import` 已被废弃,推荐使用 `@use`

---

## 8. @mixin 和 @include

### 8.1 定义 Mixin
```scss
@mixin reset-list {
  margin: 0;
  padding: 0;
  list-style: none;
}
```

### 8.2 使用 Mixin
```scss
nav ul {
  @include reset-list;
}
```

### 8.3 参数
```scss
@mixin rtl($property, $ltr-value, $rtl-value) {
  #{$property}: $ltr-value;
  [dir=rtl] & {
    #{$property}: $rtl-value;
  }
}
```

### 8.4 可选参数(默认值)
```scss
@mixin replace-text($image, $x: 50%, $y: 50%) {
  text-indent: -99999em;
  background: {
    image: $image;
    position: $x $y;
  }
}
```

### 8.5 关键字参数
```scss
@mixin square($size, $radius: 0) {
  width: $size;
  height: $size;
  @if $radius != 0 {
    border-radius: $radius;
  }
}

.avatar {
  @include square(100px, $radius: 4px);
}
```

### 8.6 任意位置参数(...)
```scss
@mixin order($height, $selectors...) {
  @for $i from 0 to length($selectors) {
    #{nth($selectors, $i + 1)} {
      position: absolute;
      height: $height;
      margin-top: $i * $height;
    }
  }
}

@include order(150px, "input.name", "input.address", "input.zip");
```

### 8.7 任意关键字参数
```scss
@use "sass:meta";

@mixin syntax-colors($args...) {
  @each $name, $color in meta.keywords($args) {
    pre span.stx-#{$name} {
      color: $color;
    }
  }
}

@include syntax-colors(
  $string: #080,
  $comment: #800,
  $variable: #60b,
);
```

### 8.8 传递任意参数
```scss
$form-selectors: "input.name", "input.address", "input.zip" !default;
@include order(150px, $form-selectors...);
```

### 8.9 内容块 @content
```scss
@mixin hover {
  &:not([disabled]):hover {
    @content;
  }
}

@include hover {
  color: red;
}
```

编译为:
```css
:not([disabled]):hover {
  color: red;
}
```

### 8.10 向内容块传递参数
```scss
@mixin media($types...) {
  @each $type in $types {
    @media #{$type} {
      @content;
    }
  }
}

@include media(screen, print) {
  width: 100%;
}
```

### 8.11 缩进 Mixin 语法(Sass 语法)
```sass
=reset-list
  margin: 0
  padding: 0
```

---

## 9. @function 和 @return

### 9.1 定义函数
```scss
@function pow($base, $exponent) {
  $result: 1;
  @for $_ from 1 through $exponent {
    $result: $result * $base;
  }
  @return $result;
}
```

### 9.2 使用函数
```scss
.sidebar {
  width: pow(4, 3) * 1px;  // 64px
}
```

### 9.3 参数
- 可选参数: 带默认值
- 关键字参数: `$param: value`
- 任意参数: `$args...`
- 任意关键字参数: `meta.keywords($args)`

### 9.4 @return
- 必须显式返回值
- 可以在函数体任何位置使用
- 返回任何值类型

### 9.5 普通 CSS 函数
```scss
@function my-function($arg) {
  @if $arg == "bold" {
    @return 700;
  }
  @return 400;
}
```

---

## 10. 流程控制 Flow Control

### 10.1 @if 和 @else
```scss
@use "sass:math";

@mixin avatar($size, $circle: false) {
  width: $size;
  height: $size;
  @if $circle {
    border-radius: math.div($size, 2);
  }
}
```

### 10.2 @else if
```scss
@mixin triangle($size, $color, $direction) {
  width: 0;
  height: 0;
  @if $direction == up {
    border-bottom: $size solid $color;
  } @else if $direction == right {
    border-left: $size solid $color;
  } @else if $direction == down {
    border-top: $size solid $color;
  } @else {
    border-right: $size solid $color;
  }
}
```

### 10.3 布尔运算符
- `and`: 逻辑与
- `or`: 逻辑或
- `not`: 逻辑非

### 10.4 真值与假值
- **假值**: `false`, `null`
- **真值**: 其他所有值(包括 `0`, `""`, `[]`)

### 10.5 @each
```scss
// 遍历列表
$sizes: 40px, 50px, 80px;
@each $size in $sizes {
  .icon-#{$size} {
    font-size: $size;
  }
}

// 遍历映射
$icons: (
  "eye": "\f112",
  "start": "\f12e",
  "stop": "\f12f"
);
@each $name, $glyph in $icons {
  .icon-#{$name}:before {
    content: $glyph;
  }
}
```

### 10.6 @for
```scss
// from through (包含结束)
@for $i from 1 through 3 {
  .col-#{$i} {
    width: 100% / $i;
  }
}

// from to (不包含结束)
@for $i from 1 to 3 {
  .col-#{$i} {
    width: 100% / $i;
  }
}
```

### 10.7 @while
```scss
$i: 1;
@while $i <= 3 {
  .col-#{$i} {
    width: 100% / $i;
  }
  $i: $i + 1;
}
```

---

## 11. 值类型 Values

### 11.1 数字 (Numbers)
```scss
12        // 无单位整数
100px     // 带单位
1.5em     // 小数
10%       // 百分比
```

**单位运算**:
- 相同单位可直接运算
- 不同单位转换遵循科学计算规则
- `+`, `-`, `*`, `/`, `%` 都支持

### 11.2 字符串 (Strings)
```scss
"Helvetica Neue"  // 带引号
bold              // 无引号
"line1\nline2"    // 转义字符
```

**字符串拼接**:
```scss
$family: "Helvetica" + " " + "Neue";  // "Helvetica Neue"
```

### 11.3 颜色 (Colors)
```scss
#c6538c                    // 十六进制
blue                       // 颜色名
rgb(107, 113, 127)        // RGB
hsl(210, 100%, 20%)       // HSL
```

### 11.4 列表 (Lists)
```scss
1.5em 1em 0 2em                    // 空格分隔
Helvetica, Arial, sans-serif        // 分隔
[col1-start]                        // 方括号
()                                  // 空列表
```

**列表函数**:
- `length($list)`: 列表长度
- `nth($list, $n)`: 第 n 个元素
- `set-nth($list, $n, $value)`: 设置第 n 个元素
- `join($list1, $list2)`: 合并列表
- `append($list, $val)`: 追加元素
- `index($list, $value)`: 查找元素位置
- `zip($lists...)`: 合并多个列表

### 11.5 映射 (Maps)
```scss
$font-weights: (
  "regular": 400,
  "medium": 500,
  "bold": 700
);
```

**映射函数**:
- `map-get($map, $key)`: 获取值
- `map-merge($map1, $map2)`: 合并映射
- `map-remove($map, $keys...)`: 删除键
- `map-keys($map)`: 所有键
- `map-values($map)`: 所有值
- `map-has-key($map, $key)`: 是否包含键

### 11.6 布尔值 (Booleans)
- `true`
- `false`

### 11.7 Null
- `null`: 表示空值
- 未赋值的变量默认为 `null`

### 11.8 计算 (Calculations)
```scss
$width: 100px + 50px;       // 150px
$height: 200px - 50px;      // 150px
$size: 10px * 2;            // 20px
$gap: 100px / 4;            // 25px
```

### 11.9 函数引用
```scss
$fn: get-function("rgb");
```

---

## 12. 运算符 Operators

### 12.1 相等运算符
```scss
$a == $b    // 相等
$a != $b    // 不相等
```

### 12.2 关系运算符
```scss
$a < $b     // 小于
$a <= $b    // 小于等于
$a > $b     // 大于
$a >= $b    // 大于等于
```

### 12.3 数字运算符
```scss
$a + $b     // 加法
$a - $b     // 减法
$a * $b     // 乘法
$a / $b     // 除法
$a % $b     // 取模
```

### 12.4 布尔运算符
```scss
$a and $b   // 逻辑与
$a or $b    // 逻辑或
not $a      // 逻辑非
```

### 12.5 字符串运算符
```scss
$a + $b     // 字符串拼接
```

### 12.6 运算优先级 (从高到低)
1. 一元运算符: `not`, `+`, `-`, `/`
2. `*`, `/`, `%`
3. `+`, `-`
4. `<`, `<=`, `>`, `>=`
5. `==`, `!=`
6. `and`
7. `or`
8. `=`

### 12.7 除法运算符的特殊规则
- 当 `/` 用于两个带单位的数字时,需要括号或变量来明确表示除法:
```scss
width: (100px / 4);    // 正确
width: $width / 4;     // 正确
width: 100px / 4;      // 错误,CSS 会将其视为列表
```

---

## 13. 父选择器 &

### 13.1 基本用法
```scss
a {
  color: blue;
  &:hover { color: red; }
  &:focus { color: green; }
}
```

### 13.2 在父选择器后添加
```scss
body.dark & {
  color: white;
}
```

### 13.3 在 Mixin 中使用
```scss
@mixin rtl($property, $ltr-value, $rtl-value) {
  #{$property}: $ltr-value;
  [dir=rtl] & {
    #{$property}: $rtl-value;
  }
}
```

---

## 14. 占位符选择器 %

### 14.1 定义
```scss
%button {
  padding: 10px 20px;
  border-radius: 4px;
}
```

### 14.2 使用 @extend
```scss
.button-primary {
  @extend %button;
  background: blue;
}

.button-danger {
  @extend %button;
  background: red;
}
```

编译为:
```css
.button-primary, .button-danger {
  padding: 10px 20px;
  border-radius: 4px;
}
.button-primary { background: blue; }
.button-danger { background: red; }
```

### 14.3 与 class 的区别
- 占位符选择器不会出现在编译后的 CSS 中
- 只有被 @extend 使用时才会生成代码

---

## 15. 内置模块 Built-In Modules

### 15.1 sass:color
```scss
@use "sass:color";

color.adjust($color, $lightness: 10%);
color.mix($color1, $color2, $weight: 50%);
color.invert($color);
color.grayscale($color);
color.alpha($color);
color.hue($color);
color.saturation($color);
color.lightness($color);
color.red($color);
color.green($color);
color.blue($color);
color.change($color, $red: null, $green: null, $blue: null);
color.scale($color, $red: null, $green: null, $blue: null);
color.ie-hex-str($color);
```

### 15.2 sass:list
```scss
@use "sass:list";

list.append($list, $val, $separator: auto);
list.index($list, $value);
list.is-bracketed($list);
list.join($list1, $list2, $separator: auto, $bracketed: auto);
list.length($list);
list.separator($list);
list.nth($list, $n);
list.set-nth($list, $n, $value);
list.slash($list1, $list2);
list.zip($lists...);
```

### 15.3 sass:map
```scss
@use "sass:map";

map.get($map, $key...);
map.has-key($map, $key...);
map.keys($map);
map.merge($map1, $map2);
map.remove($map, $keys...);
map.set($map, $key, $value);
map.values($map);
```

### 15.4 sass:math
```scss
@use "sass:math";

math.abs($value);
math.ceil($value);
math.clamp($min, $number, $max);
math.compatible($number1, $number2);
math.div($num1, $num2);        // 除法
math.floor($value);
math.is-unitless($number);
math.max($numbers...);
math.min($numbers...);
math.mod($num1, $num2);        // 取模
math.percentage($number);      // 转百分比
math.pow($base, $exponent);
math.random($limit: null);
math.round($value);
math.sqrt($value);
math.unit($number);
math.$e;                       // 自然常数
math.$pi;                      // 圆周率
```

### 15.5 sass:meta
```scss
@use "sass:meta";

meta.calc-args($calc);
meta.calc-name($calc);
meta.call($function, $args...);
meta.content-exists();
meta.feature-exists($feature);
meta.function-exists($function-name);
meta.get-function($name, $css: false);
meta.global-variable-exists($name);
meta.inspect($value);
meta.keywords($args);           // 获取关键字参数
meta.mixin-exists($name);
meta.module-functions($module);
meta.module-variables($module);
meta.type-of($value);
meta.variable-exists($name);
```

### 15.6 sass:selector
```scss
@use "sass:selector";

selector.append($selectors...);
selector.extend($selector, $extendee, $extender);
selector.is-superselector($super, $sub);
selector.nest($selectors...);
selector.parse($selector);
selector.replace($original, $replacement);
selector.simple-selectors($selector);
selector.unify($selector1, $selector2);
```

### 15.7 sass:string
```scss
@use "sass:string";

string.index($string, $substring);
string.insert($string, $substring, $index);
string.length($string);
string.quote($string);
string.slice($string, $start-at, $end-at: -1);
string.split($string, $separator, $limit: null);
string.to-lower-case($string);
string.to-upper-case($string);
string.unique-id();
string.unquote($string);
```

---

## 16. 所有 At-Rules 速查

| At-Rule | 功能 | 语法 |
|---------|------|------|
| `@use` | 加载模块 | `@use "url" [as namespace] [with (config)]` |
| `@forward` | 转发模块 | `@forward "url" [as prefix-*] [hide members] [show members]` |
| `@import` | 导入(遗留) | `@import "url"` |
| `@mixin` | 定义混入 | `@mixin name [(args)] { ... }` |
| `@include` | 使用混入 | `@include name [(args)]` |
| `@function` | 定义函数 | `@function name(args) { ... @return value }` |
| `@return` | 返回值 | `@return expression` |
| `@if` | 条件判断 | `@if expression { ... }` |
| `@else` | 否则 | `@else { ... }` |
| `@else if` | 否则如果 | `@else if expression { ... }` |
| `@each` | 遍历 | `@each $var in $list { ... }` |
| `@for` | 循环 | `@for $var from start through/to end { ... }` |
| `@while` | 当循环 | `@while condition { ... }` |
| `@extend` | 继承 | `@extend selector` |
| `@at-root` | 跳出嵌套 | `@at-root { ... }` |
| `@error` | 抛出错误 | `@error "message"` |
| `@warn` | 警告 | `@warn "message"` |
| `@debug` | 调试输出 | `@debug expression` |
| `@content` | 内容块 | `@content [(args)]` |

---

## 17. 编译输出规则

### 17.1 选择器嵌套
- 父选择器在前,子选择器在后
- 使用空格分隔表示后代关系

### 17.2 属性嵌套
- 父属性名作为前缀,子属性名作为后缀
- 使用连字符连接

### 17.3 变量替换
- 变量在编译时被其值替换
- 插值 `#{}` 返回未引号字符串

### 17.4 注释
- `// 单行注释` 不会出现在编译后的 CSS 中
- `/* 多行注释 */` 会保留在编译后的 CSS 中
- `/*! 重要注释 */` 即使压缩也会保留

### 17.5 输出样式
- `nested`: 嵌套格式(默认)
- `expanded`: 展开格式
- `compact`: 紧凑格式
- `compressed`: 压缩格式

---

## 附录: 编译器实现要点

### A.1 解析优先级
1. 词法分析: 识别 token
2. 语法分析: 构建 AST
3. 语义分析: 类型检查、作用域解析
4. 代码生成: 输出 CSS

### A.2 作用域规则
- 变量作用域: 块级作用域
- 函数作用域: 函数内可见
- 全局作用域: 文件顶层
- 模块作用域: `@use` 创建命名空间

### A.3 类型系统
- 静态类型检查
- 隐式类型转换
- 单位转换规则

### A.4 错误处理
- 语法错误: 解析阶段
- 类型错误: 语义分析阶段
- 运行时错误: 执行阶段(如除以零)

### A.5 性能考虑
- 避免重复计算
- 缓存模块加载结果
- 优化选择器嵌套深度

---

*文档版本: 基于 Sass 1.102.0+ 官方文档*
*生成时间: 2026-09-18*
