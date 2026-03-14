//! Error types and reporting for the tomi.u compiler.
//!
//! This module provides:
//! - [`CompileError`]: All possible compiler errors
//! - [`ErrorReporter`]: Pretty-prints errors with source context

mod reporter;

pub use reporter::ErrorReporter;
use thiserror::Error;

use crate::span::Span;

/// Result type for compiler operations.
pub type CompileResult<T> = Result<T, CompileError>;

/// All possible errors that can occur during compilation.
#[derive(Debug, Error, Clone)]
pub enum CompileError {
    // ═══════════════════════════════════════════════════════════════════════
    // Lexer Errors
    // ═══════════════════════════════════════════════════════════════════════
    #[error("unexpected character '{ch}'")]
    UnexpectedChar { ch: char, span: Span },

    #[error("unterminated string literal")]
    UnterminatedString { span: Span },

    #[error("unterminated multi-line comment")]
    UnterminatedComment { span: Span },

    #[error("invalid escape sequence '\\{ch}'")]
    InvalidEscape { ch: char, span: Span },

    #[error("invalid number literal")]
    InvalidNumber { span: Span },

    #[error("inconsistent indentation")]
    InconsistentIndentation { span: Span },

    // ═══════════════════════════════════════════════════════════════════════
    // Parser Errors
    // ═══════════════════════════════════════════════════════════════════════
    #[error("expected {expected}, found {found}")]
    ExpectedToken { expected: String, found: String, span: Span },

    #[error("expected expression")]
    ExpectedExpression { span: Span },

    #[error("expected identifier")]
    ExpectedIdentifier { span: Span },

    #[error("expected type annotation")]
    ExpectedType { span: Span },

    #[error("unexpected end of file")]
    UnexpectedEof { span: Span },

    #[error("expected indented block")]
    ExpectedBlock { span: Span },

    #[error("invalid decorator '{name}'")]
    InvalidDecorator { name: String, span: Span },

    // ═══════════════════════════════════════════════════════════════════════
    // Type Checking Errors
    // ═══════════════════════════════════════════════════════════════════════
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String, span: Span },

    #[error("undefined variable '{name}'")]
    UndefinedVariable { name: String, span: Span },

    #[error("undefined type '{name}'")]
    UndefinedType { name: String, span: Span },

    #[error("undefined field '{field}' on struct '{struct_name}'")]
    UndefinedField { struct_name: String, field: String, span: Span },

    #[error("undefined trait '{name}'")]
    UndefinedTrait { name: String, span: Span },

    #[error("cannot infer type")]
    CannotInferType { span: Span },

    #[error("argument count mismatch: expected {expected}, found {found}")]
    ArgCountMismatch { expected: usize, found: usize, span: Span },

    #[error("type '{ty}' is not callable")]
    NotCallable { ty: String, span: Span },

    #[error("invalid cast from '{from}' to '{to}'")]
    InvalidCast { from: String, to: String, span: Span },

    #[error("missing required trait method '{method}' for trait '{trait_name}'")]
    MissingTraitMethod { trait_name: String, method: String, span: Span },

    #[error("non-exhaustive match: missing patterns {missing_patterns:?}")]
    NonExhaustiveMatch { missing_patterns: Vec<String>, span: Span },

    #[error("trait bound not satisfied: '{ty}' does not implement '{trait_name}'")]
    TraitBoundNotSatisfied { ty: String, trait_name: String, span: Span },

    // ═══════════════════════════════════════════════════════════════════════
    // Code Generation Errors
    // ═══════════════════════════════════════════════════════════════════════
    #[error("unsupported feature for target: {feature}")]
    UnsupportedFeature { feature: String, span: Span },

    #[error("internal compiler error: {message}")]
    Internal { message: String, span: Span },
}

