# Tasks: main → rx 能力迁移

## 总体策略

> **逐 capability 推进，每个 capability 独立可测试，独立 commit。**

验证门控：每完成一个 capability，32+ 测试必须全部通过，无回退。

---

## Phase 0 — rx 基建拆分（已完成 ✅）

> pipeline.rs 已 462 行（接近上限），需拆分为子模块但保留 API。

- [x] **T0.1** `selector_stack.rs` — resolve_nested_selector
  - spec: css-ast-construction
  - est: 0.5h
- [x] **T0.2** `finalize.rs` — finalize_collecting
  - spec: css-ast-construction
  - est: 0.5h
- [x] **T0.3** `format.rs` — format_css
  - spec: css-serializer
  - est: 0.3h
- [x] **T0.4** 运行全套测试 32/32 通过
  - est: 0.2h

---

## Phase 1 — CSS AST 构建（css-ast-construction）

**目标**: 在 dispatch 阶段输出 CssNode 树而非字符串 token。

- [ ] **T1.1** 定义 `CssNode` 枚举 (`src/css/node.rs`)
  ```rust
  pub enum CssNode {
      Rule { selector: String, children: Vec<CssNode> },
      Declaration { property: String, value: String },
      AtRoot { children: Vec<CssNode> },
      AtRule { query: String, children: Vec<CssNode> },
      Comment(String),
  }
  ```
  - spec: css-ast-construction
  - est: 1h

- [ ] **T1.2** 实现 `CssBuilder` (`src/css/builder.rs`)
  - `scan_map(CssBuilder, feed)` 入口
  - 规则嵌套栈管理（push/pop Rule 上下文）
  - 字符串/声明/AtRule 解析
  - spec: css-ast-construction
  - est: 2h

- [ ] **T1.3** 集成到 `compile_pipeline`
  - 替换 `format_css` 前置步骤: CsSBuilder 入口
  - 输出 `Vec<CssNode>` 而非 `Vec<String>`
  - spec: css-ast-construction
  - est: 1h

- [ ] **T1.4** 实现 `render_node(&CssNode) -> String` (serializer)
  - 递归渲染: Rule → `selector {\n  children\n}`
  - 递归渲染: Declaration → `property: value;`
  - 递归渲染: AtRule → `query {\n  children\n}`
  - 递归渲染: AtRoot → children 直出（无 wrapper）
  - spec: css-serializer
  - est: 1.5h

- [ ] **T1.5** 编写测试
  - `tests/css_ast_test.rs`: CssBuilder 过程验证
  - `tests/css_serialize_test.rs`: 渲染输出比对
  - spec: css-ast-construction, css-serializer
  - est: 1h

---

## Phase 2 — Selector Extend（@extend placeholder 传播）

**目标**: 支持 `@extend %placeholder` 和 `@extend .class`。

- [ ] **T2.1** 扩展 `CompileState` — 添加 `extends_queued: Vec<(String, String)>`
  - spec: selector-extend
  - est: 0.3h

- [ ] **T2.2** 在 `dispatch_pass` 中检测 `@extend` token
  - `@extend %placeholder` → push onto extends_queued
  - 暂不消费规则，仅暂存关联
  - spec: selector-extend
  - est: 1h

- [ ] **T2.3** 实现 `apply_extends(nodes: &mut Vec<CssNode>)` post-process
  - 遍历 Vec<CssNode>，对每个 Rule 检查是否有 extenders
  - 将 extender 来源 Rule 的 Declaration 合并进当前 Rule
  - spec: selector-extend
  - est: 2h

- [ ] **T2.4** 处理 placeholder 规则消除
  - 收集在 `%placeholder { ... }` 中定义的 CssNode
  - 不作为顶级输出，仅作为 extend 模板
  - spec: selector-extend
  - est: 1h

- [ ] **T2.5** 编写测试
  - `tests/extend_test.rs`: 单/多 extender, !optional, mixin 内传播
  - spec: selector-extend
  - est: 1h

---

## Phase 3 — 函数求值（function-eval）

**目标**: 实现 color/math/list/string 内建函数 rxrust 纯函数 dispatcher。

