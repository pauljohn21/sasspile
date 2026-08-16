# 语义分析 ✅ 已完成

## 职责

构建作用域、解析模块依赖、验证 @extend 目标、收集函数/混入定义。

## 文件结构（实际）

```
semantic/
├── mod.rs             # 分析入口 + re-exports
├── symbol_table.rs    # 作用域栈 (Scope/ScopeKind/SymbolEntry)
├── module.rs          # ModuleGraph/CycleCheck/NamespaceRegistry
├── extend.rs          # @extend 选择器收集 (SelectorRegistry)
└── definitions.rs     # 定义收集 (DefinitionRegistry/MixinEntry/FunctionEntry)
```

## 符号表

**文件: `sasspile/src/semantic/symbol_table.rs`**

```rust
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

pub enum ScopeKind {
    Global,
    Local,
    Param,  // 函数/混入参数
}

pub struct SymbolEntry {
    pub name: String,
    pub kind: SymbolKind,
    pub span: SourceSpan,
}

pub enum SymbolKind {
    Variable(Value),
    Function(FunctionDef),
    Mixin(MixinDef),
    Placeholder(Selector),
}
```

## 模块图

**文件: `sasspile/src/semantic/module.rs`**

```rust
pub struct ModuleGraph {
    modules: HashMap<String, Module>,
    edges: Vec<(String, String)>,
}

pub struct Module {
    pub exports: SymbolTable,
    pub spans: HashMap<String, SourceSpan>,
}

pub struct NamespaceRegistry { /* ... */ }
pub struct CycleCheck { /* ... */ }
```

## @extend 注册表

**文件: `sasspile/src/semantic/extend.rs`**

```rust
pub struct SelectorRegistry {
    selectors: HashMap<String, Vec<ExtendSource>>,
}

pub fn collect_extends(ast: &Stylesheet) -> SelectorRegistry;
```

## 定义收集

**文件: `sasspile/src/semantic/definitions.rs`**

```rust
pub struct DefinitionRegistry /* MixinEntry, FunctionEntry, DuplicateInfo */;

pub enum DefinitionKind {
    Function,
    Mixin,
}
```

## 使用方式

```rust
use sasspile::semantic::{SymbolTable, DefinitionRegistry, collect_extends};

let table = SymbolTable::new();
table.push_scope(ScopeKind::Global);
// ... 遍历 AST 填充符号表

let registry = DefinitionRegistry::new();
// ... 收集 mixin/function 定义
```

## 测试

- `tests/symbol_table_spec.rs`
- `tests/definitions_spec.rs`
- `tests/extend_spec.rs`
- `tests/module_spec.rs`