impl CompileError {
    /// Get the source span for this error.
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedChar { span, .. } => *span,
            Self::UnterminatedString { span } => *span,
            Self::UnterminatedComment { span } => *span,
            Self::InvalidEscape { span, .. } => *span,
            Self::InvalidNumber { span } => *span,
            Self::InconsistentIndentation { span } => *span,
            Self::ExpectedToken { span, .. } => *span,
            Self::ExpectedExpression { span } => *span,
            Self::ExpectedIdentifier { span } => *span,
            Self::ExpectedType { span } => *span,
            Self::UnexpectedEof { span } => *span,
            Self::ExpectedBlock { span } => *span,
            Self::InvalidDecorator { span, .. } => *span,
            Self::TypeMismatch { span, .. } => *span,
            Self::UndefinedVariable { span, .. } => *span,
            Self::UndefinedType { span, .. } => *span,
            Self::UndefinedField { span, .. } => *span,
            Self::UndefinedTrait { span, .. } => *span,
            Self::CannotInferType { span } => *span,
            Self::ArgCountMismatch { span, .. } => *span,
            Self::NotCallable { span, .. } => *span,
            Self::InvalidCast { span, .. } => *span,
            Self::MissingTraitMethod { span, .. } => *span,
            Self::NonExhaustiveMatch { span, .. } => *span,
            Self::TraitBoundNotSatisfied { span, .. } => *span,
            Self::UnsupportedFeature { span, .. } => *span,
            Self::Internal { span, .. } => *span,
        }
    }

    /// Get the error code (e.g., "E0001").
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnexpectedChar { .. } => "E0001",
            Self::UnterminatedString { .. } => "E0002",
            Self::UnterminatedComment { .. } => "E0003",
            Self::InvalidEscape { .. } => "E0004",
            Self::InvalidNumber { .. } => "E0005",
            Self::InconsistentIndentation { .. } => "E0006",
            Self::ExpectedToken { .. } => "E0101",
            Self::ExpectedExpression { .. } => "E0102",
            Self::ExpectedIdentifier { .. } => "E0103",
            Self::ExpectedType { .. } => "E0104",
            Self::UnexpectedEof { .. } => "E0105",
            Self::ExpectedBlock { .. } => "E0106",
            Self::InvalidDecorator { .. } => "E0107",
            Self::TypeMismatch { .. } => "E0301",
            Self::UndefinedVariable { .. } => "E0302",
            Self::UndefinedType { .. } => "E0303",
            Self::UndefinedField { .. } => "E0304",
            Self::UndefinedTrait { .. } => "E0305",
            Self::CannotInferType { .. } => "E0306",
            Self::ArgCountMismatch { .. } => "E0307",
            Self::NotCallable { .. } => "E0308",
            Self::InvalidCast { .. } => "E0309",
            Self::MissingTraitMethod { .. } => "E0310",
            Self::NonExhaustiveMatch { .. } => "E0311",
            Self::TraitBoundNotSatisfied { .. } => "E0312",
            Self::UnsupportedFeature { .. } => "E0201",
            Self::Internal { .. } => "E9999",
        }
    }

    /// Get a help message for this error.
    pub fn help(&self) -> Option<String> {
        match self {
            Self::UnterminatedString { .. } => Some("add a closing '\"' to end the string".into()),
            Self::UnterminatedComment { .. } => {
                Some("add '###' to close the multi-line comment".into())
            }
            Self::InvalidEscape { ch: _, .. } => Some("valid escape sequences are: \\n, \\t, \\r, \\\\, \\\", \\', \\0, \\x{HH}, \\u{HHHH}".to_string()),
            Self::InconsistentIndentation { .. } => {
                Some("use consistent spaces (recommended: 4 spaces) for indentation".into())
            }
            Self::ExpectedBlock { .. } => {
                Some("indent the following lines to create a block".into())
            }
            Self::CannotInferType { .. } => {
                Some("add a type annotation to help the compiler infer the type".into())
            }
            Self::NonExhaustiveMatch { missing_patterns, .. } => {
                Some(format!("add arms for: {}", missing_patterns.join(", ")))
            }
            _ => None,
        }
    }
}
