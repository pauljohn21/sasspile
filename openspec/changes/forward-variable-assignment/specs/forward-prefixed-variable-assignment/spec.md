# forward-prefixed-variable-assignment

## Requirements

### Requirement: Namespaced variable assignment forwards to upstream module

WHEN `@forward "upstream" as prefix-*` is used AND `namespace.$prefix-var: value` is executed, the assignment SHALL update the original variable in the upstream module.

#### Scenario: Basic prefixed assignment

- GIVEN `_midstream.scss` has `@forward "upstream" as d-*`
- GIVEN `_upstream.scss` has `$a: old value`
- WHEN code does `midstream.$d-a: new value`
- THEN subsequent reads of `midstream.d-get-a()` return `new value`

#### Scenario: Assignment from nested scope

- GIVEN same forward setup
- WHEN assignment is inside a CSS rule: `a { midstream.$d-b: new value; }`
- THEN the assignment still updates the upstream module's `$b` (namespace assignment ignores block scope)
