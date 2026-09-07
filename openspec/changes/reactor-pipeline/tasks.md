# Tasks: Reactor Pipeline

## 阶段 1: Reactor 核心骨架 (可独立验证)

### 1.1 Reactor 类型定义与管线方法
- [ ] 定义 `Reactor` struct (source, tokens, ast, output, env, modules, io, trace)
- [ ] 定义 `ReactorIO` trait + `DefaultReactorIO` + `MockReactorIO`
- [ ] 实现 `Reactor::new(source)` 与 `Reactor::from_file(path)`
- [ ] 实现 `.lex() -> Result<Reactor<Lexed>>`
- [ ] 实现 `.parse() -> Result<Reactor<Parsed>>`
- [ ] 实现 `.evaluate() -> Result<Reactor<Evaluated>>`
- [ ] 实现 `.serialize(style) -> Reactor<Serialized>`
- [ ] 实现 `.finish() -> Result<String>`
- [ ] 类型状态机: `Reactor<Raw> / Reactor<Lexed> / Reactor<Parsed> / Reactor<Evaluated> / Reactor<Serialized>`

### 1.2 OTel 链式追踪集成
- [ ] 添加 `imbl` 到 Cargo.toml
- [ ] 定义 `ReactorTrace` / `CompileStage` / `IoRecord` 类型
- [ ] 为每个管线方法添加 `#[instrument]` span 标注
- [ ] 为 `eval_block` 内的 `try_fold` 添加每个规则级 span
- [ ] 实现 `ReactorTrace::snapshot()` 用于测试
- [ ] 实现 `ReactorTrace::advance(stage)` 阶段推进
- [ ] 验证 `cargo test --features otel` 输出包含正确 span 层级

## 阶段 2: Env/Scope 持久化改造

### 2.1 Scope 使用 imbl HashMap
- [ ] `Scope` 的 vars/mixins/functions 字段改用 `imbl::HashMap`
- [ ] 重构 `Env::enter_scope()` 为 `&self -> Env`
- [ ] 重构 `Env::bind()` 为 `self -> Env`
- [ ] 重构 `Env::lookup()` 保持 `&self -> Option<&Value>`
- [ ] 编写属性测试: 旧 Env 不变性验证

### 2.2 保持 Sass 作用域语义
- [ ] 确认 `@if/@for/@each` 不创建新 scope
- [ ] 确认 `@mixin`/`@function` 进入时创建新 scope / exit 时恢复
- [ ] 运行 `cargo test --test compile_test` 确认 57/57

## 阶段 3: eval_* 函数纯函数化

### 3.1 签名迁移
- [ ] `eval_block(reactor: Reactor, block: &Block) -> Result<(Reactor, Vec<CssNode>)>`
- [ ] `eval_rule(reactor: Reactor, rule: &Rule) -> Result<(Reactor, Vec<CssNode>)>`
- [ ] `eval_declaration(reactor: Reactor, decl: &Declaration) -> Result<(Reactor, Vec<CssNode>)>`
- [ ] `eval_at_rule(reactor: Reactor, rule: &AtRule) -> Result<(Reactor, Vec<CssNode>)>`
- [ ] `eval_expression(reactor: Reactor, expr: &Expr) -> Result<(Reactor, Value)>`

### 3.2 try_fold 重构
- [ ] 所有 for+push 序列改为 try_fold + accumulator
- [ ] 运行时 benchmark 对比原 God-object 版本 (应在 ±3% 内)

## 阶段 4: Color 操作链式 API (附带收益)

### 4.1 Color trait method
- [ ] `impl Color { pub fn scale(self, kw: &HashMap<String, Value>) -> Result<Self> }`
- [ ] `impl Color { pub fn adjust(self, kw: &HashMap<String, Value>) -> Result<Self> }`
- [ ] `impl Color { pub fn change(self, kw: &HashMap<String, Value>) -> Result<Self> }`
- [ ] `impl Color { pub fn to_space(self, target: ColorSpace) -> Self }`
- [ ] `impl Color { pub fn to_gamut(self, method: GamutMethod) -> Self }`
- [ ] 每次调用自动创建 span 记录输入/输出空间 + 参数

### 4.2 与 builtin 系统集成
- [ ] 保留现有 `color_scale / color_adjust / color_change` builtin 函数签名
- [ ] builtin 内部委托给 `Color::scale / adjust / change`
- [ ] 运行 `cargo test --test sass_spec_full` 确认通过率不降低

## 阶段 5: 收尾与清理

### 5.1 删除旧 God-object
- [ ] 删除 `evaluator::Evaluator` struct 及其 `&mut self` 方法群 (~2000 行)
- [ ] 删除旧 `Env` 的 interior mutability 代码
- [ ] 删除所有 `eval_xxx(&mut self, ...)` 签名的遗留代码

### 5.2 性能回归测试
- [ ] 运行 benchmarks (bootstrap.scss / 5000 imports / nested selectors)
- [ ] 验证性能在预算范围内

### 5.3 测试
- [ ] 新 Reactor 路径单测覆盖 ≥ 80%
- [ ] 全量 sass-spec 通过率不低于 7144/12131
- [ ] 核心测试 57/57 + 10/10 + 8/8 + 5/5 + 15/15 + 15/15 全通过

## 验证标准

```bash
# 红线: 不通过 = 迁移失败
cargo test --test compile_test                # 57/57
cargo test --test sass_spec_full -- --nocapture  # ≥ 7144/12131

# OTel 链式追踪: 每个阶段 span 正确嵌套
RUST_LOG=trace cargo test --features otel --test reactor_test -- --nocapture 2>&1 | tee /tmp/trace.log
```
