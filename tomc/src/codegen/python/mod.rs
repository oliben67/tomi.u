//! Python code generation backend for tomi.u
//!
//! This module translates tomi.u AST to valid Python 3.14 source code,
//! enabling the Python bridge (bootstrap runtime) described in v0.3.0.
//!
//! ## Translation Rules
//!
//! | tomi.u            | Python                          |
//! |-------------------|---------------------------------|
//! | `def name()`      | `def name():`                   |
//! | `let x = ...`     | `x = ...`                       |
//! | `mut x`           | `x = ...`                       |
//! | `struct S`        | `@dataclass class S:`           |
//! | `enum E`          | `class E(Enum):` / dataclasses  |
//! | `trait T`         | `class T(Protocol):`            |
//! | `impl T for S`    | Methods added to class body     |
//! | `@entrypoint`     | `if __name__ == "__main__":`    |
//! | `?Type`           | `Optional[Type]`                |
//! | `Int32`, `Int64`  | `int`                           |
//! | `Float32/64`      | `float`                         |
//! | `Bool`            | `bool`                          |
//! | `String`          | `str`                           |
//! | `try/catch`       | `try/except`                    |
//! | `raise`           | `raise`                         |
//! | `@python.export`  | raw function (no wrapper)       |

use crate::ast::*;
use crate::codegen::{BackendCodegen, CodeWriter, CodegenConfig};
use crate::error::CompileError;

/// Python 3.14 backend for code generation.
pub struct PythonBackend {
    config: CodegenConfig,
    writer: CodeWriter,
    in_async: bool,
    current_function: Option<String>,
    /// Tracks whether an entrypoint function was emitted.
    has_entrypoint: bool,
    /// Name of the entrypoint function (to call in `if __name__`).
    entrypoint_name: Option<String>,
    /// Accumulated impl methods keyed by target type name.
    impl_methods: std::collections::HashMap<String, Vec<Function>>,
    /// Types that have been defined (for impl merging).
    defined_types: std::collections::HashSet<String>,
    /// Whether we need dataclasses import.
    needs_dataclass: bool,
    /// Whether we need enum import.
    needs_enum: bool,
    /// Whether we need typing imports.
    needs_typing: bool,
    /// Whether we need Protocol import.
    needs_protocol: bool,
}

impl PythonBackend {
    pub fn new() -> Self {
        Self {
            config: CodegenConfig::default(),
            writer: CodeWriter::new(),
            in_async: false,
            current_function: None,
            has_entrypoint: false,
            entrypoint_name: None,
            impl_methods: std::collections::HashMap::new(),
            defined_types: std::collections::HashSet::new(),
            needs_dataclass: false,
            needs_enum: false,
            needs_typing: false,
            needs_protocol: false,
        }
    }

    fn reset(&mut self) {
        self.writer = CodeWriter::with_indent(&self.config.indent);
        self.in_async = false;
        self.current_function = None;
        self.has_entrypoint = false;
        self.entrypoint_name = None;
        self.impl_methods.clear();
        self.defined_types.clear();
        self.needs_dataclass = false;
        self.needs_enum = false;
        self.needs_typing = false;
        self.needs_protocol = false;
    }

    // ─────────────────────────────────────────────────────────────────
    // Pre-scan: detect which imports are needed
    // ─────────────────────────────────────────────────────────────────

