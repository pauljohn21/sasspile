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
    if has_bogus_combinators(&selector) {
        return String::new();
    }
    if !selector.contains('%') {
        return selector;
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
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                }
                if depth > 0 {
                    i += 1;
                }
            }
            let end = i;
            let inner = &result[paren_start + 1..end];
            let args: Vec<&str> = inner.split(',').filter(|s| !s.trim().is_empty()).collect();
            let real_args: Vec<&str> = args
                .iter()
                .filter(|s| !s.trim().starts_with('%'))
                .cloned()
                .collect();
            if real_args.is_empty() {
                if *pseudo == "not" {
                    // :not(%placeholder) → 移除 :not()
                    // 如果前面有选择器（如 a:not(%b)），直接移除 :not → a
                    // 如果 :not 是整个选择器（如 :not(%b)），替换为 *（匹配所有元素）
                    let before = &result[..pos];
                    let after = &result[end + 1..];
                    if before.trim().is_empty() {
                        result = format!("*{after}");
                    } else {
                        result = format!("{before}{after}");
                    }
                } else {
                    return String::new();
                }
            } else {
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
    result
}

/// 检查选择器是否包含无效组合器（bogus combinators）。
///
/// 规则：
/// - 顶层/:has() 内：允许单个前导组合器（`> a`），但禁止多个前导组合器、连续组合器、尾部组合器
/// - :is/:where/:not/matches 内：禁止任何前导组合器（只能有完整选择器）
/// - 所有上下文：禁止连续组合器和尾部组合器
pub(super) fn has_bogus_combinators(selector: &str) -> bool {
    check_bogus_in_selector(selector, true)
}

/// 递归检查选择器中的无效组合器。
/// `allow_leading_combinator` 控制是否允许单个前导组合器。
fn check_bogus_in_selector(selector: &str, allow_leading_combinator: bool) -> bool {
    // 对逗号分隔的每个选择器部分单独检查
    for part in selector.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // 将选择器分解为 token 序列，同时提取伪类内部内容
        let tokens = tokenize_selector_with_pseudo(part);
        if tokens_have_bogus(&tokens, allow_leading_combinator) {
            return true;
        }
    }
    false
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
        if c == '[' {
            in_brackets = true;
            current.push(c);
        } else if c == ']' {
            in_brackets = false;
            current.push(c);
        } else if c == '(' && !in_brackets {
            // 找到伪类内部——提取完整内容
            if !current.trim().is_empty() {
                tokens.push(SelToken::Selector(current.trim().to_string()));
                current = String::new();
            }
            // 向前查看伪类名（向前回看）
            let pseudo_name = find_pseudo_name(&tokens);
            // 提取括号内容
            let mut depth = 1;
            let mut inner = String::new();
            i += 1;
            while i < chars.len() && depth > 0 {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                }
                if depth > 0 {
                    inner.push(chars[i]);
                }
                i += 1;
            }
            // :is/:where/:not/matches 不允许前导组合器
            // :has 允许前导组合器（同顶层）
            let allow_leading = pseudo_name.as_deref() == Some("has");
            tokens.push(SelToken::PseudoInner(inner, allow_leading));
            continue; // 已经推进了 i
        } else if in_brackets {
            current.push(c);
        } else if c == '>' || c == '+' || c == '~' {
            if !current.trim().is_empty() {
                tokens.push(SelToken::Selector(current.trim().to_string()));
                current = String::new();
            }
            tokens.push(SelToken::Combinator);
        } else if c.is_whitespace() {
            if !current.trim().is_empty() {
                tokens.push(SelToken::Selector(current.trim().to_string()));
                current = String::new();
            }
        } else {
            current.push(c);
        }
        i += 1;
    }
    if !current.trim().is_empty() {
        tokens.push(SelToken::Selector(current.trim().to_string()));
    }
    tokens
}

