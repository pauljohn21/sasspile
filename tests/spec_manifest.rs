//! sass-spec manifest——目录索引 + 跳过列表。

use std::path::{Path, PathBuf};

/// 跳过的 spec 目录（已弃用/非标准/颜色）。
pub const SKIP_DIRS: &[&str] = &[
    "libsass",
    "libsass-closed-issues",
    "libsass-todo-issues",
    "libsass-todo-tests",
    "non_conformant",
    "core_functions/color",
    "values/colors",
];
