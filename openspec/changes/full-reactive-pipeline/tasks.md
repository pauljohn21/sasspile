# Tasks

本任务列表实现全链路响应式管线重构。每个 Phase 后运行 `cargo check` 确认编译。全部完成后运行 `cargo test` 确认所有测试通过。

每个任务 SHALL 在完成后标记为 `[x]`。顺序执行，前置依赖未完成时 SHALL NOT 开始后续任务。

---

## 1. 类型系统基础

- [ ] 1.1 修改 `src/types.rs` 删除 `use std::convert::Infallible;` 行并修改四个类型别名错误类型为 `CompileError`
  - 验证：`grep "Infallible" src/types.rs` 返回空

- [ ] 1.2 修改 `src/observable_ext.rs` 的 `collect_boxed` 函数签名加入 CompileError 错误通道处理
  - 验证：`cargo check` 编译通过

---

## 2. Lexer 适配

- [ ] 2.1 修改 `src/lexer/mod.rs` 的 `scan` 函数，`Shared::create` 闭包中的错误类型从 `Infallible` 改为 `CompileError`
  - 验证：`grep "Infallible" src/lexer/mod.rs` 返回空

- [ ] 2.2 确认 `LexerState::feed` 和 `flush_buf_checked` 函数签名不依赖 `Infallible`（仅 types.rs 已适配）
  - 验证：`cargo check` 编译通过

---

## 3. Parser 重构（核心变更）

- [ ] 3.1 在 `src/parser/state.rs` 新增 `check_unclosed_delimiters(&ParserState) -> Result<(), CompileError>` 函数
  - 验证：函数检查 brace_stack 是否为空，若非空返回 `CompileError::Parse { pos, message: "unclosed '{'" }`

- [ ] 3.2 在 `src/parser/mod.rs` 新增 `parser_feed(state: &mut ParserState, tok: Token) -> Vec<AstNode>` 函数
  - 验证：该函数调用 `state.push_token(tok)` 后循环尝试 `try_step(state)` 收集产出的 AstNode

- [ ] 3.3 在 `src/parser/mod.rs` 新增 `try_step(state: &mut ParserState) -> Option<AstNode>` 函数
  - 验证：根据缓冲区头部 token 类型分类（`$` → variable decl, `@` → at rule, ident → rule/decl），尝试从缓冲区解析完整一条语句；不完全则返回 None

- [ ] 3.4 重构 `parse_stream_with_paths` 函数，使用 `Shared::create` 桥接上游 TokenStream
  - 验证：函数体包含 `Shared::create(move |subscriber| { token_stream.subscribe_all(...) }).box_it()`
  - 验证：`grep "collect_boxed" src/parser/mod.rs` 返回空

- [ ] 3.5 删除 `parse_all_nodes` 函数（如不再被其他代码引用）
  - 验证：`grep "parse_all_nodes" src/` 仅在 `parse_import_source` 中可能还有调用——如仍在 import 中使用，保留但标记为 `#[allow(dead_code)]` 仅内部使用

---

## 4. Evaluator 重构

- [ ] 4.1 修改 `src/eval/mod.rs` 中 `eval_stream` 函数，去除 `collect_boxed(ast_stream)` 断裂点
  - 验证：`grep "collect_boxed" src/eval/mod.rs` 返回空
  - 验证：`eval_stream` 函数体为 `ast_stream.expand(...).scan_map(...).filter_map(...).box_it()`

- [ ] 4.2 将 `flat_map(emit_events)` 改为 `expand(emit_events)` 在 `eval_stream` 内
  - 验证：`eval_stream` 函数内存在 `.expand(move |node| emit_events(...))` 调用

- [ ] 4.3 删除 `build_css_stream`、`eval_nodes_sync`、`eval_ast_stream_sync` 函数（不再需要）
  - 验证：`grep -E "eval_nodes_sync|eval_ast_stream_sync|build_css_stream" src/` 仅在注释中出现

- [ ] 4.4 修改 `src/eval/emit.rs` 的 `emit_events` 函数签名：返回值从 `Infallible` 改为 `CompileError`
  - 验证：`emit_events` 返回 `SharedBoxedObservable<'static, EvalEvent, CompileError>`
  - 验证：`grep "Infallible" src/eval/emit.rs` 返回空

---

## 5. Serializer 响应式化

- [ ] 5.1 在 `src/serialize/mod.rs` 新增 `SerializeState` 结构体及 `new`/`push`/`render` 方法
  - 验证：`SerializeState { stmts: Vec<CssStmt>, options: Options }` 定义存在
  - 验证：`render` 方法内部调用 `crate::serialize::serialize(&self.stmts, &self.options)`

- [ ] 5.2 在 `src/serialize/mod.rs` 新增 `serialize_stream<S>(css_stream: S, options: Options) -> OutputStream` 函数
  - 验证：函数体为 `css_stream.fold(SerializeState::new(options), |mut s, stmt| { s.push(stmt); s }).map(|s| s.render()).box_it()`

