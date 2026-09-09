## ADDED Requirements

### Requirement: 命名空间统一规则矩阵
`unify_compound` SHALL 按以下规则矩阵统一两个 Type 选择器：

| A ns \ B ns | None | Empty | Any | Explicit("c") |
|-------------|------|-------|-----|---------------|
| None | 同名统一 | 冲突 | 同名→B drop ns | 冲突 |
| Empty | 冲突 | 同名统一 | 同名→B drop ns | 冲突 |
| Any | 同名→A drop ns | 同名→A drop ns | 同名→drop ns | 同名→keep B ns + drop name if A="*" |
| Explicit("c") | 冲突 | 冲突 | 同名→keep A ns + drop name if B="*" | 同名 ns → 统一；不同 ns → 冲突 |

#### Scenario: None + None 同名
- **WHEN** 执行 `selector.unify("c", "c")`
- **THEN** 结果为 `"c"`

#### Scenario: None + None 不同名
- **WHEN** 执行 `selector.unify("c", "d")`
- **THEN** 结果为 `null`

#### Scenario: None + Empty 冲突
- **WHEN** 执行 `selector.unify("c", "|c")`
- **THEN** 结果为 `null`

#### Scenario: None + Explicit 冲突
- **WHEN** 执行 `selector.unify("c", "d|c")`
- **THEN** 结果为 `null`

#### Scenario: None + Any 同 name
- **WHEN** 执行 `selector.unify("c", "*|c")`
- **THEN** 结果为 `"c"`（drop ns）

#### Scenario: Explicit + Explicit 同 ns 同 name
- **WHEN** 执行 `selector.unify("c|d", "c|d")`
- **THEN** 结果为 `"c|d"`

#### Scenario: Explicit + Explicit 同 ns A="*"
- **WHEN** 执行 `selector.unify("c|*", "c|d")`
- **THEN** 结果为 `"c|d"`（specific wins）

#### Scenario: Explicit + Explicit 不同 ns
- **WHEN** 执行 `selector.unify("c|*", "d|e")`
- **THEN** 结果为 `null`

#### Scenario: Any + Explicit 同 name
- **WHEN** 执行 `selector.unify("*|d", "c|d")`
- **THEN** 结果为 `"c|d"`（keep B ns, B name wins if A="*"）

#### Scenario: Universal + Explicit 类型
- **WHEN** 执行 `selector.unify("*", "c|d")`
- **THEN** 结果为 `null`（无前缀 universal 与命名空间类型不兼容）

#### Scenario: Universal + Any 类型
- **WHEN** 执行 `selector.unify("*", "*|c")`
- **THEN** 结果为 `"c"`（any namespace + name c）

#### Scenario: Universal + None 类型
- **WHEN** 执行 `selector.unify("*", "c")`
- **THEN** 结果为 `"c"`

## MODIFIED Requirements

### Requirement: unify_compound 行为变更
`unify_compound` SHALL 将 Universal（`*`）视为与任何无命名空间 Type 兼容，返回该 Type；但与有命名空间 Type（`c|d`）不兼容 → `null`。

#### Scenario: Universal + 无命名空间类型
- **WHEN** 执行 `selector.unify("*", "div")`
- **THEN** 结果为 `"div"`

#### Scenario: Universal + 命名空间类型
- **WHEN** 执行 `selector.unify("*", "svg|circle")`
- **THEN** 结果为 `null`
