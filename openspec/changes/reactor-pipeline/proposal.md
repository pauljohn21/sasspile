# Proposal: Reactor Pipeline — 全管线函数式链式调用

> **一句话**: 将 sasspile 从 "God-object + 隐式共享状态" 重构为 **Reactor 模式** — 整个编译管线作为纯函数状态传递链, 消除所有 &mut 和隐式 IO。

## 背景

当前 sasspile 在 **管线表层** 有不错的链式 API:
```rust
Source::new(input)?.lex()?.parse()?.evaluate()?.serialize(style).into_string()
```

但每个阶段内部存在严重不一致:

| 阶段 | 表面 | 内核 | 问题 |
|------|------|------|------|
| `lex()` | 类型状态机 | ✅ 纯函数 | 无 |
| `parse()` | 类型状态机 | ✅ 纯函数 | 无 |
| `evaluate()` | 方法链 | ❌ God-object + &mut Env + 隐式 IO | **痛点核心** |
| `serialize()` | 类型状态机 | ⚠️ 580 行 if-else 链 | 代码可读性差 |

`evaluate()` 内部违反了 AGENTS.md 的核心哲学 (规则 7.1-7.5):
- `Env` 通过 `Rc<RefCell<Scope>>` 隐藏了 shared mutable state
- `&mut Env` 参数广泛存在
- File IO (`std::fs::read`) 是隐式副作用
- Module cache mutation 没有追踪
- 每个 `eval_xxx` 函数内部是过程式累加器

**根本问题**: 当前 "链式 API" 只是语法糖, 不是真正的函数式反应链。数据不在管线中**流动**, 而是在 God-object 中被**反复修改**。

## 目标

引入 **Reactor** 类型作为编译世界的完整显式快照, 每个编译阶段消费旧的 Reactor, 产生新的 Reactor:

```rust
let css = Reactor::new(source)
    .with_load_paths(paths)
    .lex()?              // Reactor → Reactor<Lexed>
    .parse()?            // Reactor<Lexed> → Reactor<Parsed>
    .evaluate()?         // Reactor<Parsed> → Reactor<Evaluated>  ← 核心变革
    .serialize(style)    // Reactor<Evaluated> → Reactor<Serialized>
    .finish()?;          // Reactor<Serialized> → String
```

### 核心不变式

1. **Reactor 是数据, 不是状态** — 每个函数消费旧值, 返回新值
2. **IO 通过 Reactor 显式化** — 无隐式 `std::fs::read`, 全部通过 `Reactor::read_file` mockable 路径
3. **无 &mut self** — 全部 `self` (move) 或 `&self` (只读查询)
4. **Env 线程化** — 作用域进出通过返回新 Env, 而非 mutation

## 范围

### In Scope
- `Reactor` 核心类型定义
- Env/Scope 改造为消费-返回模式
- `eval_*` 全套函数改为纯函数签名 `fn(reactor, node) -> Result<(Reactor, Vec<CssNode>)>`
- File IO / Module cache / Importer 通过 Reactor trait 抽象
- Color 操作统一为 `Color` trait method (附带收益: `$space`/`to_gamut` 自然支持)
- 主编务: 新 Reactor 路径与旧 API 并存, 逐步迁移

### Out of Scope
- Parser/Lexer 内部实现 (已经是纯函数, 不需要改)
- CSS 序列化内部实现 (可以后续单独提案)
- 性能优化 (保持当前零成本抽象水平即可)
- AST 结构变更
- 公开 API 破坏性变更 (保持 `Source::lex().parse()...` 可用)

## 动机 (为什么是现在)

1. **sass-spec 推进受阻**: 剩余 5000+ failures 中, 大量与 `$space` / `@import` / 作用域泄露相关, 根本原因是 evaluator 内部 state 不可预测
2. **并发潜力**: Reactor 模式天然支持并行求值 (无畏并发)
3. **测试性**: 任何一段 eval 逻辑都可以通过 `Reactor::mock()` 单测, 无需编译管线
4. **代码健康度**: 当前 `color_adjust.rs` (637行) + `color_adjust_cie.rs` (243行) + `color_hwb_hsl.rs` (590行) 共 ~1500 行 God-function, 重构后预计 ~600 行
5. **符合项目哲学**: AGENTS.md 已确立函数式为核心, 但执行不彻底

## 风险与缓解

| 风险 | 缓解 |
|------|------|
| Reactor struct 变大, 传递开销 | Rust move 语义 + Copy elision, 零成本抽象 |
| 改不动 ~2000 行 God-object evaluator | 分阶段迁移: Env → Color → 核心 eval, 每个阶段独立验证 |
| 性能退化 | 先做 benchmark, 确认关键路径无回归 |
| 学习曲线陡 | try_fold + 类型签名强制, IDE 辅助可发现 |
| IO mock 复杂度 | 用 trait object + 默认实现, 生产代码无需关心 |

## 成功的标准

- [ ] `evaluate()` 入口不再包含任何 `&mut self` 方法
- [ ] 所有 `eval_*` 函数签名统一为纯函数形式
- [ ] `Reactor` 可被 mock (file IO / module cache / env 可注入)
- [ ] sass-spec 通过率不降低 (作为迁移完成的硬性标准)
- [ ] Color 操作具备 `color.scale(&kw)?.to_space(lab).to_gamut()` 链式 API
