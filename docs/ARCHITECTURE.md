# scss-rs 架构设计

## 1. 核心设计原则

### 1.1 类型状态机管线

源码流过管线，每个阶段是一个新类型，阶段转换是 `impl From<Previous> for Next`：

```
Source ──► Lexed ──► Parsed ──► Evaluated ──► Serialized
  │           │          │           │             │
  │ text      │ tokens   │ ast       │ Vec<CssNode> │ String
  │ base_path │          │ base_path │              │
  │ load_paths│          │ load_paths│              │
```

每个阶段类型**只携带该阶段需要的数据**，不透传无关字段。

### 1.2 零 clone 求值

`Env` 使用 `move` 语义：
- 所有 `eval_*` 方法签名：`fn eval_xxx(node, env: Env) -> Result<(Output, Env)>`
- `Env` 方法全部 `self -> Self`（builder 模式）
- `enter_scope(&self) -> Env`：创建子作用域（克隆 local 成员，共享 Rc 字段）
- `exit_scope(self, child: Env) -> Env`：从子作用域提取传播字段

### 1.3 内建函数注册：const 静态表

**不用 proc-macro**。用 `const` 静态数组作为单一数据源：

```rust
struct BuiltinEntry {
    module: &'static str,       // "math"
    field: &'static str,        // "is_unitless"（字段名，snake_case）
    global: &'static str,       // "is-unitless"（kebab-case 全局名）
    aliases: &'static [&'static str],  // ["math.is-unitless"]
}
```

编译期从 `field` 通过 `const fn snake_to_kebab` 生成 `global`，
确保两者永远一致。如拼错，const fn 在编译期失败。

### 1.4 文件组织：按功能拆分

每个文件 ≤ 300 行（业务逻辑）/ 500 行（类型定义）。
按功能边界拆分，不按行数凑。

## 2. 模块边界

```
src/
├── lib.rs              // 公开 API + 管线入口
├── error.rs            // SassError + Result
├── source.rs           // Source 类型 + from_file + from_str
│
├── lex/
│   ├── mod.rs          // Lexer 迭代器 + Token 定义
│   └── scan.rs         // 扫描逻辑（scan_ident, scan_string, scan_number...）
│
├── parse/
│   ├── mod.rs          // Parser + parse() 入口
│   ├── ast.rs          // Node 枚举定义
│   ├── rule.rs         // parse_rule / parse_selector
│   ├── decl.rs         // parse_decl / parse_property / check_important
│   ├── variable.rs     // parse_variable / parse_var_flags
│   ├── body.rs         // parse_body / parse_params / parse_args
│   ├── at_rules.rs     // @use / @forward / @import / @include / @mixin
│   └── expr.rs         // 表达式解析 (prefix/infix)
│
├── eval/
│   ├── mod.rs          // Evaluator + eval_nodes + eval_node 分发 + Evaluated::try_from（调用 apply_extends + hoist_css_imports）
│   ├── env.rs          // Env 定义 + builder + enter_scope/exit_scope + ModuleExports
│   ├── rule.rs         // eval_rule + nest_rule_in_children（逗号选择器笛卡尔积嵌套）
│   ├── mixin.rs        // exec_mixin + bind_params
│   ├── function.rs     // call_function + call_user_function
│   ├── module.rs       // load_module / load_import / eval_use / eval_forward
│   ├── file_resolver.rs // resolve_file + check_resolve_ambiguity（四种冲突检测）
│   ├── control.rs      // eval_if / eval_for / eval_each / eval_while
│   ├── extend.rs       // apply_extends + check_extend_targets
│   ├── plain_css.rs    // hoist_css_imports
│   ├── value/
│   │   ├── mod.rs      // Value 枚举 + Color + equals + parse_hex_color
│   │   ├── display.rs  // to_css_string + ColorFormat + color_to_css + rgb_to_hsl/hwb
│   │   └── ops.rs      // add/sub/mul/div/rem/neg
│   └── builtin/
│       ├── mod.rs      // call_builtin 入口
│       ├── dispatch.rs // const 静态表 + module_builtin_name + is_known_builtin + dispatch_builtin
│       ├── math.rs     // abs/ceil/floor/round/max/min/div/clamp/hypot/sqrt/pow/log/sin/cos/tan/asin/acos/atan/atan2/unit/is_unitless/percentage/random
│       ├── string.rs   // length/quote/unquote/to_upper/to_lower/index/insert/slice/split
│       ├── map.rs      // get/merge/remove/keys/values/has_key/deep_merge/deep_remove
│       ├── list.rs     // length/nth/set_nth/join/append/zip/index/is_bracketed/separator/slash
│       ├── color.rs    // mix/adjust/change/scale（骨架）
│       ├── meta.rs     // call/type_of/inspect/feature_exists/function_exists/mixin_exists/variable_exists/get_function/get_mixin/module_functions/module_variables/load_css/content_exists/keywords
│       └── selector.rs  // nest/append/parse/is_super_selector
│
├── css/
│   ├── mod.rs          // Serializer + serialize 入口
│   ├── node.rs         // CssNode 枚举
│   ├── transform.rs   // flatten / merge / hoist（纯函数 Vec→Vec）
│   └── serialize.rs   // serialize_expanded / serialize_compressed
│
└── tracing.rs          // 初始化 + re-export
```

