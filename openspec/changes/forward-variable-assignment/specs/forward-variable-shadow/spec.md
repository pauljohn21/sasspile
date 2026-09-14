# forward-variable-shadow

## Requirements

### Requirement: Forward chain variable shadowing works correctly

WHEN multiple modules forward the same upstream variable with different prefixes, each namespace SHALL see its own assigned value.

#### Scenario: Two namespaces, same upstream variable

- GIVEN `_upstream.scss` has `$a: original`
- GIVEN `_mid1.scss` has `@forward "upstream" as x-*`
- GIVEN `_mid2.scss` has `@forward "upstream" as y-*`
- WHEN `mid1.$x-a: first` and `mid2.$y-a: second` are executed
- THEN `mid1.x-get-a()` returns `first` and `mid2.y-get-a()` returns `second`
