# Rivulet — 纯函数式 FRP Web 框架设计

## 概述

Rivulet 是一个受 sasspile 启发的纯 Rust 函数式 Web 框架，采用 Behavior/Event 经典 FRP 模型，Flutter 风格的 Widget + Builder API，支持 WASM、SSR、小程序三平台。

### 设计灵感

sasspile 的管线架构（`Source → Lexer → Parser → Evaluator → Serializer → CSS`）直接映射到 Rivulet：

```
Events → accumulate → Behavior → view() → Widget → diff → Patches → Renderer → DOM/HTML/WXSS
```

每个阶段是纯函数转换，类型状态保证编译时阶段安全。

### 核心决策

| 维度 | 选择 |
|------|------|
| 编程模型 | FRP（Behavior/Event 经典模型，无可变 signal） |
| 视图语法 | Builder 模式（纯函数式链式调用） |
| 样式系统 | Flutter 风格 Style 对象（纯 Rust，零 CSS/SCSS DSL） |
| 组件模型 | 一切皆 Widget（`#[widget]` 宏） |
| 内存管理 | Arena 分配 + NodeId 轻量句柄 |
| 目标平台 | WASM + SSR + 小程序 |

---

## 1. 核心架构与管线

### 管线架构

```
                    ┌─────────────────────────────────────────────────────┐
                    │              Rivulet 管线架构                         │
                    │                                                      │
  DOM Events ──►   │  Event<E> ──► Behavior<S> ──► View ──► Widget ──► Diff │  ──► DOM/HTML/WXSS
  Timer ──────►    │     │            │            │         │        │   │
  Fetch ──────►    │     ▼            ▼            ▼         ▼        ▼   │
                    │  (离散事件)   (连续状态)   (纯函数)  (树diff)  (渲染)  │
                    └─────────────────────────────────────────────────────┘
```

### 两大核心类型

| 类型 | 语义 | sasspile 对应 | 时间行为 |
|------|------|-------------|---------|
| `Event<T>` | 离散事件流（可能不发生，可能多次） | Token 流 | 离散推送 |
| `Behavior<T>` | 连续时变值（任何时刻可读） | Env 环境 | 连续可读 |

### 类型状态阶段

借鉴 sasspile 的 `Source → Parsed → ... → CSS`，Rivulet 用类型标记管线阶段，保证编译时阶段安全：

```rust
Event<Raw>        // 原始事件，未处理
  → Event<Parsed> // 已解析为领域事件
  → Behavior<State> // 已累积为状态
  → View           // 已映射为视图
  → Rendered       // 已渲染
```

### Workspace Crate 结构

```
rivulet/
├── rivulet-core/       # Arena + Event<T> + Behavior<T> + 组合子 + 推送引擎
├── rivulet-vdom/       # Widget 类型 + diff 算法（纯函数）
├── rivulet-builder/    # WidgetBuilder + Style 对象 + 辅助类型 + 内置元素函数
├── rivulet-web/        # WASM/DOM Renderer + SSR Renderer
├── rivulet-mp/         # 小程序 Renderer
├── rivulet-macro/      # #[widget] 等过程宏
├── rivulet-router/     # 路由（基于 Event<Navigation>）
└── rivulet/            # 框架入口 + prelude
```

### Crate 依赖图

```
rivulet (入口 + prelude)
├── rivulet-core        (Arena + Event + Behavior + 推送引擎)
├── rivulet-vdom        (Widget + Diff——纯函数，依赖 core 的类型)
├── rivulet-builder     (WidgetBuilder + Style + 辅助类型 + 内置元素函数)
├── rivulet-macro       (#[widget] proc macro)
├── rivulet-router      (路由，依赖 core)
├── rivulet-web         (WebRenderer + SsrRenderer)
│   ├── rivulet-core
│   ├── rivulet-vdom
│   └── rivulet-builder
└── rivulet-mp          (MpRenderer)
    ├── rivulet-core
    ├── rivulet-vdom
    └── rivulet-builder
```

---

## 2. Arena-based Event<T> 与 Behavior<T>

### Arena 核心设计

所有节点分配在统一的 Arena 中，`Event`/`Behavior` 是轻量句柄（NodeId 索引），不是堆指针。

```rust
/// 全局 Arena，持有所有 Event/Behavior 节点的实际数据。
/// 类似 sasspile 的 Evaluator 持有 Env——所有状态集中管理。
///
/// WASM 单线程：使用 RefCell 内部可变性
/// SSR 多线程：使用 Mutex（feature gate）
pub struct Runtime {
    arena: Arena<Node>,
    /// 事件订阅图（谁订阅了谁）
    graph: DepGraph,
    /// 待处理事件队列（事件先入队，再批量推送）
    queue: EventQueue,
}
```

### 轻量句柄

