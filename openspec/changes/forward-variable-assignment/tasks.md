## 1. 前缀映射表

- [ ] 1.1 在 `Env` 中添加 `forward_prefix: HashMap<String, (String, String)>` 字段
- [ ] 1.2 `@forward "url" as prefix-*` 求值时注册映射：namespace → (prefix, source_module)
- [ ] 1.3 支持不同分隔符（`d-*` 和 `d_*`）

## 2. 变量名解前缀

- [ ] 2.1 修改 `eval_assign` 中命名空间变量赋值逻辑
- [ ] 2.2 当遇到 `namespace.$prefix-var` 时，查找前缀表去掉前缀
- [ ] 2.3 赋值到 source_module 的原始变量（去掉前缀后的名字）
- [ ] 2.4 验证 `midstream.$d-a: value` 正确赋值到 upstream 的 `$a`

## 3. 嵌套作用域处理

- [ ] 3.1 命名空间赋值始终转发到 upstream 全局变量（忽略块级作用域）
- [ ] 3.2 验证 CSS rule 内的命名空间赋值正确工作

## 4. 验证

- [ ] 4.1 运行核心测试 202/202 无回归
- [ ] 4.2 运行 `directives/forward/member/as/variable_assignment/*` 验证修复
- [ ] 4.3 运行 `directives/forward/member/shadowed/*` 验证修复
- [ ] 4.4 运行 `directives/forward/member/import/*` 验证修复
- [ ] 4.5 全量 sass-spec 无其他目录回归
