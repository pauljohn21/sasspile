//! Builtin 函数——const 静态表 dispatch。
//!
//! 单一数据源：所有内建函数在编译期注册。
//! 无 proc-macro，无运行时反射。

pub mod dispatch;
mod math;
mod string;
mod map;
mod list;
mod color;
mod meta;
mod selector;