```rust
/// 轻量句柄——只是一个索引，不持有任何堆数据
/// 复制、传递零成本（8 bytes vs Rc 的 16 bytes + 引用计数操作）
pub struct Event<T> {
    id: NodeId,      // slotmap key (u32 + generation)
    _phantom: PhantomData<T>,
}

pub struct Behavior<T> {
    id: NodeId,
    _phantom: PhantomData<T>,
}
```

### Arena 节点

```rust
enum Node {
    Event(EventNode),
    Behavior(BehaviorNode),
}

struct EventNode {
    /// 事件来源（DOM、Timer、上游 Event 的变换等）
    source: EventSource,
    /// 订阅了此事件的下游节点
    dependents: SmallVec<[NodeId; 4]>,
}

struct BehaviorNode {
    /// 当前值（类型擦除后存在 Arena 中，通过 type tag 安全转换）
    value: Box<dyn Any>,
    /// 上游 Event（状态由此 Event 驱动更新）
    upstream: Option<NodeId>,
    /// 更新函数（事件到达时调用）
    updater: Option<Box<dyn Fn(&mut dyn Any, &dyn Any)>>,
    /// 下游依赖此 Behavior 的节点
    dependents: SmallVec<[NodeId; 4]>,
}
```

### Arena 优于 Rc<RefCell> 的理由

| 维度 | Rc<RefCell<T>> | Arena + NodeId |
|------|-----------------|----------------|
| 句柄大小 | 16 bytes | 8 bytes |
| 复制开销 | 原子引用计数操作 | 整数复制 |
| 内存布局 | 堆分散，缓存不友好 | 连续 slab，缓存友好 |
| 生命周期 | 需追踪 Rc 环 | Arena 统一回收 |
| 依赖图遍历 | 需递归跟指针 | 直接索引查表 O(1) |
| SSR 多线程 | 需换 Arc<Mutex> | 只需 Arena 换 Mutex |

### Event 组合子

```rust
impl<T: Clone + 'static> Event<T> {
    /// 映射：Event<Click> → Event<i32>
    pub fn map<U: 'static>(self, f: impl Fn(&T) -> U + 'static) -> Event<U>;

    /// 过滤：只保留满足条件的事件
    pub fn filter(self, pred: impl Fn(&T) -> bool + 'static) -> Event<T>;

    /// 采样：当事件发生时，读取 Behavior 当前值
    pub fn sample<B>(self, behavior: &Behavior<B>) -> Event<B> where B: Clone;

    /// 合并：两条事件流合并为一条
    pub fn merge(self, other: Event<T>) -> Event<T>;
}
```

### Behavior 组合子

```rust
impl<T: Clone + 'static> Behavior<T> {
    /// 读取当前值（纯读取，不订阅）
    pub fn now(&self) -> T;

    /// 映射：Behavior<i32> → Behavior<String>
    pub fn map<U: 'static>(self, f: impl Fn(&T) -> U + 'static) -> Behavior<U>;

    /// 从事件流累积状态（核心！这是状态唯一的来源）
    /// 类似 sasspile 的 eval_nodes 逐个处理 Node 累积 CSS 输出
    pub fn accumulate<E: 'static>(
        event: Event<E>,
        initial: T,
        update: impl Fn(&mut T, &E) + 'static,
    ) -> Behavior<T>;
}
```

### 事件推送引擎

事件不是同步递归推送（会栈溢出），而是入队后批量处理：

```rust
impl Runtime {
    /// 事件入队（DOM 回调调用此方法）
    fn emit(&self, event_id: NodeId, payload: Box<dyn Any>) {
        self.queue.push((event_id, payload));
    }

    /// 批量处理事件队列（每帧调用一次）
    fn flush(&self) {
        while let Some((event_id, payload)) = self.queue.pop() {
            self.propagate(event_id, payload);
        }
    }

    /// 沿依赖图推送事件
    fn propagate(&self, source: NodeId, payload: &dyn Any) {
        let dependents = self.arena.get(source).unwrap().dependents.clone();
        for dep_id in dependents {
            match self.arena.get(dep_id).unwrap() {
                Node::Event(enode) => {
                    if let Some(transformed) = enode.transform(payload) {
                        self.propagate(dep_id, &transformed);
                    }
                }
                Node::Behavior(bnode) => {
                    if let Some(updater) = &bnode.updater {
                        (updater)(bnode.value.as_mut(), payload);
                        self.propagate(dep_id, &bnode.value);
                    }
                }
            }
        }
    }
}
```

### Runtime 生命周期

```rust
/// 创建 Runtime（类似 sasspile 的 compile() 入口）
/// 所有 Event/Behavior 必须在 Runtime 上下文中创建
pub fn runtime<F: FnOnce(&Runtime) -> R, R>(f: F) -> R {
    let rt = Runtime::new();
    f(&rt)
}
```

### Emitter（事件触发器）

