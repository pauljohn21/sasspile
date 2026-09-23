## Capability: module-loading

### ADDED Requirements

#### Requirement: Rx-wrapped module load
The compiler SHALL load @use'd modules using reactive scheduling.

#### Scenario: @use loads module content
- **WHEN** '@use "variables.scss";'
- **THEN** file "variables.scss" is loaded, its public variables exposed to scope

#### Scenario: @use with namespace
- **WHEN** '@use "lib" as l;'
- **THEN** access via `l.$var` — state stores namespace mapping

#### Scenario: @forward
- **WHEN** '@forward "upstream";'
- **THEN** upstream's public symbols become accessible downstream

#### Scenario: !default override via with
- **WHEN** '@use "theme" with ($primary: red);'
- **THEN** the default variable is overridden

---

### Requirement: Async/Sync load dual mode
For v1.0, module loading SHALL read from file system synchronously; async via tokio for production.

#### Scenario: Sync load in dispatcher
- `dispatch_pass` encounters `@use`, reads file, parses, stores Module in CompileState.modules
- Failure to find file → emit warning, do not abort
