use std::marker::PhantomData;

use rxrust::prelude::*;

/// 指令标记: @use (含 extend / forward / mixin 解析)
pub struct Use;

/// 算子壳子
pub struct UseOp<S> {
    pub source: S,
    pub _instruction: PhantomData<fn() -> Use>,
}

/// Observer 包装 — next 里写 @use 逻辑
pub struct UseObserver<O> {
    pub observer: O,
    _instruction: PhantomData<fn() -> Use>,
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
    O: Observer<Item, Err>,
{
    fn next(&mut self, value: Item) {
        // TODO: @use 逻辑 — extend / forward / mixin 解析
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
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped =
            context.transform(|observer| UseObserver { observer, _instruction: PhantomData });
        self.source.subscribe(wrapped)
    }
}
