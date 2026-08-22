## Context

sasspile 的 `Env` 使用单个扁平 HashMap 存储函数/mixin/变量定义，**不区分成员来源**。

### sass-spec 证据链

以下语义全部来自 sass-spec `.hrx` 文件的 input/output：

**1. `@forward` 成员在当前文件不可见（`inaccessible.hrx: local/`）**
```scss
// input.scss
@forward "other";   // other 定义了 $c: d
a {b: $c};           // → Error: Undefined variable.
```
→ `@forward` 只传递给下游，当前文件看不到 forwarded 成员

**2. local 定义遮蔽 forwarded 定义（`shadowed.hrx`）**
```scss
// _midstream.scss
@forward "upstream";   // upstream 定义 $c: upstream
$c: midstream;          // local 定义

// input.scss
@use "midstream";
a {b: midstream.$c}     // → output: b: midstream;  (local 优先)
```
→ 模块导出时 local 优先于 forwarded，同名时 local 遮蔽 forwarded

**3. 同一模块 forward 两次不冲突（`bare.hrx: no_conflict/`）**
```scss
// _midstream.scss
@forward "upstream";
@forward "upstream";    // 同一模块，同名函数 → 不报错

// input.scss
@use "midstream";
a {b: midstream.c()}    // → output: b: d;
```
→ 同来源（同一路径）的同名成员幂等写入

**4. 不同模块同值也冲突（`conflict.hrx: same_value/`）**
```scss
// input.scss
@forward "other1";  // $a: b
@forward "other2";  // $a: b  ← 值相同！
// → Error: Two forwarded modules both define a variable named $a.
```
→ 冲突检测基于来源路径，不是值比较！不同来源即使值相同也报错

**5. `@use as *` 同一模块两次不冲突（`use/member/global.hrx: no_conflict/`）**
```scss
@use "other" as *;
@use "other" as *;   // → 不报错，a {b: $c} → b: d
```
→ `@use as *` 同来源幂等写入

**6. `@forward` 的 `show`/`hide` 过滤（`visibility.hrx`）**
```scss
@forward "upstream" hide b;     // 隐藏 b
@forward "upstream" show $c;    // 只显示 $c
```
→ forwarded 成员需要支持 show/hide 过滤

**7. `@forward as prefix-*` 前缀重映射（`as.hrx`）**
```scss
@forward "upstream" as d-*;   // $c → $d-c
```
→ forwarded 成员名加前缀后绑定

## Goals / Non-Goals

**Goals:**
- 将 `Env` 成员管理重构为 local + forwarded 双层结构
- 精确实现 sass-spec 的 7 条语义规则
- 修复 ep_full 10/121 → 101+/121
- 通过 sass-spec `no_conflict/`、`shadowed.hrx`、`inaccessible.hrx` 等测试

**Non-Goals:**
- 不修改 `@use` 命名空间模式的行为
- 不修改 `@import` 的内联语义
- 不修改内建模块处理

## Decisions

### 决策 1：Env 双层成员结构

将 `functions`/`mixins`/`vars` 各拆分为 `local_*` 和 `forwarded_*`：

```
Env {
    // —— local：当前文件定义 + @use as * 导入（当前文件可见）——
    local_vars: HashMap<String, Value>
    local_mixins: HashMap<String, MixinDef>
    local_functions: HashMap<String, FunctionDef>

    // —— forwarded：@forward 导出（当前文件不可见，只传递给下游）——
    forwarded_vars: HashMap<String, Value>
    forwarded_mixins: HashMap<String, MixinDef>
    forwarded_functions: HashMap<String, FunctionDef>

    // —— 其他字段保持不变 ——
    ...（与 design 前版本相同）
}
```

**spec 依据**：
- 规则 1：`@forward` 成员在当前文件不可见 → forwarded 表不参与 `get_function`/`lookup` 查找
- 规则 2：local 遮蔽 forwarded → 模块导出时 local 优先
- 规则 3/4：同来源不冲突、不同来源同值也冲突 → `bind_exports` 需追踪来源路径

### 决策 2：成员查找路径

```rust
fn get_function(&self, name: &str) -> Option<&FunctionDef> {
    self.local_functions.get(name)
    // 不查 forwarded——规则 1：forwarded 成员在当前文件不可见
}
fn lookup(&self, name: &str) -> Option<&Value> {
    self.local_vars.get(name)
    // 不查 forwarded
}
```

**spec 依据**：规则 1（`inaccessible.hrx: local/`）——`@forward "other"` 后 `$c` 在当前文件仍然 Undefined。

### 决策 3：bind_exports 重构

