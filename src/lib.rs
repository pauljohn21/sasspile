mod directive;

use rxrust::prelude::*;
use directive::DirectiveOps;

pub fn compile(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let (tx, rx) = std::sync::mpsc::channel();

    Local::from_iter(input.chars().collect::<Vec<_>>())
        // ① tokenize — 过滤空白
        .filter_map(|ch| if ch.is_whitespace() { None } else { Some(ch) })
        // ② parse — 状态累积，遇分隔符发射 token
        .scan_map(String::new(), |buf, ch| {
            if ch == ';' || ch == '{' || ch == '}' {
                let token = buf.drain(..).collect::<String>();
                buf.push(ch);
                if token.is_empty() {
                    vec![ch.to_string()]
                } else {
                    vec![token, ch.to_string()]
                }
            } else {
                buf.push(ch);
                vec![]
            }
        })
        .flat_map(|tokens| Local::from_iter(tokens))
        // ③-⑧ 指令展开 — 链式自定义算子
        .use_()
        .mixin()
        .include()
        .if_()
        .for_()
        .each()
        // ⑨ serialize — token → CSS 片段
        .map(|token| token + "\n")
        // 收集者 → 最终 CSS String
        .collect::<String>()
        .last()
        .subscribe(|s| {
            let _ = tx.send(s);
        });

    rx.recv().unwrap_or_default()
}

pub fn compile_parallel(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let (tx, rx) = std::sync::mpsc::channel();

    Local::from_iter(input.chars().collect::<Vec<_>>())
        .filter_map(|ch| if ch.is_whitespace() { None } else { Some(ch) })
        .scan_map(String::new(), |buf, ch| {
            if ch == ';' || ch == '{' || ch == '}' {
                let token = buf.drain(..).collect::<String>();
                buf.push(ch);
                if token.is_empty() {
                    vec![ch.to_string()]
                } else {
                    vec![token, ch.to_string()]
                }
            } else {
                buf.push(ch);
                vec![]
            }
        })
        .flat_map(|tokens| Local::from_iter(tokens))
        .use_()
        .mixin()
        .include()
        .if_()
        .for_()
        .each()
        .map(|token| token + "\n")
        .collect::<String>()
        .last()
        .subscribe(|s| {
            let _ = tx.send(s);
        });

    rx.recv().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_empty() {
        assert_eq!(compile(""), "");
    }

    #[test]
    fn test_compile_with_delimiter() {
        assert_eq!(compile("a;b"), "a\n;\n");
    }

    #[test]
    fn test_compile_parallel_empty() {
        assert_eq!(compile_parallel(""), "");
    }

    #[test]
    fn test_compile_parallel_with_delimiter() {
        assert_eq!(compile_parallel("a;b"), "a\n;\n");
    }
}
