## Why

当前 sasspile 项目（src/ + tests/ ~60 个模块文件）在代码风格上存在 6 个维度的不一致：module header 详细程度参差、section 分隔符字符/长度混乱、测试函数声明紧凑/展开混用、错误消息语言不统一、命名前缀多样、行内注释缺失。这导致新贡献者缺乏统一参照、AI 生成代码风格飘忽、代码 review 成本增加。本次变更旨在建立可执行的单一风格规范（STYLE_GUIDE.md），批量统一全部文件，消除未来漂移。

## What Changes

- **新增** 独立文档 `STYLE_GUIDE.md`：覆盖 module header 模板、section 分隔符、函数声明、错误消息、命名规范 5 个维度
- **修改** ~40 个测试文件：补全/重写 module header、统一 section 分隔符、展开紧凑 `#[test] fn`
- **修改** ~20 个 src 模块：补全一句话以上 module header（当前部分文件仅一句或完全无 header）
- **统一** `diagnostic_runner.rs` 中 20+ 个紧凑 `#[test] fn` 为展开风格
- **移除** `// ═══` 双线分隔符（reactor_test.rs）、`// ——` em dash 分隔符（interp_test.rs），统一为 `// ───`
- **不改变** 任何逻辑/测试断言/编译行为

## Capabilities

### New Capabilities

- `style-guide`: 产出独立风格指南文档，定义 sasspile 全项目统一代码风格规范
- `module-header-standardization`: 所有文件（src + tests）统一使用分层 module header 模板
- `section-divider-unification`: 统一 section 分隔符字符、长度、对齐方式
- `test-declaration-format`: 测试函数声明统一展开风格，消除紧凑同行格式
- `naming-conventions`: 统一测试命名前缀和错误消息语言

### Modified Capabilities

无。本次为纯风格统一，不改变任何功能需求或 spec。

## Impact

| 类别 | 数量 | 说明 |
|------|------|------|
| 修改文件 | ~60 | src/ ~20 模块 + tests/ ~40 文件 |
| 新增文件 | 1 | `STYLE_GUIDE.md` |
| 破坏性变更 | 无 | 纯风格修改，不影响 public API 或编译结果 |
| 测试回归风险 | 极低 | 仅注释/格式修改，不触及逻辑 |
| 受影响管线 | 无 | 不改变 Lexer/Parser/Evaluator/Serializer 行为 |

**执行约束**：
- 中文注释保留（module header 和测试消息维持中文）
- 保持 `unexpected failure in test` 作为默认 expect 消息
- 单文件 ≤ 500 行规则继续适用（本次不触发超限）
- 按维度分批修改，每批跑 `cargo test` 确认零回归
