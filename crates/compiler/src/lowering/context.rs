//! LoweringContext — 降级转换的上下文

/// 降级转换上下文
#[derive(Debug, Clone)]
pub struct LoweringContext {
    /// 祖先选择器栈
    pub selector_stack: Vec<String>,
}

impl LoweringContext {
    /// 创建初始上下文
    pub fn new() -> Self {
        Self {
            selector_stack: Vec::new(),
        }
    }

    /// 压入选择器
    pub fn push_selector(&mut self, selector: String) {
        self.selector_stack.push(selector);
    }

    /// 弹出选择器
    pub fn pop_selector(&mut self) -> Option<String> {
        self.selector_stack.pop()
    }

    /// 查看栈顶选择器
    pub fn current_selector(&self) -> Option<&str> {
        self.selector_stack.last().map(|s| s.as_str())
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}
