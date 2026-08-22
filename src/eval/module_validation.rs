//! 模块配置验证——@use with / @forward with 的参数验证逻辑。

use crate::parse::ast::Node;
use std::collections::HashSet;

/// 从 AST 节点列表中收集所有带 `!default` 的顶层变量名。
pub(crate) fn collect_default_vars(nodes: &[Node]) -> HashSet<String> {
    let mut vars = HashSet::new();
    for node in nodes {
        if let Node::Variable { name, flags, .. } = node {
            if flags.default {
                vars.insert(name.clone());
            }
        }
    }
    vars
}
