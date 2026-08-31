# Spec: Nested Output Format

## Requirement

Expanded 模式下，嵌套 CSS 规则保留嵌套结构输出，不展平选择器。

## Scenarios

### Simple Nesting

```scss
// input.scss
a { b { c: d; } }
```

```css
// output.css
a {
  b {
    c: d;
  }
}
```

### & Reference

```scss
// input.scss
a { &.b { c: d; } }
```

```css
// output.css
a {
  &.b {
    c: d;
  }
}
```

### Deep Nesting

```scss
// input.scss
a { b { c { d: e; } } }
```

```css
// output.css
a {
  b {
    c {
      d: e;
    }
  }
}
```

### Mixed Declarations and Nested Rules

```scss
// input.scss
a {
  color: red;
  b {
    color: blue;
  }
  font-size: 10px;
}
```

```css
// output.css
a {
  color: red;
  b {
    color: blue;
  }
  font-size: 10px;
}
```

### @media Nesting

```scss
// input.scss
a {
  @media (min-width: 100px) {
    b { c: d; }
  }
}
```

```css
// output.css
a {
  @media (min-width: 100px) {
    b {
      c: d;
    }
  }
}
```

### Comma Selector Nesting

```scss
// input.scss
a, b { c { d: e; } }
```

```css
// output.css
a, b {
  c {
    d: e;
  }
}
```

## Constraints

- Compressed 模式仍然展平（`flatten_nodes` 只在 compressed 调用）
- `&` 选择器在 expanded 模式保留 `&` 符号
- `@at-root` 内容提升到顶层
