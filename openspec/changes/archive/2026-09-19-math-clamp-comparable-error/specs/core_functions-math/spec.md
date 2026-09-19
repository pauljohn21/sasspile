# math 子域规范 — clamp / comparable / max-error

## clamp 函数

### 输入输出规范

| 输入 | 期望行为 |
|------|----------|
| `clamp(MIN, VAL, MAX)` (三者单位兼容) | VAL 转换到 MIN 单位后 clamp；返回 clamp 值（保留 VAL 原单位） |
| `clamp(MIN, VAL, MAX)` (单位不兼容) | 报错 `$: incompatible units` |
| `clamp(1, 2, 0)` (无单位，min > max) | 按 CSS spec，min > max 时当作 min = max；返回 1 |

### 单位转换场景

```
clamp(180deg, 0.75turn, 360deg)
→ 0.75turn = 270deg (转换后)
→ 270 在 [180, 360] 范围
→ 保留原单位返回 0.75turn
```

## max/min 兼容性

### 错误检测

| 输入 | 输出 |
|------|------|
| `max(1px, 2s)` | Error: incompatible units |
| `min(1px, 1em)` | 2 (兼容，隐式转换) |

## comparable 边界场景

### to_inverse

```
compatible(1px, 1/1px)
→ 倒数单位的兼容性需按 CSS 规范处理
```
