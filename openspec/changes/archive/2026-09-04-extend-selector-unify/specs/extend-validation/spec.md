## Requirement: Complex Selector Extend Rejection

When `@extend` targets a complex selector (containing multiple compound selectors separated by combinators), the compiler MUST raise an error.

**Error message**: `complex selectors may not be extended.`

**Example**:
```scss
a b { a: b; }
c { @extend a b; }  // Error: complex selectors may not be extended.
```

## Requirement: Compound Selector Extend Rejection

When `@extend` targets a compound selector (a simple selector followed by a pseudo-class/pseudo-element), the compiler MUST raise an error.

**Error message**: `compound selectors may no longer be extended.`

**Example**:
```scss
a:hover { a: b; }
b { @extend a:hover; }  // Error: compound selectors may no longer be extended.
```

## Requirement: Empty Selector Extend Rejection

When `@extend` has no selector argument, the compiler MUST raise an error.

**Error message**: `expected selector.`

**Example**:
```scss
a { @extend; }  // Error: expected selector.
```

## Requirement: Comma-Separated Multi-Target Extend

When `@extend` targets a comma-separated list of selectors, each selector MUST be treated as an independent extend target.

**Example**:
```scss
.a, .b { c: d; }
.x { @extend .a, .b; }
// Equivalent to:
// .x { @extend .a; @extend .b; }
```

Output MUST include `.x` in both `.a` and `.b` selector rules.
