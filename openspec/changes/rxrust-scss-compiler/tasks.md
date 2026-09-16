# rxrust-scss-compiler 实现任务

> ***优先级 P0 = Enterprise 100% first.***
> Bootstrap + Element Plus 全量编译通过是首要验收目标 (BLOCKING)。
> sass-spec 是 P2 锦上添花,不得阻塞 P0 达成。

## 0. 🥇 Enterprise Baseline [P0 — BLOCKING TRUMPS ALL]

Bootstrap + Element Plus 端到端编译通过 → 100% = sasspile-rx 可用。

- [ ] 0.1 运行 `cargo run --bin sasspile_tracker -- enterprise` 拍当前基线 (Bootstrap 0%/EP 0%)
- [ ] 0.2 Bootstrap `bootstrap/scss/bootstrap.scss` 端到端编译无错误 (`@import` 全局共享)
- [ ] 0.3 Element Plus `element-plus/packages/theme-chalk/src/index.scss` 端到端编译无错误 (`@use as *` + `!global`)
- [ ] 0.4 验证 sass-spec 活跃目录 + Bootstrap + Element Plus 三者的合成通过率:Enterprise 100% 必须首先达成
- [ ] 0.5 每次实现 feature 后重跑 `cargo run --bin sasspile_tracker -- enterprise`,记录提升

> � 验收条件:`ENTERPRISE GATE PASSED — Bootstrap + Element Plus 100%` 显示在 tracker 输出。
> 在这之前,**不允许**推进 sass-spec 用例数。

---

## 1. rxrust Pipeline Skeleton（从零构建）

- [ ] 1.1 Create `src/pipeline.rs` — `Local::<str, Infallible>::from_iter(src).pipe(...).last().subscribe(...)` 主干
- [ ] 1.2 Create `src/error.rs` — `CompileError` 枚举(`MissingValue`/`InvalidInput`/`UndefinedVariable`/`...`)
- [ ] 1.3 Create `src/tokenize_dst.rs` — `TokenizerObserver<O, State>` + `Tokenizer<S, State>` (CoreObservable 实现)
- [ ] 1.4 Create `src/parse_dst.rs` — `ParserObserver<O, ASTBuilder>` + `Parser<S, State>` (CoreObservable 实现)
- [ ] 1.5 Create `src/evaluate_dst.rs` — `EvaluatorObserver<O, Scope>` + flat_map dispatch
- [ ] 1.6 Create `src/serialize_dst.rs` — `SerializerObserver<O>` + map 渲染
- [ ] 1.7 每个 stage 返回后 `.box_it()` 类型擦除,pipeline.rs 中链式 `.pipe(stage).box_it()`
- [ ] 1.8 添加 `Create rxrust stage with Observer wrap pattern` 文档/注释

## 2. rxrust 算子整合

- [ ] 2.1 Transformation: 所有 stage 内部必须使用 `scan` (累积) / `map` (转换) / `flat_map` (嵌套展开)
- [ ] 2.2 Filtering: 使用 `filter` 过滤空 Token,`distinct` 去重选择器
- [ ] 2.3 Utility: 使用 `tap` 注入所有 tracing/副作用,`finalize` 释放资源
- [ ] 2.4 Aggregation: `.last()` / `reduce()` 收敛为 `Result<String, CompileError>`
- [ ] 2.5 Connectable: `shared/module.rs` 使用 `.publish(Local::subject()).ref_count()` 共享模块缓存
- [ ] 2.6 Creation: `Local::from_iter` / `Local::of` / `Local::subject` 作为源的创建方式
- [ ] 2.7 每个新 operator 类别使用时必须有对应的单元测试覆盖

## 3. Tracing via tap（rxrust 副作用约定）

- [ ] 3.1 添加 `tracing` 依赖到 Cargo.toml
- [ ] 3.2 移除所有在 `map`/`scan` 闭包内的 tracing 调用
- [ ] 3.3 改用 `.tap(|item| debug!(?item, stage = "..."))` 桥接
- [ ] 3.4 定义 span 层级约定:`info_span!("sasspile", stage = %"tokenize")` 入口 → `debug_span!("sasspile.feed", module = %"...")` 子工序
- [ ] 3.5 每个 stage entry/exit MUST 在 tap 中记录 span
- [ ] 3.6 添加 `elapsed_ms` 字段到 pipeline 级 span

