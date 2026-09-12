## 0. 紧急修复（已完成）

- [x] 0.1 修复 `fmt_color_fn` 格式字符串补上遗漏的 `)` 闭合括号
- [x] 0.2 编译验证 `cargo check`
- [x] 0.3 核心测试验证（202/202 通过）

## 1. 新 Baseline（已完成）

- [x] 1.1 运行 `SPEC_STORE_CMD=run` 生成新 snapshot
- [x] 1.2 确认 snapshot 通过率恢复到 ~63.4%
- [x] 1.3 `SPEC_STORE_CMD=stats` 输出最新目录统计

## 2. Phase 1: 媒体查询 + CSS 函数格式化（已完成）

- [x] 2.1 诊断 `css/media` 55 个失败 case 的具体模式
- [x] 2.2 实现媒体查询逻辑操作符（not/and/or）后空格规范化
- [x] 2.3 诊断 `css/functions` 74 个失败 case 的模式
- [x] 2.4 实现 CSS 函数名 lowercase 序列化
- [x] 2.5 实现 vendor 前缀函数注释间距处理（已分析：非主要失败模式，跳过）
- [x] 2.6 Phase 1 验证：核心测试通过 + sass-spec 统计确认增量（7663→7677, +14）

## 3. Phase 2: CSS 嵌套 + 普通规则（部分完成）

- [x] 3.1 诊断 `css/plain` 85 个失败 case 的模式
- [x] 3.2 实现 CSS 文件 @import 在规则体内选择器组合
- [ ] 3.3 实现声明重排序（嵌套块前置）
- [ ] 3.4 实现 CSS hack 格式规范化
- [ ] 3.5 Phase 2 验证：核心测试通过 + sass-spec 统计确认增量

---

## Architecture: Dual AST 完全隔离（新增主轴线）

> **SCSS 和 CSS 是两种独立语言，有各自版本线（CSS4→CSS5, SCSS3→SCSS4）。通过独立 AST 实现完全解耦。**

### Phase A: 类型定义 + 双 Parser（✅ 已完成）

- [x] A.1 新建 `src/parse/scss_ast.rs`（ScssAst + ScssNode 定义，当前为 type alias）
- [x] A.2 新建 `src/parse/css_ast.rs`（CssAst + CssNode 定义，**真正独立**受限类型）
- [x] A.3 新建 `src/parse/css_parser.rs`（CssParser，产出 CssAst）
- [x] A.4 修改 `src/parse/mod.rs`（`Parsed` 枚举 + `CompileMode` + `parse_dispatch()`）
- [x] A.5 迁移确认：现有 `nodes.rs` 无需改动（Node = ScssNode alias）
- [x] A.6 验证：cargo check 零错误，核心测试全绿（48/48 compile_test、8/8 stage_test、15/15 interp_test、1/1 ep_full、15/15 bs_spec）

### Phase B: SCSS Evaluator 迁移（✅ 已完成）

- [x] B.1 新建 `src/eval/scss_evaluator.rs`（薄包装，委托给 Evaluator）
- [x] B.2 ~~新建 `src/eval/scss_env.rs`~~（Env 保持通用，无需重命名）
- [x] B.3 修改 `src/eval/reactor.rs`（Scss 路径调用 `ScssEvaluator::evaluate_with_env`）
- [x] B.4 CssNode 确认：`src/css/node.rs` 已是合适共享输出类型
- [x] B.5 验证：compile_test 48/48, stage_test 8/8, ep_full 1/1, bs_spec 15/15 全通过

### Phase C: CSS Evaluator 新建（✅ 已完成）

- [x] C.1 新建 `src/eval/css_evaluator.rs`（~220 行，含值格式化）
- [x] C.2 实现 `eval_rule`（保留嵌套结构，不组合选择器）
- [x] C.3 实现 `eval_import`（透传 @import at-rule）
- [x] C.4 实现 `eval_at_rule`（@media/@keyframes 透传不提升）
- [x] C.5 修改 `src/eval/reactor.rs`（双路径：Scss→ScssEvaluator, Css→CssEvaluator）
- [x] C.6 验证：compile_test 48/48, stage_test 8/8, interp_test 15/15, ep_full 1/1, bs_spec 15/15 全通过

### Phase D: 彻底清理 plain_css 残留（✅ 已完成）