```rust
impl Runtime {
    /// 创建一个新的事件源
    /// 返回 Event 句柄 + Emitter（用于在 DOM 回调中触发事件）
    pub fn create_event<T: Clone + 'static>(&self) -> (Event<T>, Emitter<T>) {
        let id = self.arena.register(EventNode {
            source: EventSource::Manual,
            dependents: SmallVec::new(),
        });
        (
            Event { id, _phantom: PhantomData },
            Emitter { id, rt: self },
        )
    }
}

/// 事件触发器——在 DOM 事件回调中调用
pub struct Emitter<T> {
    id: NodeId,
    rt: *const Runtime,
}

impl<T: Clone + 'static> Emitter<T> {
    pub fn fire(&self, payload: T) {
        // SAFETY: Runtime 生命周期覆盖所有 Emitter
        unsafe { (*self.rt).emit(self.id, Box::new(payload)); }
    }
}
```

### Context（全局状态传递）

```rust
impl Runtime {
    /// 提供 Context 值（祖先 Widget 调用）
    pub fn provide_context<T: Clone + 'static>(&self, value: T);

    /// 消费 Context 值（后代 Widget 调用）
    pub fn use_context<T: Clone + 'static>(&self) -> Option<T>;
}
```

### 与 banyan 的关键区别

| banyan（可变 signal） | Rivulet（FRP） |
|----------------------|----------------|
| `let count = create_signal(0); count.set(1);` | `let count = Behavior::accumulate(events, 0, \|s, e\| ...);` |
| 直接可变，set 即改 | 事件驱动累积，无 set |
| 响应式系统通过 effect 传播 | 管线推送，事件触发链式更新 |
| `create_memo` 需要手动追踪依赖 | `map`/`accumulate` 天然声明依赖 |

---

## 3. Widget + Builder + Style 统一体系

### 一切皆 Widget

采用 Flutter 术语——一切皆 Widget，不可变 UI 描述：

```rust
/// 一切皆 Widget——不可变数据，描述 UI 的一部分
pub enum Widget {
    /// HTML 元素 Widget
    Element(ElementWidget),
    /// 文本 Widget
    Text(TextWidget),
    /// 组合 Widget（用户定义的 #[widget] 函数）
    Composite(CompositeWidget),
    /// 多子节点无包裹
    Fragment(Vec<Widget>),
    /// 空（条件渲染）
    None,
}

pub struct ElementWidget {
    pub tag: Cow<'static, str>,
    pub attrs: Vec<(Cow<'static, str>, AttrValue)>,
    pub style: Option<Cow<'static, str>>,
    pub children: Vec<Widget>,
    pub key: Option<Key>,
}

pub struct TextWidget {
    pub content: TextContent,
    pub style: Option<Cow<'static, str>>,
}

pub struct CompositeWidget {
    /// Widget 函数的类型 ID（用于 diff 时判断类型是否相同）
    pub type_id: TypeId,
    /// Props（类型擦除）
    pub props: Box<dyn Any>,
    /// 渲染结果缓存
    pub cached: Option<Box<Widget>>,
}
```

### 属性值类型

```rust
/// 属性值——支持静态、动态、事件绑定
pub enum AttrValue {
    /// 静态属性值
    Static(Cow<'static, str>),
    /// 动态属性值（绑定 Behavior，值变化时自动更新）
    Dynamic(Behavior<String>),
    /// 事件绑定（DOM 事件 → Emitter 触发）
    Event(EventBinding),
    /// 动态样式（Behavior<Style> 驱动）
    StyleDyn(Behavior<Style>),
}

/// 事件绑定信息
pub struct EventBinding {
    pub emitter_id: NodeId,
    pub payload: Box<dyn Any>,
}
```

### 内置元素函数

所有 HTML 元素是内置函数，返回 `WidgetBuilder`，链式调用统一视图和样式：

```rust
// HTML 元素 → WidgetBuilder
pub fn div() -> WidgetBuilder { WidgetBuilder::new("div") }
pub fn span() -> WidgetBuilder { WidgetBuilder::new("span") }
pub fn button() -> WidgetBuilder { WidgetBuilder::new("button") }
pub fn input() -> WidgetBuilder { WidgetBuilder::new("input") }
pub fn h1() -> WidgetBuilder { WidgetBuilder::new("h1") }
pub fn h2() -> WidgetBuilder { WidgetBuilder::new("h2") }
pub fn p() -> WidgetBuilder { WidgetBuilder::new("p") }
pub fn a() -> WidgetBuilder { WidgetBuilder::new("a") }
pub fn img() -> WidgetBuilder { WidgetBuilder::new("img") }
// ... 所有 HTML 元素

// 文本 Widget
pub fn text(s: impl Into<Cow<'static, str>>) -> TextBuilder {
    TextBuilder { content: s.into(), style: Style::new() }
}

// 条件渲染
pub fn if_widget(cond: bool, then: impl FnOnce() -> Widget) -> Widget {
    if cond { then() } else { Widget::None }
}

// 列表映射
pub fn list<T, F>(items: &[T], render: F) -> Widget
where F: Fn(&T) -> Widget,
{
    Widget::Fragment(items.iter().map(render).collect())
}
```

