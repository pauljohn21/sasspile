/// 选择器 token——用于组合器验证。
#[derive(Debug)]
pub(super) enum SelToken {
    /// 普通选择器片段。
    Selector(String),
    /// 组合器（>, +, ~）。
    Combinator,
    /// 伪类内部的选择器列表（需递归检查）。
    /// (内部选择器字符串, 是否允许前导组合器)
    PseudoInner(String, bool),
}

/// 净化选择器——处理占位符 `%xxx` 在伪类中的移除 + 组合器验证 + 相邻复合选择器规范化。
pub(super) fn sanitize_selector(selector: &str) -> String {
    // 先规范化属性选择器（引号去除、修饰符空格）
    let selector = normalize_attr_selectors(selector);
    // 处理相邻复合选择器（[a]b → [a] b）
    let selector = normalize_adjacent_compounds(&selector);
    // 组合器验证——无效组合器返回空字符串
    match has_bogus_combinators(&selector) {
        true => return String::new(),
        false => {}
    }
    match !selector.contains('%') {
        true => return selector,
        false => {}
    }
    // 顶层逗号分隔——移除纯占位符部分
    let parts: Vec<&str> = selector
        .split(',')
        .filter(|s| !s.trim().starts_with('%'))
        .collect();
    let mut result = parts.join(",").trim().to_string();
    // 处理伪类内的占位符
    for pseudo in &["is", "not", "where", "matches"] {
        let pattern = format!(":{pseudo}(%");
        while let Some(pos) = result.find(&pattern) {
            let paren_start = pos + pattern.len() - 2; // pattern=":is(%" → -2 得到 ( 的位置
            let mut depth = 1;
            let chars: Vec<char> = result.chars().collect();
            let mut i = paren_start + 1;
            while i < chars.len() && depth > 0 {
                match chars[i] {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                match depth > 0 {
                    true => i += 1,
                    false => {}
                }
            }
            let end = i;
            let inner = &result[paren_start + 1..end];
            let args: Vec<&str> = inner.split(',').filter(|s| !s.trim().is_empty()).collect();
            let real_args: Vec<&str> = args
                .iter()
                .filter(|s| !s.trim().starts_with('%'))
                .copied()
                .collect();
            match real_args.is_empty() {
                true => match *pseudo == "not" {
                    true => {
                        let before = &result[..pos];
                        let after = &result[end + 1..];
                        match before.trim().is_empty() {
                            true => result = format!("*{after}"),
                            false => result = format!("{before}{after}"),
                        }
                    }
                    false => return String::new(),
                },
                false => {
                    let new_inner = real_args
                        .iter()
                        .map(|s| s.trim())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let before = &result[..paren_start];
                    let after = &result[end + 1..];
                    result = format!("{before}({new_inner}){after}");
                }
            }
        }
    }
    result
}

/// 检查选择器是否包含无效组合器（bogus combinators）。
///
/// 规则：
/// - `顶层/:has()` 内：允许单个前导组合器（`> a`），但禁止多个前导组合器、连续组合器、尾部组合器
/// - :is/:where/:not/matches 内：禁止任何前导组合器（只能有完整选择器）
/// - 所有上下文：禁止连续组合器和尾部组合器
pub(super) fn has_bogus_combinators(selector: &str) -> bool {
    check_bogus_in_selector(selector, true)
}

/// 递归检查选择器中的无效组合器。
/// `allow_leading_combinator` 控制是否允许单个前导组合器。
fn check_bogus_in_selector(selector: &str, allow_leading_combinator: bool) -> bool {
    // 对逗号分隔的每个选择器部分单独检查
    selector.split(',').any(|part| {
        let part = part.trim();
        !part.is_empty() && {
            let tokens = tokenize_selector_with_pseudo(part);
            tokens_have_bogus(&tokens, allow_leading_combinator)
        }
    })
}

/// 将选择器字符串分解为 token 向量，识别伪类内部内容。
fn tokenize_selector_with_pseudo(selector: &str) -> Vec<SelToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = selector.chars().collect();
    let mut i = 0;
    let mut current = String::new();
    let mut in_brackets = false;

    while i < chars.len() {
        let c = chars[i];
        match c {
            '[' => {
                in_brackets = true;
                current.push(c);
            }
            ']' => {
                in_brackets = false;
                current.push(c);
            }
            '(' if !in_brackets => {
                match !current.trim().is_empty() {
                    true => {
                        tokens.push(SelToken::Selector(current.trim().to_string()));
                        current = String::new();
                    }
                    false => {}
                }
                // 向前查看伪类名（向前回看）
                let pseudo_name = find_pseudo_name(&tokens);
                // 提取括号内容
                let mut depth = 1;
                let mut inner = String::new();
                i += 1;
                while i < chars.len() && depth > 0 {
                    match chars[i] {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    match depth > 0 {
                        true => inner.push(chars[i]),
                        false => {}
                    }
                    i += 1;
                }
                // :is/:where/:not/matches 不允许前导组合器
                // :has 允许前导组合器（同顶层）
                let allow_leading = pseudo_name.as_deref() == Some("has");
                tokens.push(SelToken::PseudoInner(inner, allow_leading));
                continue; // 已经推进了 i
            }
            _ if in_brackets => {
                current.push(c);
            }
            '>' | '+' | '~' => {
                match !current.trim().is_empty() {
                    true => {
                        tokens.push(SelToken::Selector(current.trim().to_string()));
                        current = String::new();
                    }
                    false => {}
                }
                tokens.push(SelToken::Combinator);
            }
            _ if c.is_whitespace() => {
                match !current.trim().is_empty() {
                    true => {
                        tokens.push(SelToken::Selector(current.trim().to_string()));
                        current = String::new();
                    }
                    false => {}
                }
            }
            _ => {
                current.push(c);
            }
        }
        i += 1;
    }
    match !current.trim().is_empty() {
        true => tokens.push(SelToken::Selector(current.trim().to_string())),
        false => {}
    }
    tokens
}

/// 从 token 序列中找出最近伪类的名称。
fn find_pseudo_name(tokens: &[SelToken]) -> Option<String> {
    // 回看 token 序列，找到 :pseudoName 模式
    for window in tokens.windows(2) {
        if let SelToken::Selector(name) = &window[0]
            && let SelToken::Selector(colon) = &window[1]
            && colon == ":"
        {
            return Some(name.clone());
        }
    }
    // 检查最后一个 token 是否是 `:name` 形式
    if let Some(SelToken::Selector(last)) = tokens.last()
        && last.starts_with(':')
    {
        return Some(last[1..].to_string());
    }
    None
}

/// 检查 token 序列是否包含无效组合器。
fn tokens_have_bogus(tokens: &[SelToken], allow_leading_combinator: bool) -> bool {
    match tokens.is_empty() {
        true => return false,
        false => {}
    }
    // 检查尾部组合器
    match matches!(tokens.last(), Some(SelToken::Combinator)) {
        true => return true,
        false => {}
    }
    // 检查前导组合器
    match tokens.first() {
        Some(SelToken::Combinator) => {
            match !allow_leading_combinator {
                true => return true,
                false => {}
            }
            // 单个前导组合器允许，但第二个不能是组合器
            match (tokens.len() >= 2, &tokens[1]) {
                (true, SelToken::Combinator) => return true,
                _ => {}
            }
        }
        _ => {}
    }
    // 检查中间连续组合器
    for window in tokens.windows(2) {
        if let (SelToken::Combinator, SelToken::Combinator) = (&window[0], &window[1]) {
            return true;
        }
    }
    // 递归检查伪类内部
    tokens.iter().any(|token| {
        if let SelToken::PseudoInner(inner, allow_leading) = token {
            // 处理逗号分隔的多个选择器
            inner.split(',').any(|part| {
                let part = part.trim();
                !part.is_empty() && {
                    let inner_tokens = tokenize_selector_with_pseudo(part);
                    tokens_have_bogus(&inner_tokens, *allow_leading)
                }
            })
        } else {
            false
        }
    })
}

/// 规范化相邻复合选择器——在属性选择器后跟类型选择器时添加空格。
/// 例如：`[a]b` → `[a] b`（属性选择器后紧跟类型选择器/通配符需加空格）
/// 但 `[a].b` `[a]#b` `[a]:hover` 不需要加空格（单复合选择器内）
fn normalize_adjacent_compounds(selector: &str) -> String {
    let chars: Vec<char> = selector.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    while i < chars.len() {
        result.push(chars[i]);
        // 检查是否需要插入空格
        match i + 1 < chars.len() {
            true => {
                let curr = chars[i];
                let next = chars[i + 1];
                match (curr == ']' && !next.is_whitespace(), next == '*' || next.is_ascii_alphabetic()) {
                    (true, true) => result.push(' '),
                    _ => {}
                }
            }
            false => {}
        }
        i += 1;
    }
    result
}

/// 规范化属性选择器——去除合法标识符的引号，在修饰符前加空格。
/// 例如：`[a="b"i]` → `[a=b i]`
fn normalize_attr_selectors(selector: &str) -> String {
    let chars: Vec<char> = selector.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '[' => {
                // 找到匹配的 ]
                let start = i;
                let mut depth = 1;
                i += 1;
                while i < chars.len() && depth > 0 {
                    match chars[i] {
                        '[' => depth += 1,
                        ']' => depth -= 1,
                        _ => {}
                    }
                    match depth > 0 {
                        true => i += 1,
                        false => {}
                    }
                }
                match i < chars.len() {
                    true => {
                        let inner: String = chars[start + 1..i].iter().collect();
                        let normalized = normalize_attr_content(&inner);
                        result.push('[');
                        result.push_str(&normalized);
                        result.push(']');
                        i += 1;
                    }
                    false => {
                        result.extend(&chars[start..]);
                        break;
                    }
                }
            }
            _ => {
                result.push(chars[i]);
                i += 1;
            }
        }
    }
    result
}

/// 规范化属性选择器内容。
/// `a="b"i` → `a=b i`（当 b 是合法标识符时去引号，修饰符前加空格）
fn normalize_attr_content(inner: &str) -> String {
    let chars: Vec<char> = inner.chars().collect();
    let mut result = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '"' | '\'' => {
                let quote = chars[i];
                let val_start = i + 1;
                let mut j = val_start;
                while j < chars.len() && chars[j] != quote {
                    j += 1;
                }
                match j < chars.len() {
                    true => {
                        let val: String = chars[val_start..j].iter().collect();
                        let is_ident = !val.is_empty()
                            && !val.starts_with("--")
                            && val
                                .chars()
                                .next()
                                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '-')
                            && val
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
                        match is_ident {
                            true => result.push_str(&val),
                            false => {
                                result.push(quote);
                                result.push_str(&val);
                                result.push(quote);
                            }
                        }
                        let after = j + 1;
                        match chars.get(after) {
                            Some(c) if c.is_ascii_alphabetic() => result.push(' '),
                            _ => {}
                        }
                        i = j + 1;
                    }
                    false => {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
            }
            _ => {
                result.push(chars[i]);
                i += 1;
            }
        }
    }
    result
}
