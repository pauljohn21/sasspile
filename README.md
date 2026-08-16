# sasspile

**Pure Rust + Tokio asynchronous SCSS compiler**, targeting compatibility with the Sass specification.

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](./LICENSE)
[![Rust Edition 2024](https://img.shields.io/badge/rust-2024-orange.svg)](https://blog.rust-lang.org/2024/02/19/Rust-2024.html)
[![Rust Toolchain 1.97](https://img.shields.io/badge/toolchain-1.97-blue.svg)](https://blog.rust-lang.org/2026/04/02/Rust-1.97.0.html)

## Overview

sasspile is a from-scratch SCSS compiler written in Rust, built around a 7-stage Tokio asynchronous pipeline. It aims for broad compatibility with the [Sass specification](https://github.com/sass/sass-spec) while leveraging Rust's type system and async runtime for safe, performant compilation.

**Status**: Core pipeline (Phase 1–11) complete — lexing, parsing, semantic analysis, expression evaluation, built-in modules (color/math/list/map/string/meta), CSS generation, CSS4 color spaces, incremental compilation, pipeline orchestration, and Sass spec test integration.

## Architecture

```
Source → Lex → Parse → Semantic → Transform → Evaluate → Codegen
  │         │       │         │          │          │         │
  └─────────┴───────┴─────────┴──────────┴──────────┴─────────┘
                    Tokio Tasks + mpsc Channels
```

Each stage is an independent Tokio task connected via bounded `mpsc` channels. Immutable `Value` types flow through the pipeline, with `watch` channels propagating variable changes for incremental recompilation.

### Module Structure

| Module | Purpose |
|--------|---------|
| `lexer` | Tokenization, interpolation parsing, indented syntax |
| `parser` | Recursive-descent parser, AST construction, error recovery |
| `semantic` | Symbol tables, dependency resolution, `@extend` validation |
| `eval` | Expression evaluation, operators, function dispatch |
| `builtin` | Built-in Sass modules: `sass:color`, `sass:math`, `sass:list`, `sass:map`, `sass:string`, `sass:meta` |
| `css` | CSS output generation, rule expansion, at-rule handling |
| `value` | Value system: numbers, colors, strings, maps, lists |
| `color` | CSS4 color spaces: Oklab, Oklch, HWB |
| `pipeline` | 7-stage Tokio orchestration with backpressure |
| `incremental` | Reactive environment, dependency graph, caching, propagation |
| `diagnostics` | Error reporting with source snippets |

## Installation

### From Source

```bash
git clone git@github.com:pauljohn21/sasspile-next.git
cd sasspile-next
cargo build --release -p sasspile
```

### Requirements

- Rust 1.97+ (edition 2024)
- Tokio (features: full)

## Usage

### CLI

```bash
# Compile a file
cargo run -p sasspile -- input.scss -o output.css

# Verbose mode with tracing
RUST_LOG=info cargo run -p sasspile -- input.scss -o output.css -vv

# Generate JSON trace logs for analysis
cargo run -p sasspile --example trace_parse -- input.scss output.json
```

### Library

```rust
use sasspile::Compiler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compiler = Compiler::new();
    let css = compiler.compile("$color: red; .foo { color: $color; }").await?;
    println!("{css}");
    Ok(())
}
```

### Pipeline (Advanced)

```rust
use sasspile::{Pipeline, PipelineInput};
use sasspile::css::OutputStyle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipeline = Pipeline::new();

    let input = PipelineInput {
        path: "input.scss".to_string(),
        source: ".foo { color: blue; }".to_string(),
    };

    let output = pipeline.compile_one(input).await?;
    println!("{output:?}");
    Ok(())
}
```

### Batch Compilation (Concurrent)

```rust
use sasspile::{Pipeline, PipelineInput};
use sasspile::css::OutputStyle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pipeline = Pipeline::new();

    let inputs = vec![
        PipelineInput { path: "a.scss".to_string(), source: ".a { color: red; }".to_string() },
        PipelineInput { path: "b.scss".to_string(), source: ".b { color: blue; }".to_string() },
    ];

    let results = pipeline.compile_batch(inputs, OutputStyle::Expanded).await;

    for result in results {
        match result {
            Ok(output) => println!("{}: {} bytes", output.path, output.css.len()),
            Err(e) => eprintln!("Error: {e}"),
        }
    }
    Ok(())
}
```

### Incremental Compilation

```rust
use sasspile::incremental::{ReactiveEnv, DependencyGraph, SpanCache, PropagateMsg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reactive environment: variable changes trigger downstream updates.
    let env = ReactiveEnv::new();
    env.set_var("theme", "dark");

    // Track dependencies between nodes.
    let graph = DependencyGraph::new();

    // Cache compiled spans.
    let cache = SpanCache::new();

    // Propagate changes through the pipeline.
    // (Use PropagateMsg::VarChanged, BatchChanged, or SourceEdited)
    Ok(())
}
```

## Features

### Implemented

- Lexing: identifiers, numbers, strings, operators, interpolation `#{}`
- Parsing: rules, declarations, at-rules, nesting, selectors
- Semantic analysis: symbol table, module dependencies, `@extend`
- Evaluation: arithmetic, comparisons, string ops, list/map access
- Built-in modules: `sass:color`, `sass:math`, `sass:list`, `sass:map`, `sass:string`, `sass:meta`
- CSS generation: expanded/compressed output, nested rules, at-rules
- CSS4 colors: Oklab, Oklch, HWB, `color-mix()`, relative color syntax
- Pipeline: 7-stage Tokio async pipeline with backpressure
- Incremental: reactive env, dep graph, span cache, change propagation
- Sass spec: HRX loader, spec runner, CSS4 color skip list

### CSS4 Color Spaces

| Space | Status |
|-------|--------|
| sRGB (hex, rgb(), hsl()) | ✅ |
| Oklab / Oklch | ✅ |
| HWB | ✅ |
| `color-mix()` | ✅ |
| Relative color syntax | ✅ |
| `light-dark()` | ✅ via `color-mix()` |

## Testing

```bash
# Run all unit tests
cargo test -p sasspile --lib

# Run integration tests (spec runner)
cargo test -p sasspile --test spec_runner

# Run all tests
cargo test -p sasspile --lib --tests
```

## Benchmarks

The project tracks compatibility with:

- **sass-spec**: Sass specification test suite (HRX archives)
- **Bootstrap SCSS**: Full Bootstrap 5 SCSS compilation baseline (99/99 files passing)

## Design Principles

1. **Zero `println!`** — All logging via `tracing` macros
2. **Immutable data** — Values use `Arc` for cheap sharing across tasks
3. **Modular files** — Single file ≤ 400 lines
4. **Async-first** — Tokio tasks throughout the pipeline
5. **Zero unsafe** — Pure safe Rust

## Contributing

Contributions welcome! Please ensure:

- `cargo build -p sasspile` passes
- `cargo test -p sasspile` passes
- `cargo clippy -p sasspile -- -D warnings` is clean
- All logging uses `tracing` macros (no `println!`/`eprintln!`)
- New tests go in `tests/` directory (not inline)

## License

Dual-licensed under [MIT](./LICENSE-MIT) or [Apache-2.0](./LICENSE-APACHE).

## Acknowledgments

- [Sass specification](https://sass-lang.com/documentation) — the definitive reference
- [sass-spec](https://github.com/sass/sass-spec) — official test suite
- [Bootstrap](https://github.com/twbs/bootstrap) — real-world SCSS baseline
