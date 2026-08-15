## ADDED Requirements

### Requirement: Semantic Analyzer SHALL resolve variable scopes
The analyzer SHALL track variable scopes (global, local, params) and resolve identifiers to their declarations.

#### Scenario: Local shadows global
- **WHEN** global `$color: red;` and local `$color: blue;` both exist
- **THEN** references inside the local scope resolve to `$color: blue`

#### Scenario: Undefined variable error
- **WHEN** source references undefined `$undefined-var`
- **THEN** analyzer produces error "Undefined variable: $undefined-var" with location

### Requirement: Semantic Analyzer SHALL track @use / @forward dependencies
The analyzer SHALL resolve `@use "module" as ns` and forward member access `ns.$var`.

#### Scenario: Namespace resolution
- **WHEN** `@use "lib"` then `lib.foo()` appears
- **THEN** analyzer resolves `lib.foo` to `lib::FooFunction`

#### Scenario: Circular dependency detection
- **WHEN** module A uses module B and B uses A
- **THEN** analyzer produces error "Circular dependency detected: A -> B -> A"

### Requirement: Semantic Analyzer SHALL validate @extend selectors
The analyzer SHALL verify that extended selectors exist and produce warnings for non-existent extends.

#### Scenario: Valid extend
- **WHEN** selector `.base` exists and `@extend .base` appears
- **THEN** analyzer resolves the extend target successfully

#### Scenario: Extend missing selector
- **WHEN** `@extend .nonexistent` with no matching selector
- **THEN** analyzer produces warning "The target selector was not found"

### Requirement: Semantic Analyzer SHALL collect mixin/function definitions
The analyzer SHALL build a registry of all top-level mixin and function definitions for name resolution.

#### Scenario: Duplicate definition
- **WHEN** two `@function foo(...)` appear at the global scope
- **THEN** analyzer produces error "There is already a function named 'foo'"

### Requirement: Semantic Analyzer SHALL type-check built-in functions
The analyzer SHALL validate argument count and types for built-in sass functions where possible.

#### Scenario: Wrong arity
- **WHEN** `rgb(255, 0)` is called with 2 arguments instead of 3-4
- **THEN** analyzer produces error "Only 3-4 arguments allowed, 2 passed"