/// 从 token 序列中找出最近伪类的名称。
fn find_pseudo_name(tokens: &[SelToken]) -> Option<String> {
    // 回看 token 序列，找到 :pseudoName 模式
    for window in tokens.windows(2) {
        if let SelToken::Selector(name) = &window[0]
            && let SelToken::Selector(colon) = &window[1]
                && colon == ":" {
                    return Some(name.clone());
                }
    }
    // 检查最后一个 token 是否是 `:name` 形式
    if let Some(SelToken::Selector(last)) = tokens.last()
        && last.starts_with(':') {
            return Some(last[1..].to_string());
        }
    None
}

/// 检查 token 序列是否包含无效组合器。
fn tokens_have_bogus(tokens: &[SelToken], allow_leading_combinator: bool) -> bool {
    if tokens.is_empty() {
        return false;
    }
    // 检查尾部组合器
    if matches!(tokens.last(), Some(SelToken::Combinator)) {
        return true;
    }
    // 检查前导组合器
    if let Some(SelToken::Combinator) = tokens.first() {
        if !allow_leading_combinator {
            return true;
        }
        // 单个前导组合器允许，但第二个不能是组合器
        if tokens.len() >= 2
            && let SelToken::Combinator = tokens[1] {
                return true; // 连续组合器
            }
    }
    // 检查中间连续组合器
    for window in tokens.windows(2) {
        if let (SelToken::Combinator, SelToken::Combinator) = (&window[0], &window[1]) {
            return true;
        }
    }
    // 递归检查伪类内部
    for token in tokens {
        if let SelToken::PseudoInner(inner, allow_leading) = token {
            // 处理逗号分隔的多个选择器
            for part in inner.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                let inner_tokens = tokenize_selector_with_pseudo(part);
                if tokens_have_bogus(&inner_tokens, *allow_leading) {
                    return true;
                }
            }
        }
    }
    false
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
        if i + 1 < chars.len() {
            let curr = chars[i];
            let next = chars[i + 1];
            if curr == ']' && !next.is_whitespace() {
                // ] 后紧跟类型选择器（字母）或通配符 * 时加空格
                // 但 . # : [ 后是同一复合选择器的延续，不加空格
                if next == '*' || next.is_ascii_alphabetic() {
                    result.push(' ');
                }
            }
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
        if chars[i] == '[' {
            // 找到匹配的 ]
            let start = i;
            let mut depth = 1;
            i += 1;
            while i < chars.len() && depth > 0 {
                if chars[i] == '[' {
                    depth += 1;
                } else if chars[i] == ']' {
                    depth -= 1;
                }
                if depth > 0 {
                    i += 1;
                }
            }
            if i < chars.len() {
                // 提取属性选择器内容
                let inner: String = chars[start + 1..i].iter().collect();
                let normalized = normalize_attr_content(&inner);
                result.push('[');
                result.push_str(&normalized);
                result.push(']');
                i += 1; // 跳过 ]
            } else {
                // 未闭合的 [——直接复制剩余
                result.extend(&chars[start..]);
                break;
            }
        } else {
            result.push(chars[i]);
            i += 1;
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
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            // 找到结束引号
            let val_start = i + 1;
            let mut j = val_start;
            while j < chars.len() && chars[j] != quote {
                j += 1;
            }
            if j < chars.len() {
                // 提取引号内的值
                let val: String = chars[val_start..j].iter().collect();
                // 检查是否是合法 CSS 标识符（但 --foo 需保留引号）
                let is_ident = !val.is_empty()
                    && !val.starts_with("--")
                    && val.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '-')
                    && val.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
                if is_ident {
                    // 去除引号
                    result.push_str(&val);
                } else {
                    // 保留引号
                    result.push(quote);
                    result.push_str(&val);
                    result.push(quote);
                }
                // 检查后面是否有修饰符（紧跟的字母）
                let after = j + 1;
                if after < chars.len() && chars[after].is_ascii_alphabetic() {
                    // 在修饰符前加空格
                    result.push(' ');
                }
                i = j + 1;
            } else {
                // 未闭合的引号——直接复制
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}
