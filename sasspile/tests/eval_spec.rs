//! Evaluator tests — AST → Value.

use sasspile::{
    DefinitionRegistry, EvalContext, SymbolTable, Value,
    parser, value,
};
use sasspile::semantic::SymbolEntry;

/// Helper: create an eval context with symbols and definitions.
fn make_ctx<'a>(
    symbols: &'a mut SymbolTable,
    definitions: &'a DefinitionRegistry,
) -> EvalContext<'a> {
    EvalContext::new(symbols, definitions)
}

/// Helper: define a variable in the symbol table.
fn define_var(symbols: &mut SymbolTable, name: &str, val: Value) {
    symbols.define_current(
        name.to_string(),
        SymbolEntry::mutable(
            Some(val),
            sasspile::source::SourceSpan::new(0, 0),
        ),
    );
}

// === Literals ===

#[test]
fn eval_number_literal() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Number(42.0, None);
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(42.0)));
}

#[test]
fn eval_number_with_unit() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Number(10.0, Some("px".to_string()));
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::px(10.0)));
}

#[test]
fn eval_string_literal() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::String("hello".to_string());
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(
        result,
        Value::String("hello".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn eval_boolean_literal() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    assert_eq!(
        ctx.eval_expr(&parser::Expr::Boolean(true)).unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        ctx.eval_expr(&parser::Expr::Boolean(false)).unwrap(),
        Value::Boolean(false)
    );
}

#[test]
fn eval_null_literal() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let result = ctx.eval_expr(&parser::Expr::Null).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn eval_color_literal() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let result = ctx.eval_expr(&parser::Expr::Color(0xFF0000)).unwrap();
    assert_eq!(result, Value::Color(value::SassColor::from_hex(0xFF0000)));
}

// === Variables ===

#[test]
fn eval_variable_lookup() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    define_var(&mut syms, "x", Value::Number(value::Number::unitless(100.0)));

    let mut ctx = make_ctx(&mut syms, &defs);
    let result = ctx.eval_expr(&parser::Expr::Variable("x".to_string())).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(100.0)));
}

#[test]
fn eval_undefined_variable() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let result = ctx.eval_expr(&parser::Expr::Variable("missing".to_string()));
    assert!(result.is_err());
}

// === Arithmetic ===

#[test]
fn eval_add_numbers() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Add,
        Box::new(parser::Expr::Number(3.0, None)),
        Box::new(parser::Expr::Number(4.0, None)),
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(7.0)));
}

#[test]
fn eval_subtract_numbers() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Sub,
        Box::new(parser::Expr::Number(10.0, None)),
        Box::new(parser::Expr::Number(3.0, None)),
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(7.0)));
}

#[test]
fn eval_multiply_numbers() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Mul,
        Box::new(parser::Expr::Number(6.0, None)),
        Box::new(parser::Expr::Number(7.0, None)),
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(42.0)));
}

#[test]
fn eval_divide_numbers() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Div,
        Box::new(parser::Expr::Number(10.0, None)),
        Box::new(parser::Expr::Number(2.0, None)),
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(5.0)));
}

#[test]
fn eval_divide_by_zero() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Div,
        Box::new(parser::Expr::Number(10.0, None)),
        Box::new(parser::Expr::Number(0.0, None)),
    );
    let result = ctx.eval_expr(&expr);
    assert!(result.is_err());
}

#[test]
fn eval_modulo_numbers() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Mod,
        Box::new(parser::Expr::Number(10.0, None)),
        Box::new(parser::Expr::Number(3.0, None)),
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(1.0)));
}

#[test]
fn eval_unary_negate() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Unary(
        parser::UnaryOp::Neg,
        Box::new(parser::Expr::Number(5.0, None)),
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(-5.0)));
}

#[test]
fn eval_unary_not() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Unary(
        parser::UnaryOp::Not,
        Box::new(parser::Expr::Boolean(true)),
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Boolean(false));
}

// === String operations ===

#[test]
fn eval_string_concat() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Add,
        Box::new(parser::Expr::String("hello".to_string())),
        Box::new(parser::Expr::String(" world".to_string())),
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(
        result,
        Value::String("hello world".to_string(), value::Quoted::Quoted)
    );
}

// === Comparison ===

#[test]
fn eval_equality() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Eq,
        Box::new(parser::Expr::Number(5.0, None)),
        Box::new(parser::Expr::Number(5.0, None)),
    );
    assert_eq!(ctx.eval_expr(&expr).unwrap(), Value::Boolean(true));

    let expr2 = parser::Expr::Binary(
        parser::BinaryOp::Eq,
        Box::new(parser::Expr::Number(5.0, None)),
        Box::new(parser::Expr::Number(3.0, None)),
    );
    assert_eq!(ctx.eval_expr(&expr2).unwrap(), Value::Boolean(false));
}

