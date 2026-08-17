//! Selector parsing — parse CSS selector strings into SelectorList.

use super::*;

/// Parse a CSS selector string into a SelectorList.
///
/// Handles:
/// - Type selectors: `div`, `span`
/// - Universal: `*`
/// - Class: `.foo`
/// - ID: `#bar`
/// - Attribute: `[type="text"]`
/// - Pseudo-class: `:hover`, `:nth-child(2n+1)`
/// - Pseudo-element: `::before`
/// - Placeholder: `%base`
/// - Combinators: ` ` (descendant), ` > ` (child), ` + ` (adjacent), ` ~ ` (sibling)
/// - Comma-separated lists
pub fn parse_selector_list(input: &str) -> SelectorList {
    let span = tracing::debug_span!("parse_selector", stage = "selector", input = %input);
    let _enter = span.enter();

    let mut list = SelectorList::new();
    let parts: Vec<&str> = split_top_level_commas(input);

    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let complex = parse_complex_selector(trimmed);
        list.add(complex);
    }

    tracing::trace!(stage = "selector", selector_count = list.selectors.len(), "parsed selectors");
    list
}

/// Parse a single complex selector (no commas).
fn parse_complex_selector(input: &str) -> ComplexSelector {
    // Tokenize into compound selectors and combinators
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut current = String::new();

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' => {
                if !current.is_empty() {
                    tokens.push(SelectorToken::Compound(current.clone()));
                    current.clear();
                }
                // Check if next is explicit combinator
                while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
                    chars.next();
                }
                if chars.peek() == Some(&'>') || chars.peek() == Some(&'+') || chars.peek() == Some(&'~') {
                    let comb_char = chars.next().unwrap();
                    while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
                        chars.next();
                    }
                    tokens.push(SelectorToken::Combinator(parse_combinator(comb_char)));
                } else if !tokens.is_empty() {
                    // Only add descendant if there's something before and after
                    tokens.push(SelectorToken::Combinator(Combinator::Descendant));
                }
            }
            '>' | '+' | '~' => {
                if !current.is_empty() {
                    tokens.push(SelectorToken::Compound(current.clone()));
                    current.clear();
                }
                while chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
                    chars.next();
                }
                tokens.push(SelectorToken::Combinator(parse_combinator(c)));
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(SelectorToken::Compound(current));
    }

    // Build ComplexSelector from tokens
    let mut iter = tokens.into_iter();
    let first_compound = match iter.next() {
        Some(SelectorToken::Compound(s)) => parse_compound_str(&s),
        _ => return ComplexSelector::new(CompoundSelector::new()),
    };
    let mut complex = ComplexSelector::new(first_compound);
    loop {
        let comb = match iter.next() {
            Some(SelectorToken::Combinator(c)) => c,
            Some(SelectorToken::Compound(_)) => continue, // shouldn't happen
            None => break,
        };
        let compound = match iter.next() {
            Some(SelectorToken::Compound(s)) => parse_compound_str(&s),
            _ => break,
        };
        complex.add(comb, compound);
    }
    complex
}

#[derive(Debug)]
enum SelectorToken {
    Compound(String),
    Combinator(Combinator),
}

/// Parse a compound selector from a string (e.g. "div.foo:first-child").
fn parse_compound_str(s: &str) -> CompoundSelector {
    let mut compound = CompoundSelector::new();
    let mut chars = s.chars().peekable();

    while chars.peek().is_some() {
        let c = *chars.peek().unwrap();
        match c {
            '.' | '#' | ':' | '[' | '%' | '*' => {
                let simple = parse_simple_selector(&mut chars);
                compound.add(simple);
            }
            _ if c.is_alphabetic() || c == '_' || c == '-' => {
                chars.next();
                let name = read_ident_from(&mut chars, c);
                compound.add(SimpleSelector::Type(name));
            }
            _ => {
                chars.next();
            }
        }
    }
    compound
}

/// Parse a combinator character.
fn parse_combinator(c: char) -> Combinator {
    match c {
        '>' => Combinator::Child,
        '+' => Combinator::AdjacentSibling,
        '~' => Combinator::GeneralSibling,
        _ => Combinator::Descendant,
    }
}