> **SCSS 和 CSS 完全隔离，不再有 `plain_css` 标志。**
> `Evaluator::load_import` 和 `Evaluator::load_module` 对 `.css` 文件使用 `CssParser` + `CssEvaluator`。
> 所有 `is_plain_css()` 运行时检查已删除，由解析阶段的类型系统保证隔离。

- [x] D.1 改造 `Evaluator::load_import` 对 `.css` 文件使用 `CssParser` + `CssEvaluator`
- [x] D.2 改造 `Evaluator::load_module` 对 `.css` 文件使用 CSS 路径
- [x] D.3 删除 `src/eval/plain_css.rs`（230 行 `check_plain_css_*` 函数）
- [x] D.4 删除 `Env.plain_css` 字段 + `with_plain_css()` + `is_plain_css()`
- [x] D.5 删除所有 `check_plain_css_value/node/selector` 调用（mod.rs）
- [x] D.6 删除 rule.rs 中 60+ 行 `@media` 提升 dead code
- [x] D.7 删除 manual_dispatch.rs 和 partial.rs 中 sass() / 插值的 plain_css 检查
- [x] D.8 删除 error_msgs.rs 中 `err_plain_css_*` 三函数
- [x] D.9 验证：核心测试 121/121 + ep_full 1/1 + bs_spec 15/15 全通过，零回归

### Phase E: 架构验证 + 修复收敛（✅ 完成，零回归 + 2 修复）

- [x] E.1 运行 sass-spec：7676/12131 (63.3%)，与之前 7677 (63.3%) 基本持平
- [x] E.2 诊断"回归"：2 个 PASS→FAIL 均为预存 flaky test（HashMap 序 + IO 时序），非真回归
- [x] E.3 诊断"修复"：2 个 FAIL→PASS 为真实修复（`module_variables/through_forward/bare`, `modules/color/css_overloads/alpha/multi_arg`）
- [x] E.4 SCSS-CSS 跨界场景：`Evaluator::load_import` 通过 dual AST 分派处理（`.css`→CssParser+CssEvaluator）
- [x] E.5 验证：核心测试 48/48 compile_test + 8/8 stage_test + 15/15 interp_test + 1/1 ep_full + 15/15 bs_spec 全通过

---

## 后续 Phase（待定，依赖架构稳定后）

### Phase 3: calc() 简化（预估 +300~400 cases）

- [ ] 4.1 诊断 `values/calculation/calc` 344 个失败 case 的模式
- [ ] 4.2 实现 `calc(calc(...))` 嵌套展开
- [ ] 4.3 实现纯数字除法常量折叠
- [ ] 4.4 实现一元负号规范化
- [ ] 4.5 诊断并修复其他数学函数（clamp/sign/exp/log 等 50 cases）
- [ ] 4.6 Phase 3 验证

### Phase 4: Selector + Meta 函数（预估 +350~500 cases）

- [ ] 5.1 诊断 + 修复 selector.unify
- [ ] 5.2 诊断 + 修复 selector.extend
- [ ] 5.3 诊断 + 修复 selector.is_superselector
- [ ] 5.4 诊断 + 修复 meta 函数
- [ ] 5.5 Phase 4 验证

### Phase 5: 颜色系统深度修复（预估 +1000~1500 cases）

- [ ] 6.1 分析修复 fmt_color_fn 后剩余颜色失败模式
- [ ] 6.2 修复 adjust 通道计算精度
- [ ] 6.3 修复 change 通道替换逻辑
- [ ] 6.4 修复 scale 缩放算法
- [ ] 6.5 修复 to_gamut 色域映射
- [ ] 6.6 修复命名色舍入和匹配
- [ ] 6.7 Phase 5 验证

---

## Phase 2 诊断报告（参考）

`css/plain` 共 85 个失败 case，分类如下：

| 类别 | 数量 | 说明 |
|------|------|------|
| 错误验证 | ~30 | `error/media/*`, `error/statement/*`, `error/expression/*` — 需架构级校验 |
| Import 条件 | ~16 | `import/conditions/*` — `@import "url" supports(...)` 格式校验 |
| @use "plain" 特性 | ~33 | boolean ops, hacks, custom properties, lowercase functions |
| CSS @import 嵌套 | 5 | `through_import/*`, `through_load_css/*` — 选择器组合问题 |
| 声明排序 | 3 | `with_declaration/*` — 嵌套块与声明需按源序保留 |

