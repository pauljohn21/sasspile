//! CSS 序列化器 + 后处理。

mod node;
mod serialize;

pub use node::CssNode;
pub use serialize::{Serialized, OutputStyle};
