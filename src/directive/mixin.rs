use std::collections::HashMap;
use std::convert::Infallible;

use rxrust::prelude::*;
use tracing::{info_span, Span};

#[derive(Clone)]
pub struct MixinOp<S> {
    pub source: S,
}

#[derive(Clone, Debug, Default)]
pub struct MixinState {
    defs: HashMap<String, MixinDef>,
    capturing: Option<CaptureState>,
}

#[derive(Clone, Debug)]
pub struct CaptureState {
    name: String,
    params: Vec<(String, Option<String>)>,
    body: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct MixinDef {
    params: Vec<(String, Option<String>)>,
    body: Vec<String>,
}

fn parse_mixin_sig(s: &str) -> Option<(String, Vec<(String, Option<String>)>)> {
    let (name, rest) = s.split_once('(')?;
    let name = name.trim().to_string();
    let rest = rest.trim().trim_end_matches(')').trim();
    let params = if rest.is_empty() {
        vec![]
    } else {
        rest.split(',')
            .map(|param| {
                let p = param.trim();
                p.split_once(':')
                    .map(|(n, d)| (n.trim().to_string(), Some(d.trim().to_string())))
                    .unwrap_or_else(|| (p.to_string(), None))
            })
            .collect()
    };
    Some((name, params))
}

fn parse_include_sig(s: &str) -> (String, Vec<String>) {
    match s.split_once('(') {
        Some((name, rest)) => {
            let name = name.trim().to_string();
            let rest = rest.trim().trim_end_matches(')').trim();
            let args = if rest.is_empty() {
                vec![]
            } else {
                rest.split(',').map(|a| a.trim().to_string()).collect()
            };
            (name, args)
        }
        None => (s.trim().to_string(), vec![]),
    }
}

fn substitute(body: &[String], params: &[(String, Option<String>)], args: &[String]) -> Vec<String> {
    let replacements: HashMap<String, String> = params
        .iter()
        .enumerate()
        .map(|(i, (param, default))| {
            let value = args.get(i).cloned().or_else(|| default.clone()).unwrap_or_default();
            (param.clone(), value)
        })
        .collect();

    body.iter()
        .map(|token| replacements.iter().fold(token.clone(), |acc, (p, v)| acc.replace(p, v)))
        .collect()
}

impl<S> ObservableType for MixinOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = String
    where
        Self: 'a;
    type Err = S::Err;
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
            state: MixinState::default(),
        });
        self.source.subscribe(wrapped)
    }
}

#[derive(Clone)]
pub struct MixinObserver<O> {
    observer: O,
    state: MixinState,
}

impl<O> Observer<String, Infallible> for MixinObserver<O>
where
    O: Observer<String, Infallible> + Send,
{
    fn next(&mut self, value: String) {
        let span = info_span!("mixin_op", token = %value);
        let _enter = span.enter();

        let state = self.state.clone();
        let (defs, capturing, outputs) = match state.capturing {
            Some(cap) => handle_capturing(state.defs, cap, value),
            None => handle_idle(state.defs, value),
        };
        self.state = MixinState { defs, capturing };
        outputs.into_iter().for_each(|tok| {
            if !self.observer.is_closed() {
                self.observer.next(tok);
            }
        });
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

fn handle_capturing(
    mut defs: HashMap<String, MixinDef>,
    mut cap: CaptureState,
    token: String,
) -> (HashMap<String, MixinDef>, Option<CaptureState>, Vec<String>) {
    match token.trim() {
        "}" => {
            defs.insert(cap.name, MixinDef { params: cap.params, body: cap.body });
            (defs, None, vec![])
        }
        "{" => (defs, Some(cap), vec![]),
        _ => {
            cap.body.push(token);
            (defs, Some(cap), vec![])
        }
    }
}

fn handle_idle(
    mut defs: HashMap<String, MixinDef>,
    token: String,
) -> (HashMap<String, MixinDef>, Option<CaptureState>, Vec<String>) {
    let trimmed = token.trim();

    if let Some(rest) = trimmed.strip_prefix("@mixin") {
        if let Some((name, params)) = parse_mixin_sig(rest.trim()) {
            return (defs, Some(CaptureState { name, params, body: vec![] }), vec![]);
        }
        return (defs, None, vec![]);
    }

    if let Some(rest) = trimmed.strip_prefix("@include") {
        let (name, args) = parse_include_sig(rest.trim());
        let output = defs
            .get(&name)
            .map(|def| substitute(&def.body, &def.params, &args))
            .unwrap_or_default();
        return (defs, None, output);
    }

    (defs, None, vec![token])
}
