use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CompileError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Lex error at position {pos}: {message}")]
    Lex { pos: u32, message: String },

    #[error("Parse error at position {pos}: {message}")]
    Parse { pos: u32, message: String },

    #[error("Eval error: {message}")]
    Eval { message: String },

    #[error("Type error: expected {expected}, found {found}")]
    Type { expected: String, found: String },

    #[error("Unit error: {message}")]
    Unit(String),

    #[error("Undefined variable: ${0}")]
    UndefinedVariable(String),

    #[error("Undefined mixin: {0}")]
    UndefinedMixin(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    #[error("Recursion limit exceeded: {0}")]
    RecursionLimit(String),
}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        CompileError::Io(e.to_string())
    }
}

impl From<String> for CompileError {
    fn from(msg: String) -> Self {
        CompileError::Eval { message: msg }
    }
}

impl From<&str> for CompileError {
    fn from(msg: &str) -> Self {
        CompileError::Eval { message: msg.to_string() }
    }
}