    fn scan_items(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::Struct(_) => self.needs_dataclass = true,
                Item::Enum(_) => self.needs_enum = true,
                Item::Trait(_) => {
                    self.needs_protocol = true;
                    self.needs_typing = true;
                }
                Item::TypeAlias(_) => self.needs_typing = true,
                Item::Function(f) => {
                    if f.return_type.is_some() || !f.params.is_empty() {
                        self.needs_typing = true;
                    }
                    self.scan_decorators(&f.decorators);
                }
                Item::Impl(imp) => {
                    // Collect impl methods to attach to their target class later.
                    if let Type::Named(path) = &imp.target {
                        let name = path
                            .segments
                            .iter()
                            .map(|s| s.node.as_str())
                            .collect::<Vec<_>>()
                            .join(".");
                        let methods: Vec<Function> = imp.items.clone();
                        self.impl_methods.entry(name).or_default().extend(methods);
                    }
                }
                Item::Const(_) => {}
            }
        }
    }

    fn scan_decorators(&mut self, decorators: &[Decorator]) {
        for dec in decorators {
            if dec.name.node == "entrypoint" {
                self.has_entrypoint = true;
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Prelude and imports
    // ─────────────────────────────────────────────────────────────────

    fn generate_prelude(&mut self) {
        if self.config.include_comments {
            self.writer.writeln("# Generated by tomc - the tomi.u compiler");
            self.writer.writeln("# Do not edit manually");
            self.writer.blank_line();
        }

        self.writer.writeln("from __future__ import annotations");

        if self.needs_dataclass {
            self.writer.writeln("from dataclasses import dataclass");
        }
        if self.needs_enum {
            self.writer.writeln("from enum import Enum, auto");
        }
        if self.needs_typing || self.needs_protocol {
            let mut parts = Vec::new();
            if self.needs_typing {
                parts.extend(["Any", "Optional"]);
            }
            if self.needs_protocol {
                parts.push("Protocol");
            }
            self.writer.writeln(&format!("from typing import {}", parts.join(", ")));
        }
        self.writer.blank_line();

        // Built-in function shims matching the tomi.u standard library
        self.writer.writeln("# tomi.u runtime support");
        self.writer.writeln("import sys as _sys");
        self.writer.blank_line();
    }

    // ─────────────────────────────────────────────────────────────────
    // Type mapping  tomi.u → Python
    // ─────────────────────────────────────────────────────────────────

    fn map_type_name<'a>(&self, name: &'a str) -> &'a str {
        match name {
            "Int8" | "Int16" | "Int32" | "Int64" | "UInt8" | "UInt16" | "UInt32" | "UInt64"
            | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" => "int",
            "Float32" | "Float64" | "f32" | "f64" => "float",
            "Bool" | "bool" => "bool",
            "String" => "str",
            "Char" => "str",
            "Unit" => "None",
            "Never" => "None",
            "Bytes" => "bytes",
            other => other,
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Operators
    // ─────────────────────────────────────────────────────────────────

    fn binary_op_to_python(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
        }
    }

    fn unary_op_to_python(&self, op: &UnaryOp) -> &'static str {
        match op {
            UnaryOp::Not => "not ",
            UnaryOp::Neg => "-",
            UnaryOp::BitNot => "~",
            // References don't exist in Python — pass through
            UnaryOp::Ref | UnaryOp::MutRef | UnaryOp::Deref => "",
        }
    }

    fn assign_op_to_python(&self, op: &AssignOp) -> &'static str {
        match op {
            AssignOp::Assign => "=",
            AssignOp::AddAssign => "+=",
            AssignOp::SubAssign => "-=",
            AssignOp::MulAssign => "*=",
            AssignOp::DivAssign => "/=",
            AssignOp::ModAssign => "%=",
            AssignOp::ShlAssign => "<<=",
            AssignOp::ShrAssign => ">>=",
        }
    }

    // ─────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────

    fn has_entrypoint_decorator(&self, decorators: &[Decorator]) -> bool {
        decorators.iter().any(|d| d.name.node == "entrypoint")
    }

    fn has_python_export(&self, decorators: &[Decorator]) -> bool {
        decorators.iter().any(|d| d.name.node == "python.export")
    }

    /// Generate parameters for a Python function.
    fn generate_params(&mut self, params: &[Param]) -> Result<String, CompileError> {
        let mut parts = Vec::new();
        for param in params {
            if param.name.node == "self" {
                parts.push("self".to_string());
                continue;
            }
            let ty_str = self.generate_type(&param.ty)?;
            let mut part = format!("{}: {}", param.name.node, ty_str);
            if let Some(default) = &param.default {
                let val = self.generate_expr(default)?;
                part.push_str(&format!(" = {}", val));
            }
            parts.push(part);
        }
        Ok(parts.join(", "))
    }

    /// Generate generic type parameter brackets `[T, U]`.
    fn generate_type_params_bracket(&self, params: &[TypeParam]) -> String {
        if params.is_empty() {
            return String::new();
        }
        let parts: Vec<&str> = params.iter().map(|p| p.name.node.as_str()).collect();
        format!("[{}]", parts.join(", "))
    }

    /// Generate import statement.
    fn generate_import(&mut self, import: &Import) -> Result<(), CompileError> {
        let path_parts: Vec<&str> = import.path.iter().map(|s| s.node.as_str()).collect();
        let module = path_parts.join(".");

        if let Some(items) = &import.items {
            let names: Vec<&str> = items.iter().map(|s| s.node.as_str()).collect();
            self.writer.writeln(&format!("from {} import {}", module, names.join(", ")));
        } else if let Some(alias) = &import.alias {
            self.writer.writeln(&format!("import {} as {}", module, alias.node));
        } else {
            self.writer.writeln(&format!("import {}", module));
        }
        Ok(())
    }

    /// Generate a trait definition as a Protocol class.
    fn generate_trait_def(&mut self, t: &Trait) -> Result<(), CompileError> {
        self.writer.write(&format!("class {}(Protocol)", t.name.node));
        let tp = self.generate_type_params_bracket(&t.type_params);
        if !tp.is_empty() {
            // For generic protocols we'd need Generic[T] — simplified for now
        }
        self.writer.writeln(":");
        self.writer.indent();

        if t.items.is_empty() {
            self.writer.writeln("...");
        } else {
            for item in &t.items {
                self.generate_trait_item(item)?;
                self.writer.blank_line();
            }
        }
        self.writer.dedent();
        Ok(())
    }

    fn generate_trait_item(&mut self, item: &TraitItem) -> Result<(), CompileError> {
        match item {
            TraitItem::Method(m) => {
                let func_str = self.generate_function(m)?;
                self.writer.write(&func_str);
            }
            TraitItem::Type { name, .. } => {
                self.writer.writeln(&format!("{}: Any", name.node));
            }
        }
        Ok(())
    }

    /// Generate an impl block. For inherent impls, methods are attached to
    /// the class body during struct/enum generation. For trait impls we add
    /// the methods after the class definition.
    fn generate_standalone_impl(&mut self, imp: &Impl) -> Result<(), CompileError> {
        // Only generate standalone if the target type is NOT one we already
        // defined as a class (structs/enums). For trait impls on known types
        // we monkey-patch the methods.
        let target_name = self.generate_type(&imp.target)?;

        if imp.trait_path.is_some() {
            // Trait impl — we register the class as implementing the protocol
            // by adding methods. In Python this is duck-typed so we just attach.
            if self.config.include_comments {
                if let Some(ref tp) = imp.trait_path {
                    let trait_name = self.generate_type_path(tp)?;
                    self.writer.writeln(&format!("# impl {} for {}", trait_name, target_name));
                }
            }
        }

        for method in &imp.items {
            let func_str = self.generate_function(method)?;
            self.writer.writeln(&format!(
                "{}.{} = {}  # type: ignore",
                target_name, method.name.node, method.name.node
            ));
            self.writer.write(&func_str);
            self.writer.blank_line();
        }
        Ok(())
    }

    /// Generate a type alias.
    fn generate_type_alias_item(&mut self, ta: &TypeAlias) -> Result<(), CompileError> {
        let ty_str = self.generate_type(&ta.ty)?;
        self.writer.writeln(&format!("{} = {}", ta.name.node, ty_str));
        Ok(())
    }

    /// Generate a const.
    fn generate_const_item(&mut self, c: &Const) -> Result<(), CompileError> {
        let val_str = self.generate_expr(&c.value)?;
        if let Some(ty) = &c.ty {
            let ty_str = self.generate_type(ty)?;
            self.writer.writeln(&format!("{}: {} = {}", c.name.node, ty_str, val_str));
        } else {
            self.writer.writeln(&format!("{} = {}", c.name.node, val_str));
        }
        Ok(())
    }

    /// Generate a `if __name__ == "__main__":` block calling the entrypoint.
    fn generate_main_guard(&mut self) {
        if let Some(ref name) = self.entrypoint_name {
            self.writer.blank_line();
            self.writer.writeln("if __name__ == \"__main__\":");
            self.writer.indent();
            self.writer.writeln(&format!("{}()", name));
            self.writer.dedent();
        }
    }
}

