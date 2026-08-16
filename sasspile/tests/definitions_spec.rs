//! Tests for semantic/definitions.

use sasspile::diagnostics::Diagnostics;
use sasspile::{FunctionDef, MixinDef, Param};
use sasspile::semantic::DefinitionRegistry;

#[test]
fn new_registry_is_empty() {
    let reg = DefinitionRegistry::new();
    assert!(reg.is_empty());
    assert_eq!(reg.len(), 0);
}

#[test]
fn register_function() {
    let mut reg = DefinitionRegistry::new();
    let mut diags = Diagnostics::new();

    let func = FunctionDef {
        name: "double".to_string(),
        params: vec![Param {
            name: "$x".to_string(),
            default: None,
        }],
        body: vec![],
    };

    let result = reg.register_function(&func, &mut diags);
    assert!(result.is_ok());
    assert!(reg.has_function("double"));
    assert!(!reg.has_mixin("double"));
    assert_eq!(reg.function_count(), 1);

    let entry = reg.get_function("double").unwrap();
    assert_eq!(entry.required_params, 1);
    assert_eq!(entry.total_params, 1);
    assert!(!entry.variadic);
}

#[test]
fn register_mixin_with_defaults() {
    let mut reg = DefinitionRegistry::new();
    let mut diags = Diagnostics::new();

    let mixin = MixinDef {
        name: "box".to_string(),
        params: vec![
            Param {
                name: "$w".to_string(),
                default: None,
            },
            Param {
                name: "$h".to_string(),
                default: None,
            },
            Param {
                name: "$color".to_string(),
                default: None,
            },
        ],
        body: vec![],
    };

    reg.register_mixin(&mixin, &mut diags).unwrap();
    let entry = reg.get_mixin("box").unwrap();
    assert_eq!(entry.required_params, 3);
    assert_eq!(entry.total_params, 3);
}

#[test]
fn duplicate_function_warning() {
    let mut reg = DefinitionRegistry::new();
    let mut diags = Diagnostics::new();

    let func1 = FunctionDef {
        name: "f".to_string(),
        params: vec![],
        body: vec![],
    };
    let func2 = FunctionDef {
        name: "f".to_string(),
        params: vec![Param {
            name: "$a".to_string(),
            default: None,
        }],
        body: vec![],
    };

    reg.register_function(&func1, &mut diags).unwrap();
    reg.register_function(&func2, &mut diags).unwrap();

    // Should have generated a warning for redefinition.
    assert!(!diags.is_empty());
}

#[test]
fn validate_function_call_ok() {
    let mut reg = DefinitionRegistry::new();
    let mut diags = Diagnostics::new();

    let func = FunctionDef {
        name: "add".to_string(),
        params: vec![
            Param {
                name: "$a".to_string(),
                default: None,
            },
            Param {
                name: "$b".to_string(),
                default: None,
            },
        ],
        body: vec![],
    };
    reg.register_function(&func, &mut diags).unwrap();

    diags = Diagnostics::new();
    assert!(reg.validate_function_call("add", 2, &mut diags));
}

#[test]
fn validate_function_call_undefined() {
    let reg = DefinitionRegistry::new();
    let mut diags = Diagnostics::new();

    assert!(!reg.validate_function_call("nonexistent", 1, &mut diags));
    assert!(diags.has_errors());
}

#[test]
fn validate_function_call_too_few_args() {
    let mut reg = DefinitionRegistry::new();
    let mut diags = Diagnostics::new();

    let func = FunctionDef {
        name: "add".to_string(),
        params: vec![
            Param {
                name: "$a".to_string(),
                default: None,
            },
            Param {
                name: "$b".to_string(),
                default: None,
            },
        ],
        body: vec![],
    };
    reg.register_function(&func, &mut diags).unwrap();

    diags = Diagnostics::new();
    assert!(!reg.validate_function_call("add", 1, &mut diags));
    assert!(diags.has_errors());
}

#[test]
fn validate_mixin_call_ok() {
    let mut reg = DefinitionRegistry::new();
    let mut diags = Diagnostics::new();

    let mixin = MixinDef {
        name: "flex".to_string(),
        params: vec![Param {
            name: "$dir".to_string(),
            default: None,
        }],
        body: vec![],
    };
    reg.register_mixin(&mixin, &mut diags).unwrap();

    diags = Diagnostics::new();
    assert!(reg.validate_mixin_call("flex", 1, &mut diags));
}

#[test]
fn function_names_iterator() {
    let mut reg = DefinitionRegistry::new();
    let mut diags = Diagnostics::new();

    let func1 = FunctionDef {
        name: "add".to_string(),
        params: vec![],
        body: vec![],
    };
    let func2 = FunctionDef {
        name: "sub".to_string(),
        params: vec![],
        body: vec![],
    };

    reg.register_function(&func1, &mut diags).unwrap();
    reg.register_function(&func2, &mut diags).unwrap();

    let names: Vec<&str> = reg.function_names().collect();
    assert!(names.contains(&"add"));
    assert!(names.contains(&"sub"));
}
