# Spec Delta

## Purpose

将 Bootstrap dist 对齐覆盖率门控从当前 22% 提升到 60%+，通过补全嵌套 map 迭代和响应式 mixin 展开。

## MODIFIED Requirements

### Requirement: 全量 Bootstrap dist 逐行对齐
~~系统 SHALL 将 `bootstrap.scss` 编译产物与 `dist/css/bootstrap.css` 进行逐行 diff，所有 CSS 声明块（selector + properties）在参考文件中存在且在产物中存在。~~
系统 SHALL 将 `bootstrap.scss` 编译产物与 `dist/css/bootstrap.css` 进行逐行 diff，产物的 CSS 规则集合 SHALL 包含参考文件中 >= 60% 的规则行（允许空白/注释差异），目标 >= 99%。

#### Scenario: 编译产物覆盖所有参考 CSS 规则
- **WHEN** 编译 `bootstrap/bootstrap.scss`（expanded 模式）
- **THEN** 产物的 CSS 规则集合 SHALL 包含参考文件中 >= 60% 的规则行（允许空白/注释差异）

#### Scenario: CI 门控通过条件
- **WHEN** 对照覆盖率 >= 60%
- **THEN** 测试 SHALL pass
- **WHEN** 对照覆盖率 < 60%
- **THEN** 测试 SHALL fail 并输出差异

#### Scenario: Utility API 全量展开
- **WHEN** 编译 `$utilities` map 上的 `@each` 迭代
- **THEN** 产物 SHALL 包含所有 spacing utilities (`.mt-0` 到 `.mt-5`, `.mx-auto`, `.px-lg-2` 等)

#### Scenario: 响应式断点类生成
- **WHEN** 编译 `@include media-breakpoint-up(sm) { ... }` 循环内的工具类
- **THEN** 产物 SHALL 包含 `.d-sm-none`, `.d-md-block`, `.col-lg-6` 等断点前缀类
