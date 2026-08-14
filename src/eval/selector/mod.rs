//! 选择器解析与算法模块。
//!
//! 提供选择器的结构化表示和算法实现：
//! - `parse.rs`：选择器解析为结构化表示
//! - `algorithms.rs`：is-superselector/unify/extend 算法

pub mod algorithms;
pub mod parse;

pub use parse::{
    Combinator, ComplexSelector, CompoundSelector, CompoundWithCombinator, parse_selector_list,
};
