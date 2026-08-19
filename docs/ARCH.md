# sasspile 架构指南

## 编译管线

sasspile 是一个纯 Rust 函数式 SCSS 编译器，编译管线为：

```
Source → Lexer → Parser → Evaluator → Serializer → CSS
         (lex/)   (parse/)  (eval/)     (css/)
```

### 各阶段职责

1. **Lexer** (`src/lex/`): 词法分析，将 SCSS 源代码转换为 Token 流
2. **Parser** (`src/parse/`): 语法分析，将 Token 流转换为 AST
3. **Evaluator** (`src/eval/`): 求值器，处理变量、函数调用、控制流
4. **Serializer** (`src/css/`): CSS 序列化，将求值结果转换为 CSS 字符串

## 模块结构

```
src/
├── lex/              # 词法分析器
│   ├── mod.rs        # Lexer 实现
│   └── token.rs      # Token 定义
├── parse/            # 语法分析器
│   ├── ast/           # AST 定义
│   │   ├── mod.rs     # 核心类型定义（Node/Value/Color/ColorFormat/BinOp/Param/Arg）
│   │   └── display.rs # Display for Value（ColorFormat 分派序列化）
│   ├── ast_impl.rs    # Node::to_scss()
│   ├── expr/          # 表达式解析
│   │   ├── mod.rs     # Pratt 解析器入口
│   │   └── prefix.rs  # 前缀解析
│   ├── at_rules.rs    # @规则解析
│   └── nodes.rs       # AST 节点解析
├── eval/             # 求值器
│   ├── value/         # 值运算
│   │   ├── mod.rs     # eval_value + eval_binop + units_compatible
│   │   ├── ops.rs     # 算术/比较运算实现
│   │   └── display.rs # inspect_value + 值格式化
│   ├── builtin/       # 内建函数
│   │   ├── math.rs    # 数学函数 + 命名参数合并
│   │   ├── color.rs   # 颜色函数
│   │   ├── map.rs     # Map 函数
│   │   ├── list.rs    # List 函数
│   │   ├── string.rs  # String 函数
│   │   └── selector.rs # Selector 函数
│   ├── rule.rs        # 规则求值（变量作用域隔离）
│   ├── control_flow.rs # @if/@for/@each/@while
│   ├── mixin.rs       # Mixin/Function 处理
│   ├── extend.rs      # @extend 后处理
│   ├── at_params.rs   # @media/@supports 参数插值
│   ├── module.rs      # 模块系统（@use/@forward/@import）
│   ├── builtin.rs     # call_builtin 分派入口 + meta 函数
│   └── color.rs       # 颜色辅助函数
├── css/              # CSS 序列化
│   ├── mod.rs        # Serializer 核心
│   ├── node.rs       # CssNode 定义
│   └── selector.rs   # 选择器处理
├── stage/            # 管线阶段类型
├── lib.rs            # 库入口
└── main.rs           # CLI 入口
```

## 关键设计决策

### 1. 为什么用 enum 表示 Value

```rust
pub enum Value {
    Number(f64, Option<String>),
    String(String, bool),
    Color(Color),
    // ...
}
```

使用 enum 而不是 trait object：
- **零成本抽象**: 编译期单态化，无运行时开销
- **类型安全**: 编译器确保所有分支都被处理
- **内存效率**: 栈分配，无 Box 开销（除递归类型）

### 2. 为什么手递归下降解析

选择 Pratt 解析器（算符优先）手写而非使用 lalrpop/peg：
- **更好的错误信息**: 精确控制错误位置和消息
- **CSS 透传**: 未知语法可直接保留为字符串，无需完整文法
- **SCSS 特殊语法**: `if()` 冒号语法、插值等需要上下文相关解析

### 3. 为什么 CSS 输出用 Serializer 模式

Serializer 直接写入 `String` 缓冲区：
- **避免分配**: 不产生中间 `Vec<String>` + `join`
- **流式输出**: 支持大文件不占用过多内存
- **精确控制**: 空白、换行、缩进完全可控

### 4. 不可变环境 (im::HashMap)

使用 `im-rs` crate 提供不可变 HashMap：
- **函数式风格**: 环境传递而非 mutation
- **结构共享**: 绑定新变量时复用旧环境的大部分内存
- **回滚简单**: 保留旧环境引用即可"回退"

## 性能考虑

- **直接缓冲写入**: Serializer 使用 `&mut String` 而非 `format!` 链
- **Token 过滤**: 词法分析后立即过滤 Whitespace/Eof
- **短路求值**: `and`/`or` 运算符不总是求值两侧
- **迭代器链**: Lexer 返回迭代器，支持惰性求值

## 测试

```bash
# 全部测试
cargo test --test compile_test    # 41 个
cargo test --test stage_test      # 10 个
cargo test --test ast_test        # 8 个
cargo test --test common_test     # 5 个
cargo test --test bs_spec         # 15 个 (Bootstrap 验证)

# sass-spec 完整验证
cargo test --test ep_full         # 121 个 (Element Plus 验证)

# sass-spec 全量统计（约 70 秒）
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture
# 基线：3478/11775 = 29%（全量统计，只跳过 libsass/non_conformant 弃用目录）

# 基准测试
cargo bench
```

## 调试

```bash
# 追踪编译管线
RUST_LOG=info cargo test --test compile_test <test_name> -- --nocapture

# 追踪特定模块
RUST_LOG="sasspile::color=trace" cargo test --test compile_test -- --nocapture

# sass-spec 诊断
cargo test --test cf_diag diag_<subdir> -- --nocapture
```
