//! Raw CSS expansion and simplification utilities.
//!
//! These functions handle raw CSS text from `@use` of `.css` files,
//! expanding compact single-line CSS into multi-line format and
//! simplifying CSS values (e.g. `calc(1px)` → `1px`).

/// Expand compact CSS (e.g. `a {b: val}`) into multi-line expanded format.
/// This handles raw CSS from @use of .css files.
pub fn expand_css(text: &str, base_indent: usize) -> String {
    let mut output = String::new();
    let mut depth = base_indent;

    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();

    while i < chars.len() {
        // Skip whitespace between rules
        while i < chars.len() && chars[i].is_whitespace() && chars[i] != '\n' {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        // Collect selector until '{'
        let selector_start = i;
        while i < chars.len() && chars[i] != '{' && chars[i] != '}' {
            i += 1;
        }

        if i >= chars.len() {
            break;
        }

        if chars[i] == '}' {
            // Closing brace
            depth = depth.saturating_sub(1);
            let pad = "  ".repeat(depth);
            output.push_str(&pad);
            output.push('}');
            output.push('\n');
            i += 1;
            continue;
        }

        // We have a selector followed by '{'
        let selector: String = chars[selector_start..i].iter().collect();
        let selector = selector.trim();

        if !selector.is_empty() {
            let pad = "  ".repeat(depth);
            output.push_str(&pad);
            output.push_str(selector);
            output.push_str(" {\n");
            depth += 1;
        }
        i += 1; // skip '{'

        // Collect declarations until '}'
        while i < chars.len() && chars[i] != '}' {
            // Skip whitespace
            while i < chars.len() && chars[i].is_whitespace() && chars[i] != '\n' {
                i += 1;
            }
            if i >= chars.len() || chars[i] == '}' {
                break;
            }

            // Collect declaration until ';' or '}' (but not inside parens)
            let decl_start = i;
            let mut paren_depth: usize = 0;
            while i < chars.len() {
                if chars[i] == '(' {
                    paren_depth += 1;
                } else if chars[i] == ')' && paren_depth > 0 {
                    paren_depth -= 1;
                } else if chars[i] == ';' && paren_depth == 0 {
                    break;
                } else if chars[i] == '}' && paren_depth == 0 {
                    break;
                }
                i += 1;
            }

            let decl: String = chars[decl_start..i].iter().collect();
            let decl = decl.trim();

            if !decl.is_empty() {
                let pad = "  ".repeat(depth);
                output.push_str(&pad);
                output.push_str(&simplify_css_value(decl));
                output.push_str(";\n");
            }

            if i < chars.len() && chars[i] == ';' {
                i += 1;
            }
        }

        // Closing brace
        if i < chars.len() && chars[i] == '}' {
            depth = depth.saturating_sub(1);
            let pad = "  ".repeat(depth);
            output.push_str(&pad);
            output.push('}');
            output.push('\n');
            i += 1;
        }
    }

    output
}

/// Simplify CSS values from raw CSS:
/// - `calc(<single_value>)` → `<single_value>` when calc has a single arg
/// - Remove spaces around `/` in slash-separated values
fn simplify_css_value(val: &str) -> String {
    // Split into property and value at the first colon
    // (but not inside parens)
    let mut colon_pos = None;
    let mut paren_depth: usize = 0;
    for (i, ch) in val.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            ':' if paren_depth == 0 => {
                colon_pos = Some(i);
                break;
            }
            _ => {}
        }
    }

    let (prop, value) = match colon_pos {
        Some(pos) => (&val[..pos], &val[pos + 1..]),
        None => ("", val),
    };

    let mut simplified = value.trim().to_string();

    // Simplify calc(single_value) → single_value
    // Only simplify when the inner value is a simple number with optional unit
    // (e.g. calc(1px) → 1px), not for function calls like calc(c())
    if simplified.starts_with("calc(") && simplified.ends_with(')') {
        let inner = &simplified[5..simplified.len() - 1];
        // Check if inner is a simple number (digits + optional unit, no operators or parens)
        let is_simple_number = !inner.is_empty()
            && inner.chars().next().map(|c| c.is_ascii_digit() || c == '.').unwrap_or(false)
            && !inner.contains('(')
            && !inner.contains(')');
        if is_simple_number {
            let mut pdepth: usize = 0;
            let mut has_op = false;
            for ch in inner.chars() {
                match ch {
                    '(' => pdepth += 1,
                    ')' => pdepth = pdepth.saturating_sub(1),
                    '+' | '-' | '*' | '/' if pdepth == 0 => has_op = true,
                    _ => {}
                }
            }
            if !has_op {
                simplified = inner.to_string();
            }
        }
    }

    // Remove spaces around `/` in slash-separated values (outside parens)
    let mut final_val = String::new();
    let mut pdepth: usize = 0;
    let chars: Vec<char> = simplified.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '(' {
            pdepth += 1;
            final_val.push(chars[i]);
        } else if chars[i] == ')' && pdepth > 0 {
            pdepth -= 1;
            final_val.push(chars[i]);
        } else if chars[i] == '/' && pdepth == 0 {
            while final_val.ends_with(' ') {
                final_val.pop();
            }
            final_val.push('/');
            i += 1;
            while i < chars.len() && chars[i] == ' ' {
                i += 1;
            }
            continue;
        } else {
            final_val.push(chars[i]);
        }
        i += 1;
    }

    if prop.is_empty() {
        final_val
    } else {
        format!("{}: {}", prop, final_val)
    }
}
