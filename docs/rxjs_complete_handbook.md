# RxJS 完整使用手册

> 基于 RxJS 7.x 官方文档编写,覆盖 Observable、Operator、Subject、Scheduler 全部核心概念。
> 与 rxrust 1.0.0-rc.5 共享同一响应式范式,概念一一对应。

---

## 目录

1. [核心概念](#1-核心概念)
2. [Observable](#2-observable)
3. [Observer](#3-observer)
4. [Subscription](#4-subscription)
5. [Operators](#5-operators)
6. [Subject](#6-subject) ⭐
7. [Scheduler](#7-scheduler)
8. [与 rxrust 对照表](#8-与-rxrust-对照表)

---

## 1. 核心概念

RxJS 的响应式编程基于三个核心角色:

```
┌─────────────┐     next(value)      ┌───────────┐
│  Observable  │ ──────────────────►  │  Observer  │
│  (生产者)    │     error(err)       │  (消费者)  │
│              │     complete()       │           │
└─────────────┘                      └───────────┘
       │                                   ▲
       │         Subscription              │
       │  ┌─────────────────────────┐      │
       └─ │ subscribe(observer)     │ ─────┘
          │ unsubscribe()           │
          └─────────────────────────┘
```

**三大核心:**
1. **Observable** — 可被观察的对象,负责产生数据流
2. **Observer** — 观察者,定义如何处理数据(next/error/complete)
3. **Subscription** — 订阅关系,用于取消订阅

---

## 2. Observable

### 2.1 什么是 Observable
Observable 是一个懒推送(Lazy Push)的集合,代表未来将要产生的值。

```typescript
import { Observable } from 'rxjs';

const observable = new Observable(subscriber => {
  subscriber.next(1);
  subscriber.next(2);
  subscriber.next(3);
  
  setTimeout(() => {
    subscriber.next(4);
    subscriber.complete();
  }, 1000);
});
```

### 2.2 创建方式
```typescript
import { of, from, fromEvent, interval, timer } from 'rxjs';

// 同步创建
of(1, 2, 3);                    // 发出 1, 2, 3 后完成
from([1, 2, 3]);               // 从数组/Promise/Iterable 创建
fromEvent(document, 'click');   // 从 DOM 事件创建

// 异步创建
interval(1000);                 // 每秒发出递增数字: 0, 1, 2, ...
timer(5000, 1000);             // 5秒后开始,每秒发出

// 从 Promise 创建
from(fetch('/api/data'));

// 自定义
new Observable(subscriber => {
  // 推送逻辑
});
```

### 2.3 多播 vs 单播
- **冷 Observable (Cold)**: 每个订阅者独立执行(默认行为)
- **热 Observable (Hot)**: 多个订阅者共享同一执行(Subject)

**rxrust 对应:**
- `Local::from_iter()` / `Shared::from_stream()` → Cold Observable
- `Subject` → Hot Observable

---

## 3. Observer

### 3.1 Observer 接口
```typescript
interface Observer<T> {
  next: (value: T) => void;       // 接收值
  error: (err: any) => void;      // 接收错误
  complete: () => void;           // 接收完成通知
}
```

### 3.2 使用方式
```typescript
observable.subscribe({
  next: (value) => console.log(value),
  error: (err) => console.error(err),
  complete: () => console.log('done')
});

// 简写法(只关心 next)
observable.subscribe(value => console.log(value));
```

**rxrust 对应:**
```rust
// rxrust  Observer trait
trait Observer<Item, Err> {
    fn next(&mut self, value: Item);
    fn error(self, err: Err);
    fn complete(self);
    fn is_closed(&self) -> bool;
}

// 使用
source.subscribe(|v| println!("{:?}", v));
```

---

## 4. Subscription

### 4.1 订阅与取消
```typescript
const subscription = observable.subscribe(console.log);

// 取消订阅(停止接收)
subscription.unsubscribe();
```

### 4.2 复合订阅
```typescript
const sub1 = observable1.subscribe();
const sub2 = observable2.subscribe();

// 合并管理
const combined = new Subscription();
combined.add(sub1);
combined.add(sub2);

// 一次性取消所有
combined.unsubscribe();
```

**rxrust 对应:**
```rust
let handle = source.subscribe(|v| { ... });
handle.unsubscribe();  // 取消
handle.await;          // 等待完成(TaskHandle)
```

---

## 5. Operators

### 5.1 管道(Pipe)机制
```typescript
import { filter, map, scan, take } from 'rxjs/operators';

observable.pipe(
  filter(x => x % 2 === 0),    // 过滤偶数
  map(x => x * 2),              // 乘以2
  scan((acc, x) => acc + x, 0), // 累加
  take(5)                       // 只取前5个
).subscribe(console.log);
```

**rxrust 对应:**
```rust
source
    .filter(|x| x % 2 == 0)
    .map(|x| x * 2)
    .scan_map(0, |acc, x| *acc += x)
    .take(5)
    .subscribe(|v| println!("{:?}", v));
```

### 5.2 操作符分类

#### 创建类(Creation)
| RxJS | rxrust | 功能 |
|------|--------|------|
| `of(1,2,3)` | `Local::of(1)` | 同步发出值 |
| `from([1,2])` | `Local::from_iter([1,2])` | 从迭代器 |
| `fromEvent(el, 'click')` | `Shared::from_stream(stream)` | 从事件流 |
| `interval(1000)` | `Shared::interval(Duration::from_secs(1))` | 定时发出 |
| `timer(5000)` | `Shared::timer(Duration::from_secs(5))` | 延迟发出 |
| `ajax('/api')` | 无内置 | HTTP 请求 |
| `throwError(err)` | `Shared::throw_err(err)` | 抛出错误 |

#### 转换类(Transformation)
| RxJS | rxrust | 功能 |
|------|--------|------|
| `map(x => x*2)` | `map(\|x\| x*2)` | 映射 |
| `mapTo(42)` | 无 | 映射为常量 |
| `scan((a,b)=>a+b, 0)` | `scan_map(0, \|a,b\| *a+=b)` | 累加 |
| `switchMap(x => req(x))` | 无内置 | 切换映射 |
| `mergeMap(x => from(x))` | `flat_map(\|x\| ...)` | 扁平映射 |
| `concatMap` | 无内置 | 连接映射 |
| `exhaustMap` | 无内置 | 耗尽映射 |
| `bufferCount(n)` | 无内置 | 缓冲 |
| `toArray()` | `collect::<Vec<_>>()` | 收集为数组 |

#### 过滤类(Filtering)
| RxJS | rxrust | 功能 |
|------|--------|------|
| `filter(x => x>5)` | `filter(\|x\| x>5)` | 过滤 |
| `take(n)` | `take(n)` | 取前n个 |
| `takeLast(n)` | 无内置 | 取后n个 |
| `takeWhile` | 无内置 | 条件取 |
| `takeUntil(notifier)` | 无内置 | 直到通知 |
| `skip(n)` | 无内置 | 跳过n个 |
| `skipWhile` | 无内置 | 条件跳过 |
| `distinct()` | 无内置 | 去重 |
| `distinctUntilChanged()` | 无内置 | 连续去重 |
| `debounceTime(ms)` | `debounce(Duration::from_millis(ms))` | 防抖 |
| `throttleTime(ms)` | `throttle(...)` | 节流 |
| `first()` | 无内置 | 首个 |
| `last()` | `last()` | 末尾 |
| `elementAt(n)` | 无内置 | 指定位置 |
| `ignoreElements()` | 无内置 | 忽略元素 |

#### 组合类(Combination)
| RxJS | rxrust | 功能 |
|------|--------|------|
| `concat(a, b)` | 无内置 | 顺序连接 |
| `merge(a, b)` | 无内置 | 合并 |
| `combineLatest([a,b])` | 无内置 | 最新组合 |
| `zip(a, b)` | 无内置 | 拉链组合 |
| `withLatestFrom` | 无内置 | 最新配对 |
| `startWith(0)` | 无内置 | 起始值 |
| `forkJoin([a,b])` | 无内置 | 等待完成 |

#### 多播类(Multicasting)
| RxJS | rxrust | 功能 |
|------|--------|------|
| `multicast(subject)` | 无内置 | 多播 |
| `share()` | 无内置 | 共享 |
| `shareReplay(n)` | 无内置 | 重放共享 |

#### 错误处理类(Error Handling)
| RxJS | rxrust | 功能 |
|------|--------|------|
| `catchError(err => of(0))` | `on_error(\|e\| {})` | 捕获错误 |
| `retry(n)` | 无内置 | 重试 |
| `retryWhen` | 无内置 | 条件重试 |
| `finalize(() => {})` | 无内置 | 最终执行 |

#### 条件与布尔类(Conditional)
| RxJS | rxrust | 功能 |
|------|--------|------|
| `defaultIfEmpty(0)` | 无内置 | 空默认值 |
| `every(x => x>0)` | 无内置 | 全满足 |
| `find(x => x>5)` | 无内置 | 查找首个 |
| `findIndex` | 无内置 | 查找索引 |
| `isEmpty()` | 无内置 | 是否为空 |

#### 数学与聚合类(Mathematical)
| RxJS | rxrust | 功能 |
|------|--------|------|
| `reduce((a,b) => a+b, 0)` | 无内置 | 归约 |
| `count()` | 无内置 | 计数 |
| `max()` | 无内置 | 最大值 |
| `min()` | 无内置 | 最小值 |

---

## 6. Subject ⭐

### 6.1 什么是 Subject

**Subject 是一种特殊类型的 Observable,可以多播给多个 Observer。**

```
普通 Observable (单播):
Observable A ──► Observer 1
            └─► Observer 2 (独立的执行)

Subject (多播):
Subject ──► Observer 1
        └─► Observer 2 (共享同一执行)
```

**关键区别:**
- 普通 Observable: 每个订阅者触发独立的执行(Cold)
- Subject: 所有订阅者共享同一数据流(Hot)

### 6.2 为什么需要 Subject

```typescript
// 问题: 冷 Observable,每次订阅独立执行
const cold$ = new Observable(subscriber => {
  const data = expensiveComputation(); // 每次都执行!
  subscriber.next(data);
  subscriber.complete();
});

cold$.subscribe(v => console.log('A:', v)); // expensiveComputation()
cold$.subscribe(v => console.log('B:', v)); // expensiveComputation() 再次执行!

// 解决方案: Subject,所有订阅共享
const subject = new Subject();

subject.subscribe(v => console.log('A:', v));
subject.subscribe(v => console.log('B:', v));

subject.next(42);
// 输出:
// A: 42
// B: 42
// expensiveComputation 只执行一次!
```

### 6.3 Subject 的两种身份

```typescript
const subject = new Subject<number();

// 身份1: Observer — 可以接收值
subject.next(1);
subject.next(2);
subject.complete();

// 身份2: Observable — 可以被订阅
subject.subscribe(v => console.log(v));
```

**rxrust 对应:**
```rust
use rxrust::prelude::*;
use rxrust::subject::SubjectMut;

let mut subject = SubjectMut::new();

// 身份1: 接收值
subject.next(1);
subject.next(2);
subject.complete();

// 身份2: 被订阅
subject.subscribe(|v| println!("{:?}", v));
```

### 6.4 四种 Subject

#### 6.4.1 Subject (基础)
```typescript
const subject = new Subject();

subject.subscribe(v => console.log('A:', v));
subject.next(1);  // A: 1
subject.next(2);  // A: 2

subject.subscribe(v => console.log('B:', v));
subject.next(3);  // A: 3, B: 3
                  // ⚠️ B 错过了 1 和 2!
```

**特性:**
- 不保存历史值
- 订阅前的值会丢失
- 只有订阅后才能收到后续值

**适用场景:** 事件通知、广播

#### 6.4.2 BehaviorSubject
```typescript
const subject = new BehaviorSubject(0); // 需要初始值

subject.subscribe(v => console.log('A:', v)); // A: 0 (立即收到初始值)
subject.next(1);                               // A: 1
subject.next(2);                               // A: 2

subject.subscribe(v => console.log('B:', v)); // B: 2 (立即收到最新值)
subject.next(3);                               // A: 3, B: 3
```

**特性:**
- 需要初始值
- 新订阅者立即收到当前最新值
- 保存一个最新值

**适用场景:** 状态管理、当前值查询

**rxrust 对应:** `BehaviorSubject` (rxrust 内置)

#### 6.4.3 ReplaySubject
```typescript
const subject = new ReplaySubject(3); // 缓存3个值

subject.next(1);
subject.next(2);
subject.next(3);
subject.next(4);

subject.subscribe(v => console.log('A:', v));
// A: 2 (缓存的最后3个)
// A: 3
// A: 4
```

**特性:**
- 缓存N个历史值
- 新订阅者立即收到所有缓存值
- 可配置缓存数量

**适用场景:** 历史回放、状态恢复

#### 6.4.4 AsyncSubject
```typescript
const subject = new AsyncSubject();

subject.next(1);
subject.next(2);
subject.next(3);

subject.subscribe(v => console.log('A:', v)); // 还没完成,不发出

subject.complete(); // A: 3 (只发出最后一个值)
```

**特性:**
- 只在 complete 时发出值
- 发出的值是最后一个 next 的值
- 类似 `last()` 操作符

**适用场景:** 只关心最终结果的异步操作

### 6.5 对比表

| Subject 类型 | 初始值 | 缓存 | 新订阅者收到 |
|-------------|--------|------|-------------|
| Subject | 无 | 无 | 仅后续值 |
| BehaviorSubject | 必要 | 1个最新 | 最新值 |
| ReplaySubject | 可选 | N个历史 | 所有缓存值 |
| AsyncSubject | 无 | 1个最后 | complete时的最后值 |

### 6.6 实际应用示例

#### 全局状态管理(BehaviorSubject)
```typescript
class UserService {
  private currentUser = new BehaviorSubject<User | null>(null);
  
  // 暴露为 Observable(防止外部调用 next)
  currentUser$ = this.currentUser.asObservable();
  
  login(user: User) {
    this.currentUser.next(user);
  }
  
  logout() {
    this.currentUser.next(null);
  }
}

// 使用
const userService = new UserService();

// 组件A订阅
userService.currentUser$.subscribe(user => {
  console.log('组件A:', user);
});

// 组件B订阅
userService.currentUser$.subscribe(user => {
  console.log('组件B:', user);
});

// 登录 — 两个组件同时更新
userService.login({ name: 'Alice' });
// 组件A: { name: 'Alice' }
// 组件B: { name: 'Alice' }
```

#### 事件总线(Subject)
```typescript
class EventBus {
  private events = new Subject<AppEvent>();
  
  events$ = this.events.asObservable();
  
  emit(event: AppEvent) {
    this.events.next(event);
  }
}

// 模块A发送
eventBus.emit({ type: 'USER_LOGIN', payload: user });

// 模块B接收
eventBus.events$
  .pipe(filter(e => e.type === 'USER_LOGIN'))
  .subscribe(e => console.log(e.payload));
```

#### 缓存与重放(ReplaySubject)
```typescript
class CachedDataService {
  private cache = new ReplaySubject<Data>(1); // 缓存1个
  
  data$ = this.cache.asObservable();
  
  loadData() {
    fetch('/api/data').then(data => {
      this.cache.next(data);
    });
  }
}

// 即使数据已加载过,新订阅者仍然能收到
const service = new CachedDataService();
service.loadData();

setTimeout(() => {
  service.data$.subscribe(data => {
    console.log('收到缓存数据:', data);
  });
}, 2000); // 2秒后订阅,仍然能收到之前的数据
```

---

## 7. Scheduler

### 7.1 什么是 Scheduler

Scheduler 控制订阅何时开始以及何时传递通知。

```typescript
interface Scheduler {
  now(): number;                           // 当前时间
  schedule(work, delay?, state?): Subscription; // 调度任务
}
```

### 7.2 四种调度器

| 调度器 | 功能 | 使用场景 |
|--------|------|----------|
| `asyncScheduler` | 基于 `setTimeout` | 默认异步 |
| `queueScheduler` | 同步队列 | 同步异步交替 |
| `asapScheduler` | 基于微任务(Promise) | 微任务调度 |
| `animationFrameScheduler` | `requestAnimationFrame` | DOM动画 |

### 7.3 使用方式
```typescript
import { asyncScheduler, of } from 'rxjs';
import { observeOn } from 'rxjs/operators';

of(1, 2, 3).pipe(
  observeOn(asyncScheduler) // 切换到异步调度
).subscribe(console.log);
```

### 7.4 订阅调度 vs 通知调度
```typescript
// subscribeOn: 决定订阅在哪个调度器执行
source.pipe(
  subscribeOn(asyncScheduler)
).subscribe();

// observeOn: 决定每个值在哪个调度器发出
source.pipe(
  observeOn(asyncScheduler)
).subscribe();
```

**rxrust 对应:**
| RxJS | rxrust | 功能 |
|------|--------|------|
| `asyncScheduler` | `SharedScheduler` | 多线程调度 |
| `queueScheduler` | `LocalScheduler` | 单线程调度 |
| `observeOn(scheduler)` | `observe_on(scheduler)` | 切换调度器 |
| `subscribeOn(scheduler)` | `subscribe_on(scheduler)` | 订阅调度 |

---

## 8. 与 rxrust 对照表

### 8.1 概念对照

| RxJS 概念 | rxrust 对应 | 说明 |
|-----------|------------|------|
| `Observable<T>` | `Observable` trait | 可观察对象 |
| `Observer<T>` | `Observer<Item, Err>` trait | 观察者 |
| `Subscription` | `Subscription` trait / `TaskHandle` | 订阅关系 |
| `Operator` | `CoreObservable<C>`, `ObservableType` | 操作符 |
| `Subject` | `Subject` / `SubjectMut` | 多播主体 |
| `Scheduler` | `Scheduler` trait | 调度器 |

### 8.2 Subject 对照

| RxJS | rxrust | 功能 |
|------|--------|------|
| `Subject` | `Subject` | 基础多播 |
| `BehaviorSubject` | `BehaviorSubject` | 带初始值 |
| `ReplaySubject` | `ReplaySubject` | 历史缓存 |
| `AsyncSubject` | `AsyncSubject` | 仅最后值 |

### 8.3 调度器对照

| RxJS | rxrust | 功能 |
|------|--------|------|
| `asyncScheduler` | `SharedScheduler` | tokio 多线程 |
| `queueScheduler` | `LocalScheduler` | 单线程 |
| `TestScheduler` | `TestScheduler` | 测试虚拟时间 |

### 8.4 订阅方式对照

```typescript
// RxJS
const sub = observable.subscribe({
  next: v => console.log(v),
  complete: () => console.log('done')
});
sub.unsubscribe();
```

```rust
// rxrust
let handle = source
    .map(|v| v * 2)
    .filter(|v| v > 5)
    .subscribe(|v| println!("{:?}", v));

handle.unsubscribe();  // 取消
handle.await;          // 等待完成(Shared)
```

### 8.5 多播(Subject)对照

```typescript
// RxJS
const subject = new Subject<number>();
subject.subscribe(v => console.log('A:', v));
subject.subscribe(v => console.log('B:', v));
subject.next(42);
```

```rust
// rxrust
use rxrust::subject::SubjectMut;

let mut subject = SubjectMut::new();
subject.subscribe(|v| println!("A: {:?}", v));
subject.subscribe(|v| println!("B: {:?}", v));
subject.next(42);
```

---

## 附录: 速查表

### 创建
```
of, from, fromEvent, interval, timer, range, generate, 
throwError, empty, never, defer, ajax, forkJoin, combineLatest
```

### 转换
```
map, scan, switchMap, mergeMap, concatMap, exhaustMap, 
buffer, bufferCount, bufferTime, toArray, expand, groupBy
```

### 过滤
```
filter, take, takeLast, takeWhile, takeUntil, skip, skipWhile,
skipUntil, distinct, distinctUntilChanged, first, last,
elementAt, ignoreElements, sample, sampleTime, debounce, 
debounceTime, throttle, throttleTime, audit, auditTime
```

### 组合
```
concat, merge, combineLatest, zip, withLatestFrom, 
startWith, endWith, race, pairwise, forkJoin
```

### 多播
```
multicast, publish, publishLast, publishBehavior, publishReplay,
share, shareReplay
```

### 错误处理
```
catchError, retry, retryWhen, finalize
```

### 条件
```
defaultIfEmpty, every, find, findIndex, isEmpty
```

### 工具
```
tap, delay, delayWhen, timeout, timeoutWith, 
materialize, dematerialize, observeOn, subscribeOn,
toArray, repeat
```

### 数学/聚合
```
reduce, count, max, min, find, findIndex
```

---

> **核心要点:**
> 1. Observable 是懒推送集合 — 订阅后才执行
> 2. Observer 定义响应 — next/error/complete
> 3. Subject 既是 Observable 又是 Observer — 实现多播
> 4. Operator 是纯函数 — 转换 Observable 链
> 5. Scheduler 控制执行时机 — 异步/同步/测试