### WidgetBuilder（视图 + 样式统一）

```rust
pub struct WidgetBuilder {
    tag: &'static str,
    attrs: Vec<(Cow<'static, str>, AttrValue)>,
    children: Vec<Widget>,
    key: Option<Key>,
    style: Style,
}

impl WidgetBuilder {
    // === 视图方法 ===

    pub fn child(mut self, child: impl Into<Widget>) -> Self;
    pub fn children(mut self, children: impl IntoIterator<Item = Widget>) -> Self;
    pub fn key(mut self, k: impl Into<Key>) -> Self;
    pub fn id(self, id: impl Into<Cow<'static, str>>) -> Self;
    pub fn attr(self, key: &'static str, val: impl Into<Cow<'static, str>>) -> Self;
    pub fn attr_dyn(self, key: &'static str, behavior: Behavior<String>) -> Self;

    // === 样式方法（Flutter 风格，直接在 Builder 上）===

    // 布局
    pub fn flex(mut self) -> Self;
    pub fn block(mut self) -> Self;
    pub fn grid(mut self) -> Self;
    pub fn hidden(mut self) -> Self;
    pub fn row(mut self) -> Self;
    pub fn column(mut self) -> Self;
    pub fn center(mut self) -> Self;
    pub fn justify(mut self, j: JustifyContent) -> Self;
    pub fn align(mut self, a: AlignItems) -> Self;
    pub fn gap(mut self, g: impl Into<Dimension>) -> Self;

    // 盒模型
    pub fn margin(mut self, e: EdgeInsets) -> Self;
    pub fn padding(mut self, e: impl Into<EdgeInsets>) -> Self;
    pub fn width(mut self, d: impl Into<Dimension>) -> Self;
    pub fn height(mut self, d: impl Into<Dimension>) -> Self;
    pub fn min_width(mut self, d: impl Into<Dimension>) -> Self;
    pub fn min_height(mut self, d: impl Into<Dimension>) -> Self;
    pub fn max_width(mut self, d: impl Into<Dimension>) -> Self;
    pub fn max_height(mut self, d: impl Into<Dimension>) -> Self;

    // 装饰
    pub fn bg(mut self, c: impl Into<Color>) -> Self;
    pub fn border(mut self, b: Border) -> Self;
    pub fn radius(mut self, r: impl Into<BorderRadius>) -> Self;
    pub fn shadow(mut self, s: Shadow) -> Self;
    pub fn opacity(mut self, o: f32) -> Self;

    // 文字
    pub fn font_size(mut self, d: impl Into<Dimension>) -> Self;
    pub fn font_weight(mut self, w: FontWeight) -> Self;
    pub fn font_family(mut self, f: impl Into<String>) -> Self;
    pub fn color(mut self, c: impl Into<Color>) -> Self;
    pub fn line_height(mut self, lh: f32) -> Self;

    // 其他
    pub fn cursor(mut self, c: Cursor) -> Self;
    pub fn absolute(mut self) -> Self;
    pub fn relative(mut self) -> Self;
    pub fn fixed(mut self) -> Self;
    pub fn z(mut self, z: i32) -> Self;
    pub fn overflow(mut self, o: Overflow) -> Self;

    // 动态样式
    pub fn style_dyn(self, style_behavior: Behavior<Style>) -> Self;

    // 事件
    pub fn on_click_emit<T: Clone + 'static>(self, emitter: Emitter<T>, payload: T) -> Self;

    // 逃生舱口
    pub fn raw_style(mut self, key: &'static str, val: impl Into<Cow<'static, str>>) -> Self;

    // 构建
    fn build(self) -> Widget;
}

// From 转换——让 child() 接受多种类型
impl From<WidgetBuilder> for Widget;
impl From<TextBuilder> for Widget;
impl From<&str> for Widget;
impl From<String> for Widget;
```

### TextBuilder

```rust
pub struct TextBuilder {
    content: Cow<'static, str>,
    style: Style,
}

impl TextBuilder {
    pub fn color(mut self, c: impl Into<Color>) -> Self;
    pub fn font_size(mut self, d: impl Into<Dimension>) -> Self;
    pub fn font_weight(mut self, w: FontWeight) -> Self;
    pub fn font_family(mut self, f: impl Into<String>) -> Self;
    pub fn line_height(mut self, lh: f32) -> Self;
    fn build(self) -> Widget;
}
```

### Style 对象

