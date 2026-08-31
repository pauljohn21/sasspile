# Spec: Quick Fixes

## Requirement

修复参数验证、类型转换、运算符支持中的常见错误。

## Scenarios

### String-to-Number Coercion

```scss
// input.scss
a { b: math.abs("0"); }
```

```css
// output.css
a {
  b: 0;
}
```

### Calc String Concatenation

```scss
// input.scss
$c: calc(100% - 10px);
a { b: $c + 20px; }
```

```css
// output.css
a {
  b: calc(100% - 10px + 20px);
}
```

### Function Argument Validation

```scss
// input.scss
a { b: if(true, 1, 2); }
```

```css
// output.css
a {
  b: 1;
}
```

### set-nth

```scss
// input.scss
a { b: set-nth(c d e, 2, x); }
```

```css
// output.css
a {
  b: c x e;
}
```

## Constraints

- Expected errors remain errors (only fix non-expected failures)
- Error message format matches sass-spec when possible
