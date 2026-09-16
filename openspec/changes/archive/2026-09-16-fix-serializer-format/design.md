## Context

`src/ast.rs` 中 `CssNode::render()` 的当前实现把 Rule 输出为单行：`format!("{selector} {{{inner}}}")`，其中 `inner` 是各 Declaration 渲染后直接拼接 — 于是 `format!("{prop}: {value};")` 也完全不带换行。

sass-spec `core_functions` 测试期望输出格式为：
```css
a {
  b: 0;
}
```

normalize 函数 `sass_spec_detail.rs` 用 `.lines().map(str::trim).filter(empty).join("\n")` 将多行压扁 — 但单行输出和多行输出的 line 数不同，压扁后永远不等。

当前唯一 37/10274 能够通过的 case 要么 expected 本身为空、要么正好单行格式与 spec 单行期望一致。

## Goals / Non-Goals

**Goals:**
- `CssNode::render()` Rule 分支输出为多行：selector + ` {\n` + 每行 decl 前加 2 空格缩进 + `\n}`
- Declaration 自身保持在单行内（`prop: value;`），仅通过外层换行达成多行格式
- 空 Rule (body 为空) 输出 `selector {\n}` 或直接省略（与 dart-sass 一致，可讨论）

**Non-Goals:**
- 不引入完整 pretty-printer / 缩进配置系统
- 不处理 sass-spec `output.css` 自身的所有格式变体（有的注释、有的带空行，normalize 已经处理了差异）
- 不修改 evaluate 阶段的任何逻辑

## Decisions

### Decision 1: 缩进策略

**选择**: 固定 2 空格缩进，与 dart-sass / sass-spec HRX 输出一致

**理由**: sass-spec HRX 文件使用 2 空格缩进；`normalize_css` 已 trim 首尾空白，中间的 2 空格也 trim 掉 — 但 line 仍然是独立 line，join 后保持结构。

### Decision 2: 空 Rule 处理

**选择**: body 为空时输出 `selector {\n}`（保留规则）

**理由**: 保持 AST 语义（空 rule 在 Sass 中是合法的，表示"selector 无声明"）；规范化后 `selector {}` / `selector {\n}` 等线等价。

### Decision 3: Declaration 行尾分号

**选择**: 保留 `;`，与 dart-sass 输出一致

**理由**: sass-spec 期望带分号的输出

## Risks / Trade-offs

- **[Risk]** 已有 `tests/builtins_basic.rs` 或其他测试若直接比对整串可能因格式改变而 fail
- **[Mitigation]** 检查所有现有测试 — 如有直接断言 `a {b: 0;}` 的测试，更新为多行格式或改用 normalize 比较

- **[Risk]** 大型 Bootstrap / Element Plus 编译输出体积微增（每行多一个 `\n` + 2 空格）
- **[Mitigation**: 项目 PRIMARY GATE 是功能而非文件大小，且增加量 ~10% 字符，在可接受范围
