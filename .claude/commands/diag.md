# /diag — 诊断 sass-spec 子目录失败

诊断指定 sass-spec 子目录的失败模式，统计错误类型，找出可修复的模式。
集成 CSS 逐行 diff 模块，自动显示前 3 个差异行。

## 步骤

1. 确定目标子目录（如 `core_functions/color`, `css`, `expressions` 等）
2. 运行 `cargo test --test cf_diag diag_<subdir> -- --nocapture`
3. 查看输出——FAIL 行显示分类（content_diff/missing_output/extra_output）+ 差异行
4. 如需看完整差异，加 `RUST_LOG="cssdiff=debug"`
5. 对每类错误，用 `/trace` 追踪具体链路
6. 优先修复失败数量最多的错误类型

## 使用方式

```
/diag <子目录名>
```

## 命令

```bash
# 诊断特定子目录（集成 CSS diff）
cargo test --test cf_diag diag_<subdir> -- --nocapture

# CSS diff 详情模式——显示所有差异行
RUST_LOG="cssdiff=debug" cargo test --test cf_diag diag_<subdir> -- --nocapture

# 只统计不显示差异
RUST_LOG=error cargo test --test cf_diag diag_<subdir> -- --nocapture
```

## 输出格式

```
## 诊断结果: <子目录>

### 错误类型统计
| 类型 | 数量 | 示例 |
|------|------|------|
| content_diff | N | L1: exp='a { color: red; }' act='a { color: blue; }' |
| missing_output | N | L2: exp='b { color: blue; }' act=(missing) |
| extra_output | N | L1: exp=(missing) act='a:is((b) {' |
| undefined | N | 未定义函数: xxx |
| syntax | N | 语法错误: ... |
| eval | N | 求值错误: ... |

### 修复优先级
1. xxx (N个失败) — 修复方向: ...
2. yyy (N个失败) — 修复方向: ...
```
