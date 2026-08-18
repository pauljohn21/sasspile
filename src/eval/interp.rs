//! Interpolation evaluation — handles `#{...}` in selectors, properties, etc.

use crate::env::Env;
use crate::error::SassError;
use super::css::value_to_css;
use super::expr;

/// Resolve `&` parent selector and nest selectors.
pub fn resolve_selector(selector: &str, parent: &[String]) -> String {
    if parent.is_empty() {
        return selector.to_string();
    }
    let parent_sel = parent.last().unwrap();
    if selector.contains('&') {
        selector.replace('&', parent_sel)
    } else {
        format!("{} {}", parent_sel, selector)
    }
}

/// Evaluate `#{...}` interpolation in a string (selector, property name, etc.).
pub fn eval_interpolation_in_str(
    s: &str,
    env: &mut Env,
    parent_sel: &[String],
) -> Result<String, SassError> {
    if !s.contains("#{") {
        return Ok(s.to_string());
    }
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '#' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut depth = 1;
            let mut expr_str = String::new();
            while let Some(ch) = chars.next() {
                if ch == '{' {
                    depth += 1;
                    expr_str.push(ch);
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    expr_str.push(ch);
                } else {
                    expr_str.push(ch);
                }
            }
            let val = eval_interpolation_expr(&expr_str, env, parent_sel)?;
            result.push_str(&val);
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

/// Evaluate a simple expression string (used for interpolation contexts).
fn eval_interpolation_expr(
    expr_str: &str,
    env: &mut Env,
    parent_sel: &[String],
) -> Result<String, SassError> {
    let trimmed = expr_str.trim();
    // Simple variable reference: $name
    if trimmed.starts_with('$') {
        let var_name = trimmed[1..].trim();
        match env.get_var(var_name) {
            Some(val) => return Ok(value_to_css(val)),
            None => return Err(SassError::eval(
                format!("Undefined variable: ${}", var_name),
                crate::error::SourcePos::default(),
            )),
        }
    }
    // Simple identifier
    if trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        match trimmed {
            "true" => return Ok("true".to_string()),
            "false" => return Ok("false".to_string()),
            "null" => return Ok(String::new()),
            _ => {
                if let Some(val) = env.get_var(trimmed) {
                    return Ok(value_to_css(val));
                }
                return Ok(trimmed.to_string());
            }
        }
    }
    // Fall back to tokenizing and parsing as a full expression
    let tokens = crate::lexer::tokenize(trimmed, "interpolation")?;
    let mut parser = crate::parser::Parser::new(tokens);
    let mut expr_parser = crate::parser::expr::ExprParser::new(&mut parser);
    let expr = expr_parser.parse_expr()?;
    let val = expr::eval_expr(&expr, env, parent_sel)?;
    Ok(value_to_css(&val))
}
