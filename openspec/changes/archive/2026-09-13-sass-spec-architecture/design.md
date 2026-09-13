## Context

sasspile 的 eval 模块目前残留了 dart-sass 的 SCSS/CSS 双模式思维：

1. `eval_import` 检测 `.css` URL 后确实生成了 `@import url(...)` 节点，但当 resolve 返回了磁盘上的 `.css` 路径时，`load_import` 仍然被调用（走统一 SCSS 管线），导致 `.css` 内容被当作 SCSS 解析。

2. `ScssEvaluator` 类型暗示与（已删除的）`CssEvaluator` 对偶，命名残留需要清除。

3. 测试框架将所有 HRX 文件写入同一临时目录，缺乏"output.css 不应被编译器解析"的形式化保证。

## Goals / Non-Goals

**Goals:**
- 修复 `.css` 文件被错误地作为 SCSS 加载的 bug
- 消除 `ScssEvaluator`，统一为 `Evaluator`
- 测试框架区分文件角色，output 不再写入 VFS
- 增加 `@use "foo.css"` / `@forward "foo.css"` 的明确错误

**Non-Goals:**
- 不支持 `@use "foo.css"` 作为 CSS Module（Sass 新规范特性）
- 不改变 `.scss` / `.sass` 的 import 语义
- 不修改 Reactor 类型状态机管线

## Decisions

### D1: 在 `eval_import` 中增加 path-based CSS 检测

**决策**: 在 `eval_import` 的 `is_css` 判断中增加：即使 resolve 返回了一个路径，只要该路径以 `.css` 结尾，仍然走 pass-through，不调用 `load_import`。

**现有代码分析**:
```rust
// import.rs:20-32let is_css = Path::new(url).extension()...;  // URL-based detection
```
这段只检查 URL 字符串。当 `@import "foo.css"` 进入 resolve 并返回 `some/path/foo.css` 后，代码继续执行到 `load_import`。

**修复**: 在 resolve 成功后增加 `path.extension().is_some_and(|e| e == "css")` 检查，如果是 `.css` 则返回 early（复用已有的 is_css pass-through 逻辑生成 `@import url(...)` 节点）。

**替代方案**: 在 resolve_file_import 的候选列表中排除 `.css` 文件 → 被否决，因为这会导致 `@import "foo.css"` 无法解析已存在的 `.css` 文件、产生误报"Not found"。

### D2: `@use` / `@forward` 明确拒绝 `.css`

**决策**: 在 `load_module` 入口增加 URL 后缀检查——如果 URL 以 `.css` 结尾，返回 `SassError::Eval("CSS files can't be @used")`。

**理由**: SCSS 规范明确规定 `@use` / `@forward` 只能加载 Sass 源文件，`.css` 不是合法目标。明确的错误消息优于解析失败的困惑错误。

### D3: 合并 ScssEvaluator 到 Evaluator

**决策**: 将 `ScssEvaluator` 的四个方法（`evaluate` / `evaluate_with_env` / `eval_node` / `eval_nodes`）内联到 `Evaluator` 的实现中，删除 `scss_evaluator.rs` 文件。

调用方分析:
- `eval/mod.rs:20` — `pub use scss_evaluator::ScssEvaluator;`
- `eval/reactor.rs:270` — `ScssEvaluator::evaluate_with_env(&ast, env)`
- `eval/mod.rs:149` — `load_module` 中的间接调用

**方案**: 
1. 删除 `scss_evaluator.rs` 文件
2. `reactor.rs` 中改为直接调用 `Evaluator::evaluate_with_env`
3. `lib.rs pub use` 行删除

### D4: 测试框架文件角色分类

**决策**: 在 `hrx_support.rs` 的 `parse_hrx_to_cases` 中增加文件角色分类，并修改写入逻辑：

```rust
enum FileRole { Input, Output, ScssAsset, CssAsset }

// output.css → role = Output → 存入 case.expected_output，不写入 VFS
// _helper.scss → role = ScssAsset → 写入 VFS
// vendor.css → role = CssAsset → 写入 VFS（仅用于 @import 引用）
```

**实现**: 在 `HrxCase` 结构体中保持 `files: Vec<(String, String)>` 兼容现有接口，但写入前过滤：排除 `output.css`。

## Risks / Trade-offs

| Risk | 影响 | 缓解 |
|------|------|------|
| 现有测试中依赖 output.css 被写入的测试 case | 解析差异 | SPEC_STORE_CMD=run 全量验证 |
| `is_css` 检测改变导致 @import "some.css"（实际不存在文件）行为变化 | 错误消息不同 | DIFF 对比验证 |
| @use "foo.css" 报错改变可能影响 spec 测试 | 某些 spec 测试可能依赖此行为 | diag_runner 验证相关 spec 目录 |

## Migration Plan

1. 修复 `eval_import` 的 `.css` 文件处理
2. 增加 `@use` / `@forward` 对 `.css` 的拒绝
3. 合并 `ScssEvaluator` → `Evaluator`
4. 测试框架 output.css 不写入 VFS
5. SPEC_STORE_CMD=run 全量验证无 regression
6. 202/202 核心测试必须保持通过

## Open Questions

1. 是否有 spec 测试故意使用 `@use "foo.css"` 期望成功？需 diag_runner 检查 `core_functions/meta/module-exports` 相关 case。
2. `output.css` 不写入 VFS 后，是否存在测试 case 中 `@import "output.css"` 的边界情况？需搜索 HRX 文件中是否有此类模式。