```rust
/// 样式对象——纯数据，不可变，可组合
#[derive(Debug, Clone, Default)]
pub struct Style {
    // 布局
    display: Option<Display>,
    direction: Option<FlexDirection>,
    justify_content: Option<JustifyContent>,
    align_items: Option<AlignItems>,
    gap: Option<Dimension>,
    // 盒模型
    margin: Option<EdgeInsets>,
    padding: Option<EdgeInsets>,
    width: Option<Dimension>,
    height: Option<Dimension>,
    min_width: Option<Dimension>,
    min_height: Option<Dimension>,
    max_width: Option<Dimension>,
    max_height: Option<Dimension>,
    // 装饰
    background: Option<Color>,
    border: Option<Border>,
    border_radius: Option<BorderRadius>,
    box_shadow: Option<Vec<Shadow>>,
    opacity: Option<f32>,
    // 文字
    font_size: Option<Dimension>,
    font_weight: Option<FontWeight>,
    font_family: Option<String>,
    color: Option<Color>,
    text_decoration: Option<TextDecoration>,
    line_height: Option<f32>,
    letter_spacing: Option<Dimension>,
    // 其他
    overflow: Option<Overflow>,
    cursor: Option<Cursor>,
    position: Option<Position>,
    z_index: Option<i32>,
    transition: Option<Vec<Transition>>,
    transform: Option<Vec<Transform>>,
    // 原始 CSS 属性（逃生舱口）
    raw: Vec<(Cow<'static, str>, Cow<'static, str>)>,
}

impl Style {
    pub fn new() -> Self { Self::default() }

    /// 合并两个 Style，other 覆盖 self
    pub fn merge(mut self, other: &Style) -> Self;

    /// 序列化为 CSS 字符串
    pub fn to_css(&self) -> String;

    /// 是否为空
    pub fn is_empty(&self) -> bool;
}
```

### 辅助类型

```rust
/// 尺寸
#[derive(Debug, Clone, Copy)]
pub enum Dimension {
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
    Rem(f32),
    Em(f32),
    Auto,
}

impl From<i32> for Dimension;
impl From<f32> for Dimension;

/// 边距——类似 Flutter 的 EdgeInsets
#[derive(Debug, Clone, Copy)]
pub struct EdgeInsets {
    pub top: Dimension,
    pub right: Dimension,
    pub bottom: Dimension,
    pub left: Dimension,
}

impl EdgeInsets {
    pub fn all(v: impl Into<Dimension>) -> Self;
    pub fn symmetric(horizontal: impl Into<Dimension>, vertical: impl Into<Dimension>) -> Self;
    pub fn only(top: impl Into<Dimension>, right: impl Into<Dimension>, bottom: impl Into<Dimension>, left: impl Into<Dimension>) -> Self;
    pub fn h(v: impl Into<Dimension>) -> Self;  // 仅水平
    pub fn v(v: impl Into<Dimension>) -> Self;  // 仅垂直
}

/// 颜色
#[derive(Debug, Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);  // r, g, b, a

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self;
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self;
    pub fn hex(hex: &str) -> Self;
    pub fn black() -> Self;
    pub fn white() -> Self;
    pub fn transparent() -> Self;
}

/// 边框
#[derive(Debug, Clone)]
pub struct Border {
    pub width: Dimension,
    pub style: BorderStyle,
    pub color: Color,
}

impl Border {
    pub fn all(width: impl Into<Dimension>, color: impl Into<Color>) -> Self;
}

/// 圆角
#[derive(Debug, Clone, Copy)]
pub struct BorderRadius {
    pub top_left: Dimension,
    pub top_right: Dimension,
    pub bottom_right: Dimension,
    pub bottom_left: Dimension,
}

impl BorderRadius {
    pub fn all(r: impl Into<Dimension>) -> Self;
    pub fn circular(r: impl Into<Dimension>) -> Self;
}

/// 阴影
#[derive(Debug, Clone)]
pub struct Shadow {
    pub x: Dimension,
    pub y: Dimension,
    pub blur: Dimension,
    pub spread: Dimension,
    pub color: Color,
}

/// 枚举类型
pub enum Display { Flex, Block, Grid, Inline, InlineBlock, None }
pub enum FlexDirection { Row, RowReverse, Column, ColumnReverse }
pub enum JustifyContent { Start, End, Center, SpaceBetween, SpaceAround, SpaceEvenly }
pub enum AlignItems { Start, End, Center, Stretch, Baseline }
pub enum FontWeight { Thin, Light, Normal, Medium, Bold, Black }
pub enum Position { Static, Relative, Absolute, Fixed, Sticky }
pub enum Overflow { Visible, Hidden, Scroll, Auto }
pub enum Cursor { Pointer, Default, Text, Move, NotAllowed, Grab }
pub enum BorderStyle { Solid, Dashed, Dotted, None }
pub enum TextDecoration { None, Underline, LineThrough, Overline }
```