impl Default for PythonBackend {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BackendCodegen implementation
// ═══════════════════════════════════════════════════════════════════════════════

impl BackendCodegen for PythonBackend {
    fn generate_module(&mut self, module: &Module) -> Result<String, CompileError> {
        self.reset();

        // Pre-scan to determine needed imports and collect impl methods.
        self.scan_items(&module.items);

        self.generate_prelude();

        // User imports
        for import in &module.imports {
            self.generate_import(import)?;
        }
        if !module.imports.is_empty() {
            self.writer.blank_line();
        }

        // Generate items (skipping impls whose methods are merged into classes).
        for item in &module.items {
            match item {
                Item::Impl(imp) => {
                    // If impl target is a type we defined as a class, methods
                    // were already merged during struct/enum generation.
                    let target_name = self.generate_type(&imp.target)?;
                    if !self.defined_types.contains(&target_name) {
                        self.generate_standalone_impl(imp)?;
                    }
                }
                _ => {
                    let item_str = self.generate_item(item)?;
                    self.writer.write(&item_str);
                    self.writer.blank_line();
                }
            }
        }

        // Emit `if __name__ == "__main__":` guard if there was an entrypoint.
        self.generate_main_guard();

        Ok(self.writer.take())
    }

    fn generate_item(&mut self, item: &Item) -> Result<String, CompileError> {
        let mut writer = CodeWriter::with_indent(&self.config.indent);
        std::mem::swap(&mut self.writer, &mut writer);

        match item {
            Item::Function(f) => {
                let s = self.generate_function(f)?;
                self.writer.write(&s);
            }
            Item::Struct(s) => {
                let s = self.generate_struct(s)?;
                self.writer.write(&s);
            }
            Item::Enum(e) => {
                let s = self.generate_enum(e)?;
                self.writer.write(&s);
            }
            Item::Trait(t) => self.generate_trait_def(t)?,
            Item::Impl(i) => self.generate_standalone_impl(i)?,
            Item::TypeAlias(ta) => self.generate_type_alias_item(ta)?,
            Item::Const(c) => self.generate_const_item(c)?,
        }

        std::mem::swap(&mut self.writer, &mut writer);
        Ok(writer.finish())
    }

