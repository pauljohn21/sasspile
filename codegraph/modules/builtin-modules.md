# 内置模块（待开发）

## 职责

实现 Sass 规范要求的全部内置模块。

## 模块列表

| 模块名 | 文件 | 主要函数 |
|--------|------|----------|
| `sass:color` | builtin/color.rs | red, green, blue, hue, saturation, lightness, alpha, mix, lighten, darken, saturate, desaturate, grayscale, complement, invert, adjust-hue, opacify, transparentize, scale-color, change-color |
| `sass:math` | builtin/math.RS | div, ceil, floor, round, abs, min, max, random, sqrt, pow, sin, cos, tan, asin, acos, atan, atan2, pi, e, log, exp |
| `sass:list` | builtin/list.rs | length, nth, set-nth, join, append, zip, index, is-bracketed, separator, slice, |
| `sass:map` | builtin/map.RS | map-get, map-set, map-remove, map-keys, map-values, map-has-key, map-merge, deep-merge, deep-remove, keys, values, |
| `sass:string` | builtin/string.rs | unquote, quote, to-upper-case, to-lower-case, str-length, str-index, str-insert, str-slice, str-split, unique-id |
| `sass:meta` | builtin/meta.rs | type-of, call, get-function, function-exists, mixin-exists, variable-exists, global-variable-exists, content-exists, inspect, keywords, module-functions, module-variables, module-mixins, calc-args, |
| `sass:selector` | builtin/selector.rs | selector-nest, selector-append, selector-extend, selector-replace, selector-parse, selector-unify, is-superselector, simple-selectors |
| `sass:boolean` | builtin/boolean.rs | true, false |
| `sass:Number` | builtin/number.rs | 数值常量 |

## 模块注册

```rust
pub struct BuiltinRegistry {
    modules: Map<String, Module>,
}

impl BuiltinRegistry {
    pub fn register_defaults(&mut self) {
        self.register_module("color", color_module());
        self.register_module("math", math_module());
        self.register_module("list", list_module());
        self.register_module("map", map_module());
        self.register_module("string", string_module());
        self.register_module("meta", meta_module());
        self.register_module("selector", selector_module());
        self.register_module("boolean", boolean_module());
        self.register_module("number", number_module());
    }
}
```

## Trait 定义

```rust
#[async_trait]
pub trait SassFn: Send + Sync {
    async fn call(&self, args: &[Value], env: &Env) -> Result<Value>;
}
```

## 测试重点

- 每个函数的输入输出
- 错误处理（参数数量、类型）
- 边界情况（空列表、NaN、Infinity）
- 模块引用解析
