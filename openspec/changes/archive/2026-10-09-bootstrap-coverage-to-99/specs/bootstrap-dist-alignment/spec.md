# Spec Delta

## MODIFIED Requirements

### Requirement: 架构合规基线
在推动 Bootstrap 覆盖之前，rx-scss 的 eval pipeline SHALL 符合 rxrust 框架原则和 Rust 所有权规则。具体要求：禁止 `subscribe-collect` GC 模式、禁止 `#[cfg(test)]` 在 src/ 内联、clone 使用 SHALL 最小化。

#### Scenario: eval pipeline 使用 rxrust 算子
- **WHEN** `eval_stream` 处理 AST 节点流
- **THEN** SHALL 使用 `flat_map` + `scan` 算子链（而非命令式 for loop + 手动 queue）

#### Scenario: parse 使用 collect 算子
- **WHEN** `parse_stream_with_paths` 消费 TokenStream
- **THEN** SHALL 使用 `.collect::<Vec<_>>()` 替代 `Arc<Mutex<Vec>>` + `subscribe(push)`

#### Scenario: src/ 无内联测试
- **WHEN** `cargo test` 编译 src/ 模块
- **THEN** SHALL 不编译任何 `#[cfg(test)]` 代码（测试统一在 tests/ 目录）

#### Scenario: clone 最小化
- **WHEN** Work queue 推入新 AST 节点
- **THEN** SHALL 使用 move 语义消费节点（node.into_iter().for_each(move |n| queue.push(n)) 模式），而非 `node.clone()`

### Requirement: 覆盖率目标 ≥ 99%
编译 coverage SHALL 达到 99%+（≤ 53 行不包括注释的差异）。基于 tracing 证据，缺失行应分为 4 个类别进行针对性修复。架构合规（Layer 1）和算子合规（Layer 2）完成后，再实施 Bootstrap-Specific 修复（Layer 3）。

#### Scenario: 覆盖率达到 99%
- **WHEN** 运行 `bootstrap_dist_check()`
- **THEN** `missing_count ≤ 53`（总参考行 5342 的 1%）

#### Scenario: 覆盖率渐进提升
- **WHEN** 完成 Layer 1（架构合规：subscribe-collect 消灭 + 测试迁出 + clone 治理）
- **THEN** coverage SHALL ≥ 68%（无退化）且 src/ 零 `#[cfg(test)]`
- **WHEN** 完成 Layer 2（rxrust 算子链：flat_map + scan eval pipeline）
- **THEN** coverage SHALL ≥ 68%（无退化）且 eval pipeline 使用 rxrust 算子
- **WHEN** 完成 Layer 3（Bootstrap 修复：变量 cascade + selector + vendor + 值表达式）
- **THEN** coverage SHALL 最终达到 ≥ 99%

#### Scenario: 分类修复验证
- **WHEN** CSS Variable cascade 修复后（Layer 3.1 + 3.2）
- **THEN** 缺失的 `--bs-*` variable 数量 SHALL ≤ 10
- **WHEN** Utility API 展开修复后（Layer 3.3）
- **THEN** 缺失的选择器块数量 SHALL ≤ 20
- **WHEN** vendor prefix 修复后（Layer 3.4）
- **THEN** 缺失的 `-webkit-` / `-moz-` / `-o-` 数量 SHALL ≤ 5
- **WHEN** 值表达式求值修复后（Layer 3.5）
- **THEN** 缺失的其他 declaration 数量 SHALL ≤ 20

## ADDED Requirements

### Requirement: bootstrap_dist_check SHALL 输出分类诊断证据
`bootstrap_dist_check()` SHALL 通过 tracing 输出分类统计信息，区分 selector、CSS variable、vendor prefix、declaration 四个类别的缺失数量，以便后续修复按 ROI 排序。

#### Scenario: 分类输出
- **WHEN** 运行 `bootstrap_dist_check()` 或 `bootstrap_dist_check_test`
- **THEN** tracing output SHALL 包含 `bootstrap_dist_missing_breakdown` 字段，含 `selectors_missing`、`bs_vars_missing`、`webkit_missing`、`moz_missing`、`o_missing`、`decl_other_missing`

#### Scenario: 测试初始化 tracing
- **WHEN** integration_test 运行 bootstrap_dist 相关测试
- **THEN** SHALL 调用 `init_test_tracing()` 确保 tracing subscriber 初始化，否则 debug 级别日志不可见

### Requirement: 变量 null fallback 保留 var() 引用
当 StyleDecl value 评估为 `Value::Null` 时，系统 SHALL 根据 property 类型决定 fallback 策略：CSS 自定义属性（以 `--` 开头）保留声明但 value 设为 `unset`；普通声明 emit debug warning 后丢弃。此行为防止 Bootstrap 中通过条件路径赋值的 `--bs-*` 变量因中间 null 导致整行丢失。

#### Scenario: CSS 自定义属性 null fallback
- **WHEN** SCSS 中 `--bs-dark-text-emphasis` 的 value 评估为 null
- **THEN** 编译产物 SHALL 包含 `--bs-dark-text-emphasis: unset;`（而非丢失整行）

#### Scenario: 普通声明 null 仍丢弃
- **WHEN** 非 `--` 开头的 property 的 value 评估为 null
- **THEN** SHALL 丢弃该声明并 emit `tracing::debug!` warning 记录变量名

