## Context

sasspile 颜色系统的基础设施已经完整：
- `ColorSpace` 枚举已包含 Lab/Lch/Oklab/Oklch
- `parse_lab/parse_lch/parse_oklab/parse_oklch` 解析器已实现
- `color_conv.rs` f64 精度转换函数已实现
- `display.rs` 序列化支持所有现代色彩空间
- `ChannelSet` 支持 modern 通道名（chroma/lightness/a/b）

当前 sass-spec 颜色域通过率仅 51%（core_functions）。失败根因：
1. 测试工具链缺少 `core_functions/color/utils` 共享模块
2. 部分序列化精度不匹配（需 ~10 位小数）
3. `@derive(BuiltinRegistry)` 宏未注册 lab/lch/oklab/oklch 构造函数

## Goals / Non-Goals

**Goals:**
- 补齐 lab/lch/oklab/oklch 构造函数注册（parser 已有，缺函数分派）
- 扩展序列化逻辑以精确输出现代色彩空间格式
- 修复测试工具链以加载 sass-spec 共享 utils 模块
- 提升 sass-spec 颜色域通过率至 75%+

**Non-Goals:**
- 不实现 CSS Color 4 gamut mapping（已有 gamut.rs）
- 不重构 color_conv.rs（f64 精度已实现）
- 不涉及其他核心功能修改

## Decisions

### Decision 1: 构造函数注册方式

**选择**: 在 `manual_dispatch.rs` 中新增 lab/lch/oklch 分派（oklab 已存在）

**理由**:
- 现有 `parse_color_fn` 已统一处理 lab/lch/oklab/oklch
- `manual_dispatch.rs` 已导入 `parse_color_fn`
- 只需在 match arm 中添加 `"lab" | "lch" | "oklch"` 即可

**替代方案**: 使用 `#[derive(BuiltinRegistry)]` 宏。但 lab/lch/oklch 的参数解析与标准宏不匹配（空格分隔而非逗号），手工分派更直接。

### Decision 2: 序列化精度

**选择**: 扩展 `format_num` 函数以支持动态精度（当前固定 10 位）

**理由**:
- CSS Color 4 规范要求颜色通道输出足够位数以区分可区分值
- `FLOAT_PRECISION_INV = 1e10`（10 位小数）已满足大多数场景
- 特殊需求（如 hue 的 deg 输出）通过 `DEG_UNIT` 后缀处理

### Decision 3: 共享测试模块策略

**选择**: 在 HRX 解析时将 `core_functions/color/utils` 作为隐式依赖自动加载

**理由**:
- sass-spec 的 to_space 目录需要 `_utils.scss`
- 现有 VFS 结构支持从同一 HRX 读取
- 不应修改测试用例本身

**替代方案**: 创建单独的 `@use` 路径映射。但会增加测试框架复杂度。

## Risks / Trade-offs

- [精度损失] → 使用 f64 并在最终输出时保持 10 位小数舍入
- [测试稳定性] → 共享模块加载失败不应导致 panic，应给出明确错误信息
- [回归风险] → 修改 `format_num` 可能影响 RGB/HSL 输出，需保留旧行为
