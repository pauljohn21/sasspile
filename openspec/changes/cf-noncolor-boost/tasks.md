## 1. 诊断基础设施

- [ ] 1.1 创建 `tests/tmp_cf_selector.rs` 诊断测试——用 SHOW_FAILS 模式收集 selector 前 30 个失败 case,聚类分析共性根因
- [ ] 1.2 创建 `tests/tmp_cf_meta.rs` 诊断测试——收集 meta 前 30 个失败 case,按函数分组统计
- [ ] 1.3 创建 `tests/tmp_cf_list.rs` 诊断测试——收集 list 前 30 个失败 case
- [ ] 1.4 创建 `tests/tmp_cf_math.rs` 诊断测试——收集 math 前 30 个失败 case
- [ ] 1.5 创建 `tests/tmp_cf_modules.rs` 诊断测试——收集 modules 全部失败 case

## 2. selector 子域修复 (目标: 45% → 85%+)

- [ ] 2.1 修复 `selector-nest` 多父选择器展开逻辑(处理逗号分隔和部分覆盖场景)
- [ ] 2.2 修复 `selector-merge` 选择器智能合并(处理 simple/compound/complex 合并规则)
- [ ] 2.3 修复 `selector-parse` 选择器解析为结构化 list(匹配 sass-spec 期望格式)
- [ ] 2.4 修复 `is-superselector` 超集匹配判断(处理 type/class/pseudo 的层级关系)
- [ ] 2.5 修复 `selector-extend` 扩展逻辑
- [x] 2.6 修复 `selector-unify` 选择器统一——从右向左逐位置合并 + superselector 检测（a 是 b 的 super 则返回 b）
- [ ] 2.7 修复 `simple-selectors` 选择器分解
- [ ] 2.8 修复 `selector-replace` 选择器模式替换
- [ ] 2.9 运行 `cargo test --test compile_test` 确认无回归,运行 `cargo test --test sass_spec_full` 确认 selector 子目录通过率提升

## 3. meta 子域修复 (目标: 63% → 85%+)

- [ ] 3.1 修复 `meta.type-of` 返回精确类型标识(处理空列表、calculation、argument list 等边界)
- [ ] 3.2 修复 `meta.content-exists` 在 mixin @content 上下文中的正确判断
- [ ] 3.3 修复 `meta.module-variables` 返回变量名列表
- [ ] 3.4 修复 `meta.module-functions` 返回函数签名列表
- [ ] 3.5 修复 `meta.get-function` 在获取不不存在函数时的错误处理
- [ ] 3.6 修复 `meta.keywords` 在 mixin 内部返回 kwargs map
- [ ] 3.7 修复 `meta.inspect` 的字面表示格式
- [ ] 3.8 编译测试 + sass-spec meta 子目录回归验证

## 4. list 子域修复 (目标: 70% → 90%+)

- [ ] 4.1 修复 `list.separator` 对 slash 分隔和新语法的支持
- [ ] 4.2 修复 `list.set-nth` 的负索引和边界行为
- [ ] 4.3 修复 `list.join` 的 separator 参数处理
- [ ] 4.4 修复单元素列表、嵌套列表的边界场景
- [ ] 4.5 编译测试 + sass-spec list 子目录回归验证

## 5. math 子域修复 (目标: 80% → 90%+)

- [ ] 5.1 修复 `math.percentage` 对整数/小数的精确处理
- [ ] 5.2 修复 `math.ceil/floor/round` 负数的 Sass Math 规范语义(区别于 Rust f64 原生行为)
- [ ] 5.3 修复 `math.sqrt` 负数返回 NaN 的单位化处理
- [ ] 5.4 修复 `math.clamp` 的边界行为
- [ ] 5.5 修复 `math.abs` 保持百分比单位
- [ ] 5.6 修复 `math.comparable` 的单位兼容性判断(长度/角度/时间等可换算)
- [ ] 5.7 编译测试 + sass-spec math 子目录回归验证

## 6. modules 子域修复 (目标: 54% → 80%+)

- [ ] 6.1 修复 @use 配置覆盖默认变量的行为
- [ ] 6.2 修复 namespace 变量访问路径
- [ ] 6.3 修复 @forward 链式 use 变量可达性
- [ ] 6.4 编译测试 + sass-spec modules 子目录回归验证

## 7. 清理与归档

- [ ] 7.1 删除所有 `tests/tmp_cf_*.rs` 临时诊断文件
- [ ] 7.2 运行完整验证:`cargo test --test compile_test` + `cargo test --test stage_test` + `cargo test --test ast_test` + `cargo test --test common_test` + `cargo test --test interp_test` + `cargo test --test bs_spec` + cargo test --test ep_full` = 136/136
- [ ] 7.3 运行 sass-spec 全量统计确认总体通过率从 60% → 63-65%
- [ ] 7.4 git commit(按子域分 5 个独立 commit): selector → meta → list → math → modules
- [ ] 7.5 归档变更到 `openspec/changes/archive/2026-09-08-cf-noncolor-boost`
