//! 指令算子模块
//!
//! 每个 SCSS 指令是一个独立的 rxrust 自定义算子:
//! - use_   : @use (extend / forward / mixin 解析)
//! - mixin  : @mixin (定义可复用样式块)
//! - include: @include (BEM b/e/m 展开)
//! - if_    : @if (条件分支)
//! - for_   : @for (数值循环)
//! - each   : @each (列表/Map 遍历)

pub mod use_;
pub mod mixin;
pub mod include;
pub mod if_;
pub mod for_;
pub mod each;

use rxrust::prelude::*;

pub use self::use_::UseOp;
pub use self::mixin::MixinOp;
pub use self::include::IncludeOp;
pub use self::if_::IfOp;
pub use self::for_::ForOp;
pub use self::each::EachOp;

/// 扩展 trait: 为所有 Observable 提供 .use_() .mixin() .include() .if_() .for_() .each() 链式方法
pub trait DirectiveOps: Observable
where
    Self::Inner: ObservableType,
{
    fn use_(self) -> Self::With<UseOp<Self::Inner>> {
        self.transform(|source| UseOp { source })
    }

    fn mixin(self) -> Self::With<MixinOp<Self::Inner>> {
        self.transform(|source| MixinOp { source })
    }

    fn include(self) -> Self::With<IncludeOp<Self::Inner>> {
        self.transform(|source| IncludeOp { source })
    }

    fn if_(self) -> Self::With<IfOp<Self::Inner>> {
        self.transform(|source| IfOp { source })
    }

    fn for_(self) -> Self::With<ForOp<Self::Inner>> {
        self.transform(|source| ForOp { source })
    }

    fn each(self) -> Self::With<EachOp<Self::Inner>> {
        self.transform(|source| EachOp { source })
    }
}

impl<T> DirectiveOps for T
where
    T: Observable,
    T::Inner: ObservableType,
{
}
