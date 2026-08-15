# Sass 兼容性注意事项

## 已知差异

### 1. 数值精度
- Dart Sass 使用 10 位小数精度
- Rust f64 默认精度
- 可能需要自定义格式化

### 2. 字符串转义
- 确保 Unicode 转义一致
- 单引号/双引号行为差异

### 3. @use vs @import
- Sass 正在迁移到 @use
- 需要兼容两种语法

### 4. 模块加载路径
- @use "module" 搜索规则
- _index.scss 支持
- @forward 循环处理

### 5. 空列表/Map
- `()` 作为空列表的特殊处理
- `()` 在 Map 中的区分

### 6. 除法语义
- `/` 作为除法 vs 分隔符
- 括号内 / 视为除法
- `#{...}` 内 / 视为除法

### 7. 颜色操作
- 颜色名称识别
- 透明通道处理
- 百分比 vs 数字（0-255）

### 8. @extend 限制
- 跨媒体查询 @extend
- 复杂选择器限制
- 自引用检测

## 调试技巧

1. 使用 `cargo expand` 查看宏展开
2. `RUST_LOG=trace` 获取详细日志
3. 单个 spec 用例隔离运行
4. 对比 Dart Sass CLI 输出

## 参考实现

- Dart Sass：https://github.com/sass/dart-sass
- Sass 规范：https://github.com/sass/sass
