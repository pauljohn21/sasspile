## ADDED Requirements

### Requirement: Rule renders as multi-line block

`CssNode::render()` SHALL output a `Rule` node in multi-line format with `{` on the selector line and `}` on its own line.

#### Scenario: Single declaration rule
- **WHEN** rendering `Rule { selector: "a", body: [Declaration { prop: "b", value: "0" }] }`
- **THEN** output equals `"a {\n  b: 0;\n}"`

#### Scenario: Multiple declaration rule
- **WHEN** rendering a Rule with body `[Declaration { prop: "b", value: "0" }, Declaration { prop: "c", value: "1" }]`
- **THEN** output equals `"a {\n  b: 0;\n  c: 1;\n}"`

### Requirement: Empty Rule renders as `selector {\n}`

When a Rule body is empty, `CssNode::render()` SHALL output `"selector {\n}"`.

#### Scenario: Empty rule
- **WHEN** rendering `Rule { selector: "a", body: [] }`
- **THEN** output equals `"a {\n}"`

### Requirement: Declaration inside Rule has 2-space indent

Each Declaration inside a Rule body SHALL be prefixed with exactly 2 spaces.

#### Scenario: Decl indent
- **WHEN** rendering a Rule with one Declaration `b: 0;`
- **THEN** the `b: 0;` line is prefixed with `"  "`

### Requirement: Top-level Declaration and Comment unchanged

Declaration and Comment nodes outside Rule SHALL render exactly as before (single-line, no indent prefix).

#### Scenario: Top-level declaration
- **WHEN** rendering `Declaration { prop: "color", value: "red" }` outside a Rule
- **THEN** output equals `"color: red;"`

#### Scenario: Comment
- **WHEN** rendering `Comment("hello")`
- **THEN** output equals `"/* hello */"`

### Requirement: sass-spec core_functions pass rate increases after format fix

After applying the serializer fix alone, the `sass_spec_detail` per-directory pass rate for `core_functions` SHALL increase significantly from 0/7793.

#### Scenario: math/abs/zero spec case
- **WHEN** compiling `@use "sass:math";\na {b: math.abs(0)}`
- **THEN** after `normalize_css` the result matches `normalize_css("a {\n  b: 0;\n}")`

#### Scenario: color/alpha/color case  
- **WHEN** compiling `@use "sass:color";\na {b: color.alpha(red)}`
- **THEN** after `normalize_css` the result matches `normalize_css("a {\n  b: 1;\n}")`
