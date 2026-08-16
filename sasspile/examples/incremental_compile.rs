//! Incremental compilation — reactive environment, dependency graph, and span cache.
//!
//! Demonstrates the incremental compilation subsystem:
//! - ReactiveEnv: variable changes propagate via watch channels
//! - DependencyGraph: track which nodes depend on which sources
//! - SpanCache: cache compiled results with hit/miss tracking
//!
//! Usage:
//!   cargo run -p sasspile --example incremental_compile

use std::collections::HashMap;

use sasspile::incremental::depgraph::Dependency;
use sasspile::incremental::{DependencyGraph, ReactiveEnv, SpanCache};
use sasspile::source::SourceSpan;
use sasspile::value::{Quoted, Value};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("=== Incremental Compilation Example ===");

    // --- Reactive Environment ---
    tracing::info!("--- ReactiveEnv Demo ---");

    let env = ReactiveEnv::empty();

    // Subscribe to variable changes.
    let mut rx = env.subscribe();

    // Set variables — subscribers get notified.
    env.set_var(
        "theme",
        Value::String("dark".into(), Quoted::Unquoted),
    );
    env.set_var(
        "primary-color",
        Value::String("#3498db".into(), Quoted::Unquoted),
    );

    // Check initial values.
    let snapshot = rx.borrow();
    tracing::info!(
        var_count = snapshot.len(),
        "initial variable values set"
    );
    drop(snapshot);

    // Update a variable — watch channel notifies subscribers.
    env.set_var(
        "theme",
        Value::String("light".into(), Quoted::Unquoted),
    );

    // The subscriber sees the new value.
    rx.changed().await.unwrap();
    let snapshot = rx.borrow();
    tracing::info!(
        var_count = snapshot.len(),
        "after single update"
    );
    drop(snapshot);

    // Batch update multiple variables.
    let mut updates = HashMap::new();
    updates.insert(
        "theme".to_string(),
        Value::String("auto".into(), Quoted::Unquoted),
    );
    updates.insert(
        "font-size".to_string(),
        Value::String("16px".into(), Quoted::Unquoted),
    );
    env.batch_update(updates);

    rx.changed().await.unwrap();
    let snapshot = rx.borrow();
    tracing::info!(
        var_count = snapshot.len(),
        "after batch update"
    );
    drop(snapshot);

    tracing::info!("ReactiveEnv demo complete");

    // --- Dependency Graph ---
    tracing::info!("--- DependencyGraph Demo ---");

    let mut graph = DependencyGraph::new();

    // Register nodes that depend on variables.
    let node_a = graph.register_node();
    let node_b = graph.register_node();
    let node_c = graph.register_node();

    // Node A depends on "theme".
    graph.add_dependency(node_a, Dependency::Var("theme".to_string()));

    // Node B depends on "layout".
    graph.add_dependency(node_b, Dependency::Var("layout".to_string()));

    // Node C also depends on "theme".
    graph.add_dependency(node_c, Dependency::Var("theme".to_string()));

    // Query: which nodes depend on "theme"?
    let theme_dep = Dependency::Var("theme".to_string());
    let theme_dependents = graph.dependents_of(&theme_dep);
    tracing::info!(
        ?theme_dependents,
        "nodes depending on 'theme'"
    );

    // Query: which nodes are affected if "theme" changes?
    let affected = graph.affected_nodes(&[Dependency::Var("theme".to_string())]);
    tracing::info!(
        ?affected,
        "nodes affected by 'theme' change"
    );

    tracing::info!("DependencyGraph demo complete");

    // --- Span Cache ---
    tracing::info!("--- SpanCache Demo ---");

    let mut cache = SpanCache::new();

    // Create a source span for demonstration.
    let span_a = SourceSpan::new(0, 100);

    // Cache starts empty — first access is a miss.
    let v1 = cache.get(&span_a, 0);
    tracing::info!(hit = v1.is_some(), "cache lookup (should be miss)");

    // Put a value in the cache.
    cache.put(
        span_a,
        0,
        Value::String("cached_value_1".into(), Quoted::Unquoted),
    );

    // Now it's a hit.
    let v2 = cache.get(&span_a, 0);
    tracing::info!(hit = v2.is_some(), "cache lookup (should be hit)");

    // Invalidating a range clears overlapping entries.
    cache.invalidate_range(50, 150);

    let v3 = cache.get(&span_a, 0);
    tracing::info!(
        hit = v3.is_some(),
        "cache lookup after invalidation (should be miss)"
    );

    tracing::info!(hits = cache.hits(), misses = cache.misses(), "cache stats");
    tracing::info!("SpanCache demo complete");

    tracing::info!("=== Incremental compilation example complete ===");
}
