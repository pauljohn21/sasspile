//! —— CSS 字符串转义 ——
//!
//! 概要：处理 CSS 引用字符串和标识符中的特殊字符转义。
//!
//! ## 核心概念
//! - 字符串内转义（`\n`、`\"` 等）
//! - 标识符转义（特殊字符 → `\` 前缀）
//! - Unicode 码点转义（`\xxxxxx ` 格式）

use super::Value;

impl Value {
    /// 转义引用字符串中的特殊字符为 CSS 转义序列。
    ///
    /// 返回 (`quote_char`, `escaped_content`)。
    /// - 如果字符串包含 `"` 但不包含 `'`，用单引号包裹，避免转义
    /// - 否则用双引号包裹，转义 `"`
    /// - `\` → `\\`
    /// - NULL (U+0000) → `\0 ` (with trailing space if needed)
    /// - 控制字符和私有区字符 → `\XXXX` (lowercase hex)
    /// - 其他非 ASCII 字符保持原样（会触发 @charset 前缀）
    pub(crate) fn escape_quoted_string(s: &str) -> (char, String) {
        let has_double = s.contains('"');
        let has_single = s.contains('\'');
        let quote = if has_double && !has_single { '\'' } else { '"' };

        let escaped = Self::escape_css_chars(s, |c| {
            (c == '"' && quote == '"') || (c == '\'' && quote == '\'')
        });
        (quote, escaped)
    }

    /// 对未加引号的 CSS 标识符进行转义。
    /// 反斜杠 → `\\`，控制字符 → `\XXXX`，NULL → `\0 `。
    /// 非 ASCII 字母数字/`-`/`_` 的 ASCII 字符也需要转义（如 `$`, `(`, `)` 等）。
    /// 前导数字也需要转义（如 `1u` → `\31 u`）。
    pub(crate) fn escape_css_ident(s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        let mut result = String::new();
        for (i, &c) in chars.iter().enumerate() {
            match c {
                '\\' => result.push_str("\\\\"),
                '\0' => result.push_str("\\0 "),
                c if c.is_control() || ('\u{E000}'..='\u{F8FF}').contains(&c) => {
                    let hex = format!("{:x}", c as u32);
                    result.push('\\');
                    result.push_str(&hex);
                    let next = chars.get(i + 1).copied();
                    match next.is_some_and(|nc| nc.is_ascii_hexdigit() || nc.is_whitespace()) {
                        true => result.push(' '),
                        false => {}
                    }
                }
                // 前导数字：必须转义为 \XX（当后面还有字符时添加尾随空格防止歧义）
                c if i == 0 && c.is_ascii_digit() => {
                    let hex = format!("{:x}", c as u32);
                    result.push('\\');
                    result.push_str(&hex);
                    // dart-sass 风格：只要后面跟了字符，一律加空格
                    let has_next = chars.get(i + 1).is_some();
                    if has_next {
                        result.push(' ');
                    }
                }
                // CSS 标识符中不合法的 ASCII 字符需要转义
                c if c.is_ascii() && !c.is_ascii_alphanumeric() && c != '-' && c != '_' => {
                    result.push('\\');
                    result.push(c);
                    let next = chars.get(i + 1).copied();
                    match next.is_some_and(|nc| nc.is_ascii_hexdigit() || nc.is_whitespace()) {
                        true => result.push(' '),
                        false => {}
                    }
                }
                _ => result.push(c),
            }
        }
        result
    }

    /// 规范化 CSS 标识符：解码原始转义序列后重新编码为标准 CSS ident 格式。
    ///
    /// 处理两种输入：
    /// 1. 已解码的字符（如 `1u` 来自 `\31` 的 lex 解码）→ 重新转义前导数字
    /// 2. 原始转义序列（如 `u\28` 来自字符串插值）→ 先解码再转义
pub(crate) fn normalize_css_ident(raw: &str) -> String {
    let decoded = Self::decode_css_escapes(raw);
    Self::escape_css_ident(&decoded)
}

    /// 解码 CSS 转义序列（`\XX` 或 `\XX ` 格式）为实际字符。
    fn decode_css_escapes(s: &str) -> String {
        let chars: Vec<char> = s.chars().collect();
        let mut result = String::new();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '\\' && i + 1 < chars.len() {
                let next = chars[i + 1];
                if next.is_ascii_hexdigit() {
                    let mut hex = String::new();
                    let mut j = i + 1;
                    while j < chars.len() && hex.len() < 6 && chars[j].is_ascii_hexdigit() {
                        hex.push(chars[j]);
                        j += 1;
                    }
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                            // 跳过后备空格（CSS 转义终止符）
                            if j < chars.len() && chars[j] == ' ' {
                                j += 1;
                            }
                            i = j;
                            continue;
                        }
                    }
                    // 解析失败，保留反斜杠
                    result.push('\\');
                    i += 1;
                } else if next == '\n' {
                    // 行继续符，跳过
                    i += 2;
                } else {
                    // 非十六进制转义，解码单个字符
                    result.push(next);
                    i += 2;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

/// 核心转义逻辑——遍历字符并转义特殊字符。
/// `is_quote` 判断当前字符是否为需要转义的引号。
///
/// 注意：NULL 和控制字符的 CSS 转义（如 `\0 `）本身包含反斜杠，
/// 在字符串字面量中需要再次转义为 `\\0 `，确保 CSS 解析器正确解码。
pub(crate) fn escape_css_chars(s: &str, is_quote: impl Fn(char) -> bool) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '\\' => result.push_str("\\\\"),
            c if is_quote(c) => {
                result.push('\\');
                result.push(c);
            }
            '\0' => {
                // NULL → CSS 转义 `\0 `（带尾随空格，dart-sass 规范）
                // 在字符串字面量中需双反斜杠：`\\0 `
                let next = chars.get(i + 1).copied();
                match next.is_some_and(|nc| nc.is_ascii_hexdigit()) {
                    true => result.push_str("\\\\0 "),  // 后跟 hex 数字，需要空格
                    false => result.push_str("\\\\0 "),  // dart-sass 总是加空格
                }
            }
            c if c.is_control() || ('\u{E000}'..='\u{F8FF}').contains(&c) => {
                let hex = format!("{:x}", c as u32);
                result.push_str("\\\\");  // 双反斜杠（字符串转义）
                result.push_str(&hex);
                let next = chars.get(i + 1).copied();
                match next.is_some_and(|nc| nc.is_ascii_hexdigit() || nc.is_whitespace()) {
                    true => result.push(' '),
                    false => {}
                }
            }
            _ => result.push(c),
        }
    }
    result
}
}
