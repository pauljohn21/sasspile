use rxrust::prelude::*;

pub trait ObservablePipe: Observable + Sized {
    fn pipe<Stage, Out>(self, stage: Stage) -> Out
    where
        Stage: FnOnce(Self) -> Out,
    {
        stage(self)
    }
}

impl<T: Observable> ObservablePipe for T {}
