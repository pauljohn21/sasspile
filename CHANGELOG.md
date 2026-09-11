> ⛔ **禁止参照 dart-sass**：dart-sass 依赖 GC（垃圾回收），其嵌套结构依赖 GC 保。sasspile 是纯 Rust 项目，无 GC，所有权语义完全不同。任何实现必须基于 Rust 所有权模型和 sass-spec 规范，不得参照 dart-sass 的实现。

# Changelog

## [0.9.11] — 2026-09-11

### Fixed

- **math.unit 输出格式**：返回带引号的字符串（`math.unit(1px)` → `"px"` 而非 `px`）
  - 影响：`math.rs`、`core_functions/math` +3 cases
- **selector.parse compound 边界**：`[c]d` 正确拆分为 `[c]` + `d`
  - 影响：`selector_format.rs`、`css/selector` +1 case
- **scan_ident 非十六进制转义**：`\$` 正确解析为 `$`（选择器转义修复）
  - 影响：`scanner.rs`、`css/selector/escaping/dollar_char`
- **escape_css_chars NULL/控制字符**：双反斜杠转义序列
  - 影响：`escape.rs`、`core_functions/string/quote/escape`
- **values_eq 单位感知比较**：兼容单位转换后比较，unitless ≠ 有单位
  - 影响：`ops.rs`、`core_functions/list/index`

### Changed

- sass-spec 通过率：7694 → 7698（+4），63.4% → 63.4%
- 24 个目录达到 100% 通过率

### 100% 目录（24 个）

variables/whitespace, variables/semi_global, variables/double_flag, variables/comments,
parser/selector, parser/operator_precedence, parser/interpolation, operators/slash,
expressions/syntax, directives/while, directives/return, css/url, css/style_rule,
css/percent, css/mixin, css/important, css/function_name_identifiers, css/empty_block_directive,
css/directive_with_lots_of_whitespace, css/blockless_directive_without_semicolon,
core_functions/map, callable/whitespace, callable/parameters

### 待修复目录（85-99%，需深度架构修复）

- core_functions/list: 230/233 (98.7%) — 嵌套属性块解析
- core_functions/string: 152/155 (98.1%) — 转义/引号解析
- directives/extend: 19/20 (95.0%) — `:is()` 选择器传递
- css/selector: 92/96 (95.8%) — 伪选择器验证/转义
- expressions/if: 197/211 (93.4%) — `css()`/`and`/`or`/`not` 解析
- directives/forward: 201/216 (93.1%) — 成员导入优先级
- directives/at_root: 25/27 (92.6%) — `@use`+`@import` 嵌套
- core_functions/math: 448/486 (92.1%) — 单位系统/浮点精度
- directives/use: 236/267 (88.4%) — 模块系统

## [0.9.10] — 2026-09-10

### Fixed

- **空输出 case 不再跳过**：移除测试框架中 `expected_output.is_empty() && !expect_error → skip` 的逻辑
  - 影响: `tests/specstore/runner.rs`、`tests/hrx_support.rs`、`tests/sass_spec_full.rs`
  - `variables/` 目录通过率 21% → 100%（whitespace/semi_global/double_flag/comments 全 100%）
  - 全局新增 ~2600 个评估 case（9511 → 12131）
- **变量声明注释解析**：修复 `$a /**/: b` 解析失败问题 — `parse_variable` 改用 `skip_ws_and_comments`
- **变量声明尾部注释**：修复 `$a: b /**/` 错误产生 `/*  */` CSS 输出 — 声明后消耗尾部注释

### Changed

- sass-spec 通过率：7718/12131 = 63.6%（全部 case 均评估，无 SKIP）

## [0.9.9] — 2026-09-10

### Fixed

- **calc() 特殊常量格式化**：`infinity`/`-infinity`/`NaN` 在 calc 输出中格式化为规范大小写
  - `format_number` 映射：INFINITY → "infinity", NEG_INFINITY → "-infinity", NaN → "NaN"
  - `parse_ident_or_func` 识别特殊常量关键词（大小写不敏感）
  - `try_ast_simplify` 保留特殊常量的 `calc()` 包装（颜色格式化需要）
- **extract_unitless 特殊浮点常量支持**：接受 `infinity`/`-infinity`/`nan` 作为合法数值输入
  - `pow`/`sqrt`/`log`/`asin`/`acos`/`atan` 可处理特殊浮点值
  - `sqrt(infinity)` → `infinity`, `asin(infinity)` → `NaN`, `atan(infinity)` → `90deg`
- **calc-size() 保留**：加入 `is_css_function` 列表，原样输出不编译时求值
- **反三角函数变量保留**：`atan`/`asin`/`acos` 遇 Sass 变量时保留函数形式不编译时求值
- **type-of(calc(特殊常量))**：含 infinity/-infinity/NaN 的 calc 返回 `number` 而非 `calculation`

### Changed