- [ ] **T3.1** 构建 builtin 函数表 (`src/eval/builtin.rs`)
  - 类型: `type BuiltinFn = fn(&[String]) -> Option<String>`
  - 实现模块: color/lighten, color/darken, math/round, math/clamp
  - spec: function-eval
  - est: 2h

- [ ] **T3.2** 添加 `try_eval_builtin(line: &str, state: &CompileState) -> Option<String>`
  - 解析 "funcname(args)" 形式的 token
  - 查 builtin 表求值
  - spec: function-eval
  - est: 1h

- [ ] **T3.3** 在 `dispatch_pass` 中集成函数求值
  - 对包含函数调用的属性行，优先调用 try_eval_builtin
  - 成功 → emit 解析后 token；失败 → 原样透传
  - spec: function-eval
  - est: 1h

- [ ] **T3.4** 升级 math 函数 (sqrt, pow, atan2, hypot, clamp, abs)
  - spec: function-eval
  - est: 1.5h

- [ ] **T3.5** 升级 color 函数 (mix, scale, change, adjust, alpha, opacity)
  - spec: function-eval
  - est: 2h

- [ ] **T3.6** 升级 list/string 函数 (nth, join, length, unquote, quote)
  - spec: function-eval
  - est: 1.5h

- [ ] **T3.7** 编写测试
  - `tests/builtin_fn_test.rs`: 每个函数的 input/output 对比
  - spec: function-eval
  - est: 1.5h

---

## Phase 4 — @at-root 支持（at-root-hoisting）

**目标**: 正确处理 `@at-root { ... }` 提升语义。

- [ ] **T4.1** 在 CssNode 中特殊处理 AtRoot
  - CssBuilder.feed 检测 @at-root 并设置下一批子节点为 "hoistable"
  - spec: at-root-hoisting
  - est: 1h

- [ ] **T4.2** render_node 对 AtRoot 直接展开 children
  - spec: css-serializer
  - est: 0.5h

- [ ] **T4.3** 编写测试
  - `tests/atroot_test.rs`: 基础 at-root, (without: media), 嵌套 at-root
  - spec: at-root-hoisting
  - est: 1h

---

## Phase 5 — Module Loading 完善

**目标**: 支持 @use/@forward 从文件系统加载模块。

- [ ] **T5.1** 扩展 `dispatch_pass` @use 分支 — 实际文件 IO 读取
  - 当前 @use 仅消费 token，改为读文件 + parse + 存入 CompileState
  - spec: module-loading
  - est: 1.5h

- [ ] **T5.2** 支持 @forward
  - spec: module-loading
  - est: 1h

- [ ] **T5.3** 支持 with 参数覆盖
  - spec: module-loading
  - est: 1h

- [ ] **T5.4** 提升 use_resolution 测试从 5 → 10 (新增 with, forward, async)
  - spec: module-loading
  - est: 1h

---

## Phase 6 — 集成验证 + Polish

- [ ] **T6.1** 全量重构后跑 `cargo test` 必须 100% 通过
  - 门控: 任何回退必须恢复
  - est: 0.5h

- [ ] **T6.2** cargo clippy --all-targets 零错误
  - est: 0.5h

- [ ] **T6.3** sass-spec 基线快照拍取
  - 跑 `sass_spec_stats` 记录 rx 分支基线
  - est: 0.5h

- [ ] **T6.4** EP (Element Plus) 端到端编译验证
  - 使用 `test_extend_ph.rs` 或新建 EP smoke test
  - est: 1h

- [ ] **T6.5** 文档更新 (README.md, openspec/specs/ 主 specs 同步)
  - est: 1h

---

## Phase 7 — Archive

- [ ] **T7.1** 归档 openspec (移至 `openspec/changes/archive/2026-09-23-main-to-rx-migration/`)
  - est: 0.3h

- [ ] **T7.2** 推送到 origin/rx (SSH)
  - est: 0.2h

---

## 依赖链

```
Phase 1 (css-ast)
  ├─→ Phase 2 (selector-extend)  ← 需要 CssNode tree
  ├─→ Phase 3 (function-eval)    ← 可独立推进
  ├─→ Phase 4 (at-root)          ← 依赖 CssNode
  ├─→ Phase 5 (module-loading)   ← 可独立推进
  └─→ Phase 6 (polish)
```

可并行: Phase 2 + Phase 3 + Phase 5
</longcat_think>