    fn generate_function(&mut self, func: &Function) -> Result<String, CompileError> {
        let mut writer = CodeWriter::with_indent(&self.config.indent);
        std::mem::swap(&mut self.writer, &mut writer);

        self.current_function = Some(func.name.node.clone());

        let is_entrypoint = self.has_entrypoint_decorator(&func.decorators);
        let is_exported = self.has_python_export(&func.decorators);

        // Emit non-special decorators
        for dec in &func.decorators {
            let name = &dec.name.node;
            if name != "entrypoint" && name != "python.export" && name != "constructor" {
                self.writer.writeln(&format!("@{}", name));
            }
        }

        // Async
        if func.is_async {
            self.writer.write("async ");
            self.in_async = true;
        }

        // Function name
        let fn_name = if is_entrypoint {
            self.entrypoint_name = Some("main".to_string());
            "main".to_string()
        } else {
            func.name.node.clone()
        };

        self.writer.write("def ");
        self.writer.write(&fn_name);

        // Parameters
        let params_str = self.generate_params(&func.params)?;
        self.writer.write("(");
        self.writer.write(&params_str);
        self.writer.write(")");

        // Return type annotation
        if let Some(ret) = &func.return_type {
            let ret_str = self.generate_type(ret)?;
            self.writer.write(" -> ");
            self.writer.write(&ret_str);
        }

        self.writer.writeln(":");

        // Body
        self.writer.indent();
        let block_str = self.generate_block(&func.body)?;
        if block_str.trim().is_empty() {
            self.writer.writeln("pass");
        } else {
            self.writer.write(&block_str);
        }
        self.writer.dedent();

        self.in_async = false;
        self.current_function = None;

        // Export marker (comment for documentation — Python functions are
        // already callable; the decorator is informational for tooling).
        if is_exported && self.config.include_comments {
            // Already handled by duck typing
        }

        std::mem::swap(&mut self.writer, &mut writer);
        Ok(writer.finish())
    }

    fn generate_struct(&mut self, s: &Struct) -> Result<String, CompileError> {
        let mut writer = CodeWriter::with_indent(&self.config.indent);
        std::mem::swap(&mut self.writer, &mut writer);

        self.defined_types.insert(s.name.node.clone());

        self.writer.writeln("@dataclass");
        self.writer.write("class ");
        self.writer.write(&s.name.node);
        self.writer.writeln(":");
        self.writer.indent();

        if s.fields.is_empty() && s.methods.is_empty() {
            self.writer.writeln("pass");
        } else {
            // Fields
            for field in &s.fields {
                let ty_str = self.generate_type(&field.ty)?;
                if let Some(default) = &field.default {
                    let val = self.generate_expr(default)?;
                    self.writer.writeln(&format!("{}: {} = {}", field.name.node, ty_str, val));
                } else {
                    self.writer.writeln(&format!("{}: {}", field.name.node, ty_str));
                }
            }

            // Inline methods from the struct definition
            for method in &s.methods {
                self.writer.blank_line();
                let func_str = self.generate_function(method)?;
                self.writer.write(&func_str);
            }

            // Merge impl methods
            if let Some(methods) = self.impl_methods.remove(&s.name.node) {
                for method in &methods {
                    self.writer.blank_line();
                    let func_str = self.generate_function(method)?;
                    self.writer.write(&func_str);
                }
            }
        }

        self.writer.dedent();

        std::mem::swap(&mut self.writer, &mut writer);
        Ok(writer.finish())
    }

