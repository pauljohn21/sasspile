//! Tests for semantic/module.

use sasspile::semantic::{
    CycleCheck, ModuleGraph, NamespaceRegistry,
};
use sasspile::source::SourceSpan;

#[test]
fn new_graph_is_empty() {
    let graph = ModuleGraph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.len(), 0);
}

#[test]
fn register_module() {
    let mut graph = ModuleGraph::new();
    let span = SourceSpan::new(0, 10);
    let url = graph.register("sass:color".to_string(), span);

    assert_eq!(url, "sass:color");
    assert_eq!(graph.len(), 1);

    let module = graph.get("sass:color");
    assert!(module.is_some());
    assert_eq!(module.unwrap().url, "sass:color");
}

#[test]
fn add_dependency_edge() {
    let mut graph = ModuleGraph::new();
    let span = SourceSpan::new(0, 10);

    graph.register("main".to_string(), span);
    graph.register("dep".to_string(), span);
    graph.add_dependency("main", "dep");

    let main = graph.get("main").unwrap();
    assert_eq!(main.dependencies.len(), 1);
    assert_eq!(main.dependencies[0], "dep");
}

#[test]
fn no_cycle_in_linear_graph() {
    let mut graph = ModuleGraph::new();
    let span = SourceSpan::new(0, 10);

    graph.register("a".to_string(), span);
    graph.register("b".to_string(), span);
    graph.register("c".to_string(), span);

    graph.add_dependency("a", "b");
    graph.add_dependency("b", "c");

    match graph.check_cycles_from("a") {
        CycleCheck::Clean => {}
        CycleCheck::Cycle(_) => panic!("expected no cycle"),
    }
}

#[test]
fn detect_cycle() {
    let mut graph = ModuleGraph::new();
    let span = SourceSpan::new(0, 10);

    graph.register("a".to_string(), span);
    graph.register("b".to_string(), span);
    graph.register("c".to_string(), span);

    graph.add_dependency("a", "b");
    graph.add_dependency("b", "c");
    graph.add_dependency("c", "a"); // Creates cycle

    match graph.check_all_cycles() {
        CycleCheck::Cycle(path) => {
            assert!(!path.is_empty());
        }
        CycleCheck::Clean => panic!("expected cycle"),
    }
}

#[test]
fn topological_sort_linear() {
    let mut graph = ModuleGraph::new();
    let span = SourceSpan::new(0, 10);

    graph.register("a".to_string(), span);
    graph.register("b".to_string(), span);
    graph.register("c".to_string(), span);

    graph.add_dependency("a", "b");
    graph.add_dependency("b", "c");

    let sorted = graph.topological_sort().expect("no cycle");
    assert_eq!(sorted.len(), 3);

    // c should come before b, b before a.
    let pos_c = sorted.iter().position(|s| s == "c").unwrap();
    let pos_b = sorted.iter().position(|s| s == "b").unwrap();
    let pos_a = sorted.iter().position(|s| s == "a").unwrap();
    assert!(pos_c < pos_b);
    assert!(pos_b < pos_a);
}

#[test]
fn topological_sort_fails_with_cycle() {
    let mut graph = ModuleGraph::new();
    let span = SourceSpan::new(0, 10);

    graph.register("a".to_string(), span);
    graph.register("b".to_string(), span);

    graph.add_dependency("a", "b");
    graph.add_dependency("b", "a");

    assert!(graph.topological_sort().is_err());
}

#[test]
fn reverse_dependencies() {
    let mut graph = ModuleGraph::new();
    let span = SourceSpan::new(0, 10);

    graph.register("a".to_string(), span);
    graph.register("b".to_string(), span);
    graph.register("c".to_string(), span);

    graph.add_dependency("b", "a");
    graph.add_dependency("c", "a");

    let rev = graph.reverse_dependencies("a");
    assert_eq!(rev.len(), 2);
    assert!(rev.contains(&"b".to_string()));
    assert!(rev.contains(&"c".to_string()));
}

#[test]
fn namespace_registry() {
    let mut reg = NamespaceRegistry::new();
    reg.register("math".to_string(), "sass:math".to_string());
    reg.register("col".to_string(), "sass:color".to_string());

    assert_eq!(reg.resolve("math"), Some(&"sass:math".to_string()));
    assert_eq!(reg.resolve("col"), Some(&"sass:color".to_string()));
    assert!(reg.resolve("unknown").is_none());
}

#[test]
fn duplicate_registration_idempotent() {
    let mut graph = ModuleGraph::new();
    let span = SourceSpan::new(0, 10);

    graph.register("mymod".to_string(), span);
    graph.register("mymod".to_string(), span); // Duplicate

    assert_eq!(graph.len(), 1);
}
