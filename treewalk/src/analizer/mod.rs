//! Semantic analysis for AST, which converts it without any side effects.
//! Takes O(n) time.
//!
//! A resolver is often in a parser, like clox. But in part II of the book
//! (treewalk interpreter), we make it a separate class so that we can use
//! most of our existing `Env` class.

pub mod resolver;
