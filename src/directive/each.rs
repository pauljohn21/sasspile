use rxrust::prelude::*;

#[derive(Clone)]
pub struct EachOp<S> {
    pub source: S,
}

#[derive(Clone)]
pub struct EachObserver<O> {
    observer: O,
}

impl<S> ObservableType for EachOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = S::Item<'a>
    where
        Self: 'a;
    type Err = S::Err;
}

impl<O, Item, Err> Observer<Item, Err> for EachObserver<O>
where
    O: Observer<Item, Err> + Send,
{
    fn next(&mut self, value: Item) {
        // TODO: @each 遍历展开逻辑
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

impl<S, C> CoreObservable<C> for EachOp<S>
where
    C: Context,
    S: CoreObservable<C::With<EachObserver<C::Inner>>>,
    C::Inner: Send,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| EachObserver { observer });
        self.source.subscribe(wrapped)
    }
}
