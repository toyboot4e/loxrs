pub mod env;
pub mod obj;

mod interpreter;
pub use interpreter::Interpreter;

pub type Result<T> = ::std::result::Result<T, RuntimeError>;
use thiserror::Error;

// These errors are horribly.. They don't know source positions and they are lack of context!
/// Error when evaluating expressions.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("mismatched type")]
    MismatchedType,
    /// Tried to lookup undefined variable
    #[error("looked up undefined variable \"{0}\"")]
    Undefined(String),
    #[error("duplicate declaration of \"{0}\"")]
    DuplicateDeclaration(String),
    #[error("wrong number of arguments")]
    WrongNumberOfArguments,
    #[error("not for dot operator")]
    NotForDotOperator,
    #[error("not filed found with name \"{0}\"")]
    NoFieldWithName(String),
    #[error("re-assignment is disabled")]
    ReassignDisabled,
    #[error("cannot bind")]
    CantBind,
}
