---
name: tracing-ai
description: "AI-native tracing diagnostics MCP tool. Use when debugging cargo test failures, analyzing performance bottlenecks, or detecting regressions via tracing logs. Triggers: test failure root cause analysis, span chain inspection, slow span detection, trace comparison."
---

# tracing-ai — AI-native Tracing Diagnostics

MCP Server 工具，用于分析 Rust 项目的 tracing 日志。帮助 AI Agent 自主诊断测试失败、定位性能瓶颈、检测退化。

## 工具概览

| 工具 | 用途 | 输入 |
|------|------|------|
| `diagnose_test_failure` | 诊断测试失败根因 | test_output, trace_log |
| `analyze_traces` | 分析 trace 异常（慢 span、错误） | log_file, slow_threshold_ms |
| `compare_runs` | 对比两次运行差异 | baseline_log, current_log |

## 使用前提

1. 安装 tracing-ai CLI：
   ```bash
   cargo install tracing-ai
   ```

2. 在 tracing-ai MCP 配置中注册（项目 .claude/ 或全局配置）

## 工作流程

### 测试失败诊断

```bash
# 收集 trace（项目需启用 tracing-subscriber json feature）
RUST_LOG=debug cargo test 2>trace.log | tee test_output.log

# 调用 MCP 工具 diagnose_test_failure
# 输入: test_output, trace_log
# 输出: failure_point, decision_path, anomalies, top_errors
```

### 性能分析

```bash
# 调用 analyze_traces
# 输入: log_file, slow_threshold_ms (默认 100)
# 输出: 慢 span 列表、错误聚合、异常标记
```

### 回归检测

```bash
# 1. 生成基准 trace
git checkout main
RUST_LOG=debug cargo test 2>baseline.log

# 2. 切换分支并生成当前 trace
git checkout feature-branch
RUST_LOG=debug cargo test 2>current.log

# 3. 调用 compare_runs 对比
```

## 输出格式

- **JSON**（默认）：MCP Agent 消费的结构化数据
- **Pretty**（`--format pretty`）：带 ANSI 颜色的可读格式

设置 `NO_COLOR=1` 禁用颜色。

## 日志格式兼容

tracing-ai 期望 tracing JSON 格式（每行一个 JSON 对象）。

如果日志是纯文本，tracing-ai 会尝试跳过非 JSON 行。

## 使用技巧

- 用 `RUST_LOG=info` 减少噪音，`debug` 获取详细 span
- 保留 trace 日志做对比：`cp trace.log baseline.log`
- 诊断时重点关注 `failure_point` 和 `decision_path`

## 注意事项

- tracing-ai **不分析编译错误**，只分析运行时 trace
- 慢 span 阈值不建议 > 1000ms，否则会漏掉有意义的慢调用
- JSON 输出用 JSON 解析器，不要做字符串匹配
