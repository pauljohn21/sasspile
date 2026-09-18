use std::marker::PhantomData;

use rxrust::prelude::*;

/// 指令标记: @for (数值循环展开)
pub struct For;

pub struct ForOp<S> {
    pub source: S,
    pub _instruction: PhantomData<fn() -> For>,
}

pub struct ForObserver<O> {
    pub observer: O,
    _instruction: PhantomData<fn() -> For>,
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
    O: Observer<Item, Err>,
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
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped =
            context.transform(|observer| ForObserver { observer, _instruction: PhantomData });
        self.source.subscribe(wrapped)
    }
}
