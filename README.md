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
- **Control Flow** — `@if` / `@else if` / `@else`, `@for ... through/to`, `@each ... in`
- **`@extend`** — Selector inheritance with placeholder selectors (`%placeholder`)
- **`@use`** — Module system with namespace resolution and virtual file system support
- **`@import`** — Legacy import support
- **Interpolation** — `#{...}` in selectors, values, and property names
- **`@error`, `@warn`, `@debug`** — Diagnostic at-rules
- **`@at-root`** — Root-level selector escaping

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

### With Virtual File System

For `@use "module"` resolution without filesystem access:

```rust
use sasspile::compile_with_files;
use std::collections::HashMap;

let mut vfs = HashMap::new();
vfs.insert("_colors".to_string(), "$brand: #ff6600;".to_string());

let scss = r#"
    @use "colors";
    .header { color: colors.$brand; }
"#;

let css = compile_with_files(scss, &vfs).unwrap();
```

### Output Style Control

```rust
use sasspile::{serialize_with_style, OutputStyle};

// Serialize with compressed output
// let css = serialize_with_style(&css_tree, OutputStyle::Compressed).unwrap();
```

## Testing

The test suite includes **265+ tests** covering:

- Lexer tokenization
- Parser grammar (expressions, at-rules, selectors)
- Evaluation pipeline (variables, mixins, functions, control flow)
- Built-in functions — math, color, string, list, map, meta, selector
- Selector parsing and `@extend` resolution
- sass-spec integration via HRX format
- Real-world project compilation: Bootstrap, Element Plus

```bash
# Run all tests
cargo test

# Run with trace output
RUST_LOG=trace cargo test

# Run specific module tests
cargo test --test builtins_color
```

## Project Statistics

| Metric | Value |
|--------|-------|
| Source lines | ~7,300 |
| Test lines | ~2,600 |
| Source files | 25 |
| Test files | 18 |
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
