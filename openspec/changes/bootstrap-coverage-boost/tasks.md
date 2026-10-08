# Tasks

## 1. 序列化格式修复（B6/B7）— 预期 coverage 1.25% → ~50%

- [ ] 1.1 修复 `format_expanded` depth 起始值：根节点 depth=1，每层 +1，缩进量 `(depth-1)*2` spaces。验证：integration_test.rs 中嵌套规则 decl 缩进为 2/4 空格
- [ ] 1.2 修复选择器空格：`{` 前统一插入 1 空格（`.a {` 而非 `.a{`）。验证：integration_test.rs 输出含 `.selector {` 格式
- [ ] 1.3 运行 `cargo test --test bootstrap_test -- test_bootstrap_dist_alignment --ignored` 验证 coverage 提升至 ~50%

## 2. `!important` 标记处理（B8）— 预期 coverage ~50% → ~62%

- [ ] 2.1 在 `StyleDecl` 结构体新增 `important: bool` 字段。验证：`cargo check` 通过
- [ ] 2.2 修改 parser 在解析 value 后遇到 `!Token` 时设置 `important=true`。验证：parser_test.rs 新增测试通过
- [ ] 2.3 修改 serializer 在输出 decl 时若 `important=true` 则追加 ` !important`。验证：`color: red !important;` 输出正确
- [ ] 2.4 运行 bootstrap_dist_check 验证 coverage 提升至 ~62%

## 3. 厂家前缀生成（B9）— 预期 coverage ~62% → ~80%

- [ ] 3.1 诊断 `webkit-prefixer` mixin 调用链：检查 mixin 注册、参数绑定、展开路径。验证：tracing trace 显示完整调用链
- [ ] 3.2 修复 mixin 展开以生成 `-webkit-`/`-moz-`/`-ms-` 前缀变体。验证：transition/transform 相关 decl 输出含前缀变体
- [ ] 3.3 运行 bootstrap_dist_check 验证 coverage 提升至 ~80%

## 4. 复杂 Mixin 展开（B11）— 预期 coverage ~80% → ≥ 95%

- [ ] 4.1 诊断 `media-breakpoint-up` → `media-breakpoint-between` → `@media` 调用链。验证：tracing trace 显示嵌套展开路径
- [ ] 4.2 修复 @content 在嵌套 mixin 中的传播路径。验证：@media 规则数从 25 增至 ≥ 100
- [ ] 4.3 运行 bootstrap_dist_check 验证 coverage ≥ 95%

## 5. Sass Maps 格式修复（B10）— 清理乱码输出

- [ ] 5.1 诊断 map 值在 CSS decl 中的输出路径（`value_to_string` 对 Map 的处理）。验证：tracing trace 定位泄漏点
- [ ] 5.2 修复 map 值在 CSS decl 中输出为 `null`（不输出到 CSS）。验证：map 值不再出现在 CSS decl 值中
- [ ] 5.3 运行 bootstrap_dist_check 验证 coverage ≥ 99%

## 6. 最终验证

- [ ] 6.1 运行全量 `cargo test` 确认所有测试通过。验证：exit code 0
- [ ] 6.2 运行 `cargo clippy -- -D warnings` 确认零警告。验证：exit code 0
- [ ] 6.3 运行 `bootstrap_dist_check()` 确认 `missing_count ≤ 53`（coverage ≥ 99%）。验证：assertion 通过
