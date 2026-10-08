# Tasks

## Layer 1: Architecture Compliance（架构合规）

### 1.1 消灭 eval subscribe-collect [L1]

- [x] 1.1.1 重写 `eval_stream`：移除 `Arc<Mutex<Vec<AstNode>>>` + `.subscribe(push)`，改用 `.collect::<Vec<_>>()` 算子
- [x] 1.1.2 重写 `eval_ast_stream_sync`：同上，使用 `.collect()` 替代 subscribe-collect
- [x] 1.1.3 验证: `cargo test --test integration_test` 通过（无行为变更）

### 1.2 消灭 parse subscribe-collect [L1]

- [x] 1.2.1 重写 `parse_stream_with_paths`：移除 `Arc<Mutex<Vec<Token>>>` 中间收集，改用 `.collect()` 直接获取 tokens
- [x] 1.2.2 验证: `cargo test --test parser_test` + `cargo test --test lexer_test` 通过

### 1.3 内联测试迁出 [L1]

- [x] 1.3.1 创建 `tests/serialize_test.rs`，将 `serialize/mod.rs` 中 `serialize_tests` mod 的 4 个测试函数迁入
- [x] 1.3.2 从 `src/serialize/mod.rs` 删除 `#[cfg(test)] mod serialize_tests` 及其全部内容
- [x] 1.3.3 创建 `tests/builder_test.rs`，将 `builder.rs` 中 `builder_tests` mod 的 3 个测试函数迁入
- [x] 1.3.4 从 `src/builder.rs` 删除 `#[cfg(test)] mod builder_tests` 及其全部内容
- [x] 1.3.5 验证: `cargo test` 全量通过，无测试缺失

### 1.4 Work queue clone 治理 [L1]

- [x] 1.4.1 重构 `Work` enum：确认所有 variant 使用 move 语义（无多余 clone）— 已删除旧 work-queue
- [x] 1.4.2 消除 `expand_nodes_to_events` 中 `child_ctx.clone()` 的重复 clone（move 进 Work item 后原值废弃）
- [x] 1.4.3 消除 `parent_sel.clone()` 在每次迭代的不必要 clone（改用纯函数 emit_node_events）
- [x] 1.4.4 验证: `cargo test` 全量通过

## Layer 2: rxrust Pipeline Compliance（算子合规）

### 2.1 实现 flat_map + scan eval 管线 [L2]

- [x] 2.1.1 将 `expand_nodes_to_events()` 的展开逻辑拆分为 `emit_node_events(node, ctx, bus) -> Vec<EvalEvent>` 纯函数
- [x] 2.1.2 将 `apply_event()` + `for` 循环重构为 `scan(initial_frames, fold_frame)` 算子：`fold_frame` 消费单个事件，返回新 frame 栈
- [x] 2.1.3 在 `eval_nodes_pipeline` 中组装算子链：`Shared::from_iter(nodes).flat_map(emit_events).scan(frames, fold).filter_map(emit_completed).box_it()`
- [x] 2.1.4 验证: `cargo test --test eval_test` 通过，行为与命令式版本等价

### 2.2 Parse pipeline collect 简化 [L2]

- [x] 2.2.1 简化 `parse_stream`：使用 `collect` 算子 + Arc<Mutex> 值提取通道
- [x] 2.2.2 验证: `cargo test --test parser_test` 通过

## Layer 3: Bootstrap-Specific Fixes（覆盖修复）

### 3.1 变量 null 过滤修正 [L3]

- [x] 3.1.1 修改 `eval/mod.rs` StyleDecl 分支：当 `val_str == "null"` 时不再 `continue`，改为 fallback — 若 property 以 `--` 开头（CSS 自定义属性）保留声明且 value 设为 `unset`；否则 emit `tracing::debug!` warning
- [ ] 3.1.2 验证: `--bs-*` 变量缺失从 ~396 降至 < 50

### 3.2 变量 cascade chain walk 修正 [L3]

- [ ] 3.2.1 在 `expr.rs` 中为 VariableRef 解析增加 depth 追踪（防止循环引用）
- [ ] 3.2.2 修改 `get_var` parent chain walk：确保完整回溯到 root scope
- [x] 3.2.3 对于未找到的变量引用，fallback 生成 `Value::String("var(--name)")` 而非 `Value::Null`——保留 CSS 级联引用
- [ ] 3.2.4 验证: 二级以上变量引用不再丢失

### 3.3 Utility API selector 组合修正 [L3]

- [ ] 3.3.1 诊断 `@each` + `@include` 嵌套路径中的 selector 传播链（添加 tracing span）
- [ ] 3.3.2 修正 `combine_selectors`: 当 mixin body 中 selector 不含 `&` 且为类/ID/标签选择器时，使用 descendant combinator（含空格）；仅 pseudo-class/element 使用 compound（无空格）
- [ ] 3.3.3 修复 `.navbar-expand-*` 响应式 mixin 的选择器生成
- [ ] 3.3.4 验证: `.navbar-expand-md .navbar-nav .nav-link` 等选择器生成

### 3.4 Vendor prefix 注入 [L3]

- [x] 3.4.1 在 `src/eval/prefixer.rs` 实现 vendor prefix 注入（独立模块）
- [x] 3.4.2 添加属性映射表: `file-upload-button` → `-webkit-file-upload-button`; `column-gap` → `-moz-column-gap`; `object-fit` → `-o-object-fit`; `mask-position` → `-webkit-mask-position`; `transition`/`appearance` → 多 prefix
- [x] 3.4.3 确保生成的 prefix 声明与原始声明在同一 frame（通过 EvalEvent::Terminal 序列化保证）
- [ ] 3.4.4 验证: `.form-control::-webkit-file-upload-button` 等选择器存在于产物

### 3.5 值表达式保持原样 [L3]

- [x] 3.5.1 确认 `calc()` 表达式在 StyleDecl 的 `value_to_string` 中不被截断（Value::Calc Display）
- [x] 3.5.2 确认 `rgba(var(--bs-*-rgb), alpha)` 格式保持完整（Value::String 透传）
- [x] 3.5.3 确认 `!important` 标记在复杂值后保持（format! 宏拼接）
- [ ] 3.5.4 验证: `border-color: rgba(var(--bs-white-rgb), var(--bs-border-opacity)) !important;` 完整输出

## Validation（最终验证）

### 4.1 测试验证

- [x] 4.1.1 `cargo test` 全量通过
- [ ] 4.1.2 `cargo test --test integration_test bootstrap_dist_check_test` → `coverage_pct >= 99.0`
- [x] 4.1.3 `cargo clippy -- -D warnings` 零警告

### 4.2 诊断证据确认

- [ ] 4.2.1 tracing output `bootstrap_dist_missing_breakdown` 中 `bs_vars_missing < 10`
- [ ] 4.2.2 tracing output `selectors_missing < 20`
- [ ] 4.2.3 tracing output `webkit_missing + moz_missing + o_missing + ms_missing < 5`
- [ ] 4.2.4 tracing output `decl_other_missing < 20`