### Theme 系统

```rust
/// 全局主题——类似 Flutter 的 ThemeData
#[derive(Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub danger: Color,
    pub bg: Color,
    pub text: Color,
    pub font_size_base: Dimension,
    pub radius: BorderRadius,
    pub spacing: Dimension,
}

impl Default for Theme { /* ... */ }
```

---

## 4. Diff 算法

### Patch 类型

```rust
/// DOM 补丁——diff 的输出，Renderer 消费的指令
pub enum Patch {
    Insert { parent: DomId, node: Widget, index: usize },
    Remove { parent: DomId, index: usize },
    Replace { parent: DomId, index: usize, node: Widget },
    SetAttr { node: DomId, key: Cow<'static, str>, value: AttrValue },
    RemoveAttr { node: DomId, key: Cow<'static, str> },
    SetText { node: DomId, content: TextContent },
    SetStyle { node: DomId, style: Option<Cow<'static, str>> },
    Reorder { parent: DomId, moves: Vec<Move> },
}

/// 纯函数 diff：旧 Widget + 新 Widget → Patch 列表
pub fn diff(old: &Widget, new: &Widget) -> Vec<Patch>;

/// 子节点 diff（带 key 优化）
/// 使用最长递增子序列（LIS）最小化 DOM 移动
fn diff_children(old: &[Widget], new: &[Widget], parent: DomId) -> Vec<Patch>;
```

### Diff 性能策略

| 策略 | 说明 |
|------|------|
| Key-based 复用 | 相同 key 的节点视为同一节点，只 diff 属性差异 |
| LIS 优化 | 子节点重排时用最长递增子序列，最小化 DOM move |
| 提前退出 | 类型不同（Element vs Text）直接 Replace |
| 静态跳过 | 静态属性永不 diff，只 diff Dynamic 属性 |
| Behavior 直连 | Dynamic 属性的 Behavior 变化时只推送该属性 Patch |

---

## 5. 多目标渲染器

### Renderer Trait

```rust
/// 渲染后端接口——不同平台实现不同
pub trait Renderer {
    fn render_initial(&self, widget: &Widget) -> RenderOutput;
    fn apply_patches(&self, patches: &[Patch]);
    fn bind_events(&self, widget: &Widget, rt: &Runtime);
}

pub enum RenderOutput {
    DomMount(DomId),
    SsrHtml { html: String, hydration: HydrationData },
    MpWxml { wxml: String, data_map: MpDataMap },
}
```

### WebRenderer（WASM + DOM）

```rust
pub struct WebRenderer {
    document: web_sys::Document,
    root: web_sys::Element,
    node_map: RefCell<HashMap<DomId, web_sys::Node>>,
}
```

- `render_initial`：Widget → DOM 节点，挂载到 root
- `apply_patches`：逐个 Patch 转换为 DOM 操作（createElement, setAttribute, insertBefore...）
- `bind_events`：遍历 Widget，为有事件绑定的节点注册 DOM 事件监听，事件触发时调用 `rt.emit()`

### SSRRenderer（服务端渲染）

```rust
pub struct SsrRenderer;
```

- `render_initial`：Widget → HTML 字符串 + HydrationData
- 纯字符串拼接，无 DOM 依赖
- `render_to_string` 递归遍历 Widget 树

### Hydration（SSR → WASM 衔接）

```rust
#[derive(Serialize, Deserialize)]
pub struct HydrationData {
    pub behaviors: Vec<(NodeId, serde_json::Value)>,
    pub event_bindings: Vec<EventBinding>,
}

pub fn hydrate(html_root: &web_sys::Element, data: &HydrationData);
```

客户端 hydration 流程：
1. 从 HydrationData 恢复所有 Behavior 的初始状态
2. 扫描已有 DOM，建立 Widget ↔ DOM 映射
3. 切换到 WebRenderer，后续更新走 Patch 增量

### MPRenderer（小程序）

```rust
pub struct MpRenderer {
    data_map: RefCell<MpDataMap>,
}
```

- `render_initial`：Widget → WXML + setData 映射
- `apply_patches`：Patch → setData 调用（批量 setData）
- 小程序不支持直接 DOM 操作，所有更新通过 setData

### 事件回流闭环

```
                    事件回流
    ┌──────────────────────────────────────────┐
    │                                          │
    ▼                                          │
DOM Event ──► WebRenderer::bind_events()       │
                │                              │
                ▼                              │
            rt.emit(event_id, payload)         │
                │                              │
                ▼                              │
            Runtime::flush()                   │
                │                              │
                ▼                              │
            propagate() ──► Behavior 更新       │
                │                              │
                ▼                              │
            view() 重新计算                     │
                │                              │
                ▼                              │
            diff() → Patch                     │
                │                              │
                ▼                              │
            Renderer::apply_patches() ─────────┘
```

