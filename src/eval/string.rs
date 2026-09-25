//! 字符串函数实现 (响应式风格移植自 main string.rs)

pub fn eval_unquote(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    Some(s.trim_matches('"').trim_matches('\'').to_string())
}

pub fn eval_quote(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    if s.starts_with('"') || s.starts_with('\'') {
        Some(s.to_string())
    } else {
        Some(format!("\"{s}\""))
    }
}

pub fn eval_str_length(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let unquoted = s.trim_matches('"').trim_matches('\'');
    Some(unquoted.chars().count().to_string())
}

pub fn eval_str_index(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let sub = args.get(1)?.trim();
    let s_unquoted = s.trim_matches('"').trim_matches('\'');
    let sub_unquoted = sub.trim_matches('"').trim_matches('\'');
    s_unquoted.find(sub_unquoted).map(|pos| {
        s_unquoted[..pos].chars().count() as usize + 1
    }).map(|idx| idx.to_string())
}

pub fn eval_str_slice(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let start = args.get(1)?.trim().parse::<i64>().ok()?;
    let end = args.get(2)?.trim().parse::<i64>().ok()?;
    let s_unquoted = s.trim_matches('"').trim_matches('\'');

    let start_idx = if start < 0 {
        s_unquoted.chars().count() as i64 + start
    } else {
        start - 1
    };
    let end_idx = if end < 0 {
        s_unquoted.chars().count() as i64 + end
    } else {
        end
    };

    if start_idx < 0 || end_idx <= start_idx || start_idx >= s_unquoted.chars().count() as i64 {
        return Some(String::new());
    }

    let start_byte = s_unquoted.char_indices()
        .nth(start_idx as usize)
        .map(|(b, _)| b)
        .unwrap_or(0);
    let end_byte = s_unquoted.char_indices()
        .nth(end_idx as usize)
        .map(|(b, _)| b)
        .unwrap_or(s_unquoted.len());

    Some(s_unquoted[start_byte..end_byte].to_string())
}

pub fn eval_to_upper_case(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let unquoted = s.trim_matches('"').trim_matches('\'');
    let uppered: String = unquoted.chars()
        .map(|c| if c.is_ascii_lowercase() { c.to_ascii_uppercase() } else { c })
        .collect();
    if s.starts_with('"') || s.starts_with('\'') {
        Some(format!("\"{uppered}\""))
    } else {
        Some(uppered)
    }
}

pub fn eval_to_lower_case(args: &[String]) -> Option<String> {
    let s = args.first()?.trim();
    let unquoted = s.trim_matches('"').trim_matches('\'');
    let lowered: String = unquoted.chars()
        .map(|c| if c.is_ascii_uppercase() { c.to_ascii_lowercase() } else { c })
        .collect();
    if s.starts_with('"') || s.starts_with('\'') {
        Some(format!("\"{lowered}\""))
    } else {
        Some(lowered)
    }
}
