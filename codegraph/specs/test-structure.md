# Sass-Spec 测试结构

## 概述

sass-spec 是 Sass 官方测试套件，包含 1306+ 个测试用例，验证编译器的 CSS/Sass 语义符合规范。

## 目录结构

```
sass-spec-main/
├── spec/
│   ├── basic/           # 基础语法
│   ├── scss/            # SCSS 扩展
│   ├── css/             # CSS 兼容
│   ├── colors/          # CSS 颜色模块（462 文件，跳过）
│   ├── media/           # @media 查询
│   ├── nesting/         # 嵌套行为
│   ├── directives/      # @规则
│   ├── values/          # 值类型
│   ├── functions/       # 内置函数
│   ├── operators/       # 运算符
│   ├── interpolation/   # #{} 插值
│   ...
└── .gitignore
```

## 测试格式

每个测试用例是一个 `.hrx` 文件（HRX 归档格式），包含：
- `input.scss` — 输入 Sass/SCSS
- `output.css` — 期望输出
- `options.yaml` — 编译选项

## 运行方式

```bash
# 运行 sass-spec 测试
cargo test -p sasspile --test sass_spec_parse

# 查看所有测试统计
cargo test -p sasspile -- --list
```

## 当前统计（2026-08-15）

| 指标 | 数值 |
|------|------|
| 总用例 | 1306 |
| 解析通过 | 475 |
| 通过率 | 36.4% |
| CSS4 颜色跳过 | 462 |
| 有效非颜色用例 | ~844 |

## 已知失败模式

| 优先级 | 问题 | 影响范围 |
|--------|------|----------|
| P0 | `and`/`or` 逻辑运算符缺失 | ~15% 用例 |
| P1 | `@else`/`@else if` 解析 | ~8% 用例 |
| P2 | `@if` 条件含 `and` | ~5% 用例 |
| P3 | 大括号追踪 | ~3% 用例 |
| P4 | `@extend` 多行 | ~2% 用例 |

## 测试文件

- `sasspile/tests/sass_spec_parse.rs` — 解析层集成测试
- `sasspile/tests/builtin_spec.rs` — 内置模块测试
- `sasspile/tests/eval_spec.rs` — 求值器测试