    fn generate_enum(&mut self, e: &Enum) -> Result<String, CompileError> {
        let mut writer = CodeWriter::with_indent(&self.config.indent);
        std::mem::swap(&mut self.writer, &mut writer);

        self.defined_types.insert(e.name.node.clone());

        // Determine if all variants are unit (simple enum) or have data
        let all_unit = e.variants.iter().all(|v| matches!(v.data, VariantData::Unit));

        if all_unit {
            // Simple Python Enum
            self.writer.write("class ");
            self.writer.write(&e.name.node);
            self.writer.writeln("(Enum):");
            self.writer.indent();

            for variant in &e.variants {
                self.writer.writeln(&format!("{} = auto()", variant.name.node));
            }

            // Merge impl methods
            if let Some(methods) = self.impl_methods.remove(&e.name.node) {
                for method in &methods {
                    self.writer.blank_line();
                    let func_str = self.generate_function(method)?;
                    self.writer.write(&func_str);
                }
            }

            self.writer.dedent();
        } else {
            // Sum type — generate a base class with variant subclasses using
            // dataclasses. Each variant is a nested @dataclass inside the enum
            // namespace, and the base is a plain class they inherit from.
            self.writer.write("class ");
            self.writer.write(&e.name.node);
            self.writer.writeln(":");
            self.writer.indent();
            self.writer.writeln("\"\"\"Base class for sum type variants.\"\"\"");

            for variant in &e.variants {
                self.writer.blank_line();
                match &variant.data {
                    VariantData::Unit => {
                        self.writer.writeln("@dataclass");
                        self.writer
                            .writeln(&format!("class {}({}):", variant.name.node, e.name.node));
                        self.writer.indent();
                        self.writer.writeln("pass");
                        self.writer.dedent();
                    }
                    VariantData::Tuple(types) => {
                        self.writer.writeln("@dataclass");
                        self.writer
                            .writeln(&format!("class {}({}):", variant.name.node, e.name.node));
                        self.writer.indent();
                        for (i, ty) in types.iter().enumerate() {
                            let ty_str = self.generate_type(ty)?;
                            self.writer.writeln(&format!("_{}: {}", i, ty_str));
                        }
                        self.writer.dedent();
                    }
                    VariantData::Struct(fields) => {
                        self.writer.writeln("@dataclass");
                        self.writer
                            .writeln(&format!("class {}({}):", variant.name.node, e.name.node));
                        self.writer.indent();
                        for field in fields {
                            let ty_str = self.generate_type(&field.ty)?;
                            self.writer.writeln(&format!("{}: {}", field.name.node, ty_str));
                        }
                        self.writer.dedent();
                    }
                }
            }

            // Merge impl methods onto the base class
            if let Some(methods) = self.impl_methods.remove(&e.name.node) {
                for method in &methods {
                    self.writer.blank_line();
                    let func_str = self.generate_function(method)?;
                    self.writer.write(&func_str);
                }
            }

            self.writer.dedent();
        }

        std::mem::swap(&mut self.writer, &mut writer);
        Ok(writer.finish())
    }

    fn generate_type(&mut self, ty: &Type) -> Result<String, CompileError> {
        let result = match ty {
            Type::Named(path) => {
                let raw = self.generate_type_path(path)?;
                self.map_type_name(&raw).to_string()
            }
            Type::Generic(path, args) => {
                let base = self.generate_type_path(path)?;
                let mapped_base = self.map_type_name(&base).to_string();
                let args_str: Result<Vec<_>, _> =
                    args.iter().map(|t| self.generate_type(t)).collect();
                format!("{}[{}]", mapped_base, args_str?.join(", "))
            }
            Type::Reference { inner, .. } => {
                // Python has no references; just emit the inner type.
                self.generate_type(inner)?
            }
            Type::Optional(inner) => {
                let inner_str = self.generate_type(inner)?;
                format!("Optional[{}]", inner_str)
            }
            Type::Tuple(types) => {
                let parts: Result<Vec<_>, _> =
                    types.iter().map(|t| self.generate_type(t)).collect();
                format!("tuple[{}]", parts?.join(", "))
            }
            Type::Array { element, .. } => {
                let inner = self.generate_type(element)?;
                format!("list[{}]", inner)
            }
            Type::Slice(inner) => {
                let inner_str = self.generate_type(inner)?;
                format!("list[{}]", inner_str)
            }
            Type::Function { params, return_type } => {
                // Use Callable from collections.abc style
                let params_str: Result<Vec<_>, _> =
                    params.iter().map(|t| self.generate_type(t)).collect();
                let ret = self.generate_type(return_type)?;
                format!("Callable[[{}], {}]", params_str?.join(", "), ret)
            }
            Type::Unit => "None".to_string(),
            Type::Never => "None".to_string(),
            Type::Infer => "Any".to_string(),
        };
        Ok(result)
    }

