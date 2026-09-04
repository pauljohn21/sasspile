# functional-style-enforcement Spec

## 需求 1: 禁止 else-if 链

系统代码中不得出现 `else if` 语法结构。枚举值分派必须用 `match` 表达式。字符串字面量分派必须用 `match` + `to_lowercase`/`as_str`。

### 1.1 枚举分派

```rust
// 禁止
if x == Foo::A { ... }
else if x == Foo::B { ... }
else { ... }

// 必须
match x {
    Foo::A => ...,
    Foo::B => ...,
    _ => ...,
}
```

### 1.2 字符串分派

```rust
// 禁止
if s.eq_ignore_ascii_case("true") { Token::True }
else if s.eq_ignore_ascii_case("false") { Token::False }
else { Token::Ident(s.to_string()) }

// 必须
match s.to_lowercase().as_str() {
    "true" => Token::True,
    "false" => Token::False,
    _ => Token::Ident(s.to_string()),
}
```

## 需求 2: 禁止 for + push 集合累积

集合变换必须用迭代器链（`map`/`filter`/`collect`/`try_fold`/`flat_map`），禁止 `for x in items { result.push(f(x)) }` 模式。

### 2.1 map 变换
```rust
// 禁止
let mut v = Vec::new();
for x in items { v.push(f(x)); }

// 必须
items.into_iter().map(f).collect::<Vec<_>>()
```

### 2.2 filter + map
```rust
// 必须
items.into_iter().filter(pred).map(f).collect::<Vec<_>>()
```

### 2.3 错误传播累积
```rust
// 必须
items.into_iter().try_fold(Vec::new(), |mut acc, x| {
    acc.push(f(x)?);
    Ok::<_, E>(acc)
})
```

### 2.4 展平
```rust
// 必须
items.into_iter().flat_map(f).collect::<Vec<_>>()
```

## 需求 3: 禁止 if-let 连续修改可变变量

连续的 `if let Some(v) = get(...) { mut_var = f(mut_var, v) }` 模式必须用高阶函数 `apply_kw` 替代。

```rust
// 禁止
let mut r = initial;
if let Some(v) = get_num(kw, "red")? { r += v; }
if let Some(v) = get_num(kw, "green")? { g += v; }

// 必须
let r = apply_kw(initial, kw, "red", |v, d| v + d)?;
let g = apply_kw(initial, kw, "green", |v, d| v + d)?;
```

## 需求 4: 禁止 &mut 参数（Display trait 除外）

函数不得接收 `&mut T` 参数（`std::fmt::Display::fmt` 的 `&mut Formatter` 是标准 trait 签名，保留）。状态变更必须返回新值。

```rust
// 禁止
fn f(buf: &mut String, x: &str) { buf.push_str(x); }

// 必须
fn f(x: &str) -> String { x.to_string() }
```

## 需求 5: while let 保留条件

`while let Some(c) = self.peek()` 在以下条件保留：
- 封装在 `&mut self` 方法内部
- 不作为参数跨函数传递
- 与 peek+next 循环模式一致

## 需求 6: 保留的合理模式

以下模式不视为违规：
- `for _ in 0..N` 固定次数循环（hex 解析等）
- `for window in windows(2)` 滑动窗口
- `Rc::clone` 原子计数器递增
- `Display::fmt` 的 `&mut Formatter`
