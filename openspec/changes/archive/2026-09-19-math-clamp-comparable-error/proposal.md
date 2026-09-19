# math-clamp-comparable-error

## Why

math-final-polish 收官后，math 模块仍有 16 个失败用例。其中 clamp/max/comparable 相关的 ~6 个 case 可通过函数逻辑修复（非架构重构），预估总体通过率 +6 case（math 子域约 +3%）。

## What Changes

### clamp 函数 (4 cases)
- `clamp(min, number, max)` 当前实现将 number 直接返回（无单位转换）
- 需要：将 number 转换到 min 的单位，比较后 clamp 到 [min, max] 范围
- 用例：`math.clamp(180deg, 0.75turn, 360deg)` → 应返回 `0.75turn`（保留 number 的原始单位，但数值 clamp 到范围）

### max/error/incompatible_units (1 case)
- `math.max(1px, 2s)` 应该报错 "incompatible units"
- 当前实现缺少兼容性检查（hypot 有但 max/min 没有）

### comparable/unit/to_inverse (1 case)
- `math.compatible(1px, 1/1px)` — 需检查百分比/倒数单位的特殊处理

## Impact

- 文件：`src/eval/builtin/math.rs` (clamp/max 逻辑)
- 影响范围：仅 math 子域 clamp/max/comparable 函数
- 无架构变更
