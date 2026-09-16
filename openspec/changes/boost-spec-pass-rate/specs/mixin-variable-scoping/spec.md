## mixin-variable-scoping

Mixin 参数通过局部作用域链注入，遵守词法作用域语义。

### Scenario: 参数正确替换

**Given** SCSS 源码：
```scss
@mixin pad($x) {
  padding: $x;
}
.box { @include pad(10px); }
```

**When** 编译通过 sasspile_rx::compile

**Then** 输出包含 `padding: 10px;`

### Scenario: 多个参数空格分隔

**Given**:
```scss
@mixin dual($a, $b) {
  margin: $a $b;
}
.box { @include dual(5px 10px); }
```

**Then** `args` 应为 `["5px", "10px"]`，输出 `margin: 5px 10px;`

### Scenario: 参数含默认值

**Given**:
```scss
@mixin shadow($blur: 5px) {
  box-shadow: 0 0 $blur;
}
.box { @include shadow; }
  // 注意：括号省略时 args=[]
```

**Then** 输出 `box-shadow: 0 0 5px;`（使用默认值）

### Scenario: 嵌套 mixin 隔离

**Given**:
```scss
@mixin a($x) { width: $x; }
@mixin b($x) { @include a($x); height: $x; }
.box { @include b(10px); }
```

**Then** `.box` 同时展开为 `width: 10px; height: 10px;`（不被全局变量污染）

### Scenario: 参数作用域不泄漏

**Given**:
```scss
$x: global;
@mixin m($x) { content: $x; }
.box { @include m(local); }
```

**Then** mixin body 中 `$x` 为 `local`，全局 `$x` 保持 `global`
