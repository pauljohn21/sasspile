# SCSS String Concatenation Fix Spec

## ADDED Requirements

### Requirement: String-Concat-Op
The `+` SHALL concatenate two string values into a single string value.

#### Scenario: String-Literal Concatenation
- **WHEN** evaluating `'.' + '__'`
- **THEN** the result SHALL be `"__"` (prefix dot + double-underscore)

#### Scenario: String-With-Variable Concatenation
- **WHEN** evaluating `'.' + $name + '__'` where `$name = 'el-breadcrumb'`
- **THEN** the result SHALL be `".el-breadcrumb__"` 
- **AND** SHALL NOT contain backslash escape sequences (`\.`, `\+`, `\a`, etc.)
- **AND** SHALL NOT be split into individual characters

#### Scenario: Mixed-Type Stringification
- **WHEN** evaluating `1 + 'px'` or `'item-' + 2`
- **THEN** the result SHALL be `"1px"` or `"item-2"` respectively

#### Scenario: Interpolation Context
- **WHEN** evaluating `#{$a + '.' + $b}` where `$a = 'foo'`, `$b = 'bar'`
- **THEN** the output SHALL be `"foo.bar"`