---

## 6. Pipeline 编排

- [ ] 6.1 重写 `src/pipeline.rs`：新增 `compile_pipeline` 函数组合 scan → parse → eval → serialize_stream
  - 验证：`grep "collect_boxed\|collect_stream" src/pipeline.rs` 仅在 `from_string_with_paths` 最终消费端出现一次

- [ ] 6.2 修改 `from_string_with_paths` 使用 `compile_pipeline` + `collect_boxed` 终端消费
  - 验证：`from_string_with_paths` 函数体包含 `let stream = compile_pipeline(source, options, include_paths);` 和 `let chunks = collect_boxed(stream)?;`

- [ ] 6.3 删除 `collect_stream` 辅助函数（已被 `collect_boxed` 吸收）
  - 验证：`grep "collect_stream" src/` 返回空

---

## 7. Module Exports 清理

- [ ] 7.1 检查 `src/lib.rs` 确保不导出任何已删除函数，`pipeline` 模块仅导出 `from_path`、`from_string`
  - 验证：`grep "pipeline" src/lib.rs` 仅显示 `pub mod pipeline;` 和 `pub use pipeline::{from_path, from_string};`

---

## 8. Infallible 全局清除

- [ ] 8.1 全局 grep `Infallible` 在 `src/` 中确认无残留
  - 验证：`grep -r "Infallible" src/` 返回空

- [ ] 8.2 全局 grep `Infallible` 在 `tests/` 中确认无残留并修复所有测试文件
  - 验证：`grep -r "Infallible" tests/` 返回空

---

## 9. 修复与适配

- [ ] 9.1 运行 `cargo check` 修复所有编译错误
  - 验证：`cargo check` 输出无 error

- [ ] 9.2 运行 `cargo build` 确认完整构建
  - 验证：`cargo build` 无 error/warning（pre-existing warnings 除外）

- [ ] 9.3 运行全部测试 `cargo test`，修复失败的测试
  - 验证：`cargo test` 全部通过（含 pre-existing failures 的已知 TODO 不计入）

  **注意**: 若发现某些测试调用了已删除的 `eval_ast_stream_sync` 或 `eval_nodes_sync`，修改为调用 `eval_stream` + `collect_boxed` 或管线终端 sync wrapper。

---

## 10. 最终验证

- [ ] 10.1 全链路验证：编译 Bootstrap 的 `_boot.scss` 确认输出一致
  - 验证：`cargo test --test bootstrap_test -- --nocapture` 通过

- [ ] 10.2 行为基准验证：`sass-spec` 测试通过率不变
  - 验证：`sass-spec` 测试套件结果与重构前对比，无新增失败

- [ ] 10.3 全局搜索反模式确认：
  - 验证：`grep -rn "collect_boxed" src/` 仅在 `observable_ext.rs` 定义和 `pipeline.rs` 最终消费端出现
  - 验证：`grep -rn "Shared::from_iter" src/parser/` 不再出现
  - 验证：`grep -rn "for.*push" src/` 无匹配（命令式 GC 模式清除）

---

## 原子性规则（强制）

实现者 SHALL 遵循以下规则：

1. **渐进式回滚禁止**: 不得保留旧函数作为 "兼容层"——旧实现必须彻底删除
2. **中间 collect 禁止**: 第一阶段结束后不得在源文件中新增任何 `collect_boxed` 调用
3. **同步包装禁止**: 不得新增 `eval_*_sync`、`parse_sync` 等同步包装函数
4. **Infallible 零容忍**: 任何文件中残留 `Infallible` 均视为实现未完成
5. **中间 box_it 禁止**: 任何 `.scan_map(...).box_it().flat_map(...)` 模式均违规——`box_it()` 只能在管线最末尾

---

## 参考文件清单

实现时需读取并理解的文件：

| 文件 | 变更类型 | 关键变更点 |
|------|---------|-----------|
| `src/types.rs` | 修改 | 删除 Infallible import + 修改 4 个 type alias |
| `src/observable_ext.rs` | 修改 | collect_boxed 签名 + 双参数 subscribe |
| `src/lexer/mod.rs` | 修改 | scan 闭包错误类型 |
| `src/parser/mod.rs` | 重构 | parser_feed + try_step + Shared::create 桥接 |
| `src/parser/state.rs` | 修改 | 新增 check_unclosed_delimiters |
| `src/eval/mod.rs` | 重构 | eval_stream 去除 collect_boxed + expand 替代 flat_map + 删除同步包装 |
| `src/eval/emit.rs` | 修改 | emit_events 返回类型 CompileError |
| `src/serialize/mod.rs` | 新增 | SerializeState + serialize_stream |
| `src/pipeline.rs` | 重构 | compile_pipeline 编排 + 终端 collect_boxed |
| `src/lib.rs` | 修改 | 清理 exports |
