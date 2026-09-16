## builtin-polish

List/String/Map 内置函数的边界处理与参数歧义消除。

### Scenario: nth 1-based 索引

**Given**: `$list: a, b, c; nth($list, 2)` 
**Then**: 输出 `b`（第 2 个元素）

### Scenario: length 多分隔符

**Given**: `length(a b c)` (空格分隔的 3 元素)
**Then**: `3`

### Scenario: append 追加元素

**Given**: `append(a b, c)`
**Then**: `(a, b, c)` 或 `a, b, c`（取决于测试用例格式）

### Scenario: map-get 命中

**Given**: `$map: (a: 1, b: 2); map-get($map, a)`
**Then**: `1`

### Scenario: map-get 未命中

**Given**: `map-get((a: 1), z)`
**Then**: 输出为空或 null（取决于 sass-spec，接受空字符串）

### Scenario: quote 添加引号

**Given**: `quote(foo)`
**Then**: `"foo"`

### Scenario: unquote 移除引号

**Given**: `unquote("foo")`
**Then**: `foo`

### Scenario: map-has-key true/false

**Given**: `map-has-key((a: 1, b: 2), b)`
**Then**: `true`

**Given**: `map-has-key((a: 1), z)`
**Then**: `false`
