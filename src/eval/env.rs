//! 不可变求值环境。
//!
//! 使用 `im::HashMap` 实现持久化数据结构。
//! `bind()` 返回新环境，旧环境不变。

use crate::parse::ast::Value;
use im::HashMap;

/// 求值环境——不可变变量绑定。
///
/// 使用持久化数据结构（`im::HashMap`）确保：
/// - 绑定变量返回新环境
/// - 旧环境保持不变
/// - O(1) 查找复杂度
#[derive(Debug, Clone, Default)]
pub struct Env {
    /// 变量绑定表。
    bindings: HashMap<String, Value>,
}

impl Env {
    /// 创建空环境。
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// 绑定变量——返回新环境，旧环境不变。
    pub fn bind(&self, name: String, value: Value) -> Self {
        let mut new_bindings = self.bindings.clone();
        new_bindings.insert(name, value);
        Self {
            bindings: new_bindings,
        }
    }

    /// 查找变量。
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_and_lookup() {
        let env = Env::new();
        let env2 = env.bind("x".to_string(), Value::Number(10.0, None));
        assert_eq!(env.lookup("x"), None); // 旧环境不变
        assert_eq!(env2.lookup("x"), Some(&Value::Number(10.0, None)));
    }

    #[test]
    fn test_nested_scope() {
        let env = Env::new();
        let env = env.bind("x".to_string(), Value::Number(1.0, None));
        let env2 = env.bind("y".to_string(), Value::Number(2.0, None));
        assert_eq!(env2.lookup("x"), Some(&Value::Number(1.0, None)));
        assert_eq!(env2.lookup("y"), Some(&Value::Number(2.0, None)));
    }
}