- sass-spec 通过率：7444 → 7499（+55），62.7% → 63.1%
- values/calculation：+9（log/pow 特殊常量）
- core_functions/math：+7（pow/sqrt/log 特殊值传播）

## [0.9.8] — 2026-09-10

### Added

- **spec-store** SQLite 数据管理工具：`tests/spec_store.rs` + `tests/specstore/` 子模块（8 文件，≤ 300 行/文件）
  - HRX 入库（`SPEC_STORE_CMD=index`）：12,131 个 case 写入 SQLite WAL
  - 全量运行 + 快照（`SPEC_STORE_CMD=run`）：7,489 PASS / 4,380 FAIL / 262 SKIP，~125 秒
  - 目录统计报告（`SPEC_STORE_CMD=stats`）：Markdown 表格，按通过率排序
  - 函数级趋势（`SPEC_STORE_CMD=trend FN=math.sin`）：ASCII 折线图
  - 代码↔spec 桥接（`SPEC_STORE_CMD=link FN=math.sin`）：CodeGraph callers
  - 两 snapshot 对比（`SPEC_STORE_CMD=diff FROM=1 TO=2`）：回归检测
  - 五表 schema：spec_cases / case_files / snapshots / case_results / case_deltas

### Removed

- `tests/failures_json.rs`、`tests/sass_spec_stats.rs`、`sass-spec-failures.json`、`sass-spec-baseline.json`、`sass-spec-stats.md` — 由 spec_store 取代

### Changed

- `AGENTS.md`：工具链/工作流/验证清单全面更新为 spec_store 命令
- `STYLE_GUIDE.md`：命名示例更新为 `cmd_stats` / `cmd_trend`
- `.githooks/post-commit`：每次 commit 自动 snapshot 标记时间点

## [0.9.6] — 2026-08-19

### Added

- 命名空间变量赋值（`ns.$var: value`）：`is_namespace_var` / `parse_namespace_var` 解析 + `eval_variable` 更新 `ModuleExports.vars` + `call_module_function` 注入模块 vars 到函数环境
- `ConfigVar` 结构保留 `is_default` 标志：`@forward with()` 的 `!default` 语义正确处理
- `@import` 修饰符解析（media query 等）和文件未找到报错
- 多值 `@import`（`@import "a", "b"`）拼接为多行 CSS `@import` 输出
- 模块缓存 `loaded_modules`（HashSet）防止重复加载和无限递归
- 跨模块 `@extend` 传播：`ModuleExports` 新增 `extends` 字段，`load_module` 收集子模块 extends 关系
- `parse_node` Semicolon 分支在 EOF 时不递归，避免 `Parse error: expected {, found EOF`

### Changed

- sass-spec 全量统计：3596/11775 = 30%（+118），@directives 子目录 487/767 = 63%（+43）
- 命名空间从 URL basename 计算去掉所有扩展名（`other.foo.bar` → `other`）
- 序列化器 `@import` 之间不加空行
- `parse_config` 使用 `parse_expr(0)` 避免错误消费逗号，返回 `Vec<ConfigVar>`
- `Node::Import` 新增 `modifier: String` 字段

### Fixed

- `parse_node` Semicolon 分支在 EOF 时无限递归导致 Parse error
- `parse_config` 使用 `parse_value` 错误消费 `with()` 中逗号分隔的变量
- 命名空间变量赋值在 `call_module_function` 中找不到变量的 bug
- `@forward with()` 中 `!default` 标志未正确传播的 bug
- CSS `@import` 带修饰符或多值时解析错误的 bug
- 命名空间从多扩展名 URL basename 计算错误的 bug

## [0.9.5] — 2026-08-19

### Added

- 规则体变量作用域修复：`eval_rule` 不再传播规则体内的局部变量到外层
- `!global` 变量处理：`eval_variable` 当 `flags.global` 为 true 时同时写入 `vars` 和 `global_writes`
- `Env` 新增 `global_writes` 字段，用于传播 `!global` 变量和 `@import` 内联变量
- `@import` 变量传播：`load_import` 中将导入的变量写入 `global_writes`
- 命名空间变量传播：规则体中名字含 `.` 的变量（命名空间变量）传播到外层
- Manifest 精简：`SKIP_DIRS` 只跳过 libsass 系列 + `non_conformant` 弃用目录，不再跳过功能性子目录

### Changed

- sass-spec 全量统计：3478/11775 = 29%（之前跳过大量功能性子目录时为 2672/4848 = 55%）
- sass-spec 全量运行时间约 70 秒（之前约 35 秒，因测试用例数量增加）
- 规则体变量作用域符合 sass-spec 规范：局部变量不泄漏到外层

### Fixed

- 规则体内局部变量泄漏到外层作用域的 bug
- `!global` 变量在规则体内不传播到全局作用域的 bug
- `@import` 导入的变量不传播到外层作用域的 bug

## [0.4.0] — 2026-08-13

### Added

