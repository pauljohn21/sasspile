## 1. 术语清理 — 消除 Flux 思维传染源

- [x] 1.1 修改 `src/lib.rs` — 移除所有 "Flux 思维" / "Flux → rxrust 转译" / "Flux Sinks.Many" 术语，替换为 "Rust ownership 三态 (move/&/&mut) 驱动"（参考 `specs/no-gc-patterns/spec.md`）
- [x] 1.2 修改 `src/directive/mod.rs` — 同上清理模块级 doc comment（参考 `specs/no-gc-patterns/spec.md`）
- [x] 1.3 修改 `src/directive/pipeline.rs` — 第 1-4 行标题和注释中的 "Flux 模型" 替换为 "rxrust 算子链范式"（参考 `specs/no-gc-patterns/spec.md`）
- [x] 1.4 运行 `grep -rn "Flux" src/` 确认零残留

## 2. Token 类型引入 — 为结构化替换铺路

- [x] 2.1 在 `src/directive/parse.rs` 中新增 `enum Token { Text(String), Interpolation(String), VariableRef(String) }`（参考 `specs/no-gc-patterns/spec.md` — "文本模板式字符串替换"禁令）
- [x] 2.2 新增 `fn tokenize(input: &str) -> Vec<Token>` 纯函数，将输入拆为结构化 token（参考 `specs/rxrust-ownership/spec.md`）
- [x] 2.3 新增 `fn detoken(tokens: &[Token]) -> String` 纯函数，将 token 序列还原为 String

## 3. substitute_vars 响应式化

- [x] 3.1 重构 `substitute_vars`：内部使用 `tokenize()` → `scan_map(VarScope::new(), resolve)` → `detoken()`（参考 `specs/rxrust-ownership/spec.md` — scan_map 作为唯一状态栖息地）
- [x] 3.2 重构 `eval_all_calls`：内部使用结构化 token 流 + `map(try_eval_builtin)` 而非 `while let` + `replace_range`（参考 `specs/reactive-dataflow/spec.md`）
- [x] 3.3 重构 `eval_user_functions`：同上
- [x] 3.4 运行 `cargo test` 确认无回归（3 个 pre-existing bug 与本 change 无关）

## 4. expand_while 响应式化

- [x] 4.1 定义 `struct WhileAcc { state: CollectState, iter_count: i32 }`（参考 `specs/rxrust-ownership/spec.md`）
- [x] 4.2 将 `expand_while` 重构为 scan_map(WhileAcc) 模式循环，消除 `state.clone()` + `insert()` 模式（参考 `specs/no-gc-patterns/spec.md` — 共享可变状态禁令）
- [x] 4.3 运行 `cargo test` 确认 @while 相关测试通过（5 个测试全部通过）

## 5. if/return 链重构 — process_line 枚举分发

- [x] 5.1 定义 `enum LineKind { Empty, CloseBrace, VarDef, Include, AtExtend, InlineExtend, RuleStart, Plain }`（参考 `specs/no-gc-patterns/spec.md`）
- [x] 5.2 新增 `fn classify_line(line: &str) -> LineKind` 纯函数
- [x] 5.3 将 `process_line` 重构为 `match classify_line(trimmed) { ... }` 单层匹配，消除嵌套 `if/return` 链
- [x] 5.4 运行 `cargo test` 确认 process_line 相关测试通过（仅 pre-existing 失败）

## 6. merge_import_lines 响应式化

- [x] 6.1 将 `merge_import_lines` 重构为 scan_map 语义 fold（accumulator = (pending, result) 元组），消除 `while i < lines` 索引增量模式（参考 `specs/reactive-dataflow/spec.md`）
- [x] 6.2 运行 `cargo test` 确认无回归（仅 pre-existing 失败）

## 7. 模块系统约束 spec 落实（后续 change 范围界定）

- [x] 7.1 在 `module_system.rs` 顶部添加注释引用 `specs/module-system-rx/spec.md` 的约束
- [x] 7.2 标记 `process_module_imports` 和 `expand_forwards_in_place` 为 `// TODO: 按 specs/module-system-rx/spec.md 重构为 ModuleEvent Subject + scan_map(ModuleResolver)`
- [x] 7.3 标记 `import_resolver.rs` 同理

## 8. tracing span 自查

- [ ] 8.1 检查 `pipeline.rs` 中所有 `flat_map` / `scan_map` / `map` 闭包入口是否有对应 `debug_span!` 或 `trace_span!`（参考 `specs/tracing-span/spec.md`）
- [ ] 8.2 检查 `parse.rs` 中纯函数是否需要 `info_span!`（纯解析函数可暂不插 span，跨阶段 reducer 必须有）
- [ ] 8.3 运行 `cargo test --features otel` 确认 span 正常输出

## 9. spec 集成到 CI

- [ ] 9.1 在 `.github/workflows/` (若有) 中添加 grep 检查步骤：扫描 `for.*push` / `while.*replace_range` / `Flux` 关键词
- [ ] 9.2 在 `AGENTS.md` 中新增一行引用 `openspec/changes/rxrust-spec-hardening/specs/` 路径，告知 AI 此处有追加约束
