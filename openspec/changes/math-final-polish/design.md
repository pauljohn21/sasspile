## Context

math 模块通过率 91.6%，只剩 41 个失败 case 分散在 9 个函数。17 个函数已 100% 通过。失败分为三类：

1. **ERR（编译错误）— 5 个**: 全部在 `random`，原因是 null 参数未校验和 int 边界判断过严 ($limit: 1.0000000000001 被当错误拒绝)
2. **DIFF（输出差异）— 36 个**: 输出格式/精度/单位与 sass-spec 预期不符

现有函数位于 `src/eval/math.rs`（主入口）和 `src/builtins/math_trig.rs`。

## Goals / Non-Goals

**Goals:**
- 修复 41 个 sass-spec 失败用例，math 模块通过率 → ~100%
- 遵循调试协议（4 步流程）建立证据链

**Non-Goals:**
- 不引入新函数或 API
- 不重构 math 模块整体架构
- 不改变已通过用例的行为

## Decisions

### 1. 逐函数定点修复

不一键全改，按函数逐个攻克：random → variables/unit → pow/tan/clamp/sin/div → 单例收尾。
每个函数：插桩 → 采集 → 定位 → 修复 → 验证。

### 2. random 的 int 边界放宽

`$limit` 参数目前的 `is_int` 检查对浮点噪声过严。Decision：采用 epsilon 容差（|f - f.round()| < 1e-9 视为 int），符合 SCSS 规范对实数到整数的隐式转换。

### 3. DIFF 修复优先查 actual output

spec_store 记录 `failure_type='DIFF'` 时保存了 `actual_css`。修复前先看差异常模式，定位是精度截断、单位格式还是运算逻辑。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| random 放宽 int 检查可能接受真正非法输入 | 保留 null 检查，只放宽浮点边界 |
| DIFF 修复误改逻辑影响其他 case | 修复后跑 ep_full 全量验证 |
| 某些 DIFF 是 sass-spec 预期错误 | 交叉核对 spec 文件确认预期 output |
