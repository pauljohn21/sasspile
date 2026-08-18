<div align="center">

# sasspile

A Rust-native Sass/SCSS compiler built from the official Sass specification

[![Crates.io](https://img.shields.io/crates/v/sasspile.svg)](https://crates.io/crates/sasspile)
[![Documentation](https://docs.rs/sasspile/badge.svg)](https://docs.rs/sasspile)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.97+](https://img.shields.io/badge/rust-1.97%2B-orange.svg)](https://www.rust-lang.org)

</div>

---

> **Built from spec, not from dart-sass.** Every feature is implemented by reading the
> official Sass specification and sass-spec test suite — not by translating another
> implementation. sasspile uses Rust's ownership model, leverages `tracing` for
> diagnostics, and follows Rust idioms throughout.

## Overview

sasspile is a from-scratch Sass/SCSS compiler written in pure Rust, designed for
integration into Rust toolchains where a native SCSS-to-CSS compilation step is needed.
It implements the full compilation pipeline — lexer, parser, evaluator, and serializer —
and supports the majority of Sass language features.

## Features

### Language Features

- **Variables** — `$var: value;` with `!default` and `!global` flags
- **Nesting** — Full selector nesting with `&` parent selector
- **Mixins** — `@mixin` / `@include` with parameters and `@content`
- **Functions** — `@function` / `@return` with user-defined functions
- **Control Flow** — `@if` / `@else if` / `@else`, `@for ... through/to`, `@each ... in`, `@while`
- **`@extend`** — Selector inheritance with placeholder selectors (`%placeholder`)
- **`@use`** — Module system with namespace resolution and filesystem-based `@use`/`@import` via `compile_file`
- **`@import`** — Legacy import support
- **`@supports`** — Feature query support
- **`@at-root`** — Root-level selector escaping
- **Interpolation** — `#{...}` in selectors, values, and property names
- **`@error`, `@warn`, `@debug`** — Diagnostic at-rules

### Built-in Modules

| Module | Functions |
|--------|-----------|
| `sass:math` | `abs`, `ceil`, `floor`, `round`, `max`, `min`, `percentage`, `pow`, `sqrt`, `unit`, `unitless`, `$pi` |
| `sass:color` | `rgb`, `rgba`, `hsl`, `hsla`, `red`, `green`, `blue`, `alpha`, `lighten`, `darken`, `mix`, `invert`, `complement` |
| `sass:string` | `quote`, `unquote`, `to-upper-case`, `to-lower-case`, `str-length`, `str-slice`, `str-index` |
| `sass:list` | `length`, `nth`, `append`, `join`, `index`, `separator`, `is-bracketed` |
| `sass:map` | `get`, `keys`, `values`, `has-key`, `merge` |
| `sass:meta` | `type-of`, `inspect`, `feature-exists` |
| `sass:selector` | Selector inspection and manipulation |

### Output Styles

- **Expanded** — Multi-line, 2-space indent (default)
- **Compressed** — Single line, minimal whitespace

## Architecture

```
                         ┌──────────────────────────────────────────────┐
                         │              sasspile pipeline              │
                         └──────────────────────────────────────────────┘

  Source SCSS ──►  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐  ──►  CSS Output
                  │  Lexer  │─►│  Parser  │─►│ Evaluator │─►│ Serializer  │
                  └─────────┘  └──────────┘  └──────────┘  └────────────┘
                                    │               │               │
                                    ▼               ▼               ▼
                                  AST           CSS Tree         CSS String
                                              (Env, Builtins)
```

| Module | Responsibility |
|--------|---------------|
| `lexer` | Tokenizes SCSS source into `Token` stream with source positions |
| `parser` | Builds `AST` (statements + expressions) from tokens |
| `env` | Variable, mixin, function environment with scoping |
| `eval` | Evaluates AST → `CssTree`, resolves variables, expands mixins, applies `@extend` |
| `builtins` | Registers all `sass:*` built-in functions |
| `selector` | Selector parsing and `@extend` resolution |
| `serialize` | Converts `CssTree` → CSS string (expanded or compressed) |
| `error` | Typed errors with source position tracking |
| `value` | Sass value types: numbers, colors, strings, lists, maps, booleans, null |

### Observability

All pipeline stages are instrumented with `tracing` spans:

| Span | Stage |
|------|-------|
| `compile_pipeline{stage="compile"}` | Top-level compilation span |
| `tokenize{stage="lexer"}` | Lexing phase |
| `parse{stage="parser"}` | Parsing phase |
| `evaluate{stage="eval"}` | Evaluation phase |
| `serialize{stage="serialize"}` | Serialization phase |

Enable traces with `RUST_LOG`:

```bash
RUST_LOG=sasspile=trace cargo run
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sasspile = "0.9"
```

## Usage

### Basic Compilation

```rust
use sasspile::compile;

fn main() {
    let scss = r#"
        $primary: #336699;

        .button {
            color: $primary;
            &:hover {
                color: lighten($primary, 10%);
            }
        }
    "#;

    let css = compile(scss).unwrap();
    // println!("{}", css);
}
```

Output:

```css
.button {
  color: #336699;
}
.button:hover {
  color: #6699cc;
}
```

### File-based Compilation

For `@use "module"` / `@import "file"` resolution from the filesystem:

```rust
use sasspile::compile_file;

fn main() {
    let css = compile_file("src/main.scss").unwrap();
    // println!("{}", css);
}
```

`compile_file` sets `base_dir` to the parent directory of the input file, enabling
`@use` and `@import` directives to resolve relative paths on the filesystem.

### Output Style Control

```rust
use sasspile::{tokenize, parse, evaluate, serialize_with_style, OutputStyle};

let scss = "a { color: red; }";
let tokens = tokenize(scss, "<string>").unwrap();
let ast = parse(tokens).unwrap();
let css_tree = evaluate(ast).unwrap();
let compressed = serialize_with_style(&css_tree, OutputStyle::Compressed).unwrap();
// compressed == "a{color:red}"
```

## Testing

The test suite includes **224 tests** across **35 test files** covering:

- Lexer tokenization
- Parser grammar (expressions, at-rules, selectors)
- Evaluation pipeline (variables, mixins, functions, control flow)
- Built-in functions — math, color, string, list, map, meta, selector
- Selector parsing and `@extend` resolution
- sass-spec integration via HRX format
- Real-world project compilation: Bootstrap, Element Plus (with OpenTelemetry tracing)

```bash
# Run all tests
cargo test

# Run with trace output
RUST_LOG=trace cargo test

# Run specific module tests
cargo test --test builtins_color

# Run real-world project compilation tests (with OTel tracing)
RUST_LOG=info cargo test --test bootstrap_otel -- --nocapture
RUST_LOG=info cargo test --test element_plus_otel -- --nocapture
```

## Project Statistics

| Metric | Value |
|--------|-------|
| Source lines | ~8,500 |
| Test lines | ~3,500 |
| Source files | 35 |
| Test files | 35 |
| Total tests | 224 |
| Rust edition | 2024 |
| MSRV | 1.97 |
| License | MIT |

## License

MIT — See [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Sass specification](https://sass-lang.com/documentation/) — The language specification this compiler follows
- [sass-spec](https://github.com/sass/sass-spec) — The official test suite, used for conformance testing
- [Bootstrap](https://github.com/twbs/bootstrap) — Used as a real-world integration test target
- [Element Plus](https://github.com/element-plus/element-plus) — Used as a real-world integration test target
