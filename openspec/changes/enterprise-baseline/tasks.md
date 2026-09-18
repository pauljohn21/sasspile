# Tasks: Custom Directive Operators (CDDOO)

## Phase 0 — 算子骨架 (已完成)

- [x] **T0.1** 创建 `src/directive/` 模块结构
  - rxrust: 6 个自定义算子文件 + mod.rs
  - spec: 全部 (基础设施)
  - est: 0.5h ✅

- [x] **T0.2** 实现 6 个自定义算子四件套 (Use/Mixin/Include/If/For/Each)
  - rxrust: ObservableType + Observer + CoreObservable 实现
  - spec: use-forward-resolution, mixin-include-apply, each-for-control
  - est: 1h ✅

- [x] **T0.3** 实现 `DirectiveOps` trait — 链式方法 .use_() .mixin() .include() .if_() .for_() .each()
  - rxrust: blanket impl for Observable
  - spec: 全部
  - est: 0.5h ✅

- [x] **T0.4** 整合 `lib.rs` compile / compile_parallel — 自定义算子链 + collect/last 收集
  - rxrust: Local::from_iter → filter_map → scan_map → flat_map → 自定义算子链 → map → collect → last
  - spec: 全部
  - est: 0.5h ✅

## Phase 1 — @use 指令实现

- [ ] **T1.1** 在 `UseObserver::next` 中实现 @use 路径解析 (模块加载)
  - rxrust: UseOp 内部 flat_map(load_module(chars))
  - spec: use-forward-resolution
  - est: 2h

- [ ] **T1.2** 实现 @forward — 转发上游模块的 members
  - rxrust: ForwardOp 或复用 UseOp 内部逻辑
  - spec: use-forward-resolution
  - est: 1.5h

- [ ] **T1.3** 实现 `with` 参数覆盖 (config)
  - rxrust: scan_map 捕获 config map
  - spec: use-forward-resolution
  - est: 1h

## Phase 2 — @mixin / @include 指令

- [ ] **T2.1** 在 `MixinObserver::next` 中实现 @mixin 定义捕获
  - rxrust: next 检测到 @mixin 时注册到内部 HashMap
  - spec: mixin-include-apply
  - est: 1.5h

- [ ] **T2.2** 在 `IncludeObserver::next` 中实现 @include 调用展开 (BEM b/e/m)
  - rxrust: next 查找 mixin 定义, flat_map 展开参数绑定后的 body
  - spec: mixin-include-apply
  - est: 2h

## Phase 3 — @if 条件分支

- [ ] **T3.1** 在 `IfObserver::next` 中实现 @if predicate 求值
  - rxrust: next 走 then-branch 或 else-branch (filter_map)
  - spec: each-for-control
  - est: 1.5h

## Phase 4 — @for / @each 循环展开

- [ ] **T4.1** 在 `ForObserver::next` 中实现 @for 数值循环展开
  - rxrust: next 展开 body N 次 (flat_map(from_iter(range)))
  - spec: each-for-control
  - est: 1.5h

- [ ] **T4.2** 在 `EachObserver::next` 中实现 @each 列表/Map 遍历
  - rxrust: next 展开 body per item (flat_map(from_iter(list)))
  - spec: each-for-control
  - est: 1.5h

## Phase 5 — Runtime Validation

- [ ] **T5.1** 写 `tests/enterprise_element_plus.rs` 端到端测试
  - rxrust: compile("element-plus/index.scss 内容")
  - spec: 全部
  - est: 2h

- [ ] **T5.2** 写 `tests/enterprise_bootstrap.rs` 端到端测试
  - rxrust: compile("bootstrap.scss 内容")
  - spec: scope-global-semantics
  - est: 2h

## Phase 6 — Polish

- [ ] **T6.1** 为每个算子的 `next` 添加 `tracing::info_span!` 上下文
  - rxrust: span!(level="info", "directive", op="use")
  - est: 0.5h

- [ ] **T6.2** Archive change, promote capability to main `openspec/specs/`
  - est: 0.5h

## Summary

| Phase | Status | Est |
|---|---|---|
| 0 算子骨架 | ✅ done | 2.5h |
| 1 @use | ⬜ todo | 4.5h |
| 2 @mixin/@include | ⬜ todo | 3.5h |
| 3 @if | ⬜ todo | 1.5h |
| 4 @for/@each | ⬜ todo | 3h |
| 5 Validation | ⬜ todo | 4h |
| 6 Polish | ⬜ todo | 1h |
| **Total remaining** | | **18h** |