## 4. Error-as-Value（Infallible 管道约定）

- [ ] 4.1 所有 stage 函数签名 MUST 使用 `Infallible` 作为 Observable Err 类型
- [ ] 4.2 evaluate .stage MUST 输出 `Observable<Result<CssNode, CompileError>, Infallible>`
- [ ] 4.3 所有 `unwrap()` / `expect()` 在 parse/evaluate 中被替换为 `Result::Err`
- [ ] 4.4 `.last()` 后 subscribe 闭包 MUST 通过 `match result { Ok(css) => ..., Err(e) => ... }` 显式处理错误

## 5. Shared Context（rxrust scan + Connectable 模式）

- [ ] 5.1 Create `src/shared/context.rs` — `CompilerContext` + scan 传播模式
- [ ] 5.2 Create `src/shared/scope.rs` — `Scope` 类型(变量存储)
- [ ] 5.3 Create `src/shared/module.rs` — `EvaluatedModule` + `publish/ref_count` 共享
- [ ] 5.4 CompilerContext MUST 通过 `scan(CompilerCtx::new(), apply)` 在管道内传播
- [ ] 5.5 module_cache MUST 使用 `MutRc<HashMap<PathBuf, EvaluatedModule>>` (Local scope)
- [ ] 5.6 多订阅者模块共享 MUST 通过 `publish(Local::subject()).ref_count()` 实现
- [ ] 5.7 测试:确保同一模块路径第二次 `@use` 命中缓存(不重复 parse)

## 6. Unit Tests（tests/ 目录,rxrust-first）

- [ ] 6.1 Create `tests/tokenize.rs` — 使用 `#[rxrust_macro::test(local)]` 验证 Token 流
- [ ] 6.2 Create `tests/parse.rs` — 验证 scan 累积逻辑 + Node 产出
- [ ] 6.3 Create `tests/evaluate.rs` — 验证 flat_map 指令展开 + 错误传播
- [ ] 6.4 Create `tests/serialize.rs` — 验证 map 渲染输出 char 序列
- [ ] 6.5 Create `tests/integration.rs` — 完整 pipeline subscribe → Result<Css, Error>
- [ ] 6.6 Create `tests/operators.rs` — 验证每个 rxrust 算子类别在编译器中的应用正确性
- [ ] 6.7 每个测试 MUST 从 `Observable` 构造开始,不依赖全局状态

## 7. sass-spec Grammar Coverage (P2 锦上添花 — after P0 done)

> ⚠️ 仅在 Group 0 Enterprise Gate 通过后才开始本组。

- [ ] 7.1 选取首个 sass-spec HRX 文件(如 `core_functions/color/rgb/three_args/basic.hrx`)做 pipeline trace
- [ ] 7.2 验证输出与 HRX-defined expected output 匹配
- [ ] 7.3 自检:实现过程 MUST NOT 参考 dart-sass 源码
- [ ] 7.4 创建 spec 目录白名单:`core_functions/`、`directives/`、`css/`、`expressions/`、`parser/`、`values/`、`variables/`、`operators/`
- [ ] 7.5 测试 walker MUST 显式过滤 `libsass/`、`libsass-closed-issues/`、`libsass-todo-issues/`、`libsass-todo-tests/`、`non_conformant/`

## 8. Enterprise 100% 验收 (moved — see Group 0 above)

> 为了强调优先级,企业基线已被提升为 Group 0。本组作归档。

- [x] 8.1 Bootstrap shallow clone 注册为 git submodule(已完成)
- [x] 8.2 Element Plus shallow clone 注册为 git submodule(已完成)
- [ ] ~~8.3 编译 `bootstrap/scss/bootstrap.scss`~~ → Group 0.2
- [ ] ~~8.4 编译 `element-plus/packages/theme-chalk/src/index.scss`~~ → Group 0.3
- [ ] ~~8.5 合成通过率 = 100%~~ → Group 0.4
