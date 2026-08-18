# Extend 引擎设计

## 1. 架构总览

### 1.1 当前架构（问题）

```
┌──────────────────────────────────────────────────────────┐
│  当前 @extend 数据流                                     │
│                                                          │
│  Parser ──▶ ExtendRule{selector, optional}              │
│      │                                                   │
│      ▼                                                   │
│  Eval::eval_stmt                                        │
│      │  parent_sel.last() → extender                    │
│      │  selector → extendee                             │
│      ▼                                                   │
│  Vec<ExtendEntry>  (全局，serialize 时用)                │
│      │                                                   │
│      │  eval_use_rule 传 &mut Vec::new() ← 丢弃子模块    │
│      ▼                                                   │
│  Serialize::serialize                                   │
│      │  apply_extends(): 字符串 contains 匹配            │
│      │  selector.contains(extendee) → ", extender"      │
│      ▼                                                   │
│  最终 CSS                                               │
└──────────────────────────────────────────────────────────┘
```

### 1.2 目标架构

```
┌──────────────────────────────────────────────────────────┐
│  目标 @extend 数据流                                     │
│                                                          │
│  Parser ──▶ ExtendRule{selector, optional}              │
│      │                                                   │
│      ▼                                                   │
│  Eval::eval_stmt                                        │
│      │  parent_sel → extender SelectorList              │
│      │  selector → extendee Selector (parsed)           │
│      │  module_id → 来源模块                             │
│      ▼                                                   │
│  ExtensionStore                                         │
│      │  ├─ extensions: Map<extendee, Vec<Extension>>    │
│      │  ├─ module_graph: ModuleId → Set<ModuleId>       │
│      │  └─ apply_extends(): 统一 + 传递 + 去重          │
│      │                                                   │
│      │  eval_use_rule 传 &mut store ← 子模块 extends 收集│
│      ▼                                                   │
│  CssTree { rules: (已应用 extend), extends: vec![] }    │
│      │                                                   │
│      ▼                                                   │
│  Serialize (纯输出，不做选择器逻辑)                      │
└──────────────────────────────────────────────────────────┘
```

核心原则：**extend 处理在 eval 层完成，serialize 只做输出**。

## 2. 核心数据结构

### 2.1 ExtensionStore

```rust
/// 集中管理所有 @extend 请求的核心数据结构。
pub struct ExtensionStore {
    /// extendee → 所有指向它的 Extension 列表
    /// key = extendee 选择器的字符串形式（规范化后）
    extensions: HashMap<String, Vec<Extension>>,

    /// 模块依赖图：module_id → 它 @use 的所有 module_id
    module_graph: HashMap<ModuleId, HashSet<ModuleId>>,

    /// 当前模块 ID 栈（eval 过程中 push/pop）
    module_stack: Vec<ModuleId>,

    /// 下一个模块 ID 分配器
    next_module_id: u32,
}

/// 一条 @extend 请求。
pub struct Extension {
    /// extender 的选择器列表（谁去 extend）
    pub extender: SelectorList,
    /// extendee 的选择器（被 extend 的，规范化 key）
    pub extendee: SelectorList,
    /// 来源模块 ID
    pub module_id: ModuleId,
    /// 是否 !optional
    pub optional: bool,
}

/// 模块标识符。
pub type ModuleId = u32;
```

### 2.2 模块作用域

每条 `Extension` 携带 `module_id`，标记它来自哪个模块。当应用 extend 到 CSS 规则时，需要检查：

```
规则 R 来自模块 M_rule
extend E 来自模块 M_ext
E 可影响 R ⟺ M_ext == M_rule
         ∨ M_rule ∈ downstream(M_ext)
         (即 M_ext 的 @use 链可达 M_rule)
```

`downstream(M)` = 从 M 出发，沿 `module_graph` 的边（M → M_use1 → M_use2 ...）可达的所有模块。

### 2.3 模块 ID 分配

每次 `eval_use_rule` 进入一个新模块时分配新 `ModuleId`：

```rust
// eval_use_rule 中
let module_id = store.next_module_id();
store.push_module(module_id);
// ... eval module AST ...
store.pop_module();
```

当前模块的 `@extend` 使用 `store.current_module_id()` 获取自己的 ID。

## 3. 算法设计

