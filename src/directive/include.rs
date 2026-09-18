use std::marker::PhantomData;

use rxrust::prelude::*;

/// 指令标记: @include (展开 mixin 调用, BEM b/e/m)
pub struct Include;

pub struct IncludeOp<S> {
    pub source: S,
    pub _instruction: PhantomData<fn() -> Include>,
}

pub struct IncludeObserver<O> {
    pub observer: O,
    _instruction: PhantomData<fn() -> Include>,
}

impl<S> ObservableType for IncludeOp<S>
where
    S: ObservableType,
{
    type Item<'a>
        = S::Item<'a>
    where
        Self: 'a;
    type Err = S::Err;
}

impl<O, Item, Err> Observer<Item, Err> for IncludeObserver<O>
where
    O: Observer<Item, Err>,
{
    fn next(&mut self, value: Item) {
        // TODO: @include 展开逻辑 (BEM b/e/m)
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

impl<S, C> CoreObservable<C> for IncludeOp<S>
where
    C: Context,
    S: CoreObservable<C::With<IncludeObserver<C::Inner>>>,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| IncludeObserver {
            observer,
            _instruction: PhantomData,
        });
        self.source.subscribe(wrapped)
    }
}