#[test]
fn eval_inequality() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::NotEq,
        Box::new(parser::Expr::Number(5.0, None)),
        Box::new(parser::Expr::Number(3.0, None)),
    );
    assert_eq!(ctx.eval_expr(&expr).unwrap(), Value::Boolean(true));
}

#[test]
fn eval_greater_than() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Greater,
        Box::new(parser::Expr::Number(5.0, None)),
        Box::new(parser::Expr::Number(3.0, None)),
    );
    assert_eq!(ctx.eval_expr(&expr).unwrap(), Value::Boolean(true));

    let expr2 = parser::Expr::Binary(
        parser::BinaryOp::Greater,
        Box::new(parser::Expr::Number(3.0, None)),
        Box::new(parser::Expr::Number(5.0, None)),
    );
    assert_eq!(ctx.eval_expr(&expr2).unwrap(), Value::Boolean(false));
}

#[test]
fn eval_less_than() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Less,
        Box::new(parser::Expr::Number(3.0, None)),
        Box::new(parser::Expr::Number(5.0, None)),
    );
    assert_eq!(ctx.eval_expr(&expr).unwrap(), Value::Boolean(true));
}

#[test]
fn eval_greater_eq() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::GreaterEq,
        Box::new(parser::Expr::Number(5.0, None)),
        Box::new(parser::Expr::Number(5.0, None)),
    );
    assert_eq!(ctx.eval_expr(&expr).unwrap(), Value::Boolean(true));
}

#[test]
fn eval_less_eq() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::LessEq,
        Box::new(parser::Expr::Number(4.0, None)),
        Box::new(parser::Expr::Number(5.0, None)),
    );
    assert_eq!(ctx.eval_expr(&expr).unwrap(), Value::Boolean(true));
}

// === Logical ===

#[test]
fn eval_logical_and() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::And,
        Box::new(parser::Expr::Boolean(true)),
        Box::new(parser::Expr::Boolean(false)),
    );
    // Sass: and returns the second value if the first is truthy.
    assert_eq!(ctx.eval_expr(&expr).unwrap(), Value::Boolean(false));
}

#[test]
fn eval_logical_or() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Binary(
        parser::BinaryOp::Or,
        Box::new(parser::Expr::Boolean(true)),
        Box::new(parser::Expr::Boolean(false)),
    );
    // Sass: or returns the first truthy value.
    assert_eq!(ctx.eval_expr(&expr).unwrap(), Value::Boolean(true));
}

// === Collections ===

#[test]
fn eval_list_literal() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::List(vec![
        parser::Expr::Number(1.0, None),
        parser::Expr::Number(2.0, None),
        parser::Expr::Number(3.0, None),
    ]);
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(
        result,
        Value::List(
            vec![
                Value::Number(value::Number::unitless(1.0)),
                Value::Number(value::Number::unitless(2.0)),
                Value::Number(value::Number::unitless(3.0)),
            ],
            value::Separator::Space,
        )
    );
}

#[test]
fn eval_map_literal() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Map(vec![(
        parser::Expr::String("key".to_string()),
        parser::Expr::Number(42.0, None),
    )]);
    let result = ctx.eval_expr(&expr).unwrap();

    match result {
        Value::Map(entries) => {
            assert_eq!(entries.len(), 1);
            assert_eq!(
                entries[0].1,
                Value::Number(value::Number::unitless(42.0))
            );
        }
        _ => panic!("expected map"),
    }
}

#[test]
fn eval_nth_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "nth".to_string(),
        vec![
            parser::Expr::List(vec![
                parser::Expr::Number(10.0, None),
                parser::Expr::Number(20.0, None),
                parser::Expr::Number(30.0, None),
            ]),
            parser::Expr::Number(2.0, None),
        ],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(20.0)));
}

#[test]
fn eval_nth_out_of_bounds() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "nth".to_string(),
        vec![
            parser::Expr::List(vec![parser::Expr::Number(1.0, None)]),
            parser::Expr::Number(5.0, None),
        ],
    );
    assert!(ctx.eval_expr(&expr).is_err());
}

#[test]
fn eval_length_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "length".to_string(),
        vec![parser::Expr::List(vec![
            parser::Expr::Number(1.0, None),
            parser::Expr::Number(2.0, None),
            parser::Expr::Number(3.0, None),
        ])],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(3.0)));
}

// === Built-in functions ===

#[test]
fn eval_unquote_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "unquote".to_string(),
        vec![parser::Expr::Number(42.0, None)],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(42.0)));
}

#[test]
fn eval_quote_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "quote".to_string(),
        vec![parser::Expr::String("hello".to_string())],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(
        result,
        Value::String("hello".to_string(), value::Quoted::Quoted)
    );
}

#[test]
fn eval_abs_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "abs".to_string(),
        vec![parser::Expr::Number(-5.0, None)],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(5.0)));
}

#[test]
fn eval_round_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "round".to_string(),
        vec![parser::Expr::Number(3.7, None)],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(4.0)));
}

#[test]
fn eval_ceil_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "ceil".to_string(),
        vec![parser::Expr::Number(3.2, None)],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(4.0)));
}

