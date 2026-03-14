//! Error reporter using ariadne for beautiful diagnostics.

use ariadne::{Color, Label, Report, ReportKind, Source};

use crate::error::CompileError;

/// Pretty-prints compiler errors with source context.
pub struct ErrorReporter<'a> {
    filename: &'a str,
    source: &'a str,
}

impl<'a> ErrorReporter<'a> {
    /// Create a new error reporter.
    pub fn new(filename: &'a str, source: &'a str) -> Self {
        Self { filename, source }
    }

    /// Report an error to stderr with pretty formatting.
    pub fn report(&self, error: &CompileError) {
        let span = error.span();
        let kind = ReportKind::Error;

        let mut builder = Report::build(kind, self.filename, span.start)
            .with_code(error.code())
            .with_message(error.to_string());

        // Add the primary label
        builder = builder.with_label(
            Label::new((self.filename, span.start..span.end))
                .with_color(Color::Red)
                .with_message(self.label_message(error)),
        );

        // Add help if available
        if let Some(help) = error.help() {
            builder = builder.with_help(help);
        }

        // Print the report
        let report = builder.finish();
        report
            .eprint((self.filename, Source::from(self.source)))
            .unwrap();
    }

    /// Get a concise label message for the error.
    fn label_message(&self, error: &CompileError) -> String {
        match error {
            CompileError::UnexpectedChar { ch, .. } => format!("unexpected '{}'", ch),
            CompileError::UnterminatedString { .. } => "string starts here".into(),
            CompileError::UnterminatedComment { .. } => "comment starts here".into(),
            CompileError::InvalidEscape { ch, .. } => format!("invalid escape '\\{}'", ch),
            CompileError::InvalidNumber { .. } => "invalid number".into(),
            CompileError::InconsistentIndentation { .. } => "inconsistent indentation here".into(),
            CompileError::ExpectedToken { expected, .. } => format!("expected {}", expected),
            CompileError::ExpectedExpression { .. } => "expected expression here".into(),
            CompileError::ExpectedIdentifier { .. } => "expected identifier".into(),
            CompileError::ExpectedType { .. } => "expected type".into(),
            CompileError::UnexpectedEof { .. } => "unexpected end of file".into(),
            CompileError::ExpectedBlock { .. } => "expected indented block after this".into(),
            CompileError::InvalidDecorator { name, .. } => format!("unknown decorator @{}", name),
            CompileError::UnsupportedFeature { feature, .. } => feature.clone(),
            CompileError::Internal { message, .. } => message.clone(),
            CompileError::TypeMismatch {
                expected, found, ..
            } => {
                format!("expected `{expected}`, found `{found}`")
            }
            CompileError::UndefinedVariable { name, .. } => {
                format!("not found in this scope: `{name}`")
            }
            CompileError::UndefinedType { name, .. } => format!("type `{name}` not found"),
            CompileError::UndefinedField {
                field, struct_name, ..
            } => {
                format!("no field `{field}` on type `{struct_name}`")
            }
            CompileError::UndefinedTrait { name, .. } => format!("trait `{name}` not found"),
            CompileError::CannotInferType { .. } => "cannot determine type".into(),
            CompileError::ArgCountMismatch {
                expected, found, ..
            } => {
                format!("expected {expected} arguments, found {found}")
            }
            CompileError::NotCallable { ty, .. } => format!("type `{ty}` is not callable"),
            CompileError::InvalidCast { from, to, .. } => {
                format!("cannot cast `{from}` to `{to}`")
            }
            CompileError::MissingTraitMethod {
                method, trait_name, ..
            } => {
                format!("missing `{method}` required by `{trait_name}`")
            }
            CompileError::NonExhaustiveMatch {
                missing_patterns, ..
            } => {
                format!("patterns not covered: {}", missing_patterns.join(", "))
            }
            CompileError::TraitBoundNotSatisfied { ty, trait_name, .. } => {
                format!("`{ty}` does not implement `{trait_name}`")
            }
        }
    }
}
