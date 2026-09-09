## Why

sass-spec 当前通过率 7142/12131 = 60%，其中 core_functions 占全部失败的 75.1%（3548/4727）。color 子域已深度处理（部分 70-85%），但 **非 color 子域存在大量未修复失败**：selector 45%（489 fail）、meta 63%（179 fail）、list 70%（68 fail）、math 80%（96 fail）、modules 54%（14 fail）。这些领域从未系统攻坚，预估可修复 **+350~600 cases**，将通过率从 60% 推至 63-65%。

## What Changes

按 ROI 从高到低依次处理 5 个非 color 子域：

- **selector 函数** — 修复 `selector-nest`、`selector-merge`、`selector-extend`、`selector-parse`、`selector-unify`、`is-superselector`、`simple-selectors`、`selector-replace` 的选择器解析、合并、比较逻辑。预估 +200~350 cases。
- **meta 函数** — 修复 `meta.module-*`、`meta.get-function`、`meta.keywords`、`meta.content-exists`、`meta.inspect`、`meta.type-of` 等反射函数的类型判断、模块内省行为。预估 +80~130 cases。
- **list 函数** — 修复 `list-separator`、`list-set-nth`、`list-join` 等边界行为。预估 +30~50 cases。
- **math 函数** — 修复边界和精度 case。预估 +40~70 cases。
- **modules 函数** — 修复 module 相关函数行为。预估 +5~10 cases。

**不修改** API、ABI 或用户可见行为——仅修复内建函数实现以符合 sass-spec 规范。

## Capabilities

### New Capabilities

无。本次变更不引入新功能，仅修复现有内建函数以符合规范。

### Modified Capabilities

- **`builtin-dispatch`** — selector/meta/list/math/modules 模块的现有分派逻辑需扩展以支持被修复的子函数行为
- **`css-selector`** — 选择器解析和格式化逻辑需修复以通过 selector 函数的 spec 验证

## Impact

| 范围 | 文件 | 说明 |
|------|------|------|
| 核心实现 | `src/eval/builtin/selector.rs` | selector 函数实现 |
| 核心实现 | `src/eval/builtin.rs` + `src/eval/meta_ops.rs` | meta 函数分派 |
| 核心实现 | `src/eval/builtin/list.rs` | list 函数边界 |
| 核心实现 | `src/eval/builtin/math.rs` + `math_css.rs` + `math_helpers.rs` + `math_trig.rs` | math 函数精度/边界 |
| 核心实现 | `src/css/selector*.rs` | 选择器 AST 操作（合并、解析、比较） |
| 测试 | `tests/compile_test.rs` | 可能需要新增编译测试覆盖修复案例 |
| 文档 | `openspec/specs/builtin-dispatch/spec.md` | delta spec 记录行为变更 |