`bind_exports` 用于 `@use as *`（写入 local）和 `@forward`（写入 forwarded），需追踪来源路径以支持规则 3 和 4。

```
bind_exports(env, exports, prefix, mode, source_path):
    // mode = Use | Forward
    target_table = match mode {
        Use     => env.local_*
        Forward => env.forwarded_*
    }

    source_exports = match mode {
        Use     => exports.local_*（使用方取模块的 local 成员）
        Forward => exports.forwarded_* + exports.local_*（转发方传递 forwarded + local）
    }

    for each member (k, v) in source_exports:
        key = fmt_key(k, prefix)
        if target_table.contains(key):
            // 检查来源路径
            existing_source = env.member_sources.get(key)
            if existing_source == source_path:
                skip  ← 规则 3：同来源幂等
            else:
                error ← 规则 4：不同来源，即使值相同也报错
        else:
            target_table.insert(key, v)
            env.member_sources.insert(key, source_path)
```

**spec 依据**：
- 规则 3：同来源（同路径）不冲突
- 规则 4：不同来源即使同值也报错
- 规则 2：forward 传递时 local 优先——如果模块既有 local 又有 forwarded 同名成员，forward 传递 local 版本

### 决策 4：member_sources 追踪来源路径

```
Env {
    ...
    member_sources: Rc<HashMap<String, Rc<PathBuf>>>
    // key 格式: "fn:bem", "mx:scrollbar", "var:namespace"
}
```

`member_sources` 只在 `bind_exports` 中写入，记录每个成员来自哪个模块路径。

**spec 依据**：规则 3/4——区分"同一模块 forward 两次"和"两个不同模块 forward 同名"。

### 决策 5：ModuleExports 双层结构

```
ModuleExports {
    local_vars, local_mixins, local_functions     // 模块文件内部定义 + @use as * 导入
    forwarded_vars, forwarded_mixins, forwarded_functions  // @forward 导出
    css, loaded_modules, extends, module_cache
}
```

**spec 依据**：规则 2（`shadowed.hrx`）——模块导出时 local 优先于 forwarded，需要同时携带两者。`@use` 从模块取 local 成员；`@forward` 从模块取 forwarded + local（local 遮蔽 forwarded 后的合并结果）。

### 决策 6：`@forward` 传递逻辑

`eval_forward` 加载模块后，将该模块的 forwarded 成员 + local 成员（local 遮蔽 forwarded 后）写入当前文件的 forwarded 表：

```
eval_forward(url, prefix, config, env):
    exports = load_module(path)
    // 合并 local 和 forwarded：local 遮蔽 forwarded
    merged = merge_with_local_precedence(exports.local_*, exports.forwarded_*)
    // 写入当前 env 的 forwarded 表
    bind_exports(env, merged, prefix, mode=Forward, source_path=path)
```

**spec 依据**：规则 2（`shadowed.hrx`）——`midstream.scss` 的 `$c: midstream`（local）遮蔽了 upstream 的 `$c: upstream`（forwarded），下游 `midstream.$c` 返回 `midstream`。

### 决策 7：show/hide 过滤

`bind_exports` 在 Forward 模式时应用 show/hide 过滤：

```
if mode == Forward:
    if show 非空 and member not in show: skip
    if hide 非空 and member in hide: skip
```

**spec 依据**：规则 6（`visibility.hrx`）。

### 决策 8：`@use as *` 的查找路径

`@use as *` 从模块的 **local 成员**绑定到当前文件的 local 表：

```
eval_use(url, star=true):
    exports = load_module(path)
    bind_exports(env, exports.local_*, prefix=None, mode=Use, source_path=path)
```

**spec 依据**：规则 5（`use/member/global.hrx: no_conflict/`）——同来源幂等写入。规则 1——`@use as *` 的成员在当前文件可见（写入 local）。

### 决策 9：meta 反射函数

`meta.module-functions($module)` 返回命名空间模块的 local + forwarded 合并结果（local 优先）。

**spec 依据**：模块的所有可用成员 = local + forwarded（local 遮蔽 forwarded）。

## Risks / Trade-offs

- **风险**：`Env` clone 开销增大（6 个 HashMap + 1 个 member_sources）→ 缓解：当前正确性优先，后续可用 `Rc` 共享优化
- **风险**：`@forward` 传递时需要合并 local + forwarded → 缓解：写辅助函数 `merge_with_local_precedence`
- **风险**：`member_sources` 在规则体传播、`@import` 内联等场景需要正确传播 → 缓解：`@import` 继承全部环境（包括 member_sources），规则体传播保持 local 写入 local
