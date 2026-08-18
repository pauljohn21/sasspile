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
>
> **CodeGraph-ready.** The project ships with a CodeGraph index for instant code
> navigation, caller analysis, and impact assessment via `codegraph` CLI.

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
- **`@use`** — Module system with namespace resolution, `as *` glob imports, `with ($var: value)` configuration injection, and `ModuleResolver` trait for decoupled file loading
- **`@forward`** — Basic forward rule support for module re-exports
- **`@import`** — Legacy import support
- **`@supports`** — Feature query support
- **`@at-root`** — Root-level selector escaping
- **Interpolation** — `#{...}` in selectors, values, and property names
- **`@error`, `@warn`, `@debug`** — Diagnostic at-rules

### Built-in Modules

| Module | Functions |
|--------|-----------|
| `sass:math` | `abs`, `ceil`, `floor`, `round`, `max`, `min`, `clamp`, `div`, `percentage`, `pow`, `sqrt`, `log`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `hypot`, `unit`, `unitless`, `comparable`, `random`, `$pi`, `$e` |
| `sass:color` | `rgb`, `rgba`, `hsl`, `hsla`, `red`, `green`, `blue`, `alpha`, `hue`, `saturation`, `lightness`, `lighten`, `darken`, `saturate`, `desaturate`, `adjust-hue`, `mix`, `invert`, `complement`, `grayscale`, `ie-hex-str`, `adjust`, `scale`, `change`, `channel`, `to-space`, `to-gamut`, `is-legacy`, `is-in-gamut` |
| `sass:string` | `quote`, `unquote`, `to-upper-case`, `to-lower-case`, `str-length`, `str-index`, `str-insert`, `str-slice`, `str-split`, `unique-id` |
| `sass:list` | `length`, `nth`, `set-nth`, `append`, `join`, `zip`, `index`, `separator`, `is-bracketed`, `slash` |
| `sass:map` | `get`, `set`, `merge`, `deep-merge`, `remove`, `deep-remove`, `keys`, `values`, `has-key` |
| `sass:meta` | `type-of`, `if`, `inspect`, `function-exists`, `mixin-exists`, `variable-exists`, `global-variable-exists`, `get-function`, `get-mixin`, `call`, `content-exists`, `feature-exists`, `keywords`, `module-functions`, `module-mixins`, `module-variables`, `load-css`, `accepts-content`, `apply` |
| `sass:selector` | `selector-append`, `selector-nest`, `selector-extend`, `selector-replace`, `selector-is-superselector` |

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

All pipeline stages are instrumented with `tracing` spans, bridged to
OpenTelemetry SDK for real OTel trace export:

| Span | Stage |
|------|-------|
| `compile_pipeline{stage="compile"}` | Top-level compilation span |
| `tokenize{stage="lexer"}` | Lexing phase |
| `parse{stage="parser"}` | Parsing phase |
| `evaluate{stage="eval"}` | Evaluation phase |
| `serialize{stage="serialize"}` | Serialization phase |
| `project_compile{stage="real_project"}` | Real-world project compilation (OTel tests) |
| `project_validate{stage="real_project"}` | CSS output validation (OTel tests) |

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

## Spec Conformance Infrastructure

sasspile includes a **compiler-independent** spec dataset and conformance
tooling, built on `tracing` + OpenTelemetry Metrics.

### Spec Dataset (`spec_dataset.json`)

A pure data file extracted from the official `sass-spec` HRX test suite.
**No dependency on any Sass compiler** — any Rust Sass implementation can use
it as a conformance reference.

- **20,504 test cases** across **17 domains**
- **16 MB** JSON, self-contained
- Each case includes: `id`, `domain`, `files` (input + output + error + multi-file),
  `entry`, `expected_output`, `expected_error`, `is_multi_file`

```bash
# Regenerate the dataset from sass-spec HRX files
rust-script scripts/gen_spec_dataset.rs --spec-root sass-spec/spec --output spec_dataset.json
```

### Conformance Checker (`scripts/spec_check.rs`)

Runs any compiler against the dataset. Uses `tracing` spans for evidence chains
and produces a JSON report with per-domain pass rates.

```bash
# Run sasspile against the full dataset
rust-script scripts/spec_check.rs \
  --dataset spec_dataset.json \
  --compiler ./target/release/sasspile \
  --label sasspile_full

# Run a single domain
rust-script scripts/spec_check.rs \
  --dataset spec_dataset.json \
  --compiler ./target/release/sasspile \
  --label sasspile_operators \
  --domain operators
```

Output:
```
=== Spec Check Report ===
Compiler: ./target/release/sasspile
Total: 20504 | Passed: 2346 | Failed: 18158
Pass rate: 0.1144

Per-domain:
  operators: 8/30 (0.267)
  variables: 11/14 (0.786)
  callable: 43/71 (0.606)
  ...
```

### Baseline Diff Tool (`scripts/spec_diff.rs`)

Compares two baseline/check JSONs to track progress across versions:

```bash
rust-script scripts/spec_diff.rs --old spec_check_old.json --new spec_check_sasspile_full.json
```

### Embedded OTel Metrics Test Framework

For in-process testing (no subprocess overhead), `tests/spec_otel_runner.rs`
provides `SpecOtelRunner` with:
- **Counter** `spec_tests_total` by domain + result (pass/fail/panic)
- **Histogram** `spec_test_duration_ms` by domain
- **ObservableGauge** `spec_pass_rate` by domain
- `catch_unwind` to survive compiler panics
- `tracing::error!` on failure (no mid-test panic)

```bash
# Run all 17 domains with OTel metrics + trace
cargo test --test spec_baseline -- --nocapture --ignored
```

## Testing

The test suite includes **383 tests** across **40 test files** covering:

- Lexer tokenization
- Parser grammar (expressions, at-rules, selectors)
- Evaluation pipeline (variables, mixins, functions, control flow)
- Built-in functions — math, color, string, list, map, meta, selector
- Selector parsing and `@extend` resolution
- sass-spec integration via HRX format (20,504 spec cases)
- Real-world project compilation: Bootstrap, Element Plus, Bulma, MDC-Web, Foundation (with OpenTelemetry tracing)

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
RUST_LOG=info cargo test --test bulma_otel -- --nocapture
RUST_LOG=info cargo test --test mdc_otel -- --nocapture
RUST_LOG=info cargo test --test foundation_otel -- --nocapture

# Run full sass-spec baseline (OTel metrics + trace, ~15s)
cargo test --test spec_baseline -- --nocapture --ignored
```

### CodeGraph Integration

The project uses [CodeGraph](https://github.com/nicholastimos/codegraph) for
code analysis and navigation. The index is stored in `.codegraph/codegraph.db`.

```bash
# Update the code graph index after code changes
codegraph sync

# View index statistics
codegraph status

# Find callers of a function
codegraph callers <function_name>

# Analyze impact of changing a symbol
codegraph impact <symbol_name>

# Explore a node's details
codegraph node <node_id>

# General exploration
codegraph explore <query>
```

Current index: synced via `codegraph sync`

## Project Statistics

| Metric | Value |
|--------|-------|
| Source lines | ~8,631 |
| Test lines | ~3,550 |
| Source files | 35 |
| Test files | 40 |
| Builtin functions | 147 registrations |
| Total tests | 383 |
| Spec cases | 20,504 |
| Spec pass rate | 11.44% |
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
- [Bulma](https://github.com/jgthms/bulma) — Used as a real-world integration test target
- [Material Web](https://github.com/material-components/material-components-web) — Used as a real-world integration test target
- [Foundation](https://github.com/foundation/foundation-sites) — Used as a real-world integration test target
