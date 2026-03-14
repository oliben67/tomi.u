//! # tomc - The tomi.u Compiler
//!
//! `tomc` is the official compiler for the tomi.u programming language.
//! It translates tomi.u source code into Rust (with planned support for C/C++).
//!
//! ## Architecture
//!
//! ```text
//! Source (.tu) → Lexer → Parser → AST → CodeGen → Target (Rust/C/C++)
//! ```
//!
//! ## Modules
//!
//! - [`span`]: Source location tracking for error reporting
//! - [`error`]: Compiler error types and diagnostics
//! - [`lexer`]: Tokenization of tomi.u source code
//! - [`ast`]: Abstract Syntax Tree definitions
//! - [`parser`]: Recursive descent parser
//! - [`codegen`]: Modular code generation backends

pub mod ast;
pub mod checker;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod types;

pub use error::{CompileError, CompileResult};
pub use span::Span;

/// Compiler version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
