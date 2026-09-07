# Tasks: Reactor Pipeline

> **Status**: ✅ COMPLETED (2026-09-07)
> 
> Phase 1-3 + 5 完成, Phase 4 (Color 链式 API) 延后。

## 阶段 1: Reactor 核心骨架 ✅

### 1.1 Reactor 类型定义与管线方法 ✅
- [x] 定义 `Reactor` struct (text, base_path, load_paths, tokens, ast, serialized, env, modules, io, trace)
- [x] 定义 `ReactorIO` trait + `DefaultReactorIO` + `MockReactorIO`
- [x] 实现 `Reactor::new(text)` 与 `Reactor::from_file(path)`
- [x] 实现 `.lex() -> Result<Reactor<StateLexed>>` — 直接调用 `Lexer`
- [x] 实现 `.parse() -> Result<Reactor<StateParsed>>` — 直接调用 `Parser`
- [x] 实现 `.evaluate() -> Result<Reactor<StateEvaluated>>` — 直接调用 `Evaluator::evaluate_with_env`
- [x] 实现 `.serialize(style) -> Reactor<StateSerialized>` — 直接调用 `Serializer`
- [x] 实现 `.finish() -> Result<String>`
- [x] 类型状态机: `Reactor<StateRaw/Lexed/Parsed/Evaluated/Serialized>`

### 1.2 OTel 链式追踪集成 ✅
- [x] 定义 `ReactorTrace` / `CompileStage` / `IoRecord` 类型
- [x] 为每个管线方法添加 `#[instrument]` span 标注
- [x] 实现 `ReactorTrace::advance(stage)` 阶段推进
- [x] 实现 `ReactorSnapshot` 用于测试断言
- [x] 验证 `cargo test --features otel` 输出包含正确 span 层级

## 阶段 2: Env/Scope 持久化改造 ✅

### 2.1 Scope 使用 imbl HashMap ✅
- [x] `Scope` 全部字段改用 `imbl::HashMap` (7 个 HashMap)
- [x] `Env` 的 `bindings`/`scopes` 改用 `imbl::HashMap` + `imbl::HashSet`
- [x] `ModuleExports` 改用 `imbl::HashMap`
- [x] `bind()` 消费 `self` 返回新 `Env`
- [x] `enter_scope()` / `exit_scope()` 基于 Rc clone 零拷贝
- [x] `lookup()` 保持 `&self -> Option<&Value>`

### 2.2 保持 Sass 作用域语义 ✅
- [x] 确认 `@if/@for/@each` 不创建新 scope
- [x] 确认 `@mixin`/`@function` 进入时创建新 scope
- [x] 运行 `cargo test --test compile_test` 确认 57/57

## 阶段 3: eval_* 函数纯函数化 ✅ (由高优先级 `functional-cleanup` commit 完成)

- [x] 所有 for+push 序列改为 try_fold + accumulator (76 处)
- [x] if-else 链改为 match (81 处)
- [x] else-if 链→apply_kw 链式
- [x] 运行时 benchmark 对比原 God-object 版本 (在 ±3% 内)

## 阶段 4: Color 操作链式 API ⏭️ 延后

> 延后原因: sass-spec 颜色相关算法复杂度已足够高, 当前 builtin 分派模式工作正常。
> 后续如有需要再单独提案。

- [ ] `impl Color { pub fn scale(self, ...) -> Result<Self> }`
- [ ] `impl Color { pub fn adjust(self, ...) -> Result<Self> }`
- [ ] `impl Color { pub fn change(self, ...) -> Result<Self> }`

## 阶段 5: 收尾与清理 ✅

### 5.1 删除旧 God-object 替代路径 ✅
- [x] 删除 `src/stage/` 整个目录 (Source/Lexed/Parsed/Evaluated/Serialized 类型, ~552 行)
- [x] `Reactor` 直接管理管线数据 (text/base_path/load_paths/tokens/ast/serialized)
- [x] `Reactor` 直接调用 `Lexer` / `Parser` / `Evaluator` / `Serializer`
- [x] 所有内部测试/外部 helper 改用 `compile_expanded` / `Reactor` API

### 5.2 公开 API 迁移 ✅
- [x] `lib.rs` 的 `compile()`/`compile_file()`/`compile_file_with_load_paths()` 委托 `Reactor`
- [x] 公开导出: `Reactor`, `ReactorIO`, `ReactorSnapshot`, `ReactorTrace`, `CompileStage`
- [x] 删除 `pub use stage::source::Source`

### 5.3 性能回归测试 ✅
- [x] benchmark: 8.8ms/28KB 展开式 (与旧 Source 链持平)

### 5.4 验证 ✅
- [x] 新 Reactor 路径单测覆盖 (14 reactor + 16 stage_test)
- [x] 全量 sass-spec: 7140/12131 = 60.0% (基线不变)
- [x] 核心测试: 57/57 + 16/16 + 8/8 + 5/5 + 15/15 + 15/15 + 13/13 + 8/8 + 14/14 = 136/136

## 验收标准 (全部达成)

```bash
cargo test --test compile_test                          # 57/57 ✅
cargo test --test stage_test                            # 16/16 ✅ (Reactor 管线测试)
cargo test --test reactor_test                          # 14/14 ✅
cargo test --test bs_spec -- --nocapture                # 15/15 ✅
cargo test --test ep_full -- --nocapture                # 121/121 ✅
RUST_LOG="sass_spec_full=info" cargo test --test sass_spec_full -- --nocapture  # 7140/12131 ✅
```

## OTel 链式追踪验证

```bash
RUST_LOG=trace cargo test --features otel --test reactor_test -- --nocapture
```

每个管线阶段自动生成 span:
- `stage=lex` → `stage=parse` → `stage=evaluate` → `stage=serialize` → `stage=finish`
- 父子关系由 `#[instrument]` + 链式调用自动建立
- busy_ns / idle_ns 精确到纳秒级
