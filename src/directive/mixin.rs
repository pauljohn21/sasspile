use std::marker::PhantomData;

use rxrust::prelude::*;

/// 指令标记: @mixin (定义可复用样式块)
pub struct Mixin;

/// 算子壳子
pub struct MixinOp<S> {
    pub source: S,
    pub _instruction: PhantomData<fn() -> Mixin>,
}

/// Observer 包装 — next 里写 @mixin 定义逻辑
pub struct MixinObserver<O> {
    pub observer: O,
    _instruction: PhantomData<fn() -> Mixin>,
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

impl<O, Item, Err> Observer<Item, Err> for MixinObserver<O>
where
    O: Observer<Item, Err>,
{
    fn next(&mut self, value: Item) {
        // TODO: @mixin 定义逻辑
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

impl<S, C> CoreObservable<C> for MixinOp<S>
where
    C: Context,
    S: CoreObservable<C::With<MixinObserver<C::Inner>>>,
{
    type Unsub = S::Unsub;

    fn subscribe(self, context: C) -> Self::Unsub {
        let wrapped = context.transform(|observer| MixinObserver {
            observer,
            _instruction: PhantomData,
        });
        self.source.subscribe(wrapped)
    }
}
