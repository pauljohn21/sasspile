## ADDED Requirements

### Requirement: Commit 间回归定位
系统 SHALL 定位导致指定函数退化的 commit。

#### Scenario: 自动二分定位
- **WHEN** 运行 `spec_store bisect --function math.cos --bad HEAD --good <sha>`
- **THEN** 系统在 good 和 bad 之间的 commit 历史中二分搜索，找出 first bad commit

#### Scenario: 手动对比
- **WHEN** 运行 `spec_store diff <sha1> <sha2>`
- **THEN** 系统输出两个 commit 之间所有状态变化的 case 列表（new_failures + new_passes）

#### Scenario: Commit 退化摘要
- **WHEN** 运行 `spec_store diff <sha1> <sha2> --summary`
- **THEN** 系统仅输出退化目录排名（按 fail 增量排序）
