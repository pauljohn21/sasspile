use std::collections::HashMap;
use std::sync::OnceLock;

use rxrust::prelude::*;

#[derive(Clone)]
pub struct UseOp<S> {
    pub source: S,
}

#[derive(Clone)]
pub struct UseObserver<O> {
    observer: O,
    modules: HashMap<String, Module>,
    forwarded: HashMap<String, Module>,
}

#[derive(Debug, Clone, Default)]
pub struct Module {
    pub variables: HashMap<String, String>,
}

impl Module {
    pub fn parse_from_content(content: &str) -> Self {
        let mut module = Self::default();
        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("$") {
                if let Some((name, value)) = rest.split_once(":") {
                    let name = format!("${}", name.trim());
                    let value = value
                        .trim()
                        .trim_end_matches("!default")
                        .trim_end()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    module.variables.insert(name, value);
                }
            }
        }
        module
    }

    pub fn with_overrides(&self, overrides: &HashMap<String, String>) -> Self {
        let mut module = self.clone();
        for (key, value) in overrides {
            module.variables.insert(key.clone(), value.clone());
        }
        module
    }
}

fn module_registry() -> &'static HashMap<String, String> {
    static REGISTRY: OnceLock<HashMap<String, String>> = OnceLock::new();
    REGISTRY.get_or_init(HashMap::new)
}

impl<O> UseObserver<O> {
    fn parse_path_from_token(token: &str, prefix: &str) -> Option<String> {
        let after_prefix = token.strip_prefix(prefix)?.trim();
        let path = after_prefix
            .split_whitespace()
            .find(|t| !["as", "*", "with"].contains(t) && !t.starts_with('('))?;
        Some(path.to_string())
    }

    fn parse_with_clause(token: &str) -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        if let Some(start) = token.find('(') {
            if let Some(end) = token.rfind(')') {
                let params = &token[start + 1..end];
                for pair in params.split(',') {
                    if let Some((name, value)) = pair.split_once(':') {
                        overrides.insert(name.trim().to_string(), value.trim().to_string());
                    }
                }
            }
        }
        overrides
    }

    fn load_module(path: &str) -> Module {
        if let Some(content) = module_registry().get(path) {
            Module::parse_from_content(content)
        } else {
            Module::default()
        }
    }
}

impl<S> ObservableType for UseOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = S::Item<'a>
    where
        Self: 'a;
    type Err = S::Err;
}

impl<O, Item, Err> Observer<Item, Err> for UseObserver<O>
where
    O: Observer<Item, Err> + Send,
    Item: ToString,
{
    fn next(&mut self, value: Item) {
        let val_str = value.to_string();

        if val_str.starts_with("@use") {
            if let Some(path) = Self::parse_path_from_token(&val_str, "@use") {
                let module = Self::load_module(&path);
                let overrides = Self::parse_with_clause(&val_str);
                let module = module.with_overrides(&overrides);
                self.modules.insert(path, module);
            }
            return;
        }

        if val_str.starts_with("@forward") {
            if let Some(path) = Self::parse_path_from_token(&val_str, "@forward") {
                let module = Self::load_module(&path);
                self.forwarded.insert(path, module);
            }
            return;
        }

        self.observer.next(value);
    }

    fn error(self, err: Err) {
        self.observer.error(err);
    }

    fn complete(self) {
        self.observer.complete();
    }

    fn is_closed(&self) -> bool {
        self.observer.is_closed()
    }
}

impl<S, C> CoreObservable<C> for UseOp<S>
where
    C: Context,
    S: CoreObservable<C::With<UseObserver<C::Inner>>>,
    C::Inner: Send,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| UseObserver {
            observer,
            modules: HashMap::new(),
            forwarded: HashMap::new(),
        });
        self.source.subscribe(wrapped)
    }
}
