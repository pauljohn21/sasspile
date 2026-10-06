//! Reactive compilation pipeline entry points.

pub mod compile;
pub mod serializer;

pub use serializer::{serialize, serialize_to_string, Options, OutputStyle};
