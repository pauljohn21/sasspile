//! ModuleCache — @import / @use 模块缓存

use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct ModuleCache {
    entries: HashMap<String, String>,
}

impl ModuleCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&self, path: String, content: String) -> Self {
        let mut cache = self.clone();
        cache.entries.insert(path, content);
        cache
    }

    pub fn get(&self, path: &str) -> Option<&String> {
        self.entries.get(path)
    }

    pub fn with_entry(self, path: String, content: String) -> Self {
        self.insert(path, content)
    }
}
