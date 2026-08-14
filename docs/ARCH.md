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
├── lib.rs            (405) # 公共 API + init_tracing
├── main.rs           (49)  # CLI 入口
├── error.rs          (95)  # 统一错误类型 (SassError)
├── lex/              # 词法分析器
│   ├── mod.rs        (499) # Lexer + Iterator impl
│   └── token.rs      (170) # Token 枚举定义
├── parse/            # 语法分析器（Pratt + 递归下降）
│   ├── mod.rs        (102) # Parser 结构 + parse() + paren_depth
│   ├── ast/          # AST 类型定义
│   │   ├── mod.rs    (420) # Node, Value, Color, BinOp, Separator 等
│   │   └── display.rs (348) # Display trait + escape 函数 + round_alpha
│   ├── ast_impl.rs   (289) # Node::to_scss() 实现
│   ├── at_rules.rs   (536) # 所有 @ 规则解析
│   ├── nodes.rs      (594) # parse_node/parse_rule/parse_decl/parse_body
│   └── expr/         # 表达式解析
│       ├── mod.rs    (328) # Pratt 解析 + has_other_operator_at_top_level
│       └── prefix.rs (512) # parse_number/parse_hash_color
├── eval/             # 求值器
│   ├── mod.rs        (526) # Env + Evaluator + evaluate/eval_nodes
│   ├── rule.rs       (169) # eval_rule + combine_selectors
│   ├── value/        # 值求值
│   │   ├── mod.rs    (449) # eval_value + eval_interp_str + eval_simple_expr
│   │   ├── ops.rs    (290) # add/sub/mul/div/modulo/compare
│   │   └── display.rs (186) # inspect_value
│   ├── control_flow.rs (150) # eval_if/eval_for/eval_each/eval_while
│   ├── mixin.rs      (264) # eval_include + bind_params + call_function
│   ├── extend.rs     (76)  # apply_extends
│   ├── module.rs     (302) # resolve_file + load_module + call_module_function
│   ├── color.rs      (621) # hsl_to_rgb/hwb_to_rgb + builtin_rgba/darken/lighten/mix
│   ├── builtin.rs    (497) # call_builtin 分派入口
│   ├── builtin/      # 内建函数按类别分文件
│   │   ├── color.rs  (553) # 颜色函数
│   │   ├── list.rs   (282) # 列表函数
│   │   ├── map.rs    (302) # 映射函数
│   │   ├── string.rs (281) # 字符串函数
│   │   └── selector.rs (156) # 选择器函数
│   ├── selector/    # 选择器操作
│   │   ├── parse.rs  # 选择器解析
│   │   └── algorithms.rs # 选择器算法
│   └── memory_limit.rs (92) # 内存限制器
├── css/              # CSS 序列化
│   ├── mod.rs        (350) # Serializer（选择器净化 + @规则合并）
│   └── node.rs       (93)  # CssNode 枚举
└── stage/            # 管线阶段类型
    ├── mod.rs        # Stage trait
    ├── source.rs     # Source 类型
    ├── lexed.rs      # Lexed 类型
    ├── parsed.rs     # Parsed 类型
    ├── evaluated.rs  # Evaluated 类型
    └── serialized.rs # Serialized 类型
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

### 4. 求值环境 (HashMap + Clone)

使用标准库 HashMap + clone 实现作用域隔离：
- **简单够用**: 变量数量有限，clone 开销可接受
- **作用域隔离**: 进入新作用域 clone 环境，退出自动恢复
- **减少依赖**: 移除 im-rs，仅保留 thiserror + tracing

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
