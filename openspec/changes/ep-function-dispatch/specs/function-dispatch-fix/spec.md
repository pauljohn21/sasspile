## ADDED Requirements

### Requirement: Custom @function 同文件调用
The system SHALL correctly dispatch calls to user-defined `@function` within the same source file, where the function is defined via `@function name(...) { @return ... }`.

#### Scenario: Simple function call within same file
- **WHEN** a .scss file defines `@function double($x) { @return $x * 2; }` and calls `double(5)`
- **THEN** the call SHALL return `Value::Number(10.0, None)` and NOT produce CSS passthrough output like `double(5)`

#### Scenario: Function call with list argument
- **WHEN** a .scss file defines `@function my-join($list) { ... }` and calls `my-join((a, b, c))`
- **THEN** the argument SHALL be correctly bound as a List value to the `$list` parameter

### Requirement: Custom @function 跨模块调用
The system SHALL correctly dispatch calls to user-defined `@function` from a `@use` or `@import` loaded module.

#### Scenario: Function from @use'd module
- **WHEN** file A defines `@function bem($block) { ... }` and file B `@use 'A'` then calls `bem('el-button')`
- **THEN** the call SHALL execute file A's function body and return the computed Value

#### Scenario: EP joinVarName function
- **WHEN** element-plus's `joinVarName` function is defined in utils.scss and called from button.scss as `joinVarName(('color', 'primary'))`
- **THEN** the call SHALL return the joined string value like `el-color-primary` (exact behavior per the function's logic)

### Requirement: Function definition registration in Env
The system SHALL register user-defined `@function` into the environment's function lookup table during `@function` evaluation, making it available for subsequent calls.

#### Scenario: @function definition followed by usage
- **WHEN** `eval_func_def` processes `@function foo() { @return 42; }`
- **THEN** `env.get_function("foo")` SHALL return the FunctionDef after evaluation

#### Scenario: Multiple function definitions
- **WHEN** a file defines `@function a() {}` and `@function b() {}`
- **THEN** both `env.get_function("a")` and `env.get_function("b")` SHALL succeed

### Requirement: Function call tracing diagnostics
The system SHALL emit tracing spans at function dispatch time to diagnose dispatch failures in production-like scenarios.

#### Scenario: Undefined function call
- **WHEN** a call to undefined function `nonexistent()` is made and falls through to CSS passthrough
- **THEN** a trace span SHALL be emitted recording the function name and argument count

#### Scenario: Namespace traversal for function lookup
- **WHEN** `call_function` searches namespaces for a user-defined function
- **THEN** a trace span SHALL record which namespaces were searched and whether a match was found

## MODIFIED Requirements

*(无现有 spec 被修改 — 这是 bug 修复，不改变规范行为)*

## REMOVED Requirements

*(无)*
