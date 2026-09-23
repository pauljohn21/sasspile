## Capability: function-eval

### ADDED Requirements

#### Requirement: Builtin function dispatch
The compiler SHALL dispatch builtin color/math/list/string functions to pure functions.

#### Scenario: lighten function
- **WHEN** "color: lighten(#000, 50%);"
- **THEN** output "color: #808080;" (HSL lightened)

#### Scenario: rgba alpha adjustment
- **WHEN** "color: rgba(#ff0000, 0.5);"
- **THEN** output "color: rgba(255, 0, 0, 0.5);"

#### Scenario: math clamp
- **WHEN** "width: clamp(100px, 50vw, 500px);"
- **THEN** output "width: clamp(100px, 50vw, 500px);" (passthrough when no eval possible)

#### Scenario: Variable fallback
- **WHEN** "color: var(--bg, blue);"
- **THEN** "--bg" is looked up in CompileState; if absent, uses "blue"

---

### Requirement: rxrust pure function model
Builtin functions SHALL be implemented as `fn(&[String]) -> Option<String>` pure functions, no CompileState access.

#### Scenario: Stateless function
```
fn lighten(args: &[String]) -> Option<String> {
    let color = Color::from_str(&args[0])?;
    let amount: f64 = args[1].trim_end_matches('%').parse().ok()?;
    Some(color.lighten(amount).to_css())
}
```

#### Scenario: Function lookup
- `dispatch_pass` detects "lighten(" pattern and tries `try_eval_builtin`
- If returns Some, emits the resolved string
- If returns None, emits the original token unchanged
