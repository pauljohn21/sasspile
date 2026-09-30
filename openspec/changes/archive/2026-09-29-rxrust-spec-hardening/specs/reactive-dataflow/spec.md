## ADDED Requirements

### Requirement: 管线入口必须是 Shared Subject
AI 编写多线程管线时，入口 MUST 使用 `Shared::subject::<String, Infallible>()`，不得使用 `Local::subject` 或裸 `Vec<String>`。

#### Scenario: 新建编译管线
- **WHEN** AI 创建一个新的编译管线函数
- **THEN** MUST 以 `let subject = Shared::subject::<String, Infallible>()` 开头，且入口数据类型为 `String`（满足 `'static + Send`）

#### Scenario: 入口数据为行
- **WHEN** AI 需要将输入的逐行数据注入管线
- **THEN** MUST 使用 `input.lines().for_each(|line| subject.clone().next(line.to_string()))`，且在所有 `next()` 完成后调用 `subject.clone().complete()`

### Requirement: 中间处理必须使用 rxrust 算子链
AI 不得在管线中间阶段使用 `for`/`while` 循环处理数据，MUST 使用 `map`/`filter`/`flat_map`/`scan_map`/`tap` 等算子。

#### Scenario: 变量替换
- **WHEN** AI 需要对流中每行做变量替换
- **THEN** MUST 使用 `.map(|line: String| substitute(&state, &line))` 或 `.scan_map(..., reducer)`，不得使用 `for line in lines { result.push(replace(line)) }`

#### Scenario: 过滤空行
- **WHEN** AI 需要过滤空行
- **THEN** MUST 使用 `.filter(|line: &String| !line.trim().is_empty())`，不得使用 `if line.trim().is_empty() { continue }`

#### Scenario: 展平嵌套集合
- **WHEN** AI 需要将 `Vec<Vec<T>>` 展开为 `Vec<T>`
- **THEN** MUST 使用 `.flat_map(|v: Vec<T>| Shared::from_iter(v))`，不得使用 `for inner in outer { result.extend(inner) }`

### Requirement: 终端模式固定为 collect/last + subscribe + mpsc
管线出口 MUST 使用 `collect::<Vec<T>>().last().subscribe(closure)` 模式，不得直接subscribe裸流。

#### Scenario: 编译管线出口
- **WHEN** AI 编写管线终端逻辑
- **THEN** MUST 使用以下固定模式：
  ```rust
  let (tx, rx) = std::sync::mpsc::channel::<String>();
  // ... 算子链 ...
      .collect::<Vec<String>>().last()
      .subscribe(move |v: Vec<String>| {
          let _ = tx.send(v.join("\n"));
      });
  rx.recv().unwrap_or_default()
  ```
- **不得** 省略 `collect().last()` 直接在 subscribe 中拼接

#### Scenario: 终端闭包内不再驱动源
- **WHEN** AI 在 subscribe 闭包中需要处理数据
- **THEN** MUST 保持 subscribe 闭包为纯终态操作（如 `tx.send()`），不得在 subscribe 内再调用 `subject.next()` 或触发新的流

### Requirement: flat_map 的 Inner 必须是 Observable
AI 使用 flat_map 时，闭包返回类型 MUST 实现 `Observable` trait，不得返回裸 `Vec` 或迭代器。

#### Scenario: 展开 Vec
- **WHEN** AI 需要将 `Vec<T>` 展平为流中单个 T
- **THEN** MUST 使用 `.flat_map(|v: Vec<T>| Shared::from_iter(v))`，不得使用 `.flat_map(|v: Vec<T>| v.into_iter())`（Iterator 不是 Observable）

### Requirement: 声明式优先 — chain 是声明 subscribe 是执行边界
AI MUST 明确区分"声明数据关系"和"触发执行"两个阶段。

#### Scenario: 构建算子链
- **WHEN** AI 编写管线逻辑
- **THEN** MUST 在 `.subscribe()` 之前只构建算子链（声明式），`.subscribe()` 是唯一执行入口，不得在 `.subscribe()` 之后添加算子
