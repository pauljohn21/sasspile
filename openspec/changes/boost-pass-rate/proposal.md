## Why

sass-spec 当前通过率 ~7677/12131 (63.3%)，存在多个可以快速提升的子域。通过系统性分析失败分布，识别出 Quick Wins（@extend、meta、math 的边界 case）、Root Cause（`+` 运算符缺失组合、CSS 序列化格式）、以及中期架构改进（selector AST 操作）三个层次的机会，预计可提升至 66-67%。

## What Changes

- **Phase 1 — Quick Wins**：修复 @extend (53%→88%)、meta get-function/keywords/global_variable_exists、math 边界行为（pow/atan2/unit）
- **Phase 2 — Root Cause**：补全 `+` 运算符缺失 Value variant 组合（Map+X、List+Map bracketed）、CSS 序列化格式规范化、values/numbers 精度处理
- **Phase 3 — Selector AST**：完善 selector nest/unify/extend/append 操作（selector 45% 通过率的最大失败池）

无破坏性变更，全部为 bug 修复和缺失功能补全。

## Capabilities

### New Capabilities

- `math-edge-cases`: math pow/atan2/clamp/unit 边界行为完善
- `meta-introspection-full`: get-function 返回实际引用、keywords 参数解包、global_variable_exists 全局检查
- `selector-ops-complete`: selector nest/unify/extend/append 全 operational

### Modified Capabilities

- `value-ops`: `+` 运算符补全缺失 variant 组合（当前 catch-all 返回 "Unsupported + operation"）
- `css-serializer`: CSS 输出格式规范化（空格/分号/换行/缩进统一）
- `extend-directive`: @extend 语义完善（complex selector extend、optional/mandatory 行为）

## Impact

- **受影响源码**: `src/eval/value/ops.rs`、`src/eval/builtin/math.rs`、`src/eval/builtin/meta.rs`、`src/css/serializer.rs`、`src/eval/extend.rs`、`src/parse/selector_*.rs`
- **受影响测试**: `tests/sass_spec_full.rs`（通过率数字提升）
- **不影响**: 核心架构、公共 API、环境模型
- **风险**: CSS 序列化格式调整可能影响大量 case 的精确字符串匹配，需分批验证
