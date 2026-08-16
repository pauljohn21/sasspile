# CSS 生成 ❌ 待开发

## 职责

将求值后的 AST 转换为 CSS 文本，支持多种输出格式。

## 计划文件结构

```
css/
├── mod.rs           # 入口
├── generator.rs     # 生成器主体
├── rules.rs         # 规则展开
├── atrules.rs       # @规则输出
└── sourcemap.rs     # Source Maps
```

## 输出格式

```rust
pub enum OutputStyle {
    Expanded,    // 多行，不压缩
    Compact,     // 单行多规则
    Compressed,  // 最小化
}
```

## 关键功能

1. **选择器展平**：嵌套规则展开为平面选择器
2. **属性排序**：可选的字母排序
3. **@规则生成**：`@media`, `@supports`, `@import`, `@charset`
4. **注释处理**：`/*! */` 版权注释保留
5. **Source Maps**：v3 JSON 格式

## 类型定义

```rust
pub struct CssRule {
    pub selector: String,
    pub declarations: Vec<CssDeclaration>,
}

pub struct CssDeclaration {
    pub name: String,
    pub value: String,
    pub important: bool,
}

pub struct CssAtRule {
    pub name: String,
    pub prelude: String,
    pub block: Vec<CssNode>,
}
```

## Source Maps

- 使用 vlq 编码
- v3 JSON 格式
- 包含 sources、mappings、names 字段

## 测试重点

- 基本 CSS 输出
- 嵌套选择器展平
- @规则输出
- 压缩输出
- 注释保留
- Source Maps 生成
