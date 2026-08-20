# AGENTS.md — sasspile 项目规则

## ⛔ sasspile 特定规则

1. **禁止 #[cfg(test)] 内联测试**：所有测试放在 tests/ 目录，src/ 保持纯生产代码。
2. **修复 bug 前必须 tracing 追踪**：RUST_LOG=info/debug cargo test 查看完整链路。

## 会话开始检查清单

每次新会话，先确认：
- [ ] 读 workspace 规则（本文件）
- [ ] 读用户规则（user_rules 部分）
- [ ] 检查是否有相关记忆需要加载

## 项目核心

sasspile 是纯 Rust 函数式 SCSS 编译器。架构：

```
Source → Lexer → Parser → Evaluator → Serializer → CSS
         (lex/)   (parse/)  (eval/)     (css/)
```

## 验证清单（修复后必跑）

```bash
cargo test --test compile_test    # 43 个
cargo test --test stage_test      # 10 个
cargo test --test ast_test        # 8 个
cargo test --test common_test     # 5 个
cargo test --test bs_spec -- --nocapture    # 15 个
cargo test --test ep_full -- --nocapture    # 121 个（约 28 秒）

# sass-spec 全量统计（约 70 秒）
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture
```

**通过标准**：43/43 + 10/10 + 8/8 + 5/5 + 15/15 + 10/121
**sass-spec 基线**：2828/5362 = 53%（VFS + `===` 分组隔离，跳过 libsass/color/colors 目录）
**@directives 子目录**：337/605 = 56%（at_root 50%, extend 41%, for 94%, forward 59%, function 48%, if 33%, import 64%, mixin 100%, use 47%）
**ep_full**：10/121 = 8%（@forward 模块冲突，剩余 111 个失败为 "Two forwarded modules both define a function named xxx"）
**core_functions/color 子目录**：已跳过（防止无限修复循环，需 `--ignored` 手动触发）

### 颜色测试跳过策略

颜色相关 spec 测试已全部加入跳过列表，防止在非颜色任务中反复触发颜色测试失败导致无限修复循环：

- **SKIP_DIRS**（`tests/spec_manifest.rs`）：`core_functions/color` + `values/colors` — 全量统计和诊断自动跳过
- **#[ignore] 测试函数**：
  - `sass_spec_full::test_core_functions_subdirs` — 17 个颜色子目录统计
  - `cf_diag::diag_color` — core_functions/color 诊断
  - `cf_diag::diag_values_colors` — values/colors 诊断
  - `cf_color::color_error_patterns` — 颜色错误模式统计
  - `minimize::minimize_color_error` — 颜色错误最小化
- **手动触发颜色测试**：`cargo test --test <file> -- --ignored`

## HRX 解析架构（hrx-auditor 集成）

sasspile 通过 dev-dependency 引用 `hrx-auditor` crate（路径 `../scss-rust`），直接使用其 VFS + parser：

```toml
[dev-dependencies]
hrx-auditor = { path = "../scss-rust" }
```

- `hrx_auditor::parser::parse_hrx(content)` → `HrxArchive`
- `hrx_auditor::vfs::Vfs::from_archive(&archive)` → `Vfs`（虚拟目录树）
- 测试代码按 `===` 分隔符将 entries 分成独立组，每组构建自己的 VFS，正确隔离不同测试组的文件
- 已迁移全部 8 个测试文件：`sass_spec_full.rs`、`cf_diag.rs`、`css_diag.rs`、`expr_diag.rs`、`sass_spec.rs`、`diag_detail.rs`、`minimize.rs`、`cf_color.rs`

## Git 规范

- 推送用 SSH：`git push origin main`（remote 名为 `origin`，SSH 地址 git@github.com）
- Commit 格式：`feat: 描述 — 总计 N/M`
- **每次提交后必须同步 CodeGraph**：`codegraph sync`（确保代码导航索引与最新代码一致）

## OpenSpec 归档

- 已归档变更存储在 `openspec/changes/archive/` 目录
- **spec-pass-rate-boost**（2026-08-21 归档）：参数验证修复 + meta 模块功能 + error 检测 + values/css 深度修复 — 5 个 spec 已同步到 `openspec/specs/`

## 内建函数注册架构（builtin-dispatch-macro）