### 多平台编译策略

```toml
[features]
default = ["web"]
web = ["dep:wasm-bindgen", "dep:web-sys"]
ssr = ["dep:html-escape"]
mp = []
hydrate = ["web"]
```

---

## 6. Widget 组件模型 + 路由

### Widget Trait 体系

```rust
/// Widget Props trait——#[widget] 宏自动 derive
pub trait WidgetProps: 'static {
    type Builder;
    fn builder() -> Self::Builder;
}

/// Widget 函数 trait——所有 FnOnce(P) -> Widget 自动实现
pub trait WidgetFn<P: WidgetProps>: 'static {
    fn build(self, props: P) -> Widget;
}

impl<F, P> WidgetFn<P> for F
where F: FnOnce(P) -> Widget + 'static, P: WidgetProps,
{
    fn build(self, props: P) -> Widget { self(props) }
}
```

### `#[widget]` 宏

```rust
#[widget]
fn Counter(initial: i32, label: String) -> Widget {
    let rt = use_runtime();
    let (inc_event, inc_emitter) = rt.create_event::<()>();
    let count = Behavior::accumulate(inc_event, initial, |s, _| *s += 1);

    count.map(|c| {
        div()
            .flex()
            .row()
            .gap(8)
            .child(
                button()
                    .padding(EdgeInsets::symmetric(8, 12))
                    .bg(Color::rgb(52, 152, 219))
                    .color(Color::white())
                    .radius(4)
                    .cursor(Cursor::Pointer)
                    .on_click_emit(inc_emitter.clone(), ())
                    .child(text("+"))
            )
            .child(
                span()
                    .font_size(18)
                    .font_weight(FontWeight::Bold)
                    .child(text(format!("{label}: {c}")))
            )
    })
}

// 宏展开：
// struct CounterProps { initial: i32, label: String }
// impl WidgetProps for CounterProps { type Builder = CounterPropsBuilder; ... }
// fn Counter(props: CounterProps) -> Widget { ... }
```

### Widget 组合

```rust
#[widget]
fn App() -> Widget {
    div()
        .flex()
        .column()
        .gap(16)
        .child(
            Counter.builder()
                .initial(0)
                .label("Clicks".into())
                .build()
        )
        .child(
            Counter.builder()
                .initial(10)
                .label("Score".into())
                .build()
        )
}
```

### 路由

路由是一个 `Event<Navigation>` → `Behavior<Route>` 的管线，完全 FRP：

```rust
#[derive(Clone, PartialEq)]
pub enum Route {
    Home,
    Users,
    UserDetail { id: u32 },
    NotFound,
}

pub fn router() -> Behavior<Route> {
    let rt = use_runtime();
    let nav_events = rt.navigation_events();  // Event<Navigation>

    Behavior::accumulate(
        nav_events,
        Route::Home,
        |_route, nav| match nav {
            Navigation::Path(path) => match path.as_str() {
                "/" => Route::Home,
                "/users" => Route::Users,
                p if p.starts_with("/users/") => {
                    let id = p.strip_prefix("/users/").unwrap().parse().unwrap_or(0);
                    Route::UserDetail { id }
                }
                _ => Route::NotFound,
            }
        },
    )
}

#[widget]
fn App() -> Widget {
    let route = router();

    route.map(|r| match r {
        Route::Home => Home.builder().build(),
        Route::Users => Users.builder().build(),
        Route::UserDetail { id } => UserDetail.builder().id(id).build(),
        Route::NotFound => text("404 Not Found"),
    })
}
```

---

## 7. 错误处理

```rust
#[derive(Debug)]
pub enum RivuletError {
    Runtime(RuntimeError),
    View(ViewError),
    Render(RenderError),
}

#[derive(Debug)]
pub enum RuntimeError {
    CycleDetected { from: NodeId, to: NodeId },
    DisposedNode(NodeId),
    TypeMismatch { expected: &'static str, got: &'static str },
}

#[derive(Debug)]
pub enum RenderError {
    DomNotFound(DomId),
    InvalidAttribute(String),
    JsError(String),
}

pub type Result<T> = std::result::Result<T, RivuletError>;
```

**策略**：
- 编译时（宏）：`compile_error!` 直接报错
- 运行时（DOM 操作）：`Result` 传播，调试模式 `panic` + 友好信息，发布模式 `unwrap_or` 降级
- Arena 操作：类型不匹配在调试模式下 panic，发布模式 best-effort

---

## 8. 测试策略

### 分层测试

