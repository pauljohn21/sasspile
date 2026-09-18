use rxrust::prelude::*;

#[derive(Clone)]
pub struct IfOp<S> {
    pub source: S,
}

#[derive(Clone)]
pub struct IfObserver<O> {
    observer: O,
}

impl<S> ObservableType for IfOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = S::Item<'a>
    where
        Self: 'a;
    type Err = S::Err;
}

impl<O, Item, Err> Observer<Item, Err> for IfObserver<O>
where
    O: Observer<Item, Err> + Send,
{
    fn next(&mut self, value: Item) {
        // TODO: @if 条件求值逻辑
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

impl<S, C> CoreObservable<C> for IfOp<S>
where
    C: Context,
    S: CoreObservable<C::With<IfObserver<C::Inner>>>,
    C::Inner: Send,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| IfObserver { observer });
        self.source.subscribe(wrapped)
    }
}
