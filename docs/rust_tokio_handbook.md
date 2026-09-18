# Rust std + Tokio 开发手册

> 基于 docs.rs 编写的 Rust 标准库与 Tokio 参考手册,用于辅助 sasspile-rx 编译器开发。

---

## 目录

1. [Rust std 核心](#1-rust-std-核心)
2. [Tokio 运行时](#2-tokio-运行时)
3. [Tokio 同步原语](#3-tokio-同步原语)
4. [Tokio 任务管理](#4-tokio-任务管理)
5. [Tokio I/O](#5-tokio-io)
6. [重要注意事项](#6-重要注意事项)

---

## 1. Rust std 核心

### 1.1 模块概览

| 模块 | 功能 |
|------|------|
| `std::thread` | 线程抽象 |
| `std::sync` | 共享内存原语 (atomic, mpmc, mpsc) |
| `std::sync::atomic` | 原子类型 |
| `std::sync::mpsc` | 消息传递通道 |
| `std::io` | I/O 基础类型 |
| `std::fs` | 文件系统操作 |
| `std::net` | 网络 (TCP, UDP) |
| `std::collections` | 集合 (HashMap 等) |

### 1.2 并发原语

#### Arc (原子引用计数)
```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);
let data2 = Arc::clone(&data);  // 增加引用计数
```

#### Mutex (互斥锁)
```rust
use std::sync::Mutex;

let m = Mutex::new(0);
{
    let mut guard = m.lock().unwrap();
    *guard += 1;
}  // guard 离开作用域自动解锁
```

#### RwLock (读写锁)
```rust
use std::sync::RwLock;

let lock = RwLock::new(5);
// 多个读锁
let r1 = lock.read().unwrap();
let r2 = lock.read().unwrap();
// 单个写锁
let mut w = lock.write().unwrap();
*w += 1;
```

#### mpsc (多生产者单消费者通道)
```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::channel();
tx.send(42).unwrap();
let val = rx.recv().unwrap();  // 阻塞等待
let val = rx.try_recv();       // 非阻塞尝试接收
```

### 1.3 原子类型

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

let counter = AtomicUsize::new(0);
counter.fetch_add(1, Ordering::SeqCst);
let val = counter.load(Ordering::SeqCst);
```

**内存序**:
- `Relaxed`: 无同步约束
- `Acquire`: 读操作,保证之后的读/写不重排到此之前
- `Release`: 写操作,保证之前的读/写不重排到此之后
- `AcqRel`: Acquire + Release
- `SeqCst`: 顺序一致性(最强)

---

## 2. Tokio 运行时

### 2.1 简介

Tokio 是事件驱动、非阻塞 I/O 的异步运行时平台。核心组件:
- **Tasks**: 轻量级非阻塞执行单元
- **sync**: 同步原语和通道
- **net**: 异步 TCP/UDP
- **fs**: 异步文件系统
- **runtime**: 任务调度器 + I/O 驱动 + 高性能定时器

### 2.2 Feature Flags

| Flag | 功能 |
|------|------|
| `full` | 启用所有功能 |
| `rt` | 启用 spawn 和 current-thread 调度器 |
| `rt-multi-thread` | 启用多线程 work-stealing 调度器 |
| `sync` | 启用 tokio::sync |
| `time` | 启用 tokio::time |
| `net` | 启用 tokio::net |
| `fs` | 启用 tokio::fs |
| `signal` | 启用 tokio::signal |
| `process` | 启用 tokio::process |
| `macros` | 启用 #[tokio::main] 和 #[tokio::test] |

### 2.3 运行时类型

#### Current-Thread 调度器 (单线程)
```rust
#[tokio::main(flavor = "current_thread")]
async fn main() { }
```

#### Multi-Thread 调度器 (默认多线程)
```rust
#[tokio::main]
async fn main() { }

// 或自定义
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() { }
```

### 2.4 阻塞线程池

Tokio 有两种线程:
- **Core threads**: 运行异步代码,默认每个 CPU 核心一个
- **Blocking threads**: 运行阻塞代码,按需创建

```rust
// 在阻塞线程上执行
tokio::task::spawn_blocking(|| {
    // 阻塞操作
}).await?;
```

---

## 3. Tokio 同步原语

### 3.1 通道 (Channels)

#### oneshot (单生产者单消费者,单个值)
```rust
use tokio::sync::oneshot;

let (tx, rx) = oneshot::channel::<String>();

tokio::spawn(async move {
    tx.send("result".to_string()).unwrap();
});

let result = rx.await.unwrap();
```

#### mpsc (多生产者单消费者,多个值)
```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel::<String>(100);  // 容量 100

tokio::spawn(async move {
    for i in 0..10 {
        tx.send(format!("message {}", i)).await.unwrap();
    }
});

while let Some(msg) = rx.recv().await {
    println!("got: {}", msg);
}
```

**方法**:
- `send(value).await`: 异步发送
- `recv().await`: 异步接收,返回 Option<T>
- `try_recv()`: 非阻塞接收
- `close()`: 关闭通道
- `closed().await`: 等待所有发送者断开

#### watch (单生产者多消费者,最新值)
```rust
use tokio::sync::watch;

let (tx, mut rx) = watch::channel("initial");

tx.send("updated").unwrap();
let val = rx.borrow().clone();  // "updated"
```

#### broadcast (多生产者多消费者)
```rust
use tokio::sync::broadcast;

let (tx, mut rx1) = broadcast::channel::<String>(16);
let mut rx2 = tx.subscribe();

tx.send("hello".to_string()).unwrap();
let val1 = rx1.recv().await.unwrap();
let val2 = rx2.recv().await.unwrap();
```

### 3.2 Mutex (异步互斥锁)

```rust
use tokio::sync::Mutex;

let m = Mutex::new(0);
{
    let mut guard = m.lock().await;
    *guard += 1;
}  // 自动解锁
```

**与 std::Mutex 的区别**:
- `tokio::sync::Mutex`: 异步锁,`.lock().await` 期间可以 yield
- `std::sync::Mutex`: 阻塞锁,会阻塞整个线程

### 3.3 Notify

```rust
use tokio::sync::Notify;

let notify = Notify::new();

// 通知一个等待者
notify.notify_one();

// 通知所有等待者
notify.notify_waiters();

// 等待通知
notify.notified().await;
```

### 3.4 Barrier

```rust
use tokio::sync::Barrier;

let barrier = Barrier::new(3);  // 等待 3 个任务

// 每个任务调用
barrier.wait().await;  // 第 3 个调用返回后所有任务继续
```

### 3.5 Semaphore

```rust
use tokio::sync::Semaphore;

let sem = Semaphore::new(3);  // 最多 3 个并发

let permit = sem.acquire().await?;
// 使用资源
drop(permit);  // 释放许可
```

---

## 4. Tokio 任务管理

### 4.1 spawn

```rust
use tokio::task;

let handle: JoinHandle<String> = task::spawn(async {
    "result".to_string()
});

let result = handle.await.unwrap();
```

### 4.2 JoinHandle

```rust
// 等待完成
let result = handle.await;

// 取消任务
handle.abort();

// 检查是否完成
handle.is_finished();

// 等待完成或超时
tokio::time::timeout(Duration::from_secs(5), handle).await;
```

### 4.3 spawn_local (非 Send future)

```rust
use tokio::task::LocalSet;

let local = LocalSet::new();

local.run_until(async {
    tokio::task::spawn_local(async {
        // 非 Send future
    });
}).await;
```

### 4.4 yield_now

```rust
use tokio::task;

async {
    task::yield_now().await;  // 让出执行权
};
```

### 4.5 block_in_place

```rust
use tokio::task;

let result = task::block_in_place(|| {
    // 阻塞操作,将当前 worker 线程转为阻塞线程
    "blocking completed"
});
```

### 4.6 JoinSet

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();

set.spawn(async { 1 });
set.spawn(async { 2 });

while let Some(res) = set.join_next().await {
    let val = res.unwrap();
}
```

---

## 5. Tokio I/O

### 5.1 异步 I/O 特性

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// 异步读取
let mut buf = [0u8; 1024];
let n = socket.read(&mut buf).await?;

// 异步写入
socket.write_all(b"data").await?;
```

### 5.2 TCP

```rust
use tokio::net::{TcpListener, TcpStream};

// 服务端
let listener = TcpListener::bind("127.0.0.1:8080").await?;
loop {
    let (mut socket, _) = listener.accept().await?;
    tokio::spawn(async move {
        // 处理连接
    });
}

// 客户端
let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
```

### 5.3 UDP

```rust
use tokio::net::UdpSocket;

let socket = UdpSocket::bind("0.0.0.0:8080").await?;
let mut buf = [0u8; 1024];
let (len, addr) = socket.recv_from(&mut buf).await?;
socket.send_to(&buf[..len], addr).await?;
```

### 5.4 文件系统

```rust
use tokio::fs;

// 读取文件
let content = fs::read("file.txt").await?;
let content = fs::read_to_string("file.txt").await?;

// 写入文件
fs::write("file.txt", b"data").await?;
```

### 5.5 信号

```rust
use tokio::signal;

// Ctrl+C
signal::ctrl_c().await?;
```

---

## 6. 重要注意事项

### 6.1 rxrust 项目规范

根据项目规则:

1. **禁止手写 mpsc**: 使用 rxrust 的 `from_stream` 等原生方法
2. **禁止使用 tokio 运行时手动管理**: rxrust 内置调度器
3. **使用 rxrust Shared 上下文**: 用于多线程调度
4. **异步流处理**: 使用 `from_stream` + `collect`/`last`

### 6.2 std 与 tokio 选择原则

| 场景 | 选择 | 原因 |
|------|------|------|
| 与 rxrust 集成 | rxrust 原生 | 避免额外运行时开销 |
| 独立异步应用 | tokio | 完整异步生态 |
| 同步代码 | std | 简单直接 |

### 6.3 Send + Sync 要求

```rust
// Send: 可以在线程间转移所有权
// Sync: 可以在线程间共享引用 (&T 是 Send)

// tokio::spawn 要求 Future: Send
tokio::spawn(async { /* Send */ });

// tokio::task::spawn_local 不要求 Send
tokio::task::spawn_local(async { /* !Send OK */ });
```

### 6.4 常见模式

#### 任务间通信
```rust
// 使用 oneshot 获取一次性结果
let (tx, rx) = oneshot::channel();
tokio::spawn(async move {
    let result = compute().await;
    tx.send(result).unwrap();
});
let result = rx.await.unwrap();
```

#### 资源管理
```rust
// 使用 mpsc 管理共享资源
let (cmd_tx, mut cmd_rx) = mpsc::channel::<Command>(100);

// 资源管理任务
tokio::spawn(async move {
    let mut resource = Resource::new();
    while let Some(cmd) = cmd_rx.recv().await {
        resource.handle(cmd);
    }
});
```

#### 取消模式
```rust
let handle = tokio::spawn(async { /* work */ });
handle.abort();  // 请求取消
// handle.await 返回 Err(Cancelled)
```

### 6.5 调试建议

```rust
// 使用 #[tokio::test] 测试异步代码
#[tokio::test]
async fn test_something() {
    // 测试代码
}

// 使用 RUST_LOG 控制日志
// RUST_LOG=trace cargo test
```

---

## 附录: 速查表

### 常用 tokio::sync 类型

| 类型 | 用途 | 方法 |
|------|------|------|
| `oneshot::channel` | 单次消息 | `send`, `await` |
| `mpsc::channel` | 流式消息 | `send`, `recv`, `try_recv` |
| `watch::channel` | 状态广播 | `send`, `borrow`, `changed` |
| `broadcast::channel` | 多播 | `send`, `recv`, `subscribe` |
| `Mutex` | 异步互斥 | `lock`, `try_lock` |
| `Notify` | 事件通知 | `notify_one`, `notified` |
| `Barrier` | 同步点 | `wait` |
| `Semaphore` | 限流 | `acquire`, `release` |

### 常用 tokio::task 函数

| 函数 | 用途 |
|------|------|
| `spawn` | 创建异步任务 |
| `spawn_blocking` | 创建阻塞任务 |
| `spawn_local` | 创建本地任务 |
| `block_in_place` | 当前线程执行阻塞代码 |
| `yield_now` | 让出执行权 |

---

*文档版本: Tokio 1.53.1, Rust std 1.97*
*生成时间: 2026-09-18*