/// Parse a single simple selector starting with a special char.
fn parse_simple_selector(chars: &mut std::iter::Peekable<std::str::Chars>) -> SimpleSelector {
    let c = chars.next().unwrap();
    match c {
        '.' => {
            let name = read_ident(chars);
            SimpleSelector::Class(name)
        }
        '#' => {
            let name = read_ident(chars);
            SimpleSelector::Id(name)
        }
        '*' => SimpleSelector::Universal,
        '%' => {
            let name = read_ident(chars);
            SimpleSelector::Placeholder(name)
        }
        '[' => {
            // Attribute selector — read until ]
            let mut content = String::new();
            while let Some(ch) = chars.peek().cloned() {
                if ch == ']' {
                    chars.next();
                    break;
                }
                content.push(ch);
                chars.next();
            }
            SimpleSelector::Attribute(content)
        }
        ':' => {
            // Pseudo-class or pseudo-element
            let is_element = chars.peek() == Some(&':');
            if is_element {
                chars.next(); // consume second ':'
            }
            let name = read_ident(chars);
            let arg = parse_pseudo_args(chars);
            if is_element {
                SimpleSelector::PseudoElement(name, arg)
            } else {
                SimpleSelector::PseudoClass(name, arg)
            }
        }
        _ => SimpleSelector::Type(read_ident_from(chars, c)),
    }
}

/// Parse pseudo-class/element arguments in parens: (arg)
fn parse_pseudo_args(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    if chars.peek() != Some(&'(') {
        return None;
    }
    chars.next(); // consume '('
    let mut content = String::new();
    let mut depth = 1;
    while let Some(ch) = chars.peek().cloned() {
        chars.next();
        if ch == '(' {
            depth += 1;
            content.push(ch);
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                break;
            }
            content.push(ch);
        } else {
            content.push(ch);
        }
    }
    if content.is_empty() {
        None
    } else {
        Some(content)
    }
}

/// Read an identifier from the char stream.
fn read_ident(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut s = String::new();
    while let Some(c) = chars.peek().cloned() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }
    s
}

/// Read an identifier starting with an already-consumed char.
fn read_ident_from(chars: &mut std::iter::Peekable<std::str::Chars>, first: char) -> String {
    let mut s = String::new();
    s.push(first);
    while let Some(c) = chars.peek().cloned() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }
    s
}

/// Split a string on top-level commas (not inside brackets/parens).
fn split_top_level_commas(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth: i32 = 0;

    for (i, c) in input.chars().enumerate() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&input[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < input.len() {
        parts.push(&input[start..]);
    } else if start == input.len() && parts.is_empty() {
        parts.push(input);
    }
    parts
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_class() {
        let list = parse_selector_list(".foo");
        assert_eq!(list.selectors.len(), 1);
        assert_eq!(list.selectors[0].leading.components.len(), 1);
        assert_eq!(list.to_string(), ".foo");
    }

    #[test]
    fn test_parse_compound() {
        let list = parse_selector_list("div.foo#bar");
        assert_eq!(list.selectors[0].leading.components.len(), 3);
        assert_eq!(list.to_string(), "div.foo#bar");
    }

    #[test]
    fn test_parse_complex() {
        let list = parse_selector_list("div > .foo + p");
        assert_eq!(list.selectors.len(), 1);
        assert_eq!(list.selectors[0].rest.len(), 2);
        assert_eq!(list.to_string(), "div > .foo + p");
    }

    #[test]
    fn test_parse_list() {
        let list = parse_selector_list("div, .foo, #bar");
        assert_eq!(list.selectors.len(), 3);
    }

    #[test]
    fn test_parse_placeholder() {
        let list = parse_selector_list("%base");
        assert!(list.selectors[0].leading.has_placeholder());
    }

    #[test]
    fn test_parse_pseudo() {
        let list = parse_selector_list("a:hover");
        assert_eq!(list.to_string(), "a:hover");

        let list = parse_selector_list("::before");
        assert!(list.to_string().contains("::before"));

        let list = parse_selector_list(":nth-child(2n+1)");
        assert!(list.to_string().contains("nth-child"));
    }
}
