# 增量编译（待开发）

## 职责

实现文件监听、依赖图追踪、变更传播和缓存，支持热重载。

## 计划文件结构

```
incremental/
├── mod.rs           # 入口
├── env.rs           # 响应式环境
├── depgraph.rs      # 依赖图
├── cache.rs         # 缓存层
└── propagate.rs     # 变更传播
```

## 响应式环境

```rust
pub struct ReactiveEnv {
    vars: watch::Sender<Map<String, Value>>,
}

impl ReactiveEnv {
    pub fn subscribe(&self) -> watch::Receiver<Map<String, Value>> {
        self.vars.subscribe()
    }

    pub fn set_var(&self, name: &str, value: Value) {
        self.vars.send_modify(|map| { map.insert(name.into(), value); });
    }
}
```

## 依赖图

- 变量 → 使用它的表达式映射
- 文件 → 导出变量映射
- 拓扑排序确定编译顺序

## 缓存策略

- `moka` crate 用于 LRU 缓存
- 基于 SourceSpan 的缓存键
- 输入内容哈希（快速变更检测）

## 防抖策略

- `debounce_ms: u64`（默认 200ms）
- `CompileMode::Watch | CompileMode::Ci`（CI 禁用防抖）
- fsnotify 事件 + sleep 实现

## 变更传播

1. 文件变更检测
2. 读取内容
3. 哈希比较
4. 标记脏节点
5. 沿依赖图传播
6. 重编译受影响模块

## 测试重点

- 变量变更自动传播
- 依赖图正确性
- 缓存命中/失效
- 防抖行为
