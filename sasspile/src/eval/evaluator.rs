//! Evaluation context — walks AST expressions and produces Values.

use crate::eval::error::EvalError;
use crate::parser::Expr;
use crate::semantic::{SymbolEntry, SymbolTable, DefinitionRegistry};
use crate::value::{SassColor, Value};

/// Maximum recursion depth for function calls.
const MAX_CALL_DEPTH: usize = 64;

/// Evaluation context holding symbol table and definition registry.
#[derive(Debug)]
pub struct EvalContext<'a> {
    /// Reference to the symbol table for variable resolution.
    pub(crate) symbols: &'a mut SymbolTable,
    /// Reference to the definition registry for function/mixin lookup.
    pub(crate) definitions: &'a DefinitionRegistry,
    /// Function call depth (to detect infinite recursion).
    call_depth: usize,
}

impl<'a> EvalContext<'a> {
    /// Create a new evaluation context.
    pub fn new(
        symbols: &'a mut SymbolTable,
        definitions: &'a DefinitionRegistry,
    ) -> Self {
        Self {
            symbols,
            definitions,
            call_depth: 0,
        }
    }

    /// Evaluate an expression to a Value.
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, EvalError> {
        #[allow(unreachable_patterns)]
        match expr {
            Expr::Variable(name) => self.eval_variable(name),
            Expr::Number(value, unit) => {
                let u = unit.as_deref().and_then(crate::value::Unit::parse)
                    .unwrap_or(crate::value::Unit::None);
                Ok(Value::Number(crate::value::Number::new(*value, u)))
            }
            Expr::String(s) => Ok(Value::String(s.clone(), crate::value::Quoted::Quoted)),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::Color(c) => Ok(Value::Color(SassColor::from_hex(*c))),
            Expr::Url(u) => Ok(Value::String(u.clone(), crate::value::Quoted::Unquoted)),
            Expr::Interpolation(inner) => {
                let val = self.eval_expr(inner)?;
                Ok(Value::String(val.to_string_value(), crate::value::Quoted::Unquoted))
            }
            Expr::Binary(op, lhs, rhs) => {
                let l = self.eval_expr(lhs)?;
                let r = self.eval_expr(rhs)?;
                crate::eval::ops::binary(op, &l, &r)
            }
            Expr::Unary(op, operand) => {
                let v = self.eval_expr(operand)?;
                crate::eval::ops::unary(op, &v)
            }
            Expr::Call(name, args) => crate::eval::functions::call(name, args, self),
            Expr::List(items) => {
                let evaluated: Result<Vec<_>, _> =
                    items.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::List(evaluated?, crate::value::Separator::Space))
            }
            Expr::Map(entries) => {
                let mut map = Vec::new();
                for (k, v) in entries {
                    let key = self.eval_expr(k)?;
                    let val = self.eval_expr(v)?;
                    map.push((key, val));
                }
                Ok(Value::Map(map))
            }
            Expr::Parens(inner) => self.eval_expr(inner),
            Expr::SlashList(items) => {
                let evaluated: Result<Vec<_>, _> =
                    items.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::List(evaluated?, crate::value::Separator::Slash))
            }
            Expr::SpaceList(items) => {
                let evaluated: Result<Vec<_>, _> =
                    items.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::List(evaluated?, crate::value::Separator::Space))
            }
            // Named argument: evaluate to its value (function dispatch handles the name).
            Expr::NamedArg(_name, value) => self.eval_expr(value),
            // Spread in arg list: evaluate inner expr
            Expr::Spread(inner) => self.eval_expr(inner),
        }
    }

    /// Resolve a variable from the symbol table.
    fn eval_variable(&self, name: &str) -> Result<Value, EvalError> {
        match self.symbols.lookup(name) {
            Some(entry) => match &entry.value {
                Some(val) => Ok(val.clone()),
                None => Err(EvalError::UndefinedVariable(name.to_string())),
            },
            None => Err(EvalError::UndefinedVariable(name.to_string())),
        }
    }

    /// Call a user-defined function with proper scoping and argument binding.
    pub(super) fn call_user_function(
        &mut self,
        func: &crate::semantic::FunctionEntry,
        args: &[Expr],
    ) -> Result<Value, EvalError> {
        if self.call_depth >= MAX_CALL_DEPTH {
            return Err(EvalError::MaxDepthExceeded(MAX_CALL_DEPTH));
        }

        // Evaluate arguments.
        let arg_values: Result<Vec<_>, _> =
            args.iter().map(|e| self.eval_expr(e)).collect();
        let arg_values = arg_values?;

        // Check arity.
        if arg_values.len() < func.required_params {
            return Err(EvalError::ArityMismatch(
                func.name.clone(),
                format!("at least {}", func.required_params),
                arg_values.len(),
            ));
        }
        if !func.variadic && arg_values.len() > func.total_params {
            return Err(EvalError::ArityMismatch(
                func.name.clone(),
                format!("{}", func.total_params),
                arg_values.len(),
            ));
        }

        // Push new param scope with arguments bound.
        self.symbols.push_param();
        for (i, param) in func.definition.params.iter().enumerate() {
            let value = arg_values.get(i).cloned().unwrap_or(Value::Null);
            let entry = SymbolEntry::new(
                Some(value),
                crate::source::SourceSpan::new(0, 0),
            );
            self.symbols.define_current(param.name.clone(), entry);
        }

        self.call_depth += 1;
        let result = self.eval_function_body(&func.definition.body);
        self.call_depth -= 1;

        self.symbols.pop();
        result
    }

    /// Evaluate the body of a user-defined function.
    fn eval_function_body(&mut self, body: &[crate::parser::Node]) -> Result<Value, EvalError> {
        for node in body {
            if let crate::parser::Node::AtRule(crate::parser::AtRule::Return(expr)) = node {
                return self.eval_expr(expr);
            }
            // Process variable assignments.
            self.eval_node_for_defs(node)?;
        }
        Ok(Value::Null)
    }

    /// Evaluate nodes to extract variable definitions.
    fn eval_node_for_defs(&mut self, node: &crate::parser::Node) -> Result<(), EvalError> {
        match node {
            crate::parser::Node::Declaration(decl) => {
                let value = self.eval_expr(&decl.value)?;
                let entry = SymbolEntry::mutable(
                    Some(value),
                    decl.span,
                );
                self.symbols.define_current(decl.name.clone(), entry);
            }
            crate::parser::Node::Rule(rule) => {
                self.symbols.push_local();
                for inner in &rule.nodes {
                    self.eval_node_for_defs(inner)?;
                }
                self.symbols.pop();
            }
            _ => {}
        }
        Ok(())
    }
}
