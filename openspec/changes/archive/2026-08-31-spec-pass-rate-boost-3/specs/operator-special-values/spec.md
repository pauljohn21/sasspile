## ADDED Requirements

### Requirement: 运算符对 calc 值的处理

系统 SHALL 支持 `+`/`-` 运算符对 `calc()` 值的操作，生成 CSS 原生 `calc()` 表达式而非报错。

#### Scenario: calc + number
- **WHEN** 执行 `calc(1px + 2px) + 3px` 运算
- **THEN** 系统 SHALL 返回 `calc(1px + 2px + 3px)` 或等效 CSS，不报 "Unsupported + operation" 错误

#### Scenario: calc - number
- **WHEN** 执行 `calc(100% - 10px) - 5px` 运算
- **THEN** 系统 SHALL 返回等效 CSS calc 表达式，不报 "Unsupported - operation" 错误

#### Scenario: calc + calc
- **WHEN** 执行 `calc(1px + 2px) + calc(3px + 4px)` 运算
- **THEN** 系统 SHALL 返回合并后的 CSS calc 表达式

### Requirement: 运算符对 get-mixin 值的处理

系统 SHALL 在 `get-mixin()` 值参与算术运算时报有意义的的错误消息，而非 "Unsupported +/- operation" 通用错误。

#### Scenario: get-mixin 乘法报错
- **WHEN** 执行 `get-mixin("a") * get-mixin("b")` 运算
- **THEN** 系统 SHALL 报有意义的错误消息说明 mixin 引用不能参与算术运算

#### Scenario: get-mixin 加法报错
- **WHEN** 执行 `get-mixin("a") + 1` 运算
- **THEN** 系统 SHALL 报有意义的错误消息说明 mixin 引用不能参与加法运算

### Requirement: 模块循环检测

系统 SHALL 在模块加载链中检测循环引用并报 "Module loop: this module is already being loaded." 错误。

#### Scenario: 直接循环引用
- **WHEN** 模块 A @use 模块 B，模块 B @use 模块 A
- **THEN** 系统 SHALL 报 "Module loop: this module is already being loaded." 错误

#### Scenario: 间接循环引用
- **WHEN** 模块 A @use 模块 B，B @use C，C @use A
- **THEN** 系统 SHALL 报模块循环错误

#### Scenario: 无循环的钻石依赖
- **WHEN** 模块 A @use B 和 C，B 和 C 都 @use D
- **THEN** 系统 SHALL 正常加载，不报循环错误
