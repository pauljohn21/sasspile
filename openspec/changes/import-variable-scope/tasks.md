## 1. Env 作用域链继承

- [ ] 1.1 在 `Env` 上添加 `with_import_scope()` 方法，返回新 Env 其 current scope 的 parent 指向当前 scope
- [ ] 1.2 确保 `with_import_scope()` 不 clone 变量 HashMap（只 clone `Rc<Scope>` 引用）
- [ ] 1.3 验证 parent chain 查找正确（沿链向上搜索变量）

## 2. @import 求值改造

- [ ] 2.1 修改 `eval_import`：加载模块后使用 `env.with_import_scope()` 创建子环境
- [ ] 2.2 子环境求值 imported module 的 AST
- [ ] 2.3 确保 imported file 中的 `!default` 赋值在 parent 已有值时跳过
- [ ] 2.4 确保 imported file 中普通赋值不影响 importing context

## 3. 嵌套 Import 支持

- [ ] 3.1 确保 import 链 a→b→c 中，c 可以读取 a 和 b 的变量
- [ ] 3.2 验证循环 import 检测不因子作用域链而失效

## 4. 验证

- [ ] 4.1 运行核心测试 202/202 无回归
- [ ] 4.2 运行 `directives/import/configuration/*` 验证修复（期望 +20 passes）
- [ ] 4.3 运行 `directives/import/with*` 验证修复（期望 +12 passes）
- [ ] 4.4 全量 sass-spec 无其他目录回归
