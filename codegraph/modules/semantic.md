# 语义分析（待开发）

## 职责

构建作用域、解析模块依赖、验证 @extend 目标、收集函数/混入定义。

## 计划文件结构

```
semantic/
├── mod.rs             # 分析入口
├── symbol_table.rs    # 作用域栈
├── module.rs          # @use/@forward 解析
├── extend.rs          # @extend 验证
└── definitions.rs     # 定义收集
```

## 符号表

```rust
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

pub enum Scope {
    Global,
    Local,
    Param,  // 函数/混入参数
}

pub struct Symbol {
    name: String,
    kind: SymbolKind,
    span: SourceSpan,
}

pub enum SymbolKind {
    Variable(Value),
    Function(FunctionDef),
    Mixin(MixinDef),
    Placeholder(Selector),
}
```

## 模块解析

- 构建依赖有向图
- Kahn 算法拓扑排序
- 循环依赖检测
- 入度为 0 的模块可并行编译

## @extend 验证

- 检查目标选择器存在性
- 验证伪类和组合子的有效性

## 定义收集

- Mixin 注册表
- Function 注册表
- 重复定义检测

## 测试重点

- 作用域链
- @use / @forward 解析
- 模块依赖图拓扑排序
- 循环依赖检测
- @extend 验证