- **sasspile-macros** proc-macro crate（workspace 成员）：通过 `#[derive(BuiltinRegistry)]` 将三处重复 match 合并为单一数据源
- 依赖：syn 3.0 + quote + proc-macro2（未用 darling，改用 syn 3.0 原生 `parse_nested_meta`）
- 7 个结构体：MathBuiltins / StringBuiltins / MapBuiltins / ListBuiltins / ColorBuiltins / MetaBuiltins / SelectorBuiltins
- 宏自动生成：`module_builtin_name`（模块限定名 → 全局名）、`is_known_builtin`（已知函数检查）、`dispatch_builtin_module`（模块分派）
- `#[builtin(module = "math", dispatch = "math")]` 声明模块名和分派目标
- `#[builtin(alias = "math.div")]` 声明字段别名（模块限定名）
- 宏自动生成 `module.kebab-case` 默认别名
- `dispatch = "none"` 表示只参与名称映射不分派（meta 模块）
- 手工保留：rgba/rgb/darken/lighten/mix 的分派和 is_known_builtin

## 颜色系统架构

sasspile 颜色系统基于 `ColorFormat` 枚举追踪颜色创建方式，影响序列化输出：

| 格式 | 用途 | 示例 |
|------|------|------|
| `Auto` | hex / 命名颜色 / rgba（默认） | `#ff0000`, `red`, `rgba(0,0,0,0.5)` |
| `Rgb` | rgb(r,g,b) / rgba(r,g,b,a)（不转 hex） | `rgb(255, 0, 0)` |
| `RgbPercent(h,s,l)` | rgb(r%,g%,b%) 百分比输出（HSL 操作结果） | `rgb(72%, 0%, 0%)` |
| `Hsl(h,s,l)` | hsl(h,s%,l%) / hsla(...)（保留原始 HSL） | `hsl(120, 50%, 50%)` |
| `Hwb(h,w,b)` | hwb(h w% b%) / hwb(h w% b% / a) | `hwb(0 30% 40%)` |
| `Lab(l,a,b)` | lab(L% a b)（CSS Color 4 Lab） | `lab(50% 40 59.5)` |
| `Lch(l,c,h)` | lch(L% C Hdeg)（CSS Color 4 LCH） | `lch(50% 50 270)` |
| `Oklab(l,a,b)` | oklab(L% a b)（CSS Color 4 OkLab） | `oklab(59% 0.1 0.1)` |
| `Oklch(l,c,h)` | oklch(L% C Hdeg)（CSS Color 4 OKLCH） | `oklch(70% 0.1 180)` |
| `DisplayP3(r,g,b)` | color(display-p3 r g b) | `color(display-p3 1 0 0)` |
| `Srgb(r,g,b)` | color(srgb r g b) | `color(srgb 1 0 0)` |
| `XyzD65(x,y,z)` / `XyzD50(x,y,z)` | color(xyz r g b) / color(xyz-d50 r g b) | `color(xyz 0.5 0.5 0.5)` |

**关键规则**：
- `hsl()`/`hsla()` 创建的颜色保留 HSL 格式输出
- `darken`/`lighten`/`saturate`/`adjust-hue`/`complement`/`invert`/`grayscale` 等操作函数用 `RgbPercent` 输出
- `adjust-color`/`change-color`/`scale-color` 修改 HSL/HWB 参数时用 `RgbPercent`，纯 RGB 参数时用 `Auto`
- **CSS Color 4 现代空间**：`color_conv.rs` 使用 W3C 有理数分数矩阵（sRGB↔XYZ/Lab/Oklab），`color_adjust.rs` 支持现代空间 adjust/change/scale，`color_gamut.rs` 实现 clip + local-minde 色域映射
- 依赖 `color` crate v0.3 提供色彩空间转换参考

## 参考文档（需要时查阅）

- **代码导航**：CodeGraph（动态查询，优先）/ `docs/CODE_INDEX.md`（静态参考）
  - 每次 git 提交后必须运行 `codegraph sync` 同步索引
- **综合开发技能**：根目录 `skill.md`（编译管线 + 内建函数 + CSS 序列化 + 调试追踪）
- **调试技能**：`.claude/skills/tracing-debug/SKILL.md`
- **OpenSpec 工作流**：`.claude/skills/openspec-*/SKILL.md`
- **源文件结构**：见 `docs/CODE_INDEX.md`

## 常用命令（需要时查阅）

```bash
# 追踪错误链路
RUST_LOG=info cargo test --test compile_test <test_name> -- --nocapture
RUST_LOG="sasspile::color=trace" cargo test --test compile_test -- --nocapture

# sass-spec 诊断
cargo test --test cf_diag diag_<subdir> -- --nocapture
RUST_LOG="minimize=info" cargo test --test minimize minimize_<subdir>_error -- --nocapture

# CodeGraph 查询（优先使用）
codegraph sync                # 同步索引（每次提交后必跑）
codegraph impact <symbol>      # 影响分析
codegraph callers <function>   # 谁调了这个函数
codegraph explore "query"      # 自然语言探索
codegraph node <symbol>       # 查看符号源码 + 调用链路
```
