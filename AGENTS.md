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
cargo test --test compile_test    # 41 个
cargo test --test stage_test      # 10 个
cargo test --test ast_test        # 8 个
cargo test --test common_test     # 5 个
cargo test --test bs_spec -- --nocapture    # 15 个
cargo test --test ep_full -- --nocapture    # 121 个（约 28 秒）

# sass-spec 全量统计（约 35 秒）
RUST_LOG="sass_spec_full=info,sasspile=warn" cargo test --test sass_spec_full -- --nocapture
```

**通过标准**：41/41 + 10/10 + 8/8 + 5/5 + 15/15 + 121/121
**sass-spec 基线**：2678/4848 = 55%（core_functions 1757/2985 = 59%）

## Git 规范

- 推送用 SSH：`git push github main`
- Commit 格式：`feat: 描述 — 总计 N/M`

## 颜色系统架构

sasspile 颜色系统基于 `ColorFormat` 枚举追踪颜色创建方式，影响序列化输出：

| 格式 | 用途 | 示例 |
|------|------|------|
| `Auto` | hex / 命名颜色 / rgba（默认） | `#ff0000`, `red`, `rgba(0,0,0,0.5)` |
| `Rgb` | rgb(r,g,b) / rgba(r,g,b,a)（不转 hex） | `rgb(255, 0, 0)` |
| `RgbPercent(h,s,l)` | rgb(r%,g%,b%) 百分比输出（HSL 操作结果） | `rgb(72%, 0%, 0%)` |
| `Hsl(h,s,l)` | hsl(h,s%,l%) / hsla(...)（保留原始 HSL） | `hsl(120, 50%, 50%)` |
| `Hwb(h,w,b)` | hwb(h w% b%) / hwb(h w% b% / a) | `hwb(0 30% 40%)` |

**关键规则**：
- `hsl()`/`hsla()` 创建的颜色保留 HSL 格式输出
- `darken`/`lighten`/`saturate`/`adjust-hue`/`complement`/`invert`/`grayscale` 等操作函数用 `RgbPercent` 输出
- `adjust-color`/`change-color`/`scale-color` 修改 HSL/HWB 参数时用 `RgbPercent`，纯 RGB 参数时用 `Auto`
- 依赖 `color` crate v0.3 提供色彩空间转换参考

## 参考文档（需要时查阅）

- **代码导航**：`docs/CODE_INDEX.md`（静态）/ CodeGraph（动态查询）
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

# CodeGraph 查询
codegraph impact <symbol>      # 影响分析
codegraph callers <function>   # 谁调了这个函数
codegraph explore "query"      # 自然语言探索
```