## 3. 数据流

### 3.1 字符串编译
```
Source::new(input)
    .lex()?
    .parse()?
    .evaluate()?
    .serialize(style)
    .into_string()
```

### 3.2 文件编译
```
Source::from_file(path)?
    .with_load_paths(paths)
    .lex()?
    .parse()?
    .evaluate()?
    .serialize(style)
    .into_string()
```

### 3.3 后处理链（纯函数）
```
let (mut css, final_env) = eval_nodes(ast, env)?;  // (Vec<CssNode>, Env)
let extends = final_env.get_extends().to_vec();
apply_extends(&mut css, &extends);     // 选择器匹配 + 替换
check_extend_targets(&css, &extends)?; // 验证非 optional extend
hoist_css_imports(&mut css);          // CSS @import 提升到顶部
```

## 4. Env 设计

```rust
pub struct Env {
    // 可变状态（enter_scope 时克隆）
    local_vars: HashMap<String, Value>,
    local_mixins: HashMap<String, MixinDef>,
    local_functions: HashMap<String, FunctionDef>,
    forwarded_vars: HashMap<String, Value>,
    forwarded_mixins: HashMap<String, MixinDef>,
    forwarded_functions: HashMap<String, FunctionDef>,
    global_writes: HashMap<String, Value>,
    
    // 共享状态（Rc 引用计数，enter_scope 时 clone Rc）
    content: Option<Rc<Vec<Node>>>,
    content_env: Option<Rc<Env>>,
    namespaces: HashMap<String, Rc<ModuleExports>>,
    extends: Rc<Vec<(String, String, bool)>>,
    loaded_modules: Rc<HashSet<PathBuf>>,
    module_cache: Rc<HashMap<PathBuf, ModuleExports>>,
    
    // 管线配置
    base_path: Option<PathBuf>,
    load_paths: Vec<PathBuf>,
    current_selector: Option<String>,
    depth: usize,
    plain_css: bool,
    pending_config: HashMap<String, Value>,
    builtin_modules: Vec<String>,
}
```

Builder 方法命名规范：
- `with_xxx(xxx) -> Self` — 覆盖字段
- `add_xxx(xxx) -> Self` — 追加到集合
- `define_xxx(name, val) -> Self` — 插入到 map
- `get_xxx() -> &T` — 只读访问

## 5. 内建函数 dispatch 设计

```rust
// 单一数据源——编译期 const
const BUILTIN_TABLE: &[BuiltinEntry] = &[
    // math
    BuiltinEntry { module: "math", field: "abs",      global: "", aliases: &["math.abs"] },
    BuiltinEntry { module: "math", field: "div",      global: "", aliases: &["math.div"] },
    BuiltinEntry { module: "math", field: "is_unitless", global: "", aliases: &["math.is-unitless"] },
    // ... 每个字段一行
];

// 编译期验证：global 从 field 自动生成
const fn snake_to_kebab(s: &str) -> [u8; 64] { ... }

// 三个函数从同一张表生成
pub fn module_builtin_name(name: &str) -> &str { ... }
pub fn is_known_builtin(name: &str) -> bool { ... }
pub fn dispatch_builtin_module(...) -> Option<Result<Value>> { ... }
```

## 6. 测试策略

- `tests/` 目录，无 `#[cfg(test)]` 内联测试
- 每个管线阶段有独立测试文件
- sass-spec 集成测试从第一天就有
- 颜色测试单独跳过列表，防止无限修复循环

## 7. 测试基线（align-sasspile 归档后）

| 测试套件 | 通过/总数 | 通过率 |
|----------|-----------|--------|
| compile_test | 19/19 | 100% |
| lex_test | 29/29 | 100% |
| bs_spec | 15/15 | 100% |
| sass_spec | 1235/5362 | 23% |
| ep_full | 10/121 | 8% |

### sass-spec 各目录通过率

| 目录 | 通过/总数 | 通过率 |
|------|-----------|--------|
| variables | 3/3 | 100% |
| values | 141/1169 | 12% |
| css | 133/830 | 16% |
| operators | 6/30 | 20% |
| expressions | 64/212 | 30% |
| directives | 222/775 | 36% |
| core_functions | 660/2519 | 26% |
| parser | 1/4 | 25% |
| callable | 5/71 | 11% |
