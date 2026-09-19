# math-clamp-comparable-error 任务清单

## 1. clamp 单位转换 + 范围限制 (4 cases)

- [x] 1.1 读取 4 个 clamp 失败 case 的完整 actual vs expected 差异
- [x] 1.2 在 `math.rs` 新增 `call_clamp` 函数：
  - 提取 (min, number, max) 三个参数
  - 检查所有三个参数的单位兼容性
  - 将 number/max 转换到 min 单位后进行 CSS spec 数值比较
  - 胜出区段决定输出单位（min/number/max）
  - min > max 时回退到 min
- [x] 1.3 在 clamp 调用入口添加 tracing span 记录 min/number/max 值和单位
- [x] 1.4 验证：4 个 clamp case 全部通过（`min_greater_than_max`、`preserves_units/{min,number,max}`）

## 2. max/min 单位兼容性检查 (2 cases)

- [x] 2.1 在 `math.rs` 的 max/min 分支，前置检查所有参数对单位兼容性
- [x] 2.2 不兼容时返回错误：`Incompatible units 1px and 1s.`
- [x] 2.3 验证 `max(1px, 2s)` 和 `min(1px, 1s)` 报错正确

## 3. comparable 函数补全 (1 case)

- [x] 3.1 定位 `compatible(1px, 1/1px)` 差异（actual=true, expected=false）
- [x] 3.2 修复 `ops.rs::div`：分子无 unit 分母有时，结果单位为倒数 `1/{u}`
  - `1/1px` → result_unit = "1/px"
  - `units_compatible("px", "1/px")` 返回 false（不在任何兼容组）
- [x] 3.3 验证 `comparable/unit/to_inverse` 通过

## 4. 回归验证

- [x] 4.1 运行 `diag_math` 确认 math 模块总失败数从 16 降至 10
- [x] 4.2 核心测试 241/241 全通过
- [x] 4.3 sass-spec math 统计确认净增 6 cases（472/482 vs 466/482 = +6 net）

## 结果

| 指标 | 修复前 | 修复后 | delta |
|------|--------|--------|-------|
| math module | 466/482 (97%) | 472/482 (98%) | +6 |
| 4 clamp | FAIL | PASS | +4 |
| 2 incompatible_units | FAIL | PASS | +2 |
| comparable/to_inverse | FAIL | PASS | +1 |
| 核心测试 | 241/241 | 241/241 | 0 |
