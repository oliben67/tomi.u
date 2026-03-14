//! Code generation module for tomi.u
//!
//! This module provides a modular, backend-agnostic code generation system.
//! New backends (Rust, C, C++) can be added by implementing the [`BackendCodegen`] trait.
//!
//! ## Architecture
//!
//! ```text
//! AST → CodeGenerator → Backend (Rust/C/C++) → Target Code
//! ```
//!
//! ## Adding a New Backend
//!
//! 1. Create a new module (e.g., `src/codegen/c/mod.rs`)
//! 2. Implement [`BackendCodegen`] for your backend
//! 3. Add the variant to [`Backend`] enum
//! 4. Update [`CodeGenerator::new`] to construct your backend

pub mod rust;

use crate::ast::{Module, Item, Function, Struct, Enum, Type, TypePath, Expr, Stmt, Pattern, Block};
use crate::error::CompileError;
use crate::span::Span;

/// Supported code generation backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// Generate Rust code
    Rust,
    // Future backends:
    // C,
    // Cpp,
}

impl Backend {
    /// Get the file extension for this backend.
    pub fn extension(&self) -> &'static str {
        match self {
            Backend::Rust => "rs",
            // Backend::C => "c",
            // Backend::Cpp => "cpp",
        }
    }

    /// Get a display name for this backend.
    pub fn name(&self) -> &'static str {
        match self {
            Backend::Rust => "Rust",
            // Backend::C => "C",
            // Backend::Cpp => "C++",
        }
    }
}

/// Trait for implementing code generation backends.
///
/// Each backend implements this trait to generate target-specific code.
/// The trait provides methods for generating different AST node types,
/// allowing backends to customize output for their target language.
pub trait BackendCodegen {
    /// Generate code for a complete module.
    fn generate_module(&mut self, module: &Module) -> Result<String, CompileError>;

    /// Generate code for a single item (function, struct, etc.).
    fn generate_item(&mut self, item: &Item) -> Result<String, CompileError>;

    /// Generate code for a function.
    fn generate_function(&mut self, func: &Function) -> Result<String, CompileError>;

    /// Generate code for a struct.
    fn generate_struct(&mut self, s: &Struct) -> Result<String, CompileError>;

    /// Generate code for an enum.
    fn generate_enum(&mut self, e: &Enum) -> Result<String, CompileError>;

    /// Generate code for a type reference.
    fn generate_type(&mut self, ty: &Type) -> Result<String, CompileError>;

    /// Generate code for a type path.
    fn generate_type_path(&mut self, path: &TypePath) -> Result<String, CompileError>;

    /// Generate code for an expression.
    fn generate_expr(&mut self, expr: &Expr) -> Result<String, CompileError>;

    /// Generate code for a statement.
    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<String, CompileError>;

    /// Generate code for a block.
    fn generate_block(&mut self, block: &Block) -> Result<String, CompileError>;

    /// Generate code for a pattern.
    fn generate_pattern(&mut self, pattern: &Pattern) -> Result<String, CompileError>;

    /// Get backend-specific configuration.
    fn config(&self) -> &CodegenConfig;

    /// Set backend-specific configuration.
    fn set_config(&mut self, config: CodegenConfig);
}

/// Configuration options for code generation.
#[derive(Debug, Clone)]
pub struct CodegenConfig {
    /// Indentation string (default: 4 spaces)
    pub indent: String,
    /// Whether to include source comments in generated code
    pub include_comments: bool,
    /// Whether to format the output (pretty-print)
    pub format_output: bool,
    /// Generate debug assertions
    pub debug_mode: bool,
}

impl Default for CodegenConfig {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(),
            include_comments: true,
            format_output: true,
            debug_mode: false,
        }
    }
}

/// The main code generator that delegates to specific backends.
pub struct CodeGenerator {
    config: CodegenConfig,
    backend_type: Backend,
}

