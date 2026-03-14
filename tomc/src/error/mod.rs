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
            Self::InvalidEscape { ch, .. } => Some(format!(
                "valid escape sequences are: \\n, \\t, \\r, \\\\, \\\", \\', \\0, \\x{{HH}}, \\u{{HHHH}}"
            )),
            Self::InconsistentIndentation { .. } => {
                Some("use consistent spaces (recommended: 4 spaces) for indentation".into())
            }
            Self::ExpectedBlock { .. } => {
                Some("indent the following lines to create a block".into())
            }
            _ => None,
        }
    }
}
