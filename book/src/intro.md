# 简介

## 什么是 sasspile？

sasspile 是一个纯 Rust 实现的 SCSS 编译器，使用函数式编程范式构建。项目的设计初衷是学习和探索 Rust 语言的特性，同时提供一个可用的 SCSS 编译工具。

## 核心特性

### 类型状态机管线

sasspile 使用类型状态机模式（Type-State Pattern）构建编译管线，确保每个阶段的正确转换：

```
Source → Lexed → Parsed → Evaluated → Serialized
```

每个阶段都是一个独立的类型，通过方法调用实现状态转换。这种设计在类型层面保证了编译流程的正确性。

### 纯函数式风格

- 使用 Iterator 实现惰性求值
- 使用 `fold` 和 `try_fold` 替代可变状态
- 不可变数据结构（`im::HashMap`）

### 零依赖

纯 Rust 实现，无需外部 C 库或依赖，易于集成和分发。

## 设计理念

### 1. 类型安全优先

通过 Rust 的类型系统，在编译期捕获错误，减少运行时问题。

### 2. 性能与可读性平衡

使用函数式风格保证代码清晰，同时通过 Iterator 实现高效处理。

### 3. 完整的测试覆盖

- 75+ 单元测试
- 21+ sass-spec 合规测试
- 13+ Bootstrap 5.3.8 验证测试

## 项目状态

当前版本：0.2

sasspile 已通过以下验证：

- **sass-spec**: 核心功能测试套件
- **Bootstrap 5.3.8**: 实际项目编译测试

## 下一步

阅读 [快速开始](quickstart.md) 了解如何使用 sasspile。