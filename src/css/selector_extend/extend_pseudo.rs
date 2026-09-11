//! —— 伪类检测与解析辅助函数 ——

use super::super::selector_ast::{CompoundSelector, Selector, SimpleSelector};

/// 检测 compound 是否包含 `:not()` 伪类。
pub(super) fn has_not_pseudo(compound: &CompoundSelector) -> bool {
    compound.0.iter().any(|s| matches!(s, SimpleSelector::PseudoClass { name, .. } if name == "not"))
}

/// 提取 `:not()` 伪类的参数（如 `.c` 或 `.c, .d`）。
pub(super) fn extract_not_arg(compound: &CompoundSelector) -> Option<String> {
    compound.0.iter().find_map(|s| match s {
        SimpleSelector::PseudoClass { name, arg } if name == "not" => arg.clone(),
        _ => None,
    })
}

/// 在 compound 上追加新的 `:not()` 伪类。
#[allow(dead_code)]
pub(super) fn append_not_pseudo(compound: &CompoundSelector, new_not: &SimpleSelector) -> CompoundSelector {
    let mut simples: Vec<SimpleSelector> = compound.0.clone();
    simples.push(new_not.clone());
    CompoundSelector(simples)
}

/// 检查 extender 是否包含任何 `:not()` 伪类。
/// 如果包含，根据 Sass known limitation，扩展是 no-op。
pub(super) fn extender_contains_not(extender: &Selector) -> bool {
    extender.0.iter().any(|c| {
        c.compounds.iter().any(|(_, comp)| has_not_pseudo(comp))
    })
}

/// 规范化"伪" simple（__compound__/__selector__/__complex__）为实际 SimpleSelector。
pub(super) fn normalize_pseudo_compound(s: &SimpleSelector) -> SimpleSelector {
    match s {
        SimpleSelector::PseudoClass { name, arg: Some(arg) }
            if (name == "__compound__" || name == "__selector__" || name == "__complex__") =>
        {
            parse_single_simple_selector(arg).unwrap_or_else(|| s.clone())
        }
        _ => s.clone(),
    }
}

/// 将单个 simple selector 字符串解析为 SimpleSelector（仅支持基础 simple）。
pub(super) fn parse_single_simple_selector(input: &str) -> Option<super::super::selector_ast::SimpleSelector> {
    use super::super::selector_ast::Namespace;
    use super::super::selector_ast::SimpleSelector;
    let input = input.trim();
    if input.starts_with('.') && !input[1..].contains('.') && !input[1..].contains(':') && !input[1..].contains(' ') {
        Some(SimpleSelector::Class(input[1..].to_string()))
    } else if input.starts_with('#') && !input[1..].contains('#') && !input[1..].contains(':') && !input[1..].contains(' ') {
        Some(SimpleSelector::Id(input[1..].to_string()))
    } else if !input.starts_with(['.', '#', ':', '[', '*']) && !input.contains(' ') {
        Some(SimpleSelector::Type { namespace: Namespace::None, name: input.to_string() })
    } else {
        None
    }
}

/// 将逗号分隔的选择器列表字符串解析为 SimpleSelector 向量。
pub(super) fn parse_selector_list_to_simples(input: &str) -> Vec<SimpleSelector> {
    use super::super::selector_ast::SimpleSelector;
    let mut result = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '(' => { depth += 1; current.push(ch); }
            ')' => { depth = depth.saturating_sub(1); current.push(ch); }
            ',' if depth == 0 => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    result.push(SimpleSelector::PseudoClass {
                        name: "__selector__".to_string(),
                        arg: Some(trimmed),
                    });
                }
                current = String::new();
                continue;
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        result.push(SimpleSelector::PseudoClass {
            name: "__selector__".to_string(),
            arg: Some(trimmed),
        });
    }
    result
}

/// 将 extender 中的选择器展开为 simple selector 列表。
pub(super) fn flatten_extender_simples(extender: &Selector) -> Vec<SimpleSelector> {
    use super::super::selector_ast::SimpleSelector;
    if extender.0.len() == 1 {
        let complex = &extender.0[0];
        if complex.compounds.len() == 1 {
            let compound = &complex.compounds[0].1;

            let is_only_is_where_matches = compound.0.len() == 1 && matches!(&compound.0[0],
                SimpleSelector::PseudoClass { name, arg: Some(_) }
                if name == "is" || name == "where" || name == "matches"
            );

            if !is_only_is_where_matches {
                if compound.0.len() == 1 {
                    return vec![compound.0[0].clone()];
                }
                return vec![SimpleSelector::PseudoClass {
                    name: "__compound__".to_string(),
                    arg: Some(compound.to_string()),
                }];
            }

            if let Some(arg) = compound.0.iter().find_map(|s| match s {
                SimpleSelector::PseudoClass { name, arg } if name == "is" || name == "where" || name == "matches" => arg.clone(),
                _ => None,
            }) {
                return parse_selector_list_to_simples(&arg);
            }
        }
    }

    extender.0.iter().flat_map(|complex| {
        if complex.compounds.len() == 1 {
            let compound = &complex.compounds[0].1;
            if compound.0.len() == 1 {
                vec![compound.0[0].clone()]
            } else {
                vec![SimpleSelector::PseudoClass {
                    name: "__compound__".to_string(),
                    arg: Some(compound.to_string()),
                }]
            }
        } else {
            vec![SimpleSelector::PseudoClass {
                name: "__complex__".to_string(),
                arg: Some(complex.to_string()),
            }]
        }
    }).collect()
}
