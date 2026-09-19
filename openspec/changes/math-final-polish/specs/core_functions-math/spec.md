## ADDED Requirements

### Requirement: random 函数参数校验容错

math.random() 的 $limit 参数 SHALL 接受接近整数的浮点值（epsilon 容差 1e-9），SHALL 拒绝 null 参数并给出清晰错误。

#### Scenario: 接受浮点近似整数
- **WHEN** 调用 `math.random($limit: 1.0000000000001)`
- **THEN** 不报错，将 limit 视为 1 并返回有效随机数

#### Scenario: 拒绝 null 参数
- **WHEN** 调用 `math.random($limit: null)`
- **THEN** 返回错误 "$limit: null is not a number."

#### Scenario: random 返回值在范围内
- **WHEN** 调用 `math.random($limit: 5)`
- **THEN** 返回整数 n 满足 1 <= n <= 5

### Requirement: math 函数 DIFF 输出对齐

math.pow / math.tan / math.clamp / math.sin / math.div 的输出格式 SHALL 与 sass-spec 预期一致，包括数值精度、单位拼接方式。

#### Scenario: pow 运算输出
- **WHEN** 调用 sass-spec 中的 math.pow 测试用例
- **THEN** 输出 CSS 与预期字符串逐字符匹配

#### Scenario: tan 角度转换
- **WHEN** 调用 sass-spec 中的 math.tan 测试用例（可能涉及 deg/rad 单位）
- **THEN** 换算为正确弧度后计算正切值

### Requirement: 只读数学变量支持

math.$e / math.$pi / math.$epsilon 等常量 SHALL 在 `variables` 和 `unit` 模块中正确引用和解析。

#### Scenario: 变量引用数学常量
- **WHEN** SCSS 代码中通过 math.$pi 引用数学常量
- **THEN** 正确获取常量数值并参与表达式运算
