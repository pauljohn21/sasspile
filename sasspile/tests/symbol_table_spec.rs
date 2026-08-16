//! Tests for semantic/symbol_table.

use sasspile::semantic::{
    Scope, ScopeKind, SymbolEntry, SymbolTable,
};
use sasspile::source::SourceSpan;
use sasspile::value::Value;

#[test]
fn new_table_has_global_scope() {
    let table = SymbolTable::new();
    assert_eq!(table.depth(), 1);
    assert!(table.is_global());
    assert_eq!(table.current_kind(), ScopeKind::Global);
}

#[test]
fn push_and_pop_scopes() {
    let mut table = SymbolTable::new();

    table.push_local();
    assert_eq!(table.depth(), 2);
    assert_eq!(table.current_kind(), ScopeKind::Local);

    table.push_param();
    assert_eq!(table.depth(), 3);
    assert_eq!(table.current_kind(), ScopeKind::Param);

    let popped = table.pop();
    assert_eq!(popped.kind, ScopeKind::Param);
    assert_eq!(table.depth(), 2);

    table.pop();
    assert!(table.is_global());
}

#[test]
fn define_and_lookup_in_current_scope() {
    let mut table = SymbolTable::new();
    let span = SourceSpan::new(0, 10);
    let entry = SymbolEntry::new(Some(Value::Null), span);

    table.define_current("$color".to_string(), entry);

    assert!(table.is_defined_in_current("$color"));
    let found = table.lookup("$color");
    assert!(found.is_some());
}

#[test]
fn lookup_through_scope_chain() {
    let mut table = SymbolTable::new();
    let span = SourceSpan::new(0, 10);
    table.define_current(
        "$global".to_string(),
        SymbolEntry::new(Some(Value::Null), span),
    );

    table.push_local();
    table.define_current(
        "$local".to_string(),
        SymbolEntry::new(Some(Value::Null), span),
    );

    // Should find both local and global from inner scope.
    assert!(table.lookup("$local").is_some());
    assert!(table.lookup("$global").is_some());
}

#[test]
fn shadowing_in_inner_scope() {
    let mut table = SymbolTable::new();
    let span = SourceSpan::new(0, 10);

    let first = SymbolEntry::new(Some(Value::Null), span);
    table.define_current("$x".to_string(), first);

    table.push_local();
    // In inner scope, lookup still finds $x from global (lexical scoping).
    assert!(table.lookup("$x").is_some());

    // Defining $x in inner scope does NOT affect the global binding --
    // define_current only replaces in the current (innermost) scope.
    let second = SymbolEntry::new(Some(Value::Null), span);
    let old = table.define_current("$x".to_string(), second);

    // No previous binding in current scope.
    assert!(old.is_none());

    // After popping local, global $x is unchanged.
    table.pop();
    assert!(table.lookup("$x").is_some());
}

#[test]
fn mutable_entry() {
    let span = SourceSpan::new(0, 10);
    let entry = SymbolEntry::mutable(Some(Value::Null), span);
    assert!(entry.is_mutable);

    let immutable = SymbolEntry::new(Some(Value::Null), span);
    assert!(!immutable.is_mutable);
}

#[test]
fn scope_constructor() {
    let global = Scope::global();
    assert_eq!(global.kind, ScopeKind::Global);
    assert!(global.bindings.is_empty());
}

#[test]
fn lookup_nonexistent_returns_none() {
    let table = SymbolTable::new();
    assert!(table.lookup("$undefined").is_none());
}

#[test]
fn lookup_mut_current_scope() {
    let mut table = SymbolTable::new();
    let span = SourceSpan::new(0, 10);
    table.define_current(
        "$var".to_string(),
        SymbolEntry::mutable(Some(Value::Null), span),
    );

    let entry = table.lookup_mut("$var");
    assert!(entry.is_some());
    assert!(entry.unwrap().is_mutable);
}
