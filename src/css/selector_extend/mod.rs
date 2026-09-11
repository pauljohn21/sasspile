//! —— 选择器 extend/replace 算法 ——

pub mod extend;
pub mod extend_build;
pub mod extend_complex;
pub mod extend_pseudo;
pub mod replace;

pub use extend::{extend_selector, extend_selector_with_mode};
pub use replace::replace_selector;