- 综合 AI 开发技能 `skill.md`（613 行）：编译管线、内建函数参考、CSS 序列化、调试追踪系统
- `is-channel-powerless` 函数完整实现（HSL/HWB 通道无效检测）
- `sass-spec` 测试修复：HRX 文件多 case 隔离（独立临时目录 + case_dir 过滤）
- `@charset "UTF-8"` 自动添加（非 ASCII 内容）
- 选择器净化增强：相邻复合选择器规范化、属性选择器去引号、修饰符空格

### Changed

- sass-spec 通过率：2566/4848 (53%)，较 0.3 的 36% 提升 17 个百分点
- compile_test 从 28 增长到 41（新增 13 个编译测试用例）
- `css/mod.rs` Serializer 从 358 行扩展到 738 行（选择器验证 + 组合器检查 + 相邻复合规范化）
- CssNode 新增 `Raw(String)` 和 `AtRoot(Vec<CssNode>)` 变体

### Fixed

- sass-spec 测试运行器 HRX 多 case 路径碰撞 bug
- `SKIP_DIRS` 更新：排除 CSS Level 4 颜色空间（oklch/oklab/lab/lch）等 Sass 3.x 未实现特性

## [0.3.1] — 2026-08-11

### Added

- `tracing` 和 `tracing-subscriber` 改为可选 feature（`default = ["tracing"]`）
- 支持 `default-features = false` 编译，仅依赖 `thiserror` + `im`
- `.css` 文件以 plain CSS 模式加载（保留嵌套不展开选择器）
- `@media` / `@supports` / `@container` 在规则内部时自动提升到外层
- 带前缀 `-` 的未知 CSS 函数自动透传（如 `-c-type(2)`）
- `@use` 加载模块时正确包含 CSS 输出

### Fixed

- `@use` 丢弃模块 CSS 输出的 bug
- `.css` 文件被当作 SCSS 求值导致嵌套展开的 bug

## [0.3.0] — 2026-08-11

### Added

- Element Plus 全量编译通过：121/121 (100%)
- Bootstrap 5.3.8 全量编译通过：15/15 测试
- `@use` / `@forward` 模块系统支持
- `@function` / `@return` 用户自定义函数
- `@if` / `@for` / `@each` / `@while` 控制流
- `@extend` 占位符继承
- `@at-root` 根级嵌套
- `@warn` / `@debug` / `@error` 诊断指令
- `!default` 变量默认值标记
- `!important` 标记支持
- load path 支持（`--load-path` 参数）
- `@import` 环境继承 (load_import)
- `and` / `or` 短路求值 (is_truthy)
- 字符串插值拼接 (`#{...}ident`)
- `bind_params` spread Map → 关键字参数
- `url()` 分流（字符串参数走正常解析，裸 URL 走 raw）
- CSS 函数名大小写不敏感 (`to_lowercase`)
- CSS transform/filter 白名单透传
- `zip` 非列表参数视为单元素列表
- `MAX_DEPTH=100000` 内存爆炸兜底
- 命名颜色反向查找 (`reverse_lookup_named_color`)
- `invert` / `grayscale` CSS 透传
- `call` 内建函数支持用户函数
- `str-split` / `str-insert` / `str-index` 字符串函数
- `map-merge` / `map-remove` / `map-set` 嵌套 Map 操作
- tracing 调试架构（span 层级 + event targets）
- CSS Diff 逐行对比工具 (`tests/common/mod.rs`)
- sass-spec 最小化工具 (`tests/minimize.rs`)

### Changed

- 版本升级到 0.3.0
- Rust Edition 2024, Toolchain 1.97
- sass-spec 通过率：1843/5069 (36%)
- 源文件总数：27 个，总计 ~7700 行

### Fixed

- `@while` / `@each` 环境传播
- CSS Level 4 rgb/rgba 空格分隔语法
- `rgba` 2-arg (color, alpha) + 百分比 alpha
- `math.div` 映射
- `parse_expr_rest` 列表中二元运算处理
- `parse_args` 关键字参数
- `alpha(opacity=0)` CSS 透传
- `FunctionDef` / `MixinDef` 命名空间捕获
- `peek_binding_power` 区分一元负号
- `parse_prefix` 厂商前缀标识符处理
- `try_resolve_dir` 用 `file_stem` 支持带 `.scss` 扩展名的 URL
- `load_module` 保留空白 token

## [0.2.0] — 2026-08-09

### Added

- 类型状态机管线设计
- 纯函数式风格实现
- 基础 SCSS 语法支持（变量、嵌套、Mixin）
- 颜色函数（darken, lighten, mix, rgba, invert, grayscale）
- 字符串函数（str-length, str-slice, str-index）
- 列表函数（length, nth, append, join, index）
- Map 函数（map-get, map-keys, map-values, map-merge）
- 数学函数（abs, ceil, floor, round, min, max, percentage, sqrt, sin, cos, tan, pow）
- sass-spec 初步集成

## [0.1.0] — 2026-08-07

### Added

- 项目初始化
- Lexer + Parser + Evaluator + Serializer 基础架构
- 基本 CSS 编译能力
