## color-functions

sass-spec/core_functions/color 子目录的所有色彩操作函数。

### Scenario: darken 基本

**Given**: `$c: #ff0000; .a { color: darken($c, 20%); }`
**Then**: 输出 #cc0000（HSL L 通道减 20% 再转回 RGB）

### Scenario: lighten 基本

**Given**: `$c: #336699; .a { color: lighten($c, 10%); }`
**Then**: HSL L 通道加 10%

### Scenario: mix 50%

**Given**: `mix(#ff0000, #0000ff, 50%)`
**Then**: 输出 #800080（紫色）或等价的 rgb 表达

### Scenario: rgba 4 参数

**Given**: `rgba(255, 0, 0, 0.5)`
**Then**: 输出 `rgba(255, 0, 0, 0.5)`

### Scenario: alpha 提取

**Given**: `$c: rgba(0,0,0,0.7); alpha($c)`
**Then**: `0.7`

### Scenario: grayscale

**Given**: `grayscale(#ff8800)`
**Then**: 输出灰度等价色（ luminance = 0.299R+0.587G+0.114B ）

### Scenario: invert

**Given**: `invert(#ff0000)`（红色）
**Then**: `#00ffff`（青色）

### Scenario: hex 3 位扩展

**Given**: `#f00` 作为颜色传入函数
**Then**: 正确解析为 (255, 0, 0)
