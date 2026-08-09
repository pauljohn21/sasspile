//! 阶段 5: Serialized —— 最终 CSS 字符串。

/// 序列化产物——最终 CSS 输出。
#[derive(Debug, Clone)]
pub struct Serialized {
    /// CSS 文本。
    pub css: String,
}

impl Serialized {
    /// 获取 CSS 字符串引用。
    pub fn as_str(&self) -> &str {
        &self.css
    }

    /// 消费自身，返回 CSS 字符串。
    pub fn into_string(self) -> String {
        self.css
    }
}

impl AsRef<str> for Serialized {
    fn as_ref(&self) -> &str {
        &self.css
    }
}

impl std::fmt::Display for Serialized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.css)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialized_display() {
        let s = Serialized {
            css: "a{color:red;}".to_string(),
        };
        assert_eq!(format!("{}", s), "a{color:red;}");
    }

    #[test]
    fn test_serialized_as_ref() {
        let s = Serialized {
            css: "test".to_string(),
        };
        assert_eq!(s.as_ref(), "test");
    }
}
