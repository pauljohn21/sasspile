# 函数式 Rust 优先级表

> **触发场景**：任何 Rust 开发、调试、测试任务。在生成代码前必须查阅此表。

当多个方案冲突时，按此优先级选择（高 → 低）：

## 🏆 决策优先级

| 优先级 | 原则 | 示例 |
|--------|------|------|
| 1 | **move 语义** | `fn f(env: Env) -> (Output, Env)` |
| 2 | **迭代器链** | `vec.into_iter().map(f).collect()` |
| 3 | **match** | `match x { A => .., B => .. }` |
| 4 | **? 传播** | `let x = parse(tokens)?;` |
| 5 | **try_fold** | `vec.into_iter().try_fold(init, fold_fn)` |
| 6 | **partition** | `vec.into_iter().partition(pred)` |
| 7 | **flat_map** | `nested.into_iter().flat_map(|v| v).collect()` |
| 8 | **self -> Self** | `fn with_x(mut self, x: X) -> Self` |
| 9 | **新类型包装** | `struct Lexed(tokens: Vec<Token>)` |
| 10 | **纯函数 + 返回值** | `fn transform(input: Input) -> Output` |

## ❌ 反模式检测（出现即停止）

| 反模式 | 检测方法 | 正确替代 |
|--------|----------|----------|
| `clone()` 满天飞 | grep `\.clone()` > 3次/函数 | move 语义 + 返回新值 |
| `&mut` 参数 | grep `&mut [A-Z]` | `self -> Self` 或 `(T, State)` 返回 |
| `for + push` | grep `for .* in .* \{.*push` | `.map().collect()` 或 `try_fold` |
| `if-else` 链 > 3 分支 | 检查连续 `else if` | `match` |
| `match Err` 分支 | grep `Err(e) => return` | `?` 操作符 |
| `Rc<RefCell<T>>` | grep `RefCell` | 按值传递 + 返回新值 |
| 可变累积 `Vec` | `let mut result = Vec::new()` | `.collect()` / `fold` |

## 📐 类型签名模板

| 场景 | 签名 | 说明 |
|------|------|------|
| 数据变换 | `fn transform(input: Input) -> Output` | 消费输入，返回新值 |
| 带状态变换 | `fn step(state: State, input: Input) -> (Output, State)` | move 语义，返回新状态 |
| 管线阶段 | `fn next_stage(self) -> Result<NextStage>` | `self` 消费，类型状态机 |
| 链式构建 | `fn with_x(mut self, x: X) -> Self` | builder 模式 |
| 只读查询 | `fn query(&self, key: &str) -> Option<&Value>` | 纯函数，不可变借用 |
| 累积迭代 | `fn fold_fn(acc: Acc, item: T) -> Result<Acc>` | try_fold 回调 |

## 🔄 正反对比速查

### 所有权

```rust
// ❌ clone + 修改
fn eval(nodes: &[Node], env: &Env) -> Vec<CssNode> {
    let mut env = env.clone();
    env.bind("x", 1.0);
    // ...
}

// ✅ move + 返回新状态
fn eval(nodes: &[Node], env: Env) -> (Vec<CssNode>, Env) {
    let env = env.bind("x", 1.0);  // bind: self -> Self
    // ...
}
```

### 迭代器

```rust
// ❌ 命令式累积
let mut result = Vec::new();
for node in nodes {
    if node.is_css() {
        result.push(transform(node));
    }
}

// ✅ 函数式链
nodes.into_iter()
    .filter(|n| n.is_css())
    .map(transform)
    .collect::<Vec<_>>()

// ✅ 带错误传播
nodes.into_iter()
    .try_fold(Vec::new(), |mut acc, node| {
        acc.push(transform(&node)?);
        Ok::<_, SassError>(acc)
    })
```

### 分流

```rust
// ❌ 两个可变 Vec + for + if
let (mut css, mut errors) = (Vec::new(), Vec::new());
for node in nodes {
    if node.is_valid() { css.push(node); }
    else { errors.push(node); }
}

// ✅ partition
let (css, errors): (Vec<_>, Vec<_>) = nodes.into_iter()
    .partition(|n| n.is_valid());
```

### 错误处理

```rust
// ❌ 显式 match Err
let result = match parse(tokens) {
    Ok(ast) => ast,
    Err(e) => return Err(e),
};

// ✅ ? 传播
let ast = parse(tokens)?;
```

### 枚举分派

```rust
// ❌ if-else 链
if name == "media" { ... }
else if name == "supports" { ... }
else if name == "keyframes" { ... }
else { ... }

// ✅ match
match name {
    "media" => ...,
    "supports" => ...,
    "keyframes" => ...,
    _ => ...,
}
```

## 📋 自检清单

生成代码后逐项检查：

- [ ] 无 `clone()` 满天飞（理解所有权设计后再用）
- [ ] 无 `&mut` 参数（改用 `self -> Self` 或 `(T, State)` 返回）
- [ ] 集合变换用 `map/filter/collect` 而非 `for + push`
- [ ] 枚举分派用 `match` 而非 `if-else` 链
- [ ] 错误传播用 `?` 而非 `match ... Err(e) => return`
- [ ] 状态变更返回新值（`self -> Self`）而非 `&mut self`
- [ ] 管线阶段消费 `self`（类型状态机）而非 `&self` + clone
- [ ] 累积操作用 `try_fold` / `fold` 而非可变 `Vec` + push
- [ ] 分流用 `partition` 而非两个 `Vec` + for + if
