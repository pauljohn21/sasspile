use std::collections::HashMap;
use std::convert::Infallible;

use rxrust::prelude::*;

#[derive(Clone)]
pub struct MixinOp<S> {
    pub source: S,
}

#[derive(Clone)]
pub struct MixinObserver<O> {
    observer: O,
    buffer: Vec<String>,
    defs: HashMap<String, Vec<String>>,
}

impl<S> ObservableType for MixinOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = S::Item<'a>
    where
        Self: 'a;
    type Err = S::Err;
}

impl<O> Observer<String, Infallible> for MixinObserver<O>
where
    O: Observer<String, Infallible> + Send,
{
    fn next(&mut self, value: String) {
        if value == "@mixin" || value == "@include" {
            self.buffer.clear();
            self.buffer.push(value);
            return;
        }

        if !self.buffer.is_empty() {
            let is_end = value == "}" || value == ";";
            self.buffer.push(value);

            if is_end {
                let stmt: Vec<String> = self.buffer.drain(..).collect();
                for tok in self.process(&stmt) {
                    if !self.observer.is_closed() {
                        self.observer.next(tok);
                    }
                }
            }
            return;
        }

        self.observer.next(value);
    }

    fn error(self, err: Infallible) {
        match err {}
    }

    fn complete(self) {
        self.observer.complete();
    }

    fn is_closed(&self) -> bool {
        self.observer.is_closed()
    }
}

impl<O> MixinObserver<O> {
    fn process(&mut self, stmt: &[String]) -> Vec<String> {
        if stmt.is_empty() {
            return vec![];
        }

        match stmt[0].as_str() {
            "@mixin" => {
                if let Some(name_idx) = stmt.iter().position(|t| t != "@mixin" && !t.is_empty()) {
                    let name = stmt[name_idx].clone();
                    let body: Vec<String> = stmt[name_idx + 1..]
                        .iter()
                        .filter(|t| *t != "{" && *t != "}")
                        .cloned()
                        .collect();
                    self.defs.insert(name, body);
                }
                vec![]
            }
            "@include" => {
                if let Some(name_idx) = stmt.iter().position(|t| t != "@include" && !t.is_empty()) {
                    let name = &stmt[name_idx];
                    if let Some(body) = self.defs.get(name) {
                        return body.clone();
                    }
                }
                vec![]
            }
            _ => stmt.to_vec(),
        }
    }
}

impl<S, C> CoreObservable<C> for MixinOp<S>
where
    C: Context,
    S: CoreObservable<C::With<MixinObserver<C::Inner>>>,
    C::Inner: Send,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| MixinObserver {
            observer,
            buffer: Vec::new(),
            defs: HashMap::new(),
        });
        self.source.subscribe(wrapped)
    }
}
