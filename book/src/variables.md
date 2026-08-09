# 变量系统

sasspile 支持完整的 SCSS 变量系统，包括变量定义、使用和作用域。

## 定义变量

使用 `$` 前缀定义变量：

```scss
$primary: #3498db;
$font-size: 16px;
$spacing: 10px;
```

## 使用变量

在属性值中直接引用变量：

```scss
.button {
    background: $primary;
    font-size: $font-size;
    padding: $spacing * 2;
}
```

编译为：

```css
.button {
  background: #3498db;
  font-size: 16px;
  padding: 20px;
}
```

## 变量类型

sasspile 支持以下类型的变量值：

### 颜色

```scss
$color1: #3498db;
$color2: rgb(52, 152, 219);
$color3: rgba(52, 152, 219, 0.8);
```

### 数字（带单位）

```scss
$width: 100px;
$height: 50%;
$margin: 1em;
```

### 字符串

```scss
$message: "Hello, World!";
$name: 'Sass';
```

### 布尔值

```scss
$enabled: true;
$disabled: false;
```

### 列表

```scss
$colors: (#f00, #0f0, #00f);
$sizes: (small, medium, large);
```

## 变量运算

变量可以参与数学运算：

```scss
$base-size: 16px;
$spacing: $base-size / 2;
$padding: $spacing * 2;
```

## 变量作用域

sasspile 支持变量作用域：

```scss
$global: red;

.card {
    background: $global; // red
    $local: blue;

    .title {
        color: $local; // blue
        background: $global; // red
    }
}
```

## 示例：配置文件

```scss
// ===== 配置 =====
$primary: #3498db;
$secondary: #2ecc71;
$danger: #e74c3c;
$text-dark: #333;
$text-light: #fff;
$spacing-unit: 8px;

// ===== 使用 =====
.button {
    background: $primary;
    color: $text-light;
    padding: $spacing-unit * 2;

    &:hover {
        background: darken($primary, 10%);
    }
}
```