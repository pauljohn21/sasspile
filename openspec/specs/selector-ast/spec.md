# selector-ast Spec

## 需求 1: 选择器 AST 类型层级

### 1.1 类型定义

系统必须定义以下类型层级来表示 CSS 选择器：

- `Selector` — 顶层选择器，包含 `Vec<ComplexSelector>`（逗号分隔列表）
- `ComplexSelector` — 组合器分隔的复合选择器序列，包含 `Vec<(Option<Combinator>, CompoundSelector)>`
- `CompoundSelector` — 无空格的简单选择器序列，包含 `Vec<SimpleSelector>`
- `SimpleSelector` — 最小选择器单元，是以下之一的 enum：
  - `Universal` — `*`
  - `Type(String)` — `div`, `a`, `span`
  - `Class(String)` — `.btn`
  - `Id(String)` — `#main`
  - `Attribute` — `[type="text"]`，含 name/op/value/modifier 字段
  - `PseudoClass` — `:hover`, `:nth-child(2n+1)`，含 name + arg
  - `PseudoElement` — `::before`, 含 name + arg
  - `Placeholder(String)` — `%button`
- `Combinator` — enum: `Descendant`(空格) / `Child`(`>`) / `Adjacent`(`+`) / `Sibling`(`~`)

### 1.2 所有类型必须 derive `Debug, Clone, PartialEq`

### 1.3 Selector 必须实现 `std::fmt::Display`，序列化为规范 CSS 字符串

## 需求 2: 选择器解析器

### 2.1 parse_selector 函数

`pub fn parse_selector(input: &str) -> Selector` — 将选择器字符串解析为 `Selector` AST。

### 2.2 解析能力

解析器必须正确处理：
- 逗号分隔的选择器列表 → `Vec<ComplexSelector>`
- 空格/`>`/`+`/`~` 组合器 → `Combinator`
- 类型选择器 → `Type`
- 通配符 → `Universal`
- 类选择器 → `Class`
- ID 选择器 → `Id`
- 属性选择器 → `Attribute`（含引号去除、修饰符提取）
- 伪类 → `PseudoClass`（含参数，递归处理 `:not()`/`:is()` 内部）
- 伪元素 → `PseudoElement`（`::` 双冒号和 `:` 单冒号旧语法）
- 占位符 → `Placeholder`（`%` 前缀）
- 命名空间前缀（`ns|type`）

### 2.3 降级策略

无法解析的输入必须返回包含原始字符串的降级 AST，不 panic。

## 需求 3: unify 算法

### 3.1 unify_compound

`pub fn unify_compound(a: &CompoundSelector, b: &CompoundSelector) -> Option<CompoundSelector>`

规则：
- 两个 `Type` 不同 → `None`
- 两个 `Id` 不同 → `None`
- 两个 `PseudoElement` 不同 → `None`
- `Universal` + `Type` → `Type`（Universal 被收窄）
- `Class`/`PseudoClass`/`Attribute` → 并集去重
- 结果顺序：Type → Universal → Id → Class → Attribute → PseudoClass → PseudoElement

### 3.2 unify_complex

`pub fn unify_complex(a: &ComplexSelector, b: &ComplexSelector) -> Option<ComplexSelector>`

从右端开始匹配复合选择器，对每对调用 `unify_compound`。任一对返回 `None` 则整体返回 `None`。

### 3.3 unify

`pub fn unify(a: &Selector, b: &Selector) -> Option<Selector>`

对 a 和 b 的所有 `ComplexSelector` 笛卡尔积调用 `unify_complex`，过滤 `None`。全部为 `None` 时返回 `None`。

## 需求 4: is_superselector 算法

### 4.1 is_super_compound

`fn is_super_compound(super_c: &CompoundSelector, sub_c: &CompoundSelector) -> bool`

规则：
- `super_c` 的所有 `SimpleSelector` 必须在 `sub_c` 中找到匹配
- `PseudoElement` 例外：`super_c` 无 `PseudoElement` 或与 `sub_c` 相同
- `Type` 匹配：名称相同，或 `super_c` 为 `Universal`
- `Class` 匹配：名称相同
- `PseudoClass` 匹配：名称相同，或 `super_c` 无参数而 `sub_c` 有参数

### 4.2 is_super_complex

`fn is_super_complex(super_c: &ComplexSelector, sub_c: &ComplexSelector) -> bool`

规则：
- `super_c` 的复合选择器序列是 `sub_c` 的子序列（LCS 匹配）
- 每个匹配对调用 `is_super_compound`
- 组合器必须兼容（`Descendant` 在 super 中可匹配任何组合器在 sub 中）

### 4.3 is_superselector

`pub fn is_superselector(super_sel: &Selector, sub_sel: &Selector) -> bool`

当且仅当 `super_sel` 中每个 `ComplexSelector` 都是 `sub_sel` 中至少一个 `ComplexSelector` 的超选择器时返回 `true`。

## 需求 5: extend/replace 算法

### 5.1 extend_selector

`pub fn extend_selector(selector: &Selector, extendee: &Selector, extender: &Selector) -> Selector`

规则：
- 对 `selector` 中每个 `ComplexSelector`，检查是否匹配 `extendee` 的某个 `ComplexSelector`
- 匹配时：用 `unify(匹配部分, extender)` 生成新选择器
- 统一冲突时不追加（保持原选择器）
- 返回原选择器 + 所有扩展选择器的并集（去重）

### 5.2 replace_selector

`pub fn replace_selector(selector: &Selector, original: &Selector, replacement: &Selector) -> Selector`

规则同 extend，但匹配部分直接替换为 `replacement`（而非 unify）。

## 需求 6: @extend 指令重写

### 6.1 apply_extends 重写

`apply_extends` 函数必须使用 `extend_selector` 替代字符串 `replace`：
- 对每个 `CssNode::Rule`，`parse_selector(selector)` 转为 AST
- 对每个 extend `(extender, target)`，调用 `extend_selector`
- 序列化回字符串设置 `Rule.selector`
- 移除未被继承的占位符选择器（`%xxx`）

### 6.2 模块 scope 保持

现有模块 scope 检查逻辑（`module_selectors`）不变。

## 需求 7: 内建函数重写

### 7.1 selector-unify

使用 `parse_selector` + `unify` + `Display` 实现。

### 7.2 selector-extend

使用 `parse_selector` + `extend_selector` + `Display` 实现。

### 7.3 selector-replace

使用 `parse_selector` + `replace_selector` + `Display` 实现。

### 7.4 selector-is-superselector / selector-is-super

使用 `parse_selector` + `is_superselector` 实现。

### 7.5 其他函数保持不变

`selector-append`、`selector-nest`、`selector-parse`、`selector-simple-selectors` 保持现有实现。
