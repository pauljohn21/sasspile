use std::marker::PhantomData;

use rxrust::prelude::*;

/// 指令标记: @each (列表/Map 遍历展开)
pub struct Each;

pub struct EachOp<S> {
    pub source: S,
    pub _instruction: PhantomData<fn() -> Each>,
}

pub struct EachObserver<O> {
    pub observer: O,
    _instruction: PhantomData<fn() -> Each>,
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
    O: Observer<Item, Err>,
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
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped =
            context.transform(|observer| EachObserver { observer, _instruction: PhantomData });
        self.source.subscribe(wrapped)
    }
}
