## Phase 1: Quick Wins — 高ROI单点多修

### 1. value-ops: + 运算符补全

- [ ] 1.1 在 ops.rs::add() 中添加 Map+Map 合并 arm（后值优先覆盖）
- [ ] 1.2 在 ops.rs::add() 中添加 Map+Null 和 Null+Map identity arm
- [ ] 1.3 在 ops.rs::add() 中添加 Bool+Bool 字符串拼接
- [ ] 1.4 在 ops.rs::add() 中添加 Null+Null → Null
- [ ] 1.5 在 tests/ 添加 value-ops 单元测试（Map+Map 等场景）

### 2. math edge cases

- [ ] 2.1 修复 math.pow() 负底数+分数指数 → NaN
- [ ] 2.2 修复 math.pow() 溢出 → Infinity
- [ ] 2.3 修复 math.atan2() 带符号零处理
- [ ] 2.4 修复 math.clamp() 单位保留
- [ ] 2.5 修复 math.unit() 复合单位输出

### 3. meta 内省完善

- [ ] 3.1 修复 get-function 返回可调用的 Function value
- [ ] 3.2 修复 keywords($args) 返回 Map 而非 List
- [ ] 3.3 修复 global_variable_exists 跨作用域检查
- [ ] 3.4 修复 function-exists 2-arg form 和 builtin 识别

### 4. @extend 语义修复

- [ ] 4.1 修复 @extend 在 @media 内的作用域
- [ ] 4.2 修复 @extend !optional 静默失败
- [ ] 4.3 修复 selector-extend programmatic form
- [ ] 4.4 在 tests/ 添加 extend 单元测试

### 5. CSS 序列化基础

- [ ] 5.1 修复 declaration `property: value` 空格规范化
- [ ] 5.2 修复 selector list 逗号后空格
- [ ] 5.3 修复 @rule 花括号前空格

## Phase 2: Root Cause — 一鱼多吃

### 6. selector-ops 完善

- [ ] 6.1 实现 selector-nest 的 & 父引用解析
- [ ] 6.2 完善 selector-append compound selector 处理
- [ ] 6.3 修复 selector-unify type+class 组合
- [ ] 6.4 完善 selector-extend complex 选择器扩展

### 7. list 边界修复

- [ ] 7.1 修复 list.join 多参数和空列表
- [ ] 7.2 修复 list.set-nth 越界和 Map 强制
- [ ] 7.3 修复 list.zip 0/1 参数场景
- [ ] 7.4 修复 list.is-bracketed 参数校验

### 8. values/numbers 精度

- [ ] 8.1 修复浮点格式化精度截断（消除 0.30000000000000004）
- [ ] 8.2 修复 number 百分比转换边界值
- [ ] 8.3 修复 NaN/Infinity 的 CSS 输出格式

## Phase 3: 验证与提交

### 9. 回归验证

- [ ] 9.1 cargo test 核心测试 202/202 通过
- [ ] 9.2 cargo test --test sass_spec_full 统计 + 确认无退化
- [ ] 9.3 cargo test --test ep_full 确认失败数减少
- [ ] 9.4 cargo clippy --all-targets 零警告

### 10. 归档

- [ ] 10.1 openspec archive（汇总实际通过率变化）
- [ ] 10.2 codegraph sync 更新代码索引
