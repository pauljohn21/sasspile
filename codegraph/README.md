# CodeGraph — 项目知识图谱

CodeGraph 是本项目的"外部大脑"，由两部分组成：

1. **SQLite 图谱数据库**（`.codegraph/codegraph.db`）：代码结构索引，追踪函数、类、模块、调用关系
2. **Markdown 文档**（本目录）：架构设计、模块知识、决策记录、开发进度

两者结合，用于在长对话、多轮开发中保持对项目架构、决策、模块关系的持续认知。

---

## 一、SQLite 图谱数据库

### 概述

`.codegraph/codegraph.db` 是一个代码知识图谱，使用 SQLite 存储代码节点（函数、类、模块、变量等）和它们之间的关系（调用、依赖、继承等）。

### 当前统计

| 指标 | 数值 |
|------|------|
| 索引文件 | 2,500 |
| 节点 | 20,068 |
| 边（关系） | 52,230 |
| 数据库大小 | ~71 MB |

### 数据库 Schema

| 表 | 用途 |
|----|------|
| `nodes` | 代码节点（函数、类、变量、文件等） |
| `edges` | 节点间关系（调用、导入、继承等） |
| `files` | 文件索引信息 |
| `unresolved_refs` | 未解析的引用 |
| `nodes_fts` | 全文搜索索引（FTS5） |
| `name_segment_vocab` | 名称分词词汇表 |
| `project_metadata` | 项目元数据 |

### Node 类型（kind）

| 类型 | 数量 | 说明 |
|------|------|------|
| import | 7,586 | 导入/use 语句 |
| constant | 4,178 | 常量定义 |
| function | 2,268 | 函数定义 |
| file | 2,192 | 文件节点 |
| component | 1,109 | 组件（Vue/React） |
| type_alias | 958 | 类型别名 |
| interface | 390 | 接口定义 |
| property | 659 | 属性/字段 |
| method | 502 | 方法 |
| variable | 126 | 变量 |

### CLI 命令

```bash
# 初始化/重新索引
codegraph init [path]

# 同步增量变更
codegraph sync [path]

# 查看状态
codegraph status [path]

# 搜索符号
codegraph query <search> [--limit N]

# 查看单个符号详情（源码 + 调用链）
codegraph node <name>

# 探索代码区域（相关符号 + 调用路径）
codegraph explore <query...>

# 查看文件结构
codegraph files
```

### 使用示例

```bash
# 搜索 parse 函数
codegraph query "parse" --limit 10

# 查看 hrx::parse 的源码和调用链
codegraph node parse

# 探索 hrx 模块
codegraph explore hrx module
```

---

## 二、Markdown 文档

### 目录结构

```
codegraph/
├── README.md          # 本文件：使用说明
├── overview.md        # 项目概览：目标、范围、当前状态
├── architecture.md    # 架构设计：管道、模块依赖、数据流
├── modules/           # 各模块详细知识
│   ├── hrx.md         # HRX 解析器（已完成）
│   ├── value-system.md
│   ├── lexer.md
│   ├── parser.md
│   ├── semantic.md
│   ├── eval.md
│   ├── builtin-modules.md
│   ├── css-gen.md
│   ├── pipeline.md
│   └── incremental.md
├── decisions/         # 关键设计决策记录
│   ├── 001-pipeline-tokio-mpsc.md
│   ├── 002-immutable-value-arc.md
│   ├── 003-module-dependency-graph.md
│   ├── 004-async-builtin-function.md
│   ├── 005-error-accumulation.md
│   ├── 006-debounce-strategy.md
│   └── 007-css-ast-formatter.md
├── specs/             # sass-spec 测试相关知识
│   ├── test-structure.md
│   ├── css4-colors-skip.md
│   └── compat-notes.md
└── progress.md        # 开发进度跟踪
```

### 使用指南

1. **每次会话开始时**：读取 `overview.md` 和 `progress.md` 恢复上下文
2. **涉及模块开发**：读取对应 `modules/` 下的知识文件
3. **遇到设计问题**：查看 `decisions/` 中的相关决策
4. **完成任务后**：更新对应模块知识和 `progress.md`
5. **新决策产生**：在 `decisions/` 中新增条目

### 维护原则

- **原子性**：每个文件聚焦单一主题，≤ 300 行
- **索引性**：开头提供快速导航和关键概念列表
- **溯源性**：关键结论标注来源（设计文档/代码位置）
- **时效性**：更新时保留历史决策，标注变更原因

---

## 三、两者配合使用

| 场景 | 工具 | 操作 |
|------|------|------|
| 理解代码结构 | SQLite | `codegraph query/explore/node` |
| 查找函数调用链 | SQLite | `codegraph node <name>` |
| 回顾项目目标 | Markdown | 读 `overview.md` |
| 理解架构决策 | Markdown | 读 `architecture.md` + `decisions/` |
| 了解模块详情 | Markdown | 读 `modules/<name>.md` |
| 查看开发进度 | Markdown | 读 `progress.md` |
| 测试规范知识 | Markdown | 读 `specs/` |
