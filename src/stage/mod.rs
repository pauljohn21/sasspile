//! 编译阶段类型状态机。
//!
//! 每个阶段是一个新类型，阶段转换是该类型的方法。
//! 类型系统保证编译顺序不可颠倒。
//!
//! ```text
//! Source.lex() -> Lexed.parse() -> Parsed.evaluate() -> Evaluated.serialize() -> Serialized
//! ```

pub mod evaluated;
pub mod lexed;
pub mod parsed;
pub mod serialized;
pub mod source;
