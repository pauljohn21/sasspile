use rxrust::prelude::*;

#[derive(Clone)]
pub struct ForOp<S> {
    pub source: S,
}

#[derive(Clone)]
pub struct ForObserver<O> {
    observer: O,
}

impl<S> ObservableType for ForOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = S::Item<'a>
    where
        Self: 'a;
    type Err = S::Err;
}

impl<O, Item, Err> Observer<Item, Err> for ForObserver<O>
where
    O: Observer<Item, Err> + Send,
{
    fn next(&mut self, value: Item) {
        // TODO: @for 循环展开逻辑
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

impl<S, C> CoreObservable<C> for ForOp<S>
where
    C: Context,
    S: CoreObservable<C::With<ForObserver<C::Inner>>>,
    C::Inner: Send,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| ForObserver { observer });
        self.source.subscribe(wrapped)
    }
}
