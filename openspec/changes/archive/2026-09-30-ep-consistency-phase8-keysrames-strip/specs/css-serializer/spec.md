## ADDED Requirements

### Requirement: Pseudo-element SHALL use double-colon format (CSS3)
The CSS serializer SHALL output pseudo-elements using double-colon notation (`::`) consistently, matching CSS3 specification and EP reference output.

#### Scenario: ::before serialization
- **WHEN** `.a::before { content: ""; }` is compiled
- **THEN** output SHALL contain `::before` (double colon), never single colon `:before`

#### Scenario: ::after serialization
- **WHEN** `.a::after { content: ""; }` is compiled
- **THEN** output SHALL contain `::after` (double colon)

#### Scenario: ::placeholder serialization
- **WHEN** `input::placeholder { color: gray; }` is compiled
- **THEN** output SHALL contain `::placeholder` (double colon)

#### Scenario: ::first-line and ::first-letter
- **WHEN** `.a::first-line { }` or `.a::first-letter { }` is compiled
- **THEN** output SHALL preserve double-colon format

#### Scenario: CSS2 pseudo-elements stay single-colon when source uses single
- **WHEN** `.a:before { content: ""; }` is compiled (SCSS source uses CSS2 syntax)
- **THEN** output SHALL normalize to `::before` (CSS3 double-colon format)

### Requirement: @keyframes empty blocks SHALL be preserved
When normalizing or serializing @keyframes output, the serializer SHALL preserve empty `0%` and `100%` blocks to match EP reference output structure.

#### Scenario: Empty 0% keyframe
- **WHEN** `@keyframes name { 0% {} 100% { opacity: 1; } }` is compiled
- **THEN** output SHALL contain `0% {}` block even when empty

#### Scenario: Empty 100% keyframe alongside non-empty
- **WHEN** `@keyframes name { 0% { opacity: 0; } 100% {} }` is compiled
- **THEN** output SHALL contain `100% {}` block

#### Scenario: Both 0% and 100% empty
- **WHEN** `@keyframes name { 0% {} 100% {} }` is compiled
- **THEN** output SHALL preserve both empty blocks

## MODIFIED Requirements

### Requirement: @keyframes
The CSS serializer SHALL preserve empty `0%` and `100%` blocks in @keyframes and SHALL output correct indentation for all keyframe selectors.

#### Scenario: Empty keyframe block preservation
- **WHEN** `@keyframes name { 0% {} 100% { opacity: 1; } }` is compiled
- **THEN** output SHALL contain `0% {}` block even when empty, matching EP reference output
