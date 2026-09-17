# Tasks: 所有权 + rxrust 重构

按优先级 P0→P3 依次执行,每步跑 `cargo test` 验证。

## P0: lib.rs 终点收集（最安全,先做）

- [ ] **T0.1**: `compile()` 函数中 `Rc::new(RefCell::new(Vec::new()))` + `subscribe(move |ch| push)` → 改为 `pipeline::build(input).last().subscribe(|result: String| ...)`. 验证 `cargo test --test e2e_api` 通过
- [ ] **T0.2**: `compile_at()` 函数同 T0.1 处理

## P1: evaluate_dst/mod.rs helper 函数迭代器化

- [ ] **T1.1**: `split_args(s: &str)` — 32 行 `for ch` + depth 计数 + `current.push` → 改为 `s.chars().scan(ParseState::new(), feed).filter_map(...).collect()` 或自定义 iterator. 验证 `cargo test --test core_functions_smoke` 不退化
- [ ] **T1.2**: `resolve_vars_only(vars, s)` — 55 行 `while i < chars_vec.len()` 重复模式 → 合并到 `split_args` 类似的 scan-based 实现
- [ ] **T1.3**: `substitute_vars(ctx, s)` — 125 行旗舰函数,改为: `Local::from_iter(s.chars()).scan_map(SubstState::new(), feed).flat_map(from_iter).last().subscribe(|chunks| join)` 或等价的纯函数迭代器版本

## P2: evaluate_dst/builtins.rs 命令式循环消除

- [ ] **T2.1**: `split_list(s)` — 50 行 `for ch in inner.chars()` + depth + `current.push` → 同 T1.1 替换为 scan-based iterator
- [ ] **T2.2**: `arg_to_map(s)` — 简单 `for pair in inner.split(',')` → `inner.split(',').filter_map(|p| p.split_once(':')).map(...).collect()`
- [ ] **T2.3**: `builtin_get_css_var(args, ctx)` — `for arg in args` 累积 → `args.iter().map(...).collect::<Vec<_>>().join(", ")`

## P3: evaluate_dst/eval_ctx.rs 累积模式替换

- [ ] **T3.1**: `evaluate_for` — `for i in from_n..=to_n { ctx.var = i; for node in body { out.extend(eval)?; } }` → 把迭代展开为 `Local::from_iter(from..=to).flat_map(|i| expand_body(i, body, body_fn))`
- [ ] **T3.2**: `evaluate_if` — `for node in branch { out.extend(eval)?; }` → `Local::from_iter(branch.iter()).flat_map(|node| eval_owned(node))`
- [ ] **T3.3**: `evaluate_mixin_call` — `for param ... {}` + `for node in &def.body { out.extend(eval)?; }` → 同上模式
- [ ] **T3.4**: `nodes_to_css_nodes(nodes)` — `for n in nodes { out.extend(eval)?; }` → `nodes.iter().flat_map(|n| eval_owned(n,)).collect()`
- [ ] **T3.5**: `parse_compiled_css(css)` — `for line in css.lines()` → `css.lines().filter_map(|line| parse_line(line)).collect()`

## P4: parse_dst 模块命令式 while → 迭代器

- [ ] **T4.1**: `parse_rule_body` — `while i < tokens.len()` 多次嵌套 → 改为 `tokens.iter().scan(ParseState::new(), feed).filter_map(...).collect()`
- [ ] **T4.2**: `parse_declarations` — 137 行最深重的函数,全部 `while i < tokens` 循环 → 迭代器 state machine
- [ ] **T4.3**: `parse_block` — `while i < tokens` + `while j < tokens` 双重循环 → 分阶段 scan
- [ ] **T4.4**: `try_flush_variable` — `while idx < buffer.len()` 跳过空白 → `buffer.iter().skip_while(whitespace).collect()`
- [ ] **T4.5**: `parse_arg_list_at` / `parse_param_list` / `parse_arg_list` — 三个类似的 `while pos < tokens.len()` → 统一为 scan-based

## P5: 副作用隔离

- [ ] **T5.1**: 扫描所有 `map` / `flat_map` 闭包内部是否有 `tracing::` 调用 → 移到 `.tap()`

## P6: 验证

- [ ] **T6.1**: `cargo test` 全量通过
- [ ] **T6.2**: `cargo clippy -- -W clippy::all` 无新增 warning
- [ ] **T6.3**: `grep -rn "while.*<.*\.len" src/` 数量归零或仅剩 tokenize 状态机（合理）
- [ ] **T6.4**: `grep -rn "subscribe(move |" src/` 数量归零
- [ ] **T6.5**: `cargo run --bin sasspile_tracker -- enterprise` Bootstrap/Element Plus 基线不退化
