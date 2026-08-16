//! Incremental compilation — reactive state, dependency tracking, caching.
//!
//! Provides watch-based variable propagation, fine-grained dependency graphs,
//! source-span cache, and change propagation for efficient hot-reload.

pub mod cache;
pub mod depgraph;
pub mod env;
pub mod propagate;

pub use cache::SpanCache;
pub use depgraph::{DependencyGraph, NodeId};
pub use env::{ReactiveEnv, VarId};
pub use propagate::Propagator;
