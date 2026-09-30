## 1. 伪元素双冒号格式规范化

- [ ] 1.1 在 `src/css/serializer.rs` 中添加伪元素名列表常量（`::before`, `::after`, `::placeholder`, `::first-line`, `::first-letter`, `::selection`, `::marker`, `::file-selector-button` 等）
- [ ] 1.2 在序列化选择器时检测伪元素 token，对匹配名称强制输出 `::` 双冒号格式
- [ ] 1.3 运行 `cargo test --test ep_full -- --nocapture`，确认 lightningcss 规范化后对比无回归
- [ ] 1.4 运行 `cargo test --test compile_test -- --nocapture`，确认核心测试 43/43 仍通过

## 2. Keyframes 空块保留修复

- [ ] 2.1 在 `tests/ep_normalized_test.rs` 中核查 LightningCSS minifier 配置，尝试 `minify: false` 或定制 `targets` 以保留空 keyframe 块
- [ ] 2.2 若 LightningCSS 无法配置保留，则在规范化函数中添加后处理：检测 `0% {}` / `100% {}` 缺失并补回占位空块
- [ ] 2.3 验证相关 EP DIFF 文件（dialog.scss 等）的 keyframe 段不再产生差异
- [ ] 2.4 运行 `cargo test --test ep_normalized_test -- --nocapture`，确认一致率提升

## 3. 模块变量嵌套解析诊断

- [ ] 3.1 使用 `codegraph callers load_module` 和 `codegraph callers bind_params` 分析变量传递链路
- [ ] 3.2 在 `src/directive/ops.rs` 的 `@include` 调用路径添加 `#[instrument]` span，记录入口 env 的 scope chain 长度
- [ ] 3.3 收集 EP DIFF 文件（含模块变量嵌套引用场景）的 trace 证据
- [ ] 3.4 根据 trace 结果定位 env 作用域链断裂点，定向修复

## 4. Extend 跨模块传播修复

- [ ] 4.1 查阅 `src/directive/ops.rs` 中 extend 的 selector injection 逻辑，确认 `@use` 边界下的行为
- [ ] 4.2 添加 `#[instrument]` span 追踪 extend target 解析路径
- [ ] 4.3 对比 EP reference 输出，确认选择器注入位置与 scope 边界行为匹配
- [ ] 4.4 修复 selector group ordering 或 nesting depth 偏差

## 5. 函数求值路径修复

- [ ] 5.1 使用 `cargo run --bin diag -- <scss_file>` 诊断 `getCssVar()` 和 `map.get()` 未求值的具体位置
- [ ] 5.2 在相关函数 eval 路径加 `#[instrument]` span，追踪返回值展开逻辑
- [ ] 5.3 定位引用/解引用缺失或 builtin 注册遗漏的根因
- [ ] 5.4 实施修复，逐一验证各函数输出

## 6. 全量验证

- [ ] 6.1 运行 `cargo test --test ep_normalized_test -- --nocapture`，目标 121/121 (100%)
- [ ] 6.2 运行全量核心测试 `cargo test --test compile_test --test reactor_test --test stage_test --test ast_test --test common_test --test bs_spec --test ep_full`，确认 202/202 通过
- [ ] 6.3 运行 `cargo clippy --all-targets` 确认无新增警告
