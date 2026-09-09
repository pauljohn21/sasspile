## ADDED Requirements

### Requirement: list-separator 返回列表分隔符类型

`list.separator` MUST 返回列表的分隔符: `comma`, `space`, 或 `slash`。

#### Scenario: 逗号分隔列表
- **WHEN** 调用 `list.separator(1, 2, 3)`
- **THEN** 返回 `"comma"`

#### Scenario: 空格分隔列表
- **WHEN** 调用 `list.separator((1 2 3))`
- **THEN** 返回 `"space"`

#### Scenario: 斜杠分隔列表(新版语法)
- **WHEN** 调用 `list.separator(list.join(1 2, 3 4, $separator: slash))`
- **THEN** 返回 `"slash"`

### Requirement: list-set-nth 修改指定索引元素

`list.set-nth` MUST 在指定索引位置替换列表元素(1-based),返回新列表。

#### Scenario: 修改中间元素
- **WHEN** 调用 `list.set-nth(1.5 2.5 3.5, 2, 99)`
- **THEN** 返回 `1.5 99 3.5`

#### Scenario: 首元素替换
- **WHEN** 调用 `list.set-nth(a b c, 1, z)`
- **THEN** 返回 `z b c`

#### Scenario: 末元素替换
- **WHEN** 调用 `list.set-nth(a b c, 3, z)`
- **THEN** 返回 `a b z`

#### Scenario: 负索引规范
- **WHEN** 调用 `list.set-nth(a b c, -1, z)`(负索引从尾部计数)
- **THEN** 返回 `a b z`

### Requirement: list-join 按指定分隔符合并列表

`list.join` MUST 将两个列表按指定分隔成一个列表。

#### Scenario: 逗号分隔 join
- **WHEN** 调用 `list.join(1 2, 3 4, $separator: comma)`
- **THEN** 返回 `1, 2, 3, 4`

#### Scenario: 空格分隔 join
- **WHEN** 调用 `list.join(a b, c)`
- **THEN** 返回 `a b c`

### Requirement: 列表边界行为

对单元素列表、空列表和嵌套列表的操作 MUST 规范处理。

#### Scenario: 单元素列表 separator
- **WHEN** 调用 `list.separator((1,))`(显式逗号)
- **THEN** 返回 `"comma"`
