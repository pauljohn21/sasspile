# Tasks: main → rx 能力迁移

## 总体策略

> **逐 capability 推进，每个 capability 独立可测试，独立 commit。**
> **每个 capability 必须先读 rxrust 源码, 从框架内部实现。**

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

## Phase 2 — 扩展能力 (待做)

### 2.1 @media 查询合并

- [ ] **T2.1** CssBuilder 跟踪 @media query, 相同 query 合并 children
- [ ] **T2.2** 独立测试: @media 合并后结构正确

### 2.2 @extend 选择器分组

- [ ] **T2.2** CompileState 暂存 extends_queued
- [ ] **T2.3** 规则闭合时应用 extend (选择器合并)
- [ ] **T2.4** 占位符 %placeholder extend 支持

### 2.3 @at-root 提升

- [ ] **T2.3** CssBuilder 遇到 AtRoot 时 children 提升到 output
- [ ] **T2.4** 测试: @at-root 输出到顶层

### 2.4 函数求值

- [ ] **T2.5** 内置函数注册表 (BUILTINS: &[(name, fn)])
- [ ] **T2.6** dispatch_pass 内 try_eval_builtin 替换字面量
- [ ] **T2.7** 内置: lighten/darken/rgba/round/nth 等

### 2.5 @keyframes 格式化

- [ ] **T2.8** render_node 百分比节点特殊格式化
- [ ] **T2.9** 测试: @keyframes 输出结构正确

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

- 单文件 ≤ 500 行
- 0 个 Rc<RefCell> / Arc<Mutex>
- 0 个命令式 for + push
- tracing 在 tap, 不在 map
