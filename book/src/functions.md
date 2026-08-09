# 内建函数

sasspile 提供了丰富的内建函数，涵盖颜色、字符串、列表、数学等操作。

## 颜色函数

### `rgba(r, g, b, a)`

创建 RGBA 颜色值。

```scss
.box {
    background: rgba(52, 152, 219, 0.8);
}
```

### `darken(color, amount)`

加深颜色。

```scss
.button {
    background: darken(#3498db, 10%);
}
```

### `lighten(color, amount)`

减淡颜色。

```scss
.button {
    background: lighten(#3498db, 10%);
}
```

### `mix(color1, color2, weight)`

混合两种颜色。

```scss
.mix {
    background: mix(#3498db, #e74c3c, 50%);
}
```

### `invert(color)`

反转颜色。

```scss
.invert {
    background: invert(#3498db);
}
```

### `grayscale(color)`

将颜色转为灰度。

```scss
.gray {
    background: grayscale(#3498db);
}
```

## 数学函数

### 基础运算

```scss
.container {
    width: 100% - 20px;
    height: 10px * 2;
    font-size: 16px / 2;
}
```

### `abs(number)`

绝对值。

```scss
.test {
    value: abs(-10); // 10
}
```

### `round(number)`, `ceil(number)`, `floor(number)`

四舍五入、向上取整、向下取整。

```scss
.box {
    padding: round(16.6px); // 17px
    margin: ceil(5.1px); // 6px
}
```

### `min(...)`, `max(...)`

最小值、最大值。

```scss
.container {
    padding: min(10px, 15px);
    font-size: max(14px, 12px);
}
```

### `sqrt(number)`

平方根。

```scss
.box {
    width: sqrt(64px); // 8px
}
```

### `sin(number)`, `cos(number)`, `tan(number)`

三角函数（角度制）。

```scss
.rotate {
    transform: rotate(45deg);
    value: sin(45deg);
}
```

## 字符串函数

### `str-length(string)`

字符串长度。

```scss
.test {
    content: str-length("hello"); // 5
}
```

### `str-index(string, substring)`

查找子串位置。

```scss
.test {
    content: str-index("hello", "e"); // 2
}
```

### `str-slice(string, start, end)`

切片字符串。

```scss
.test {
    content: str-slice("hello", 1, 3); // "hel"
}
```

### `to-upper-case(string)`, `to-lower-case(string)`

大小写转换。

```scss
.test {
    content: to-upper-case("hello"); // "HELLO"
}
```

## 列表函数

### `list-length(list)`

列表长度。

```scss
$colors: (#f00, #0f0, #00f);

.test {
    count: list-length($colors); // 3
}
```

### `nth(list, n)`

获取列表第 n 个元素（从 1 开始）。

```scss
.palette {
    background: nth($colors, 2); // #0f0
}
```

### `append(list, value)`

添加元素到列表末尾。

```scss
$new-list: append($colors, #000);
```

### `join(list1, list2)`

连接两个列表。

```scss
$combined: join($colors, (#fff, #000));
```

## Map 函数

### `map.get(map, key)`

获取 Map 中的值。

```scss
$font-weights: (
    "normal": 400,
    "bold": 700
);

.text {
    font-weight: map.get($font-weights, "bold"); // 700
}
```

### `map.keys(map)`, `map.values(map)`

获取所有键或值。

```scss
.keys {
    content: map.keys($font-weights); // "normal", "bold"
}
```

### `map.merge(map1, map2)`

合并两个 Map。

```scss
$merged: map.merge($font-weights, ("light": 300));
```

## 完整示例

```scss
// ===== 颜色操作 =====
$primary: #3498db;
$danger: #e74c3c;

.button {
    background: $primary;

    &:hover {
        background: darken($primary, 10%);
    }

    &:active {
        background: darken($primary, 15%);
    }
}

.alert {
    background: rgba(231, 76, 60, 0.8);
    border: 1px solid darken($danger, 20%);
}

// ===== 数学运算 =====
.container {
    width: 100% - 20px;
    padding: max(10px, 15px);
    font-size: round(16.6px);
}

// ===== 字符串 =====
.debug {
    content: str-length("hello world");
}
```