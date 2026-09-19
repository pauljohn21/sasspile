## Why

math 模块通过率 91.6%（445/486），17 个函数已 100% 通过，但仍有 41 个失败分散在 9 个函数中。这些失败是sass-spec 总体通过率突破 66% 的低垂果实——修复难度低、影响面明确，属于纯打磨收尾。

## What Changes

修复以下 math 函数的 sass-spec 失败用例：
- **random**: 5 ERR — null 参数校验、浮点 int 边界过严 ($limit: 1.0000000000001)
- **variables**: 7 DIFF — 只读变量 $/math-variables 等处理
- **unit**: 5 DIFF — 单位一致性和转换
- **pow**: 5 DIFF — 幂运算边角场景
- **tan**: 4 DIFF — 正切函数（可能角度单位残留）
- **clamp**: 4 DIFF — 钳制边界
- **sin**: 2 DIFF — 正弦残留场景
- **div**: 2 DIFF — 除法边角
- **comparable, unitless, round, min, max, hypot, atan2**: 各 1 DIFF — 单一场景修复

## Capabilities

### New Capabilities

_(无新增能力)_

### Modified Capabilities

- `core_functions-math`: 修正 random 参数校验、unit/variables 只读常量、pow/tan/clamp/sin/div 的 sass-spec 预期输出格式

## Impact

- 受影响文件：`src/eval/math.rs`、`src/builtins/math_trig.rs`、`src/builtins/registry.rs`
- 测试影响：sass-spec +41 cases（预估 41/486 → 接近 100%，总体 +0.34%）
- 无 API 变更，无依赖变更
