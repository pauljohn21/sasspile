## Why

Rivulet 是一个受 sasspile 启发的纯 Rust 函数式 FRP Web 框架。M0 是整个框架的地基——Arena-based FRP 核心抽象（Event/Behavior/Runtime），所有后续里程碑（Widget、Diff、Renderer、路由、小程序）都依赖此核心。需要先实现核心 FRP 逻辑，才能构建上层 UI 系统。

## What Changes

- 新建 `rivulet` 仓库和 workspace（位于 `/Users/pauljohn/rust/rivulet/`）
- 新建 `rivulet-core` crate——Arena-based FRP 核心库
- 实现 `Event<T>` 离散事件流类型，支持 `map`/`filter`/`merge`/`sample` 组合子
- 实现 `Behavior<T>` 连续时变值类型，支持 `now`/`map`/`accumulate` 组合子
- 实现 `Runtime` 统一 Arena + 事件队列 + 依赖图推送引擎
- 实现 `Emitter<T>` 事件触发器，用于 DOM 回调中触发事件
- 实现 `Context` 全局状态传递（provide/use_context）
- 所有组合子纯函数式——返回新句柄，不修改原始流
- 状态唯一来源是 `Behavior::accumulate`——无可变 signal

## Capabilities

### New Capabilities

- `frp-core`: Arena-based FRP 核心——Event/Behavior 类型、组合子、Runtime 推送引擎、Emitter、Context

### Modified Capabilities

（无——全新项目，无现有 spec 需要修改）

## Impact

- **新仓库**：`/Users/pauljohn/rust/rivulet/`（全新，与 sasspile/banyan 独立）
- **新 crate**：`rivulet-core`（依赖 slotmap、smallvec、tracing）
- **无对现有项目的影响**：完全独立的全新仓库
- **后续依赖**：M1（Widget + Diff + Builder）依赖此 M0 核心抽象
