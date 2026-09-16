# sasspile-rx — rxrust-first SCSS Compiler

> **rxrust 1.0.0-rc.5 · Rust 1.97 · 企业基线 100%**

[![bootstrap](https://img.shields.io/badge/Bootstrap_5-100%25_(TARGET)-red)](#测试与验收)
[![element-plus](https://img.shields.io/badge/Element_Plus-100%25_(TARGET)-red)](#测试与验收)
[![rxrust](https://img.shields.io/badge/rxrust-1.0.0_RC5-blue)](https://github.com/rxRust/rxRust)

`sasspile-rx` 是一个 **rxrust 四原语** 驱动的 SCSS 编译器——Observable / Observer / Operator / Subscription。每一个编译阶段(tokenize → parse → evaluate → serialize)都是算子的纯组合,没有命令式循环、没有共享状态、没有 dart-sard 参考。

**首要任务:Prioritize Bootstrap + Element Plus 全量编译通过 (企业基线 100%)**。sass-spec 作为语法契约参考,在企业目标达成后锦上添花。

---

## 快速开始

```rust
use sasspile_rx::compile;

fn main() -> Result<(), sasspile_rx::CompileError> {
    let css = compile("a { color: red; }")?;
    println!("{css}");
    Ok(())
}
```

```bash
# 编译并运行
cargo run --bin sasspile_tracker -- snapshot   # 拍基线
cargo build --release
```

---

## rxrust 哲学

```
Source (chars)
   │
   ▼  scan_map(ScannerState)                   ← Transformation
Token stream
   │
   ▼  filter(¬Whitespace)                      ← Filtering
   ▶ tap(tracing)                             ← Utility
   │
   ▼  scan_map(AstBuilder)                     ← Transformation
Node stream
   │
   ▼  flat_map(directive_expand)               ← Transformation + Combination
   ▶ publish(module_cache).ref_count()         ← Connectable (多订阅者共享)
   │
   ▼  map(CssNode::render)                     ← Transformation
char stream
   │
   ▼  last()                                   ← Aggregation
Result<String, CompileError>                   ← error-as-value (Infallible 管道)
```

**核心约定:**
- **Infallible 管道 + Result<T,CompileError> 作为 Item** — 错误是数据,不终止流
- **tap = 唯一副作用窗口** — 所有 tracing、缓存写入、metric 递增
- **scan = 状态累积** — tokenizer/parse 的内部状态
- **flat_map = 嵌套展开** — @extend/@mixin/@import/@use 的递归
- **publish/ref_count = 模块共享** — 同一模块多订阅者共享一次 parse
- **box_it = 类型擦除** — 每个 stage 之后擦除为 `LocalBoxedObservable<T, Infallible>`

---

## 项目结构

```
sasspile-rx/
├── src/
│   ├── lib.rs               ← 唯一公开 API: compile()
│   ├── pipeline.rs          ← 4-stage rxrust 编排
│   ├── tokenize_dst.rs      ← Stage 1: char → Token
│   ├── parse_dst.rs         ← Stage 2: Token → Node
│   ├── evaluate_dst.rs      ← Stage 3: Node → Result<CssNode, _> (待建)
│   ├── serialize_dst.rs     ← Stage 4: CssNode → char (待建)
│   ├── error.rs             ← CompileError 枚举
│   ├── shared/
│   │   ├── context.rs       ← CompilerContext (scan 传播)
│   │   └── module_cache.rs  ← SharedModule (publish/ref_count)
│   └── bin/
│       └── sasspile_tracker.rs  ← 外部对照库: snapshot/history/diff/trend
├── tests/
│   ├── integration.rs       ← 端到端编译测试
│   ├── sass_spec.rs         ← sass-spec HRX 基线
│   ├── sass_spec_detail.rs  ← per-directory 通过率分析
│   └── operators.rs         ← rxrust 算子覆盖验证
├── sass-spec/               ← git submodule (--depth 1)
├── bootstrap/               ← git submodule (--depth 1)
├── element-plus/            ← git submodule (--depth 1)
├── .codegraph/              ← CodeGraph DB + tracker snapshots (git-ignored)
│   ├── codegraph.db         ← 代码图数据库
│   └── snapshots/           ← 通过率时间序列
└── openspec/                ← 规范驱动工作流
    ├── config.yaml          ← AI 上下文 + 规则
    └── changes/rxrust-scss-compiler/
        ├── README.md
        ├── proposal.md
        ├── design.md
        ├── specs/            ← 6 spec 模块
        └── tasks.md         ← 8 个 task group (按 rxrust 算子分类)
```

---

## 测试与验收

### 优先级 P0 — 企业基线 (BLOCKING)

| 项目 | 入口文件 | 范式 | 验收命令 | 目标 |
|------|---------|------|---------|------|
| **Bootstrap 5** | `bootstrap/scss/bootstrap.scss` | `@import` 全局共享 | `cargo run --bin sasspile_tracker -- enterprise` | **100% 编译无错误** |
| **Element Plus** | `element-plus/packages/theme-chalk/src/index.scss` | `@use as *` + `!global` | `cargo run --bin sasspile_tracker -- enterprise` | **100% 编译无错误** |

### 优先级 P1 — 内核健康 (必须持续通过)

| 维度 | 命令 | 状态 |
|------|------|------|
| 单元测试 | `cargo test --lib` | ✅ |
| 端到端 Rx 管道 | `cargo test --test integration` | ✅ |
| rxrust 算子覆盖 | `cargo test --test operators` | ✅ |

### 优先级 P2 — sass-spec (锦上添花)

| 维度 | 命令 | 状态 |
|------|------|------|
| sass-spec 基线 | `cargo run --bin sasspec_tracker -- snapshot` | � 28/10274 (0.27%) |
| per-dir 分析 | `cargo test --test sass_spec_detail` | ✅ 目录级报告 |

sass-spec 仅在 P0 + P1 达成后扩展,永远不得阻塞企业目标。

---

## 外部对照库 (sasspile_tracker)

每次拍版本前先 `snapshot`,实现后再 `snapshot` + `diff`:

```bash
cargo run --bin sasspile_tracker -- snapshot
# ... 实现某 feature ...
cargo run --bin sasspile_tracker -- snapshot
cargo run --bin sasspile_tracker -- diff snap_a.json snap_b.json
cargo run --bin sasspile_tracker -- trend   # ASCII 折线图
```

snapshot 存于 `.codegraph/snapshots/snap_<ms>.json`,含 per-dir 通过率。

---

## 禁止项（违反 = 任务失败）

1. ❌ **Python** — 不用 python3/pip;测试一律 #[test];脚本用 rust-script
2. ❌ **println!/eprintln!** — 所有代码（含 src/、tests/、bins/）一律用 tracing 宏
3. ❌ **无 span 的跨阶段处理** — pipeline 各入口/出口 MUST 有 tracing::span!
4. ❌ **内联测试** — src/ 保持纯生产代码;测试放 tests/
5. ❌ **单文件 > 500 行** — 源码和测试分别计算
6. ❌ **非 SSH 推送** — git push 走 SSH 方式

---

## License

MIT © 2026 sasspile-rx Contributors
