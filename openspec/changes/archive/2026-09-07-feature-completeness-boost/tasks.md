## 1. Phase 0: Quick Win — 合并 fix-module-dispatch

- [x] 1.1 将 `openspec/changes/archive/2026-09-06-fix-module-dispatch/` 的改动应用到 src/eval/builtin/dispatch.rs
- [x] 1.2 验证 string_dispatch/map_dispatch/list_dispatch/math_dispatch/selector_dispatch 的模块限定名转换正确
- [x] 1.3 运行 202 核心测试确认不回归
- [x] 1.4 运行 sass-spec 全量确认 +30~80 case 增量
- [x] 1.5 Commit: "fix: 模块限定名 dispatch 统一转换 — Phase 0"

## 2. Phase 1: Meta 反射函数

- [x] 2.1 在 MetaBuiltins 结构体增加 get-function/get-mixin 字段和 BuiltinRegistry derive
- [x] 2.2 实现 `get-function($name, $css: false)` —— 从 env 查找函数定义返回可调用的 Function 值
- [x] 2.3 实现 `get-mixin($name)` —— 从 env 查找 mixin 定义
- [x] 2.4 实现 `module-variables($module)` —— 返回指定模块变量名映射
- [x] 2.5 实现 `module-functions($module)` —— 返回指定模块函数名列表
- [x] 2.6 实现 `module-mixins($module)` —— 返回指定模块 mixin 名列表
- [x] 2.7 实现 `load-css($module, $with)` —— 运行时动态加载模块 CSS
- [x] 2.8 实现 `apply($function, $args...)` —— 按引用动态调用
- [x] 2.9 实现 `keywords($args)` —— 返回 mixin 命名参数 Map
- [x] 2.10 实现 `feature-exists($feature)` —— 检测 Sass 特性可用性
- [x] 2.11 实现 `content-exists()` —— 检测 @content 是否存在
- [x] 2.12 添加对应 sass-spec 测试 case，运行 meta/ 子目录确认增量
- [x] 2.13 Meta 反射函数已实现（2026-09-06 前的提交）

## 3. Phase 2: 颜色函数深度修复

- [x] 3.0 Round 1: color.channel() 修复 (6424→6431, +7)
- [x] 3.0 Round 2: lab/lch/oklab/oklch 构造函数修复 (color/ +169 cases)
  - calc(NaN/infinity) 透传、百分位转换、CIE lightness clamp [NaN→0, -0→0, 超range clamp]
