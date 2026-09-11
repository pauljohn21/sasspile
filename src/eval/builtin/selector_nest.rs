//! `selector-nest` 实现。

use crate::css::selector_format;
use crate::error::Result;
use crate::parse::ast::*;

/// `selector-nest($selectors...)` — 将多个选择器嵌套组合。
pub fn call_nest(args: &[Value]) -> Result<Option<Value>> {
    if args.is_empty() {
        return Ok(Some(Value::List(Vec::new(), Separator::Space, false)));
    }
    if args.len() == 1 {
        let fmt = selector_format::value_to_selector_format(&args[0])?;
        return Ok(Some(selector_format::selector_format_to_value(fmt)));
    }
    // 解析所有参数
    let all_formats: Vec<Vec<Vec<String>>> = args
        .iter()
        .map(|a| selector_format::value_to_selector_format(a))
        .collect::<Result<Vec<_>>>()?;
    let last = all_formats
        .last()
        .expect("selector-nest: len > 1 guaranteed by caller")
        .clone();
    let leading = &all_formats[..all_formats.len() - 1];
    // 构建父级列表
    let parents = build_parents(leading)?;
    // 检查最后一个 arg 是否包含 &
    let has_amp = last
        .iter()
        .any(|complex| complex.iter().any(|c| c.contains('&')));
    let result: Vec<Vec<String>> = if has_amp {
        nest_with_amp(&parents, &last)?
    } else {
        nest_no_amp(&parents, &last)
    };
    Ok(Some(selector_format::selector_format_to_value(result)))
}

/// 将所有前置参数笛卡尔积后代连接为父级 complex 列表。
/// 参数从外到内排列：leading[0] 是最内层，leading[last] 是最外层。
/// 结果按外→内顺序排列，以便与 last（最外层父级）拼接。
fn build_parents(leading: &[Vec<Vec<String>>]) -> crate::error::Result<Vec<Vec<String>>> {
    match leading {
        [] => Ok(vec![vec![]]),
        [single] => Ok(single.clone()),
        _ => {
            // 从最后一个（最外层）向第一个（最内层）折叠
            let mut iter = leading.iter().rev();
            let outermost = iter
                .next()
                .expect("build_parents: _ branch guarantees non-empty")
                .clone();
            iter.try_fold(outermost, |acc, next| cartesian_descendant(&acc, next))
        }
    }
}

/// 笛卡尔积后代连接两个 complex 列表。
fn cartesian_descendant(a: &[Vec<String>], b: &[Vec<String>]) -> crate::error::Result<Vec<Vec<String>>> {
    let results: Vec<Vec<String>> = a
        .iter()
        .flat_map(|ca| {
            b.iter().map(move |cb| {
                match (ca.is_empty(), cb.is_empty()) {
                    (true, _) => cb.clone(),
                    (_, true) => ca.clone(),
                    _ => ca.iter().chain(cb.iter()).cloned().collect::<Vec<_>>(),
                }
            })
        })
        .collect();
    Ok(results)
}

/// 无 & 的 nest：父级 × 内层，后代连接。
/// parents = 嵌套选择器（前面的参数），last = 父级（最后一个参数）
/// 结果 = 父级 + 嵌套选择器（父级在前）
fn nest_no_amp(parents: &[Vec<String>], last: &[Vec<String>]) -> Vec<Vec<String>> {
    last.iter()
        .flat_map(|p| parents.iter().map(move |c| descendant_join(p, c)))
        .collect()
}

/// 后代连接两个 complex：A + descendant + B。
fn descendant_join(a: &[String], b: &[String]) -> Vec<String> {
    match (a.is_empty(), b.is_empty()) {
        (true, _) => b.to_vec(),
        (_, true) => a.to_vec(),
        _ => a.iter().chain(b.iter()).cloned().collect(),
    }
}

/// 带 & 的 nest 处理。
fn nest_with_amp(parents: &[Vec<String>], last: &[Vec<String>]) -> crate::error::Result<Vec<Vec<String>>> {
    let max_k = last
        .iter()
        .map(|complex| complex.iter().filter(|c| c.contains('&')).count())
        .max()
        .unwrap_or(0);
    let results: Vec<Vec<String>> = if max_k >= 2 {
        // 含多个 &：按 complex 顺序遍历，k>=2 的使用全局笛卡尔
        last.iter()
            .flat_map(|complex| {
                let k = complex.iter().filter(|c| c.contains('&')).count();
                match k >= 2 {
                    true => cartesian_replace(parents, complex, k),
                    false => parents
                        .iter()
                        .map(|parent| replace_amp_in_complex(complex, parent))
                        .collect::<Vec<_>>(),
                }
            })
            .collect()
    } else {
        // k <= 1：按 parent × complex 遍历
        parents
            .iter()
            .flat_map(|parent| {
                last.iter()
                    .map(|complex| replace_amp_in_complex(complex, parent))
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    Ok(results)
}

/// 生成所有笛卡尔积索引：k 个位置 × parents.len() 种选择。
///
/// 用 flat_map + scan 替代 fold + push 累积模式。
fn cartesian_combos(k: usize, n_parents: usize) -> Vec<Vec<usize>> {
    (0..k).fold(vec![vec!()], |acc, _| {
        acc.iter()
            .flat_map(|prefix| {
                (0..n_parents).map(move |i| {
                    prefix.iter().chain(std::iter::once(&i)).copied().collect::<Vec<_>>()
                })
            })
            .collect()
    })
}

/// 用笛卡尔积索引替换 complex 中的多个 &。
fn apply_indices_to_complex(complex: &[String], indices: &[usize], parents: &[Vec<String>]) -> Vec<String> {
    complex
        .iter()
        .map(|compound| {
            indices.iter().fold(compound.clone(), |acc, &idx| {
                let parent_str = parent_display(&parents[idx]);
                acc.replacen('&', &parent_str, 1)
            })
        })
        .collect::<Vec<_>>()
}

/// 全局笛卡尔替换多个 &。
fn cartesian_replace(parents: &[Vec<String>], complex: &[String], k: usize) -> Vec<Vec<String>> {
    let all_combos = cartesian_combos(k, parents.len());
    all_combos
        .iter()
        .map(|indices| apply_indices_to_complex(complex, indices, parents))
        .collect()
}

/// 替换 complex 中的 & 为父级字符串。
fn replace_amp_in_complex(complex: &[String], parent: &[String]) -> Vec<String> {
    if complex.iter().any(|c| c.contains('&')) {
        let parent_str = parent_display(parent);
        complex.iter().map(|c| c.replace('&', &parent_str)).collect()
    } else {
        // 无 & 的 complex 做后代连接
        descendant_join(parent, complex)
    }
}

/// 获取父级的显示字符串。
fn parent_display(parent: &[String]) -> String {
    parent.join(" ")
}
