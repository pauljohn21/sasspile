# /trace — 追踪 sasspile 错误链路

使用 tracing span + event 追踪 sasspile 的错误链路，定位根因。

## 步骤

1. 读取 `.claude/skills/tracing-debug/SKILL.md` 了解 span + event 架构
2. 用户提供错误信息或测试失败用例
3. 在 `src/lib.rs` 的 `test_debug_bs_close` 中设置对应的输入
4. 用 `RUST_LOG` 追踪错误链路
5. 分析 span 嵌套 + event 值快照，定位根因
6. 提出修复方案，修复后验证

## 使用方式

```
/trace <错误描述或测试用例名>
```

## 命令

```bash
# Level 1: 只看错误（最快）
RUST_LOG=error cargo test --lib test_debug_bs_close -- --nocapture

# Level 2: 看函数调用链（推荐）
RUST_LOG=info cargo test --lib test_debug_bs_close -- --nocapture

# Level 3: 看完整 span 嵌套（最详细）
RUST_LOG=debug cargo test --lib test_debug_bs_close -- --nocapture

# Level 4: 看所有 trace（含值表达式）
RUST_LOG=trace cargo test --lib test_debug_bs_close -- --nocapture

# Per-target 过滤——只看颜色转换值快照
RUST_LOG="sasspile::color=trace" cargo test --lib -- --nocapture

# Per-target 过滤——只看 @extend 匹配
RUST_LOG="sasspile::extend=debug" cargo test --lib -- --nocapture

# Per-target 过滤——只看 BinOp 运算值
RUST_LOG="sasspile::binop=trace" cargo test --lib -- --nocapture

# 组合多个 target
RUST_LOG="sasspile::color=debug,sasspile::extend=info" cargo test --lib -- --nocapture
```

## 输出格式

```
## 错误追踪

### Span 链路
eval_nodes → eval_node_item → eval_node → eval_rule → ...

### Event 值快照（如果有）
TRACE sasspile::color: h=120 s=0.5 l=0.5 converting HSL to RGB
TRACE sasspile::color: r=128 g=255 b=0 HSL to RGB result
TRACE sasspile::binop: op=Sub left=c right=d binop operands evaluated
TRACE sasspile::binop: op=Sub result=c-d binop result

### 根因分析
- 错误类型: 未定义函数 / 语法错误 / ...
- 位置: eval_builtin / parse_args / ...
- 原因: ...

### 修复方案
- 修改文件: ...
- 修改内容: ...
```

## Event Target 速查

| Target | Level | 场景 |
|--------|-------|------|
| `sasspile::color` | trace | 颜色转换函数输入/输出 |
| `sasspile::color` | debug | 颜色 builtin 函数入口/结果 |
| `sasspile::extend` | info | @extend 匹配成功 |
| `sasspile::extend` | debug | 选择器替换细节 |
| `sasspile::binop` | trace | 二元运算操作数值 + 结果 |
| `cssdiff` | info/debug | CSS diff 检测 + 行级差异 |
| `minimize` | info/debug | 最小化轮次 + 移除尝试 |
