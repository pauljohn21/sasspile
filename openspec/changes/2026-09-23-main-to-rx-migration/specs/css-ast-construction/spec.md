## Capability: css-ast-construction

### ADDED Requirements

#### Requirement: CssNode enum construction
CssBuilder SHALL construct a `CssNode` tree from a flat stream of style-line tokens.

#### Scenario: Nested rule construction
- **WHEN** the input stream has nested selectors ".parent { .child { color: red; } }"
- **THEN** CssBuilder outputs a `CssNode::Rule { selector: ".parent .child", children: [Declaration("color", "red")] }`
- **AND** the parent ".parent" has been merged with nested ".child"

#### Scenario: AtRule nesting
- **WHEN** input contains "@media (max-width: 768px) { .foo { color: red; } }"
- **THEN** output CssNode::AtRule { query: "@media (max-width: 768px)", children: [Rule(".foo")] }

#### Scenario: AtRoot hoisting
- **WHEN** ".foo { @at-root .bar { color: red; } }" is processed
- **THEN** ".bar" rule is hoisted to top-level output (not nested under ".foo")

#### Scenario: Value interpolation
- **WHEN** ".foo { width: #{$size}px; }" with $size = 10
- **THEN** output Declaration has value "10px" after interpolation

---

### Requirement: rxrust pipeline integration  
CssBuilder SHALL be integrated into the `scan_map` chain so that line dispatch AST construction is reactive.

#### Scenario: scan_map consumes line stream
```
Shared::from_stream(futures::stream::iter(lines))
    .scan_map(CssBuilder::new(), |builder, line| builder.feed(line))
    .flat_map(from_iter)
    .collect::<Vec<CssNode>>()
    .last()
    .subscribe(|nodes| render_all(nodes))
```

#### Scenario: state machine drives CssBuilder dispatch
- CssBuilder internally tracks a stack of open Rule scopes
- When seeing "{", push new current Rule context
- When seeing "}", pop and attach to parent
- When seeing "key: value", push Declaration to current Rule

---

### Requirement: Selector stack preservation
`CompileState.selector_stack` SHALL be preserved across AST construction for descendant selector merging.

#### Scenario: Multi-level nesting with stack
- **WHEN** three-level nesting ".a { .b { .c { ... } } }"
- **THEN** selector_stack maintains [".a", ".a .b"] during processing
- **AND** ".c" is expanded to ".a .b .c" using the topmost parent