    fn generate_type_path(&mut self, path: &TypePath) -> Result<String, CompileError> {
        let segments: Vec<&str> = path.segments.iter().map(|s| s.node.as_str()).collect();
        Ok(segments.join("."))
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<String, CompileError> {
        let result = match expr {
            Expr::IntLiteral { value, .. } => value.to_string(),
            Expr::FloatLiteral { value, .. } => {
                let s = value.to_string();
                if s.contains('.') { s } else { format!("{}.0", s) }
            }
            Expr::StringLiteral { value, .. } => {
                format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
            }
            Expr::InterpolatedString { parts, .. } => {
                let mut result = String::from("f\"");
                for part in parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Expr(e) => {
                            result.push('{');
                            result.push_str(&self.generate_expr(e)?);
                            result.push('}');
                        }
                    }
                }
                result.push('"');
                result
            }
            Expr::BoolLiteral { value, .. } => if *value { "True" } else { "False" }.to_string(),
            Expr::CharLiteral { value, .. } => format!("\"{}\"", value),
            Expr::Identifier { name, .. } => name.clone(),
            Expr::Path { segments, .. } => {
                let parts: Vec<&str> = segments.iter().map(|s| s.node.as_str()).collect();
                parts.join(".")
            }
            Expr::Binary { left, op, right, .. } => {
                let left_str = self.generate_expr(left)?;
                let right_str = self.generate_expr(right)?;
                let op_str = self.binary_op_to_python(op);
                // Python integer division for integer context
                if matches!(op, BinaryOp::Div) {
                    format!("({} // {})", left_str, right_str)
                } else {
                    format!("({} {} {})", left_str, op_str, right_str)
                }
            }
            Expr::Unary { op, operand, .. } => {
                let operand_str = self.generate_expr(operand)?;
                let op_str = self.unary_op_to_python(op);
                format!("{}{}", op_str, operand_str)
            }
            Expr::Call { callee, args, .. } => {
                let callee_str = self.generate_expr(callee)?;
                let args_str: Result<Vec<_>, _> =
                    args.iter().map(|e| self.generate_expr(e)).collect();
                format!("{}({})", callee_str, args_str?.join(", "))
            }
            Expr::MethodCall { object, method, args, .. } => {
                let obj_str = self.generate_expr(object)?;
                let args_str: Result<Vec<_>, _> =
                    args.iter().map(|e| self.generate_expr(e)).collect();
                format!("{}.{}({})", obj_str, method.node, args_str?.join(", "))
            }
            Expr::FieldAccess { object, field, .. } => {
                let obj_str = self.generate_expr(object)?;
                format!("{}.{}", obj_str, field.node)
            }
            Expr::Index { object, index, .. } => {
                let obj_str = self.generate_expr(object)?;
                let idx_str = self.generate_expr(index)?;
                format!("{}[{}]", obj_str, idx_str)
            }
            Expr::Array { elements, .. } => {
                let elems: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.generate_expr(e)).collect();
                format!("[{}]", elems?.join(", "))
            }
            Expr::Tuple { elements, .. } => {
                let elems: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.generate_expr(e)).collect();
                let joined = elems?.join(", ");
                if elements.len() == 1 { format!("({},)", joined) } else { format!("({})", joined) }
            }
            Expr::StructInit { path, fields, .. } => {
                let path_str = self.generate_type_path(path)?;
                let flds: Result<Vec<_>, _> = fields
                    .iter()
                    .map(|f| {
                        if let Some(value) = &f.value {
                            let val = self.generate_expr(value)?;
                            Ok(format!("{}={}", f.name.node, val))
                        } else {
                            Ok(format!("{}={}", f.name.node, f.name.node))
                        }
                    })
                    .collect();
                format!("{}({})", path_str, flds?.join(", "))
            }
            Expr::If { condition, then_block, elif_clauses, else_block, .. } => {
                // Python if/elif/else as an expression (ternary) only works for
                // single-expression cases. We generate a full statement form.
                let cond_str = self.generate_expr(condition)?;
                let then_str = self.generate_block(then_block)?;
                let mut result = format!("if {}:\n{}", cond_str, then_str);

                for (elif_cond, elif_block) in elif_clauses {
                    let elif_cond_str = self.generate_expr(elif_cond)?;
                    let elif_block_str = self.generate_block(elif_block)?;
                    result.push_str(&format!("elif {}:\n{}", elif_cond_str, elif_block_str));
                }

                if let Some(else_blk) = else_block {
                    let else_str = self.generate_block(else_blk)?;
                    result.push_str(&format!("else:\n{}", else_str));
                }

                result
            }
            Expr::Match { scrutinee, arms, .. } => {
                let val = self.generate_expr(scrutinee)?;
                let mut result = format!("match {}:\n", val);

                for arm in arms {
                    let pat = self.generate_pattern(&arm.pattern)?;
                    result.push_str(&self.config.indent);
                    result.push_str(&format!("case {}", pat));

                    if let Some(guard) = &arm.guard {
                        let guard_str = self.generate_expr(guard)?;
                        result.push_str(&format!(" if {}", guard_str));
                    }
                    result.push_str(":\n");

                    // arm body — indent one more level
                    let body_str = self.generate_expr(&arm.body)?;
                    result.push_str(&self.config.indent);
                    result.push_str(&self.config.indent);
                    result.push_str(&body_str);
                    result.push('\n');
                }
                result
            }
            Expr::Reference { inner, .. } | Expr::Deref { inner, .. } => {
                // No references in Python
                self.generate_expr(inner)?
            }
            Expr::Range { start, end, inclusive, .. } => {
                let start_str = start
                    .as_ref()
                    .map(|e| self.generate_expr(e))
                    .transpose()?
                    .unwrap_or_else(|| "0".to_string());
                let end_str =
                    end.as_ref().map(|e| self.generate_expr(e)).transpose()?.unwrap_or_default();
                if *inclusive && !end_str.is_empty() {
                    format!("range({}, {} + 1)", start_str, end_str)
                } else if !end_str.is_empty() {
                    format!("range({}, {})", start_str, end_str)
                } else {
                    format!("range({}, ...)", start_str)
                }
            }
            Expr::Lambda { params, body, .. } => {
                let params_str: Vec<String> = params.iter().map(|p| p.name.node.clone()).collect();
                let body_str = self.generate_expr(body)?;
                format!("lambda {}: {}", params_str.join(", "), body_str)
            }
            Expr::Await { inner, .. } => {
                let inner_str = self.generate_expr(inner)?;
                format!("await {}", inner_str)
            }
            Expr::Try { inner, .. } => {
                // The `?` operator — Python doesn't have this; just emit the expr.
                self.generate_expr(inner)?
            }
            Expr::Cast { expr, ty, .. } => {
                let expr_str = self.generate_expr(expr)?;
                let ty_str = self.generate_type(ty)?;
                format!("{}({})", ty_str, expr_str)
            }
            Expr::Block { block, .. } => {
                // Python doesn't have block expressions — generate inline.
                self.generate_block(block)?
            }
        };
        Ok(result)
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<String, CompileError> {
        let result = match stmt {
            Stmt::Let { pattern, ty, value, .. } => {
                let pat_str = self.generate_pattern(pattern)?;
                let mut s = String::new();

                if let Some(val) = value {
                    let val_str = self.generate_expr(val)?;
                    if let Some(annotation) = ty {
                        let ty_str = self.generate_type(annotation)?;
                        s.push_str(&format!("{}: {} = {}", pat_str, ty_str, val_str));
                    } else {
                        s.push_str(&format!("{} = {}", pat_str, val_str));
                    }
                } else if let Some(annotation) = ty {
                    let ty_str = self.generate_type(annotation)?;
                    s.push_str(&format!("{}: {}", pat_str, ty_str));
                } else {
                    s.push_str(&format!("{} = None", pat_str));
                }
                s
            }
            Stmt::Expr(expr) => self.generate_expr(expr)?,
            Stmt::Assignment { target, op, value, .. } => {
                let target_str = self.generate_expr(target)?;
                let op_str = self.assign_op_to_python(op);
                let val_str = self.generate_expr(value)?;
                format!("{} {} {}", target_str, op_str, val_str)
            }
            Stmt::For { pattern, iterable, body, .. } => {
                let pat = self.generate_pattern(pattern)?;
                let iter_str = self.generate_expr(iterable)?;
                let body_str = self.generate_block(body)?;
                format!("for {} in {}:\n{}", pat, iter_str, body_str)
            }
            Stmt::While { condition, body, .. } => {
                let cond = self.generate_expr(condition)?;
                let body_str = self.generate_block(body)?;
                format!("while {}:\n{}", cond, body_str)
            }
            Stmt::Loop { body, .. } => {
                let body_str = self.generate_block(body)?;
                format!("while True:\n{}", body_str)
            }
            Stmt::Return { value, .. } => {
                if let Some(val) = value {
                    let val_str = self.generate_expr(val)?;
                    format!("return {}", val_str)
                } else {
                    "return".to_string()
                }
            }
            Stmt::Break { .. } => "break".to_string(),
            Stmt::Continue { .. } => "continue".to_string(),
            Stmt::TryCatch { try_block, handlers, finally_block, .. } => {
                let mut result = String::from("try:\n");
                result.push_str(&self.generate_block(try_block)?);

                for handler in handlers {
                    if let Some(ty) = &handler.exception_type {
                        let ty_str = self.generate_type(ty)?;
                        if let Some(binding) = &handler.binding {
                            result.push_str(&format!("except {} as {}:\n", ty_str, binding.node));
                        } else {
                            result.push_str(&format!("except {}:\n", ty_str));
                        }
                    } else if let Some(binding) = &handler.binding {
                        result.push_str(&format!("except Exception as {}:\n", binding.node));
                    } else {
                        result.push_str("except Exception:\n");
                    }
                    result.push_str(&self.generate_block(&handler.body)?);
                }

                if let Some(finally) = finally_block {
                    result.push_str("finally:\n");
                    result.push_str(&self.generate_block(finally)?);
                }

                result
            }
            Stmt::Raise { exception, .. } => {
                let expr_str = self.generate_expr(exception)?;
                format!("raise {}", expr_str)
            }
        };
        Ok(result)
    }

    fn generate_block(&mut self, block: &Block) -> Result<String, CompileError> {
        let mut result = String::new();

        if block.stmts.is_empty() {
            result.push_str(&self.config.indent);
            result.push_str("pass\n");
            return Ok(result);
        }

        for stmt in &block.stmts {
            let stmt_str = self.generate_stmt(stmt)?;
            for line in stmt_str.lines() {
                result.push_str(&self.config.indent);
                result.push_str(line);
                result.push('\n');
            }
        }
        Ok(result)
    }

    fn generate_pattern(&mut self, pattern: &Pattern) -> Result<String, CompileError> {
        let result = match pattern {
            Pattern::Wildcard { .. } => "_".to_string(),
            Pattern::Identifier { name, .. } => name.node.clone(),
            Pattern::Literal { value, .. } => self.generate_expr(value)?,
            Pattern::Tuple { elements, .. } => {
                let elems: Result<Vec<_>, _> =
                    elements.iter().map(|p| self.generate_pattern(p)).collect();
                format!("({})", elems?.join(", "))
            }
            Pattern::Struct { path, fields, rest, .. } => {
                let path_str = self.generate_type_path(path)?;
                let flds: Vec<String> = fields
                    .iter()
                    .map(|f| {
                        if let Some(pat) = &f.pattern {
                            let ps = self.generate_pattern(pat).unwrap_or_default();
                            format!("{}={}", f.name.node, ps)
                        } else {
                            f.name.node.clone()
                        }
                    })
                    .collect();
                let mut parts = flds.join(", ");
                if *rest {
                    if !parts.is_empty() {
                        parts.push_str(", ");
                    }
                    parts.push_str("**_");
                }
                format!("{}({})", path_str, parts)
            }
            Pattern::Variant { path, data, .. } => {
                let path_str = self.generate_type_path(path)?;
                if let Some(data) = data {
                    let inner = self.generate_pattern(data)?;
                    format!("{}({})", path_str, inner)
                } else {
                    format!("{}()", path_str)
                }
            }
            Pattern::Slice { elements, .. } => {
                let elems: Result<Vec<_>, _> =
                    elements.iter().map(|p| self.generate_pattern(p)).collect();
                format!("[{}]", elems?.join(", "))
            }
            Pattern::Or { patterns, .. } => {
                let pats: Result<Vec<_>, _> =
                    patterns.iter().map(|p| self.generate_pattern(p)).collect();
                pats?.join(" | ")
            }
            Pattern::Range { start, end, inclusive, .. } => {
                let s =
                    start.as_ref().map(|e| self.generate_expr(e)).transpose()?.unwrap_or_default();
                let e =
                    end.as_ref().map(|e| self.generate_expr(e)).transpose()?.unwrap_or_default();
                // Python structural patterns don't distinguish inclusive/exclusive ranges directly.
                let _ = inclusive;
                format!("({}, {})", s, e)
            }
            Pattern::Binding { name, pattern, .. } => {
                // Python 3.10+ structural pattern matching doesn't have `@` binding
                // but walrus operator `:=` works on the outer level only.
                // Simplified: just emit the name.
                let _ = self.generate_pattern(pattern)?;
                name.node.clone()
            }
        };
        Ok(result)
    }

    fn config(&self) -> &CodegenConfig {
        &self.config
    }

    fn set_config(&mut self, config: CodegenConfig) {
        self.config = config;
        self.writer = CodeWriter::with_indent(&self.config.indent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::TomiParser;

    fn compile_to_python(source: &str) -> String {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("Lexer should succeed");
        let mut parser = TomiParser::new(tokens).with_source(source.to_string());
        let module = parser.parse().expect("Parser should succeed");

        let config = CodegenConfig { include_comments: false, ..CodegenConfig::default() };
        let mut backend = PythonBackend::new();
        backend.set_config(config);
        backend.generate_module(&module).expect("Codegen should succeed")
    }

    #[test]
    fn empty_module() {
        let py = compile_to_python("");
        assert!(py.contains("from __future__ import annotations"));
    }

    #[test]
    fn simple_function() {
        let py = compile_to_python("def hello():\n    return 0\n");
        assert!(py.contains("def hello()"));
        assert!(py.contains("return 0"));
    }

    #[test]
    fn bool_literals_capitalised() {
        let py = compile_to_python("def f():\n    let x = true\n    let y = false\n");
        assert!(py.contains("x = True"));
        assert!(py.contains("y = False"));
    }

    #[test]
    fn entrypoint_becomes_main() {
        let py = compile_to_python("@entrypoint\ndef main():\n    return 0\n");
        assert!(py.contains("def main()"));
        assert!(py.contains("if __name__ == \"__main__\":"));
        assert!(py.contains("main()"));
    }

    #[test]
    fn int_type_mapped() {
        let py = compile_to_python("def add(a: Int32, b: Int32) -> Int32:\n    return a + b\n");
        assert!(py.contains("a: int, b: int"));
        assert!(py.contains("-> int"));
    }
}
