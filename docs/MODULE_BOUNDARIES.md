# 模块边界与依赖方向

## 规则：依赖只能向下流动

```
lib.rs  ────────────────────────────────────────  (公开 API)
  │
  ├── source.rs ──── error.rs
  │
  ├── lex/ ──────── error.rs
  │
  ├── parse/ ────── lex/ , error.rs
  │
  ├── eval/ ────── parse/ , css/ , error.rs
  │
  └── css/ ──────── error.rs
```

## 禁止的依赖方向

| 禁止 | 原因 |
|------|------|
| `css/` → `eval/` | 序列化器不应依赖求值器 |
| `parse/` → `eval/` | 解析器不应依赖求值器 |
| `lex/` → `parse/` | 词法分析器不应依赖解析器 |
| `builtin/` → `eval/mod.rs` | 内建函数不应反向依赖求值器 |

## 模块职责矩阵

| 模块 | 输入 | 输出 | 职责 |
|------|------|------|------|
| `source.rs` | `&str` / `&Path` | `Source` | 封装源码文本 + 文件路径 + load_paths |
| `lex/` | `Source` | `Vec<Token>` | 词法分析，产出 token 流 |
| `parse/` | `Vec<Token>` | `Ast` (Vec\<Node\>) | 语法分析，构建 AST |
| `eval/` | `Ast` + `Env` | `Vec<CssNode>` | 求值，变量绑定，mixin 展开 |
| `css/` | `Vec<CssNode>` | `String` | 后处理（flatten/hoist/extend）+ 序列化 |

## 文件大小限制

| 类型 | 上限 | 超出处理 |
|------|------|---------|
| 类型定义（ast.rs, node.rs） | 500 行 | 按变体拆分 |
| 业务逻辑 | 300 行 | 按功能拆分 |
| 测试 | 500 行 | 按 spec 子目录拆分 |
| 数据表（const 表） | 不限 | 但必须纯数据 |

## 公开 API

```rust
// lib.rs —— 仅这 4 个函数是 pub
pub fn compile(input: &str, style: OutputStyle) -> Result<String>;
pub fn compile_expanded(input: &str) -> Result<String>;
pub fn compile_file(path: &Path, style: OutputStyle) -> Result<String>;
pub fn compile_file_with_paths(path: &Path, load_paths: &[PathBuf], style: OutputStyle) -> Result<String>;

// OutputStyle
pub enum OutputStyle { Expanded, Compressed }
```

所有内部类型（`Source`/`Lexed`/`Parsed`/`Evaluated`/`Serialized`）都是 `pub(crate)`。