### Requirement: 变量 Cascade Chain Walk
变量解析 SHALL 完整遍历 parent chain（child_scope → enclosing scope → ... → root_scope）。当变量在当前链中未找到 binding 时，fallback 生成 `var(--name)` 引用以保留 CSS 级联语义，而非直接 evaluated 为 null。

#### Scenario: 多链变量引用解析
- **WHEN** Bootstrap SCSS 中 `--bs-list-group-color: var(--bs-dark-text-emphasis)`，且 `--bs-dark-text-emphasis` 在祖先 scope 中定义
- **THEN** 编译产物 SHALL 保留该声明，值 either 为解析后的色值或 `var(--bs-dark-text-emphasis)` CSS 引用

#### Scenario: 循环引用检测
- **WHEN** 变量 A 引用变量 B，B 又引用 A（直接或间接）
- **THEN** SHALL 在 depth > 5 时终止 walk 并 emit `tracing::warn!` 标注循环引用涉及的变量名

### Requirement: Utility API / 响应式 mixin 展开
`$utilities` map 上的 `@each` 循环 SHALL 完整展开所有 utility class 选择器及 `@media` 响应式变体。`.navbar-expand-md .navbar-nav .nav-link` 等响应式选择器 SHALL 生成。在 mixin body 嵌套上下文中，`combine_selectors` 须正确分派 combinator：`&` 存在时替换；pseudo-class/element 时使用 compound（无空格）；class/ID/标签选择器使用 descendant combinator（空格）。

#### Scenario: 响应式 utility 选择器
- **WHEN** 编译 Bootstrap `_utilities.scss` + `_api.scss`
- **THEN** SHALL 生成 `.navbar-expand-sm`、`.navbar-expand-md`、`.navbar-expand-lg`、`.navbar-expand-xl`、`.navbar-expand-xxl` 等响应式变体

#### Scenario: 复合选择器组合
- **WHEN** 编译包含 aria 属性选择器的 utility（如 `[type="checkbox"]`）
- **THEN** SHALL 生成完整的复合选择器块如 `.btn-check:checked:focus-visible + .btn`

#### Scenario: 伪元素生成
- **WHEN** 组件 SCSS 中包含 `::after`、`::before` 等伪元素
- **THEN** SHALL 完整生成伪元素选择器及其声明块

#### Scenario: mixin body selector 分派
- **WHEN** mixin body 中嵌套 `&-hover` 且 mixin 被 `@include` 在 `.btn` 下
- **THEN** 组合结果 SHALL 为 `.btn-hover`（& 替换）
- **WHEN** mixin body 中嵌套 `.subtitle` 且 mixin 被 `@include` 在 `.card` 下
- **THEN** 组合结果 SHALL 为 `.card .subtitle`（descendant combinator）
- **WHEN** mixin body 中嵌套 `:hover` 且 mixin 被 `@include` 在 `.card` 下
- **THEN** 组合结果 SHALL 为 `.card:hover`（compound combinator）

### Requirement: Vendor prefix 自动注入
eval 层 SHALL 提供 vendor prefix 注入 mixin 或 eval 钩子，当声明以下属性时自动补全 prefix 变体：`file-upload-button` → `-webkit-file-upload-button`（伪元素）; `column-gap` → `-moz-column-gap`; `object-fit` → `-o-object-fit`; `mask-position` → `-webkit-mask-position`; `transition` → `-webkit-transition`/`-moz-transition`; `appearance` → `-webkit-appearance`/`-moz-appearance`。

#### Scenario: -webkit-file-upload-button
- **WHEN** 编译 Bootstrap form-control 相关 SCSS
- **THEN** SHALL 生成 `.form-control::-webkit-file-upload-button` 选择器

#### Scenario: -moz- 前缀延伸
- **WHEN** 编译含 column-gap 属性的规则
- **THEN** SHALL 同时输出 `-moz-column-gap` variant

#### Scenario: -o- 前缀补全
- **WHEN** 编译含 object-fit 属性的规则
- **THEN** SHALL 同时输出 `-o-object-fit` variant

#### Scenario: transition 多前缀
- **WHEN** 编译含 transition 属性的规则
- **THEN** SHALL 同时输出 `-webkit-transition` 和 `-moz-transition` variant

### Requirement: 值表达式求值完整性
涉及 `calc()`、负值乘法、颜色函数返回值的声明 SHALL 在编译产物中原样存在。`Value::Calc` 序列化时 SHALL 保留完整表达式（包括嵌套 var() 调用和算术运算）。

#### Scenario: calc() 表达式保持
- **WHEN** SCSS 声明 `margin-top: calc(-1 * (var(--bs-popover-arrow-height)) - var(--bs-popover-border-width));`
- **THEN** 编译输出 SHALL 包含该声明（不截断不求值）

#### Scenario: 颜色函数输出
- **WHEN** 编译 `--bs-btn-focus-shadow-rgb: 211, 212, 213;`
- **THEN** 输出 SHALL 精确包含 `211, 212, 213`（RGB 三元组）

#### Scenario: !important 与 complex value
- **WHEN** 编译 `border-color: rgba(var(--bs-white-rgb), var(--bs-border-opacity)) !important;`
- **THEN** 输出 SHALL 完整保留该声明（含 rgba var 嵌套 + !important）

#### Scenario: Calc 类型序列化
- **WHEN** Value::Calc 包含 `100% - 20px`
- **THEN** `value_to_string` SHALL 输出 `calc(100% - 20px)` 完整表达式

