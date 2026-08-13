# Changelog

## [0.5.1] — 2026-08-13

### Fixed

- **slash_separated 修复（13 用例）**：顶层属性值中 `/` 视为斜杠分隔列表而非除法
  - 新增 `Separator::SlashDiv` 变体（显示为 `/`，无空格）
  - 保持 `Separator::Slash` 用于嵌套列表（显示为 ` / `，有空格）
  - Parser 添加 `paren_depth` 字段追踪括号深度
  - 添加 `has_other_operator_at_top_level()` 前瞻检测顶层其他运算符
- **插值转义修复**：`Value::Interp` 求值改用 `eval_simple_expr` + `Value::Raw` 避免 CSS 标识符转义（如 `\31` → `1`）
- **alpha_hex 4位颜色解析**：#RGBA 格式改为 R=AA, G=BB, B=CC, A=DD/255
- **alpha 精度截断**：`format_alpha` 和新增 `round_alpha` 截断到 10 位小数
- **precision 修复**：绝对值 < 1e-10 显示为 0；接近整数（距离 < 1e-10）四舍五入为整数

### Stats

- values 子目录通过率：maps 100% (8/8)、lists 100% (18/18)、colors 96% (26/27)、numbers 63% (91/143)
- 全部测试通过：41/41 + 10/10 + 8/8 + 5/5 + 15/15 + 121/121

## [0.5.0] — 2026-08-13

### Changed

- **模块拆分**：`src/parse/ast.rs` 拆分为 `ast/mod.rs` + `ast/display.rs`（+238 行）
- **模块拆分**：`src/parse/expr.rs` 拆分为 `expr/mod.rs` + `expr/prefix.rs`（+177 行）
- **模块拆分**：`src/eval/value/ops.rs` 独立模块（+209 行）
- **基准测试**：新增 `benches/` 性能基准测试套件
- **文档**：新增 `docs/` 架构文档和 API 文档

### Stats

- 42 文件变更，+1946 / -1387 行
- 测试：252/252 全部通过

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