| 层级 | 范围 | 运行环境 | 示例 |
|------|------|---------|------|
| 单元测试 | 单个组合子/函数 | 纯 Rust | `Event::map` 正确变换 |
| 管线测试 | 多阶段串联 | 纯 Rust | Event → accumulate → map → Widget |
| Diff 测试 | Widget 对比 | 纯 Rust | `diff(old, new)` 产出正确 Patch |
| SSR 测试 | 完整渲染 | 纯 Rust | `render_to_string(widget)` == 期望 HTML |
| WASM 测试 | DOM 交互 | wasm-pack | 点击 → 状态更新 → DOM 变更 |
| 集成测试 | 全栈 | 纯 Rust | TodoApp 完整功能验证 |

### 测试示例

```rust
#[test]
fn accumulate_counts_events() {
    runtime(|rt| {
        let (event, emitter) = rt.create_event::<i32>();
        let count = Behavior::accumulate(event, 0, |s, n| *s += n);

        assert_eq!(count.now(), 0);
        emitter.fire(1);
        rt.flush();
        assert_eq!(count.now(), 1);
        emitter.fire(5);
        emitter.fire(3);
        rt.flush();
        assert_eq!(count.now(), 9);
    });
}

#[test]
fn diff_replaces_text() {
    let old = div().child(text("Hello")).build();
    let new = div().child(text("World")).build();
    let patches = diff(&old, &new);
    assert_eq!(patches.len(), 1);
    assert!(matches!(&patches[0], Patch::SetText { content, .. }
        if content.as_str() == "World"));
}

#[test]
fn ssr_renders_styled_element() {
    let widget = div()
        .flex()
        .row()
        .center()
        .padding(8)
        .bg(Color::rgb(52, 152, 219))
        .child(text("Hello"))
        .build();

    let html = SsrRenderer.render_to_string(&widget);
    assert!(html.contains("display:flex"));
    assert!(html.contains("background:rgb(52,152,219)"));
    assert!(html.contains("Hello"));
}
```

---

## 9. 里程碑

| 里程碑 | 内容 | 验证标准 |
|--------|------|---------|
| **M0: 核心 FRP** | `rivulet-core`：Arena + Event/Behavior + 组合子 + 推送引擎 | 单元测试：accumulate/map/filter/merge/sample |
| **M1: Widget + Diff + Builder** | `rivulet-vdom` + `rivulet-builder`：Widget 类型、WidgetBuilder API（含 Style）、Diff 算法 | 单元测试：build/diff/SSR 渲染 |
| **M2: SSR 渲染** | `rivulet-web` SSR 部分：`render_to_string` + HydrationData | SSR 测试：完整 HTML 输出 |
| **M3: WASM 渲染** | `rivulet-web` WASM 部分：DOM Renderer + 事件绑定 + Hydration | WASM 测试：点击 → DOM 变更 |
| **M4: Widget 宏** | `rivulet-macro`：`#[widget]` 宏 + WidgetProps derive | 集成测试：Widget 组合 |
| **M5: 路由** | `rivulet-router`：Event<Navigation> → Behavior<Route> | 单元测试：路由匹配 |
| **M6: 小程序** | `rivulet-mp`：MpRenderer + WXML/WXSS 输出 | 输出测试：WXML 比对 |
| **M7: 示例应用** | TodoApp + Dashboard 完整示例 | 手动验证 + 截图 |

### 开发顺序

```
M0 → M1 → M2 → M3 → M4 → M5 → M6 → M7
```

线性顺序，每个里程碑是前一个的依赖。

---

## 10. 完整示例

```rust
use rivulet::prelude::*;

#[widget]
fn TodoApp() -> Widget {
    let rt = use_runtime();
    let (add_event, add_emitter) = rt.create_event::<String>();
    let todos = Behavior::accumulate(add_event, Vec::new(), |todos, text| {
        todos.push(Todo::new(text));
    });

    todos.map(|items| {
        div()
            .flex()
            .column()
            .gap(16)
            .padding(EdgeInsets::all(20))
            .max_width(500)
            .child(
                h1()
                    .color(Color::rgb(52, 152, 219))
                    .font_size(24)
                    .child(text("Todo App"))
            )
            .child(
                div()
                    .flex()
                    .row()
                    .gap(8)
                    .child(
                        input()
                            .padding(EdgeInsets::symmetric(8, 8))
                            .border(Border::all(1, Color::rgb(150, 150, 150)))
                            .attr("placeholder", "Add todo...")
                    )
                    .child(
                        button()
                            .padding(EdgeInsets::symmetric(12, 8))
                            .bg(Color::rgb(52, 152, 219))
                            .color(Color::white())
                            .radius(4)
                            .cursor(Cursor::Pointer)
                            .on_click_emit(add_emitter.clone(), "new todo".to_string())
                            .child(text("Add"))
                    )
            )
            .child(
                list(&items, |todo| {
                    div()
                        .padding(EdgeInsets::all(8))
                        .raw_style("border-bottom", "1px solid #eee")
                        .child(text(todo.text.clone()))
                })
            )
    })
}

fn main() {
    rivulet::web::mount_to_body(TodoApp);
}
```
