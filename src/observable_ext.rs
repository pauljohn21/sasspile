use rxrust::prelude::*;
use rxrust::observer::Observer;
use rxrust::context::MutArc;
use std::sync::mpsc;
use crate::types::CompileError;

pub trait ObservablePipe: Observable + Sized {
    fn pipe<Stage, Out>(self, stage: Stage) -> Out
    where
        Stage: FnOnce(Self) -> Out,
    {
        stage(self)
    }
}

impl<T: Observable> ObservablePipe for T {}

/// 同步收集 boxed Observable<T> 为 Vec<T> —— 终端收集器
///
/// 因为 collect 终止算子的 subscribe 单参数版本仅兼容 Infallible 错误类型，
/// 而我们的管道已升级到 CompileError，所以使用 MutArc<Option<O>> 包装
/// 自定义 Observer，通过 subscribe_with 处理 collect 算子的输出和错误通道。
pub fn collect_boxed<T: Send + 'static>(
    stream: SharedBoxedObservable<'static, T, CompileError>,
) -> Result<Vec<T>, CompileError> {
    let (tx, rx) = mpsc::channel::<Vec<T>>();
    let (err_tx, err_rx) = mpsc::channel::<CompileError>();

    struct TermObserver<T: Send> {
        tx: mpsc::Sender<Vec<T>>,
        err_tx: mpsc::Sender<CompileError>,
    }

    impl<T: Send> Observer<Vec<T>, CompileError> for TermObserver<T> {
        fn next(&mut self, value: Vec<T>) {
            let _ = self.tx.send(value);
        }
        fn error(self, err: CompileError) {
            let _ = self.err_tx.send(err);
        }
        fn complete(self) {}
        fn is_closed(&self) -> bool { false }
    }

    let observer = TermObserver { tx, err_tx };
    // 使用 MutArc 包装以满足 RcDerefMut trait bound
    stream.collect::<Vec<_>>().subscribe_with(MutArc::from(Some(observer)));

    if let Ok(e) = err_rx.try_recv() {
        return Err(e);
    }
    rx.recv().map_err(|_| {
        err_rx.recv().unwrap_or(CompileError::Eval("channel disconnected".to_string()))
    })
}
