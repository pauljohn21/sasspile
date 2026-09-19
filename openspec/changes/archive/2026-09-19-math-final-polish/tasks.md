## 1. random 函数 ERR 修复（5 cases）

- [x] 1.1 查询 spec_store 获取 random 5 个失败 case 的 actual_css/error，整理错误模式
- [x] 1.2 在 math.rs 的 random 调用入口添加 #[instrument] span，记录 $limit 输入值
- [x] 1.3 修复 int 边界检查：将严格 equality 改为 epsilon 容差（|f - f.round()| < 1e-9）
- [x] 1.4 确认 null 参数校验返回 "$limit: null is not a number." 错误消息 — 改为 null ≡ 无参数（SCSS spec）
- [x] 1.5 核心测试 46+14+15 全部通过
- [ ] 1.6 剩余 3 random 失败（one/two/one_hundred）依赖 `not Bool` 在属性值中的求值 bug
  - 根因：`not true` 序列化为字面量 "not true" 而非 "false"
  - 影响：`@if not meta.call(...)` 永远进入 true 分支
  - 属于 sasspile core eval bug，独立处理

## 2. variables 模块修复（7 cases → 6 PASS）

- [x] 2.1 提取 7 个失败 case 的 actual_css 与 expected 对比集
- [x] 2.2 定位根因：parse_number 把 `1e15` 的 'e' 当作单位分隔符 → 产生错误的 Value::Number(1.0, Some("e15"))
- [x] 2.3 修复 scan_number（lexer/scanner.rs）支持科学计数法 token
- [x] 2.4 修复 parse_number（prefix.rs）先尝试整体 f64 parse，失败再扫描
- [x] 2.5 修复 format_num（display.rs）：|n| >= 1e15 时跳过精度截断避免整数漂移
- [x] 2.6 验证：6/7 PASS（min_number 因 f64 次正规数精度差异待独立处理）

## 3. unit 模块修复（5 cases — 延后）

- [x] 3.1 提取 5 个失败 case 的错误模式
- [x] 3.2 定位根因：sasspile 单字符串 unit 不支持 Dart Sass 风格的 `px*em*rad`、`px/em`、`(px*em*rad)^-1` 单位算术
- [ ] 3.3 待重构——需将 Value::Number 的 unit: Option<String> 改为 numerator/denominator 结构（独立变更）

## 4. pow + tan + clamp + sin + div DIFF 修复（18 cases）

- [ ] 4.1 批量提取 actual_css，按差异模式分组（精度/单位/格式化/计算逻辑）
- [ ] 4.2 pow（5 cases）：插桩 → 定位 → 修复 → 验证
- [ ] 4.3 tan（4 cases）：检查是否 deg→rad 转换遗漏
- [ ] 4.4 clamp（4 cases）：边界值处理
- [ ] 4.5 sin（2 cases）：残留角落场景
- [ ] 4.6 div（2 cases）：除法格式化

## 5. 单例收尾（6 cases）

- [ ] 5.1 comparable（1 case）— 快速定位 + 修复
- [ ] 5.2 unitless（1 case）— 快速定位 + 修复
- [ ] 5.3 round（1 case）— 快速定位 + 修复
- [ ] 5.4 min / max / hypot / atan2（4 cases）— 逐个验证

## 6. 验证

- [ ] 6.1 运行 compile_test + stage_test + ast_test + common_test + bs_spec — 202/202 维持
- [ ] 6.2 运行 ep_full — 121/121 维持
- [ ] 6.3 运行 sass-spec math 全量 — 预计从 445/486 → 482+/486
- [ ] 6.4 更新 CHANGELOG.md
- [ ] 6.5 codegraph sync
- [ ] 6.6 提交 — 等用户确认后 push