- [ ] 3.1 诊断 hsl/ 子目录 176 个失败 case 的根因分类
- [ ] 3.2 修复 HSL 序列化 —— hue NaN → "none"、浮点精度截断、百分比格式化
- [ ] 3.3 诊断 hwb/ 子目录 107 个失败 case 的根因（全白全黑混合规范、NaN 处理）
- [ ] 3.4 修复 HWB 序列化 —— whiteness+blackness ≥ 100% 时规范化为 HSL
- [ ] 3.5 诊断 lab/ 子目录 115 个失败 case 的根因（chroma=0 hue→none、百分比格式化）
- [ ] 3.6 修复 Lab 序列化 —— L% a b 格式、NaN 处理
- [ ] 3.7 诊断 lch/ 子目录 52 个失败 case 的根因
- [ ] 3.8 修复 Lch 序列化 —— chroma=0 时 hue→none
- [ ] 3.9 诊断 oklab/ 子目录 52 个失败 case 的根因
- [ ] 3.10 修复 Oklab 序列化 —— L 0-1 → 0%-100%、NaN 处理
- [ ] 3.11 诊断 oklch/ 子目录 52 个失败 case 的根因
- [ ] 3.12 修复 Oklch 序列化 —— 同 Lch 边界规则
- [ ] 3.13 诊断 mix/ 子目录 72 个失败 case 的根因（算法精度、权重边界）
- [ ] 3.14 修复 color-mix 算法 —— srgb/oklch 空间插值
- [ ] 3.15 诊断 invert/ 子目录 48 个失败 case 的根因
- [ ] 3.16 修复 invert($space) —— 按色彩空间求反色
- [ ] 3.17 诊断 change/ 子目录 234 个失败 case 的根因分类
- [ ] 3.18 修复 change-color 在现代色彩空间的通道修改
- [ ] 3.19 诊断 scale/ 子目录 233 个失败 case 的根因分类
- [ ] 3.20 修复 scale-color 在现代色彩空间的通道缩放
- [ ] 3.21 运行 color/* 全部子目录统计，对比基线确认增量
- [ ] 3.22 Commit: "fix: 颜色函数 hsl/hwb/lab/lch/oklab/oklch/mix/invert/change/scale 深度修复 — Phase 2"

## 4. Phase 3: Values + CSS Spec 合规

- [ ] 4.1 诊断 values/ 644 个失败 case 的根因分类（数字格式、字符串序列化、null 处理、列表/map 边界）
- [ ] 4.2 修复数字解析边界 —— 科学计数法、负零、前导小数点
- [ ] 4.3 修复字符串序列化 —— 空字符串、特殊字符转义
- [ ] 4.4 修复 null 值处理 —— 布尔判断、序列化抑制、列表中的 null
- [ ] 4.5 修复 boolean 字面量处理 —— true/false 与字符串区分
- [ ] 4.6 修复列表/Map 边界 —— 空括号、单元素、嵌套
- [ ] 4.7 修复插值边界 —— url() 内插值、空插值
- [ ] 4.8 诊断 css/ 374 个失败 case 的根因分类
- [ ] 4.9 修复 @supports 查询语法 —— 属性值查询、not/and/or 组合
- [ ] 4.10 修复 @media 嵌套和范围语法
- [ ] 4.11 修复自定义属性（--*）和 var() 默认值保留
- [ ] 4.12 运行 values/ 和 css/ 子目录统计确认增量
- [ ] 4.13 Commit: "fix: values 解析序列化边界 + CSS spec 合规 — Phase 3"

## 5. Phase 4: 模块系统高级特性

- [ ] 5.1 在 parse_use 中增加命名空间冲突检测 —— 同一模块多次 @use 无 as 时报错
- [ ] 5.2 在 parse_forward 中完善 show/hide 逻辑 —— show 白名单、hide 黑名单
- [ ] 5.3 在 parse_forward 中完善 prefix 拼接 —— `as prefix-*`
- [ ] 5.4 在 parse_forward 中完善 with() config 传递
- [ ] 5.5 在 file_resolver.rs 中增强 @import 歧义检测 —— 列出候选文件
- [ ] 5.6 运行 directives/use/、directives/forward/、directives/import/ 确认增量
- [ ] 5.7 Commit: "feat: @use/@forward 高级特性 + @import 歧义检测 — Phase 4"

## 6. Phase 5: Directives + Operators + 错误消息

- [ ] 6.1 修复 @for 边界 —— 浮点步长、反向循环、单位一致性检查
- [ ] 6.2 修复 @each 边界 —— 空列表、多变量解构、嵌套列表
- [ ] 6.3 修复 @while 边界 —— 初始 false、复合条件
- [ ] 6.4 修复 @if falsy/truthy —— 空字符串 truthy、0 truthy、空列表 truthy
- [ ] 6.5 修复 @function 边界 —— 可选参数、可变参数、@return 终止
- [ ] 6.6 修复 @mixin 边界 —— 空 mixin、复杂默认值
- [ ] 6.7 修复 operators —— 字符串拼接引号传播、null 传播、布尔短路、不兼容单位比较
- [ ] 6.8 修复除法运算符优先级 —— calc 中 / 保留、括号中做除法
- [ ] 6.9 修复错误消息格式 —— 未定义变量/函数附行号、模块搜索路径、@extend 建议修复
- [ ] 6.10 运行 directives/ 和 operators/ 子目录统计确认增量
- [ ] 6.11 Commit: "fix: directives /operators 边界 + 错误消息格式 — Phase 5"

## 7. 全局验证 + 归档

- [ ] 7.1 运行 202 核心测试确认全通过
- [ ] 7.2 运行 sass-spec 全量统计确认目标通过率 ≥63%
- [ ] 7.3 按 phase 分 commit，等待用户确认后 push
- [ ] 7.4 归档 openspec 变更到 archive
