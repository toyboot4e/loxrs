pub mod env;
mod interpreter;
pub mod obj;
mod visitor;

pub use interpreter::Interpreter;
pub use visitor::{ExprVisitor, StmtVisitor};

pub type Result<T> = ::std::result::Result<T, RuntimeError>;

/// Error when evaluating expressions.
#[derive(Debug)]
pub enum RuntimeError {
    // TODO: use more detailed context
    MismatchedType,
    /// Tried to lookup undefined variable
    Undefined(String),
    // TODO: enable overwriting
    DuplicateDeclaration(String),

    WrongNumberOfArguments,
}