### 3.1 选择器统一（Unification）

#### 3.1.1 Compound 统一

当 extender `.a .b` extends extendee `.e`，且规则选择器是 `.e.f`：

```
原 compound:  .e.f
extendee:     .e  (compound 的一部分)
extender:     .a .b  (complex，最后一个 compound 是 .b)

步骤:
1. 在原 compound 中找到 extendee 的所有 simple
   → .e 匹配
2. 将匹配的 simple 替换为 extender 的最后一个 compound
   → .b.f  (保留 .f)
3. 如果 extender 是 complex（有前缀），拼接前缀
   → .a .b.f  (descendant combinator)

结果: .a .b.f
```

已有 `unify()` 函数在 `selector/mod.rs` 处理 compound 统一。需要扩展为支持 partial 替换。

#### 3.1.2 Complex 统一（Weave）

当 extender 和 extendee 都是 complex（有后代/组合关系）时，需要 weave（交织）：

```
extender:  .a .b
extendee:  .e
原选择器:  .e .f

weave 步骤:
1. 在原选择器中找到 extendee 匹配的 compound
   → .e (leading compound)
2. 用 extender 替换匹配部分
   → .a .b .f  (extender 整体替换 .e)
3. 但如果原选择器是 .e.g .f
   → unify(.e.g, .a .b 的最后 compound .b) → .b.g
   → .a .b.g .f
```

更复杂的 `nested-compound-unification` 场景：

```
.a .b {@extend .e}
.c .d {@extend .f}
.e.f {x: y}

第一轮: 对 .e.f 应用 @extend .e (extender = .a .b)
  → .e.f, .a .b.f   (unify(.e, .b) = .b, 保留 .f)

第二轮: 对 .e.f 和 .a .b.f 应用 @extend .f (extender = .c .d)
  .e.f → .e.d → .c .d.e  (unify(.f, .d) = .d, 保留 .e)
  .a .b.f → .a .b.d → .c .a .b.d  (前缀 .c + 替换 .f→.d)

还需传递性: .a .b.f 被 @extend .f 后产生 .c .a .b.d
  再对 .c .a .b.d 应用 @extend .e? 不需要，.e 已经处理过

spec output: .e.f, .a .f.b, .c .e.d, .a .c .b.d, .c .a .b.d
```

注意 `.a .f.b` 而非 `.a .b.f`——顺序由 unify 的 merge 规则决定。

#### 3.1.3 :is() 伪类穿透

```
:is(midstream) {@extend upstream}
downstream {@extend midstream}
upstream {a: b}

步骤:
1. :is(midstream) extends upstream → upstream, :is(midstream) {a: b}
2. downstream extends midstream
   → 在 :is(midstream) 内部找到 midstream → 替换为 downstream
   → :is(midstream, downstream)
3. 最终: upstream, :is(midstream), :is(midstream, downstream) {a: b}
```

需要识别 `:is()`/`:matches()`/`:where()` 等选择器伪类，递归应用 extend 到内部。

### 3.2 传递性解析

```
in-other-extender {@extend in-other-extendee}
in-input {@extend in-other-extender}

传递: in-input extends in-other-extender
      in-other-extender extends in-other-extendee
      ⟹ in-input extends in-other-extendee

构建 extend 依赖图:
  in-other-extender → [in-other-extendee]
  in-input → [in-other-extender]

BFS from in-input:
  visit in-other-extender
  visit in-other-extendee (transitive)
  → in-input extends in-other-extendee
```

循环检测使用 `visited: HashSet<String>`：

```
.a {@extend .b}
.b {@extend .a}

BFS from .a:
  visit .b (direct)
  visit .a (transitive) ← 已在 visited 中，跳过
  → 不无限递归
```

### 3.3 冗余消除

```
原选择器列表: [.c, .a .b .c, .a .c .b]
extend .c → 不添加新选择器（已有 .c）

.a.a → .a  (duplicate elimination)
.a 是 .a.b 的 superselector → 如果 .a 和 .a.b 都在列表中，移除 .a.b
```

`is_superselector(A, B)`: A 匹配 B 匹配的所有元素（A ⊇ B）。

已有 `is_superselector` 实现在 `selector/mod.rs`，但过于简陋（字符串 startsWith）。需要改为结构化匹配。

## 4. 应用流程

