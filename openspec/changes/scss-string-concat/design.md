## Context

EP 全量对比发现 91/121 文件与官方 dist 不一致。`!global` 作用域修复已解决 BEM `$B` 变量可见性问题，但字符串拼接运算产生乱码输出。

### 问题链定位

```
SCSS:  $sel: '.' + $B + '__' + $unit + ','
                 ↓ eval Op(Add, [String(.), Var(B), String(__), Var(unit), String(,)])
                 ↓ 输出: \.\' \ \+ \a \  \  \  \  \  \ null\ \+ \a \ __\ \+ \a \ separator\ \+ ...
Expected: ".el-breadcrumb__separator,"
```

输出特征分析：
- 每个字符被 `\` 转义：`\.` `\'` `\ ` `\+` `\a` `\,`
- 字符间用空格分隔
- 变量值（`separator`）无转义原样显示
- `null` 出现（变量未解析时的占位符）

### 根因假设

1. **`Add` 运算符将字符串视为 List**：`Value::String(".")` 可能被当作字符序列处理，导致逐字符拆分
2. **`to_string()` 序列化异常**：`Value::String` 的 `Display` impl 可能使用 `Debug`-style escape
3. **中间结果类型错误**：加法中间结果变成 `Value::List` 而非 `Value::String`
4. **`ArgList` 残留**：某些路径生成了 `ArgList(seperator=Comma)` 混合类型

### 代码关键路径

| 路径 | 文件 | 说明 |
|------|------|------|
| `eval_op(Add, ...)` | `src/eval/value/ops.rs` | `+` 运算符分派 |
| `Value::Add` `(String, String)` | 同上 | 字符串连接分支 |
| `CalcNode::Op { op: Add, ... }` | `src/eval/value/calc_ast.rs` | calc 表达式中的 Add |
| `eval_interp_str` | `src/eval/value/display.rs` | `#{}` 插值求值 |
| `eval_simple_expr` | 同上 | 简单表达式求值 |

### 约束

- 不可参照 dart-sass 实现
- 修复不能破坏现有核心测试 202/202 + sass-spec 66% 基线
- 函数式风格（move 语义 + 无 clone 滥用）

## Goals / Non-Goals

**Goals:**
- `'a' + 'b'` → `"ab"` 正确连接
- `$a + '.' + $b` → `"value_a.value_b"` 正确连接
- EP 一致性从 21/121 (17.4%) 显著提升
- sass-spec 中 `values/strings` + `values/calculation` 相关 case 不受影响

**Non-Goals:**
- 不改变数字算术运算逻辑
- 不修改内建字符串函数（`unquote`/`quote`/`str-insert` 等）
- 不处理 `{` `}` 插值语法之外的特殊场景

## Decisions

### Decision 1: 定位 `Add` 字符串分支

**现状**: `ops.rs` 中 `Add` 操作可能将 `(String, String)` 分派到错误的路径，或返回了 `ArgList` 而非 `String`。

**选择**:
1. 在 `eval_op(Add)` 入口添加 trace 日志记录左右操作数类型
2. 构造最小 repro：`@debug 'a' + 'b'`、`@debug '.' + $var + '__'`
3. 确认哪个路径产出乱码

### Decision 2: 修复字符串连接语义

根据 trace 结果，修复方向可能是：
- 路径 A：`String + String` → 直接 `format!("{a}{b}")` 返回 `Value::String`
- 路径 B：如果创建了 `ArgList`，改为 `Value::String`
- 路径 C：修复中间值的 `to_string()` / 序列化逻辑

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| 修复影响 sass-spec 现有行为 | 全量 sass-spec 回归 |
| 更改字符串化逻辑影响其他 operator | 专项测试覆盖 |
| ArgList 修复影响函数参数传递 | 回归测试确认 |

## Migration Plan

1. **诊断阶段**: 插桩 `ops.rs` 的 Add 分支 + 最小 repro
2. **修复阶段**: 根据证据修改字符串连接逻辑
3. **回归验证**: `cargo test --tests` + `cargo test --test ep_normalized_test` + sass-spec
4. **清理阶段**: 降级临时 span

## Open Questions

1. `ops.rs` 中 `Add` 的 `(String, String)` 分支是否存在？
2. 乱码中的 `\a` `\.` 等 escape 序列由哪个格式化器产生？
3. 中间结果类型是 `String`、`List` 还是 `ArgList`？
4. EP 的 BEM 代码是否有特殊的字符串处理函数（如 `selector-nest`、`selector-append`）被错误分派？
