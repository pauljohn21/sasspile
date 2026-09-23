## Capability: at-root-hoisting

### ADDED Requirements

#### Requirement: @at-root directive processing
The compiler SHALL process `@at-root` by hoisting its inner rules to the top-level output, bypassing enclosing parent selectors.

#### Scenario: Basic at-root
- **WHEN** ".wrapper { @at-root .item { color: red; } }"
- **THEN** output contains ".item { color: red; }" at top level
- **AND** no ".wrapper .item" generated

#### Scenario: at-root with selector list
- **WHEN** ".wrapper { @at-root { .a { color: red; } .b { color: blue; } } }"
- **THEN** both ".a" and ".b" are hoisted to top-level

#### Scenario: at-root with keyword
- **WHEN** ".wrapper { @at-root (without: media) { ... } }"
- **THEN** the "without: media" clause is respected (media query stripped)

---

### Requirement: AtRootDirect in CssNode
`CssNode::AtRoot { children: Vec<CssNode> }` SHALL be tracked through builder as hoisting wrapper.

#### Scenario: CssBuilder recognizes AtRoot
- When consuming CssNode::AtRoot, iterate children and emit each directly to output
- Do not push AtRoot onto the nesting stack
