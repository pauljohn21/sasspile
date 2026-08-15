# HRX 模块（已完成）

## 概述

HRX (Human Readable Archive) 是一种纯文本归档格式，用于表示虚拟文件系统。本项目使用它来读取 sass-spec 测试用例。

## 文件结构

```
hrx/src/
├── lib.rs        # 公共 API 导出
├── main.rs       # CLI 入口
├── models.rs     # Archive, Entry, FileEntry, DirEntry
├── parser.rs     # HRX 解析器
├── writer.rs     # HRX 写入器（如需创建测试用例）
└── error.rs      # 错误类型定义
```

## 核心 API

```rust
// 解析 HRX 文本
pub fn parse(input: &str) -> Result<Archive>;

// 解析字节输入
pub fn parse_bytes(input: &[u8]) -> Result<Archive>;

// 写入 HRX
pub fn write(archive: &Archive) -> String;
```

## 数据模型

```rust
pub struct Archive {
    entries: Vec<Entry>,
}

pub enum Entry {
    File(FileEntry),
    Dir(DirEntry),
}

pub struct FileEntry {
    path: String,
    contents: String,
}

pub struct DirEntry {
    path: String,
    children: Vec<Entry>,
}
```

## HRX 格式规则

1. 每个条目以边界线开始：`<===> <path>`
2. 空 `<===>` + `====...` 表示目录边界
3. `#` 开头的行是注释
4. 文件内容持续到下一个边界或 EOF
5. 最大行长 1MB

## 边界标记常量

```rust
pub const BOUNDARY_MARKER: &str = "<===>";
```

## 测试覆盖

- `parser_test.rs`：单元测试
- `writer_test.rs`：写入器测试
- `integration_test.rs`：集成测试

## 依赖

- `tracing`：日志
- `thiserror`：错误定义
- `tokio`：异步运行时（CLI 二进制）
- `clap`：命令行解析
