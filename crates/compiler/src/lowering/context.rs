//! LoweringContext — 降级转换的上下文

use std::collections::HashMap;

/// 降级转换上下文
#[derive(Debug, Clone)]
pub struct LoweringContext {
    /// 祖先选择器栈
    pub selector_stack: Vec<String>,
    /// 变量环境：变量名 → 值
    pub variables: HashMap<String, String>,
}

impl LoweringContext {
    /// 创建初始上下文
    pub fn new() -> Self {
        Self {
            selector_stack: Vec::new(),
            variables: HashMap::new(),
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

    /// 获取完整的祖先选择器路径（空格分隔的多级嵌套）
    pub fn ancestor_path(&self) -> String {
        self.selector_stack.join(" ")
    }

    /// 检查变量是否已存在
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 获取变量值
    pub fn get_variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}
