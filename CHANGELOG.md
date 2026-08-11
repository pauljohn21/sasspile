# Changelog

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
