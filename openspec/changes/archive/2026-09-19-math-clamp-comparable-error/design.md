# math-clamp-comparable-error 设计

## 1. clamp 单位转换与 clamp 逻辑

**当前**：`clamp(min, number, max)` 直接返回 `number` 的无单位值或 number 原值，完全忽略单位。

**正确行为**：CSS `clamp(MIN, VAL, MAX)`:
1. MIN/MAX 单位必须兼容（否则报错）
2. VAL 单位必须与 MIN/MAX 兼容
3. 将 VAL 转换到 MIN 单位，比较：如果 VAL（转换后数值）< MIN → 返回 MIN；> MAX → 返回 MAX；否则返回 VAL（保留原始单位）

**实现**：
```rust
fn call_clamp(args: &[Value]) -> Result<Option<Value>> {
    // 提取三个参数 (min, number, max)
    // 单位兼容性检查
    // 将 number 转换到 min 单位比较
    // 返回 clamp 后的值（保留 number 原始单位或 min/max 单位）
}
```

## 2. max/min 兼容性检查增强

**当前**：`clamp` 在 `math.rs` 的 min/max 分支只做数值比较，不做单位兼容性检查。

**修正**：在提取数字参数后、比较前，检查：
- 所有有单位参数必须互相兼容（调用 `units_compatible`）
- 不兼容时返回 Err，格式：`$numbers[{}]: {}{} and $numbers[1]: {}{} have incompatible units.`

## 3. comparable 函数补全

**当前状态**：`compatible` 函数的 decimal/percentage 倒数运算逻辑可能遗漏边界。

**需验证**：比较 6 个 clamp case 的实际输出与期望差异，按差异类型分派修复。

## 数据结构变更

无

## 测试覆盖

已有诊断测试 `diag_math` 可回归验证。新增 cases：
- clamp angle 转换（deg↔turn↔rad↔grad）
- max/min incompatible units 错误路径
- compatible 百分比/倒数边角
