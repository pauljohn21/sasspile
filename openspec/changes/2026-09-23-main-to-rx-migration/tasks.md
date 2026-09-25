# Tasks: main → rx 能力迁移

## 总体策略

> **逐 capability 推进，每个 capability 独立可测试，独立 commit。**
> **每个 capability 必须先读 rxrust 源码, 从框架内部找答案。**

验证门控：每完成一个 capability，核心测试必须全部通过，无回退。

---

## Phase 0 — rx 基建 (已完成 ✅)

- [x] **T0.1** 创建模块结构 (css/, directive/)
- [x] **T0.2** CssNode AST 定义 (src/css/node.rs)
- [x] **T0.3** CssBuilder scan_map reducer (src/css/builder.rs)
- [x] **T0.4** compile_pipeline 入口 (src/directive/pipeline.rs)
- [x] **T0.5** 消除 Arc<Mutex> GC 模式, 改用 scan_map 状态机 + oneshot 终端

---

## Phase 1 — 指令展开 + CSS AST + 渲染 (已完成 ✅)

- [x] **T1.1** CssNode 枚举 (Rule/Declaration/AtRoot/AtRule/Comment)
- [x] **T1.2** CssBuilder.feed scan_map reducer
- [x] **T1.3** render_node AST→String (借用 &CssNode, 零 clone)
- [x] **T1.4** dispatch_pass 选择器嵌套展开 (& 替换)
- [x] **T1.5** for/each/mixin/include 指令展开
- [x] **T1.6** 三阶段管线: scan_map(CompileState) → flat_map → scan_map(CssBuilder) → flat_map → map(render) → collect/last → oneshot

---

## Phase 2 — 扩展能力 (已完成 ✅)

### 2.1 @media 查询合并

- [x] **T2.1** 相同 @media query 合并 children → `merge_media_nodes` (pipeline.rs)
- [x] **T2.2** 独立测试: @media 合并后结构正确 → `at_root_media_test.rs::media_query_merge`

### 2.2 @extend 选择器分组

- [x] **T2.3** CompileState 暂存 extends_queued + placeholder_defs
- [x] **T2.4** 规则闭合时应用 extend (选择器合并) → `resolve_extend_markers`
- [x] **T2.5** 占位符 %placeholder extend 支持 → `handle_inline_extend`
- [x] **T2.6** 选择器级 @extend → `build_extend_marker` + `CssNode::ExtendMarker`
- [x] **T2.7** 测试: 链式 extend、多目标、!optional → `placeholder_extend_test.rs`

### 2.3 @at-root 提升

- [x] **T2.8** CssBuilder 遇到 AtRoot 时 children 提升到 output → `RuleFrame.at_root` flag
- [x] **T2.9** 测试: @at-root 输出到顶层 → `at_root_media_test.rs::at_root_hoists_to_top_level`

### 2.4 函数求值

- [x] **T2.10** 内置函数注册表 (BUILTINS: &[(name, fn)]) → `src/eval/mod.rs`
- [x] **T2.11** dispatch_pass 内 try_eval_builtin 替换字面量 → `substitute_vars` 调用 `eval_all_calls`
- [x] **T2.12** 内置: lighten/darken/rgba/round/nth 等全套实现
- [x] **T2.13** 独立测试 → `builtins_test.rs` (22 测试) + `color_builtins_test.rs` (9 测试)

### 2.5 @keyframes 格式化

- [x] **T2.14** @keyframes 作为 AtRule 通用处理 (结构保留)
- [x] **T2.15** 测试: @keyframes 输出结构正确 → `comprehensive_e2e_test.rs::keyframes_full_structure`

---

## Phase 3 — 模块结构规范化 (已完成 ✅)

### 3.1 ops.rs 拆分

- [x] **T3.1** 原 ops.rs (610 行) 拆分为以下子模块:
  - `ops.rs` (248 行) — process_block 入口 + process_line + include 展开
  - `extend_ops.rs` (145 行) — @extend 标记构建、inline extend 处理、placeholder 提取
  - `while_ops.rs` (136 行) — @while 展开、@if 条件求值
- [x] **T3.2** 全部源文件 ≤ 500 行约束满足
- [x] **T3.3** 拆分后 32 个测试全通过，零回退

---

## 开发约定

### 源码优先

写代码前必须先读:
- `src/observable.rs` — 算子签名
- `src/factory.rs` — 创建方式
- `src/subscription/source_with_dynamic.rs` — TaskHandle 嵌套

### 所有权规则

- scan_map 算子管理状态 (CompileState, CssBuilder)
- flat_map 有序组合 (Shared::from_iter)
- 终端 oneshot channel (非 Arc<Mutex>)
- render_node(&node) 借用

### 质量门控

- 单文件 ≤ 500 行 ✅ 全部满足
- 0 个 Rc<RefCell> / Arc<Mutex> ✅
- 0 个命令式 for + push ✅ (管线全用算子链)
- tracing 在 tap, 不在 map ✅