### 4.1 eval 阶段

```
evaluate(stmts, resolver) {
    let mut store = ExtensionStore::new();
    // Phase 1: 收集 extends + 生成 CSS rules
    let rules = eval_stmts(stmts, &mut store, resolver)?;
    // Phase 2: 应用 extends 到 CSS rules
    let extended_rules = store.apply_to_rules(&rules)?;
    Ok(CssTree { rules: extended_rules, extends: vec![] })
}
```

### 4.2 apply_to_rules 流程

```
apply_to_rules(rules):
    for rule in rules:
        match rule:
            Style { selector, declarations, nested }:
                new_selector = apply_extends_to_selector(selector, store)
                new_nested = apply_to_rules(nested)
                yield Style { new_selector, declarations, new_nested }
            AtRule { name, value, body }:
                // @media 内的 extend 有独立作用域
                new_body = apply_to_rules(body)
                yield AtRule { name, value, new_body }
            _ => yield rule
```

### 4.3 apply_extends_to_selector 流程

```
apply_extends_to_selector(selector_str, store):
    original_list = parse_selector_list(selector_str)
    result_list = SelectorList::new()

    for complex in original_list.selectors:
        result_list.add(complex)  // 保留原选择器

        // 查找匹配此 complex 的所有 extensions
        for (extendee, extensions) in store.extensions:
            for ext in extensions:
                // 模块作用域检查
                if !store.is_reachable(ext.module_id, rule_module_id):
                    continue

                // 尝试统一
                for ext_complex in ext.extender.selectors:
                    unified = unify_complex(complex, extendee, ext_complex)
                    if let Some(u) = unified:
                        // 传递性：递归应用其他 extends 到新产生的选择器
                        result_list.add(u)

    // 冗余消除
    remove_redundant(&mut result_list)
    // placeholder 移除
    remove_placeholders(&mut result_list)

    result_list
```

## 5. Tracing Span 设计

遵循项目规则：跨函数/跨阶段管道必须用 tracing span。

```rust
// eval 层入口
let span = tracing::info_span!("extend_collect",
    stage = "eval",
    module = "extend",
    module_id = current_module_id,
    selector = %selector
);

// apply 阶段入口
let span = tracing::info_span!("extend_apply",
    stage = "eval",
    module = "extend",
    rule_count = rules.len(),
    extend_count = store.extensions.len(),
);

// 单条 extend 应用
let span = tracing::debug_span!("extend_unify",
    stage = "extend",
    module = "unify",
    extendee = %extendee,
    extender = %extender,
);

// 传递性解析
let span = tracing::debug_span!("extend_transitive",
    stage = "extend",
    module = "transitive",
    start = %start_selector,
);

// 冗余消除
let span = tracing::trace_span!("extend_dedup",
    stage = "extend",
    module = "dedup",
    before = list.len(),
    after = deduped.len(),
);
```

## 6. 文件拆分

按 400 行限制：

```
src/selector/extend/
    mod.rs          (≈150 行) ExtensionStore + 公共接口 + apply_to_rules
    unify.rs        (≈350 行) compound/complex 统一 + weave
    transitive.rs   (≈200 行) 传递性 BFS + 循环检测
    merge.rs        (≈200 行) 列表合并 + 冗余消除 + placeholder 移除
```

旧 `src/selector/extend.rs` 删除，替换为 `src/selector/extend/` 目录。

## 7. 渐进式实现计划

| Phase | 目标 | 通过的 spec 测试 |
|-------|------|------------------|
| 1 | ExtensionStore 基础 + eval_use_rule 传递 extends + serialize 调用 selector/extend | `upstream/near`, `upstream/placeholder`, `directives/extend/after_target` |
| 2 | 传递性解析 + 循环检测 | `extended/from_same_file`, `extended/from_other_file`, `diamond/dependency`, `180_basic_extend_loop` |
| 3 | Compound/Complex 统一 + weave | `nested-compound-unification`, `046`-`077` (unification 系列) |
| 4 | 模块作用域隔离 | `scope/sibling`, `scope/diamond`, `scope/downstream`, `scope/private` |
| 5 | :is() 穿透 + 冗余消除 + placeholder 移除 | `pseudo.hrx`, `091_redundant_selector_elimination`, `187_basic_placeholder` |
