# 嵌套规则

sasspile 支持选择器嵌套，让你可以更自然地组织 CSS 规则。

## 基础嵌套

```scss
.navbar {
    background: #333;

    .logo {
        font-size: 24px;
        color: white;
    }

    .menu {
        display: flex;
    }
}
```

编译为：

```css
.navbar {
  background: #333;
}
.navbar .logo {
  font-size: 24px;
  color: white;
}
.navbar .menu {
  display: flex;
}
```

## 父选择器引用

使用 `&` 引用父选择器，常用于伪类和伪元素：

```scss
.button {
    background: #3498db;
    color: white;

    &:hover {
        background: darken(#3498db, 10%);
    }

    &:active {
        transform: scale(0.95);
    }

    &::before {
        content: "→";
    }
}
```

编译为：

```css
.button {
  background: #3498db;
  color: white;
}
.button:hover {
  background: #2980b9;
}
.button:active {
  transform: scale(0.95);
}
.button::before {
  content: "→";
}
```

## 多层嵌套

```scss
.container {
    width: 100%;

    .header {
        background: #f0f0f0;
        padding: 10px;

        .title {
            font-size: 24px;
        }

        .subtitle {
            font-size: 14px;
            color: #666;
        }
    }

    .content {
        padding: 20px;
    }
}
```

## 嵌套媒体查询

```scss
.responsive {
    width: 100%;

    @media (max-width: 768px) {
        width: 100%;
        font-size: 14px;
    }

    @media (min-width: 769px) {
        width: 80%;
        margin: 0 auto;
    }
}
```

编译为：

```css
.responsive {
  width: 100%;
}
@media (max-width: 768px) {
  .responsive {
    width: 100%;
    font-size: 14px;
  }
}
@media (min-width: 769px) {
  .responsive {
    width: 80%;
    margin: 0 auto;
  }
}
```

## 组合使用

```scss
.card {
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);

    .card-header {
        padding: 10px;
        border-bottom: 1px solid #eee;

        h2 {
            margin: 0;
            font-size: 18px;
        }
    }

    .card-body {
        padding: 15px;

        p {
            margin: 0 0 10px 0;
            line-height: 1.6;

            &:last-child {
                margin-bottom: 0;
            }
        }
    }
}
```