impl CodeGenerator {
    /// Create a new code generator for the specified backend.
    pub fn new(backend: Backend) -> Self {
        Self { 
            config: CodegenConfig::default(),
            backend_type: backend,
        }
    }

    /// Create a new code generator with custom configuration.
    pub fn with_config(backend: Backend, config: CodegenConfig) -> Self {
        Self { 
            config,
            backend_type: backend,
        }
    }

    /// Generate code from an AST module.
    pub fn generate(&self, module: &Module) -> Result<String, CompileError> {
        let mut backend: Box<dyn BackendCodegen> = match self.backend_type {
            Backend::Rust => Box::new(rust::RustBackend::new()),
        };
        backend.set_config(self.config.clone());
        backend.generate_module(module)
    }
}

/// Helper struct for building output with proper indentation.
#[derive(Default, Clone)]
pub struct CodeWriter {
    /// The accumulated output
    output: String,
    /// Current indentation level
    indent_level: usize,
    /// Indentation string
    indent_str: String,
    /// Whether we're at the start of a line
    at_line_start: bool,
}

impl CodeWriter {
    /// Create a new code writer.
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_str: "    ".to_string(),
            at_line_start: true,
        }
    }

    /// Create a code writer with custom indentation.
    pub fn with_indent(indent: &str) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            indent_str: indent.to_string(),
            at_line_start: true,
        }
    }

    /// Write a string to the output.
    pub fn write(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        for ch in s.chars() {
            if self.at_line_start && ch != '\n' {
                for _ in 0..self.indent_level {
                    self.output.push_str(&self.indent_str);
                }
                self.at_line_start = false;
            }

            self.output.push(ch);

            if ch == '\n' {
                self.at_line_start = true;
            }
        }
    }

    /// Write a string followed by a newline.
    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.newline();
    }

    /// Write a newline.
    pub fn newline(&mut self) {
        self.output.push('\n');
        self.at_line_start = true;
    }

    /// Write an empty line.
    pub fn blank_line(&mut self) {
        if !self.output.ends_with("\n\n") {
            self.newline();
        }
    }

    /// Increase indentation level.
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level.
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Execute a closure with increased indentation.
    pub fn indented<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.indent();
        let result = f(self);
        self.dedent();
        result
    }

    /// Get the final output, consuming the writer.
    pub fn finish(self) -> String {
        self.output
    }

    /// Take the output, leaving the writer empty.
    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.output)
    }

    /// Get the output length.
    pub fn len(&self) -> usize {
        self.output.len()
    }

    /// Check if output is empty.
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }
}

impl std::fmt::Write for CodeWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.write(s);
        Ok(())
    }
}

/// Extension trait for generating code with error handling.
pub trait CodegenExt {
    /// Generate code or return an error with span information.
    fn generate_or_error<F, T>(&self, span: Span, f: F) -> Result<T, CompileError>
    where
        F: FnOnce() -> Option<T>;
}

impl CodegenExt for () {
    fn generate_or_error<F, T>(&self, span: Span, f: F) -> Result<T, CompileError>
    where
        F: FnOnce() -> Option<T>,
    {
        f().ok_or(CompileError::Internal {
            message: "code generation failed".into(),
            span,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_writer() {
        let mut writer = CodeWriter::new();
        writer.writeln("fn main() {");
        writer.indented(|w| {
            w.writeln("println!(\"Hello\");");
        });
        writer.writeln("}");

        let expected = "fn main() {\n    println!(\"Hello\");\n}\n";
        assert_eq!(writer.finish(), expected);
    }

    #[test]
    fn test_nested_indentation() {
        let mut writer = CodeWriter::new();
        writer.writeln("outer {");
        writer.indented(|w| {
            w.writeln("middle {");
            w.indented(|w| {
                w.writeln("inner");
            });
            w.writeln("}");
        });
        writer.writeln("}");

        let output = writer.finish();
        assert!(output.contains("        inner"));
    }
}