#[test]
fn eval_floor_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "floor".to_string(),
        vec![parser::Expr::Number(3.8, None)],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(3.0)));
}

#[test]
fn eval_min_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "min".to_string(),
        vec![
            parser::Expr::Number(5.0, None),
            parser::Expr::Number(2.0, None),
            parser::Expr::Number(8.0, None),
        ],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(2.0)));
}

#[test]
fn eval_max_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call(
        "max".to_string(),
        vec![
            parser::Expr::Number(5.0, None),
            parser::Expr::Number(2.0, None),
            parser::Expr::Number(8.0, None),
        ],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(8.0)));
}

#[test]
fn eval_undefined_function() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Call("nonexistent".to_string(), vec![]);
    assert!(ctx.eval_expr(&expr).is_err());
}

// === Parens and Interpolation ===

#[test]
fn eval_parenthesized() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Parens(Box::new(parser::Expr::Number(42.0, None)));
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(42.0)));
}

#[test]
fn eval_interpolation() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    let expr = parser::Expr::Interpolation(Box::new(parser::Expr::Number(42.0, None)));
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(
        result,
        Value::String("42".to_string(), value::Quoted::Unquoted)
    );
}

// === User-defined functions ===

#[test]
fn eval_user_function_call() {
    let mut syms = SymbolTable::new();
    let mut defs = DefinitionRegistry::new();

    // Register a user function: @function double($x) { @return $x * 2; }
    let func_def = parser::FunctionDef {
        name: "double".to_string(),
        params: vec![parser::Param {
            name: "$x".to_string(),
            default: None,
        }],
        body: vec![parser::Node::AtRule(parser::AtRule::Return(
            parser::Expr::Binary(
                parser::BinaryOp::Mul,
                Box::new(parser::Expr::Variable("$x".to_string())),
                Box::new(parser::Expr::Number(2.0, None)),
            ),
        ))],
    };

    defs.register_function(
        &func_def,
        &mut sasspile::diagnostics::Diagnostics::new(),
    )
    .unwrap();

    let mut ctx = make_ctx(&mut syms, &defs);
    let expr = parser::Expr::Call(
        "double".to_string(),
        vec![parser::Expr::Number(5.0, None)],
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(10.0)));
}

#[test]
fn eval_user_function_arity_mismatch() {
    let mut syms = SymbolTable::new();
    let mut defs = DefinitionRegistry::new();

    let func_def = parser::FunctionDef {
        name: "add".to_string(),
        params: vec![
            parser::Param {
                name: "$a".to_string(),
                default: None,
            },
            parser::Param {
                name: "$b".to_string(),
                default: None,
            },
        ],
        body: vec![parser::Node::AtRule(parser::AtRule::Return(
            parser::Expr::Binary(
                parser::BinaryOp::Add,
                Box::new(parser::Expr::Variable("$a".to_string())),
                Box::new(parser::Expr::Variable("$b".to_string())),
            ),
        ))],
    };

    defs.register_function(
        &func_def,
        &mut sasspile::diagnostics::Diagnostics::new(),
    )
    .unwrap();

    let mut ctx = make_ctx(&mut syms, &defs);
    // Call with wrong arity (1 instead of 2).
    let expr = parser::Expr::Call(
        "add".to_string(),
        vec![parser::Expr::Number(1.0, None)],
    );
    assert!(ctx.eval_expr(&expr).is_err());
}

// === Complex expressions ===

#[test]
fn eval_pemdas_like_expression() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    // (3 + 4) * 2 = 14
    let expr = parser::Expr::Binary(
        parser::BinaryOp::Mul,
        Box::new(parser::Expr::Parens(Box::new(parser::Expr::Binary(
            parser::BinaryOp::Add,
            Box::new(parser::Expr::Number(3.0, None)),
            Box::new(parser::Expr::Number(4.0, None)),
        )))),
        Box::new(parser::Expr::Number(2.0, None)),
    );
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Number(value::Number::unitless(14.0)));
}

#[test]
fn eval_comparison_chain() {
    let mut syms = SymbolTable::new();
    let defs = DefinitionRegistry::new();
    let mut ctx = make_ctx(&mut syms, &defs);

    // (5 > 3) and (2 < 4)
    let expr = parser::Expr::Binary(
        parser::BinaryOp::And,
        Box::new(parser::Expr::Binary(
            parser::BinaryOp::Greater,
            Box::new(parser::Expr::Number(5.0, None)),
            Box::new(parser::Expr::Number(3.0, None)),
        )),
        Box::new(parser::Expr::Binary(
            parser::BinaryOp::Less,
            Box::new(parser::Expr::Number(2.0, None)),
            Box::new(parser::Expr::Number(4.0, None)),
        )),
    );
    // Sass: 'and' returns the second value when first is truthy.
    let result = ctx.eval_expr(&expr).unwrap();

    assert_eq!(result, Value::Boolean(true));
}
