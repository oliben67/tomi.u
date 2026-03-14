//! Type checking for the tomi.u compiler.
//!
//! This module walks the AST and validates that all expressions, statements,
//! and declarations are type-correct. It also performs type inference for
//! bindings without explicit annotations.
//!
//! ## Architecture
//!
//! 1. **Collection pass** — registers all top-level types, traits, and function
//!    signatures into the [`TypeRegistry`].
//! 2. **Checking pass** — visits every function body, infers types, and validates
//!    constraints. Type inference uses unification (see [`InferCtx`]).
//!
//! ## Sub-modules
//!
//! - [`infer`]: Unification-based type inference with constraint solving.
//! - [`traits`]: Trait bound verification and method resolution.
//! - [`exhaustiveness`]: Match-expression exhaustiveness analysis.

pub mod exhaustiveness;
pub mod infer;
pub mod traits;

use std::collections::HashMap;

use crate::ast;
use crate::error::CompileError;
use crate::span::Span;
use crate::types::*;

use self::infer::InferCtx;
use self::traits::TraitResolver;

// ═══════════════════════════════════════════════════════════════════════════════
// Scoped Environment
// ═══════════════════════════════════════════════════════════════════════════════

/// Variable binding in the local scope.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Binding {
    ty: Ty,
    is_mut: bool,
}

/// Lexical scope containing local variable bindings.
#[derive(Debug)]
struct Scope {
    bindings: HashMap<String, Binding>,
}

impl Scope {
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Type Checker
// ═══════════════════════════════════════════════════════════════════════════════

/// The main type checker.
pub struct TypeChecker {
    pub registry: TypeRegistry,
    /// Function signatures by name (top-level and methods).
    pub functions: HashMap<String, FunctionSig>,
    /// Stack of lexical scopes.
    scopes: Vec<Scope>,
    /// Inference context for the current function body.
    infer: InferCtx,
    /// Collected errors.
    errors: Vec<CompileError>,
    /// The return type expected for the current function.
    current_return_type: Option<Ty>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            registry: TypeRegistry::new(),
            functions: HashMap::new(),
            scopes: Vec::new(),
            infer: InferCtx::new(),
            errors: Vec::new(),
            current_return_type: None,
        };
        checker.register_builtins();
        checker
    }

    /// Register built-in functions available in every tomi.u program.
    fn register_builtins(&mut self) {
        let builtins = [
            ("print", vec![Ty::String], Ty::Unit),
            ("println", vec![Ty::String], Ty::Unit),
            ("io.println", vec![Ty::String], Ty::Unit),
            ("io.print", vec![Ty::String], Ty::Unit),
            ("io.eprintln", vec![Ty::String], Ty::Unit),
            ("dbg", vec![Ty::String], Ty::Unit),
        ];
        for (name, params, ret) in builtins {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    type_params: Vec::new(),
                    params,
                    ret,
                    span: Span::DUMMY,
                },
            );
        }
    }

    /// Type-check a module. Returns collected errors (empty on success).
    pub fn check_module(&mut self, module: &ast::Module) -> Vec<CompileError> {
        // Pass 1: collect all top-level declarations
        self.collect_module(module);

        // Pass 2: check function bodies
        for item in &module.items {
            self.check_item(item);
        }

        std::mem::take(&mut self.errors)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 1: Collection
    // ─────────────────────────────────────────────────────────────────────

    fn collect_module(&mut self, module: &ast::Module) {
        for item in &module.items {
            self.collect_item(item);
        }
    }

    fn collect_item(&mut self, item: &ast::Item) {
        match item {
            ast::Item::Struct(s) => self.collect_struct(s),
            ast::Item::Enum(e) => self.collect_enum(e),
            ast::Item::Trait(t) => self.collect_trait(t),
            ast::Item::Impl(i) => self.collect_impl(i),
            ast::Item::Function(f) => self.collect_function(f),
            ast::Item::TypeAlias(t) => self.collect_type_alias(t),
            ast::Item::Const(_) => { /* collected during check pass */ }
        }
    }

    fn collect_struct(&mut self, s: &ast::Struct) {
        let type_params: Vec<String> = s
            .type_params
            .iter()
            .map(|tp| tp.name.node.clone())
            .collect();
        let fields: Vec<FieldDef> = s
            .fields
            .iter()
            .map(|f| {
                let ty = self.resolve_type(&f.ty);
                FieldDef {
                    name: f.name.node.clone(),
                    ty,
                    span: f.span,
                }
            })
            .collect();

        let type_id = self.registry.register_struct(StructDef {
            name: s.name.node.clone(),
            type_params,
            fields,
            span: s.span,
        });

        // Collect methods
        for method in &s.methods {
            let sig = self.build_function_sig(method);
            let qualified = format!("{}.{}", s.name.node, method.name.node);
            self.functions.insert(qualified, sig);
        }

        // Register the type itself as an ADT
        let _ = type_id;
    }

    fn collect_enum(&mut self, e: &ast::Enum) {
        let type_params: Vec<String> = e
            .type_params
            .iter()
            .map(|tp| tp.name.node.clone())
            .collect();
        let variants: Vec<VariantDef> = e
            .variants
            .iter()
            .map(|v| {
                let data = match &v.data {
                    ast::VariantData::Unit => VariantDataDef::Unit,
                    ast::VariantData::Tuple(types) => {
                        VariantDataDef::Tuple(types.iter().map(|t| self.resolve_type(t)).collect())
                    }
                    ast::VariantData::Struct(fields) => VariantDataDef::Struct(
                        fields
                            .iter()
                            .map(|f| FieldDef {
                                name: f.name.node.clone(),
                                ty: self.resolve_type(&f.ty),
                                span: f.span,
                            })
                            .collect(),
                    ),
                };
                VariantDef {
                    name: v.name.node.clone(),
                    data,
                    span: v.span,
                }
            })
            .collect();

        self.registry.register_enum(EnumDef {
            name: e.name.node.clone(),
            type_params,
            variants,
            span: e.span,
        });

        // Collect methods
        for method in &e.methods {
            let sig = self.build_function_sig(method);
            let qualified = format!("{}.{}", e.name.node, method.name.node);
            self.functions.insert(qualified, sig);
        }
    }

    fn collect_trait(&mut self, t: &ast::Trait) {
        let type_params: Vec<String> = t
            .type_params
            .iter()
            .map(|tp| tp.name.node.clone())
            .collect();

        let methods: Vec<TraitMethodDef> = t
            .items
            .iter()
            .filter_map(|item| match item {
                ast::TraitItem::Method(f) => {
                    let params: Vec<Ty> = f
                        .params
                        .iter()
                        .filter(|p| p.name.node != "self")
                        .map(|p| self.resolve_type(&p.ty))
                        .collect();
                    let ret = f
                        .return_type
                        .as_ref()
                        .map(|t| self.resolve_type(t))
                        .unwrap_or(Ty::Unit);
                    Some(TraitMethodDef {
                        name: f.name.node.clone(),
                        params,
                        ret,
                        has_default: !f.body.stmts.is_empty(),
                        span: f.span,
                    })
                }
                _ => None,
            })
            .collect();

        // Resolve super traits
        let super_traits: Vec<TraitRef> = t
            .bounds
            .iter()
            .filter_map(|b| {
                let name = b
                    .segments
                    .iter()
                    .map(|s| s.node.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                let trait_id = self.registry.resolve_trait_name(&name)?;
                Some(TraitRef {
                    trait_id,
                    args: Vec::new(),
                })
            })
            .collect();

        self.registry.register_trait(TraitDef {
            name: t.name.node.clone(),
            type_params,
            super_traits,
            methods,
            span: t.span,
        });
    }

    fn collect_impl(&mut self, i: &ast::Impl) {
        let target_ty = self.resolve_type(&i.target);

        if let Some(trait_path) = &i.trait_path {
            let trait_name = trait_path
                .segments
                .iter()
                .map(|s| s.node.as_str())
                .collect::<Vec<_>>()
                .join(".");
            if let Some(trait_id) = self.registry.resolve_trait_name(&trait_name) {
                let method_names: Vec<String> =
                    i.items.iter().map(|f| f.name.node.clone()).collect();
                self.registry.register_impl(TraitImpl {
                    trait_ref: TraitRef {
                        trait_id,
                        args: Vec::new(),
                    },
                    target_ty: target_ty.clone(),
                    methods: method_names,
                    span: i.span,
                });
            }
        }

        // Collect method signatures
        for method in &i.items {
            let sig = self.build_function_sig(method);
            // Build a qualified name using the target type
            let target_name = format!("{}", target_ty);
            let qualified = format!("{}.{}", target_name, method.name.node);
            self.functions.insert(qualified, sig);
        }
    }

    fn collect_function(&mut self, f: &ast::Function) {
        let sig = self.build_function_sig(f);
        self.functions.insert(f.name.node.clone(), sig);
    }

    fn collect_type_alias(&mut self, t: &ast::TypeAlias) {
        let ty = self.resolve_type(&t.ty);
        self.registry.register_alias(t.name.node.clone(), ty);
    }

    fn build_function_sig(&mut self, f: &ast::Function) -> FunctionSig {
        let type_params: Vec<String> = f
            .type_params
            .iter()
            .map(|tp| tp.name.node.clone())
            .collect();
        let params: Vec<Ty> = f
            .params
            .iter()
            .filter(|p| p.name.node != "self")
            .map(|p| self.resolve_type(&p.ty))
            .collect();
        let ret = f
            .return_type
            .as_ref()
            .map(|t| self.resolve_type(t))
            .unwrap_or(Ty::Unit);

        FunctionSig {
            name: f.name.node.clone(),
            type_params,
            params,
            ret,
            span: f.span,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pass 2: Checking
    // ─────────────────────────────────────────────────────────────────────

    fn check_item(&mut self, item: &ast::Item) {
        match item {
            ast::Item::Function(f) => self.check_function(f),
            ast::Item::Struct(s) => {
                for method in &s.methods {
                    self.check_function(method);
                }
            }
            ast::Item::Enum(e) => {
                for method in &e.methods {
                    self.check_function(method);
                }
            }
            ast::Item::Impl(i) => {
                self.check_impl(i);
            }
            ast::Item::Trait(t) => {
                // Check default method implementations
                for item in &t.items {
                    if let ast::TraitItem::Method(f) = item {
                        if !f.body.stmts.is_empty() {
                            self.check_function(f);
                        }
                    }
                }
            }
            ast::Item::Const(c) => self.check_const(c),
            ast::Item::TypeAlias(_) => { /* already handled in collection */ }
        }
    }

    fn check_function(&mut self, f: &ast::Function) {
        // Reset inference context for this function
        self.infer = InferCtx::new();

        let ret_ty = f
            .return_type
            .as_ref()
            .map(|t| self.resolve_type(t))
            .unwrap_or(Ty::Unit);
        self.current_return_type = Some(ret_ty);

        self.push_scope();

        // Bind parameters
        for param in &f.params {
            let ty = if param.name.node == "self" {
                // `self` type resolved contextually — use a placeholder
                Ty::Error
            } else {
                self.resolve_type(&param.ty)
            };
            self.define_local(&param.name.node, ty, param.is_mut);
        }

        // Bind type parameters as opaque types
        for tp in &f.type_params {
            // Type parameters are treated as placeholder type variables
            let var = self.registry.fresh_type_var();
            self.define_local(&tp.name.node, var, false);
        }

        // Check body
        self.check_block(&f.body);

        self.pop_scope();
        self.current_return_type = None;
    }

    fn check_impl(&mut self, i: &ast::Impl) {
        // Verify that all required trait methods are implemented
        if let Some(trait_path) = &i.trait_path {
            let trait_name = trait_path
                .segments
                .iter()
                .map(|s| s.node.as_str())
                .collect::<Vec<_>>()
                .join(".");
            if let Some(trait_id) = self.registry.resolve_trait_name(&trait_name) {
                let resolver = TraitResolver::new(&self.registry);
                let impl_method_names: Vec<&str> =
                    i.items.iter().map(|f| f.name.node.as_str()).collect();
                if let Some(missing) =
                    resolver.check_impl_completeness(trait_id, &impl_method_names)
                {
                    for method_name in missing {
                        self.errors.push(CompileError::MissingTraitMethod {
                            trait_name: trait_name.clone(),
                            method: method_name,
                            span: i.span,
                        });
                    }
                }
            } else {
                self.errors.push(CompileError::UndefinedTrait {
                    name: trait_name,
                    span: trait_path.span,
                });
            }
        }

        for method in &i.items {
            self.check_function(method);
        }
    }

    fn check_const(&mut self, c: &ast::Const) {
        let expr_ty = self.infer_expr(&c.value);
        if let Some(ref ann) = c.ty {
            let expected = self.resolve_type(ann);
            self.unify(&expected, &expr_ty, c.span);
        }
    }

    fn check_block(&mut self, block: &ast::Block) {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Statement checking
    // ─────────────────────────────────────────────────────────────────────

    fn check_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::Let {
                is_mut,
                pattern,
                ty,
                value,
                span,
            } => {
                let declared_ty = ty.as_ref().map(|t| self.resolve_type(t));
                let inferred_ty = value.as_ref().map(|v| self.infer_expr(v));

                let final_ty = match (declared_ty, inferred_ty) {
                    (Some(decl), Some(inf)) => {
                        self.unify(&decl, &inf, *span);
                        decl
                    }
                    (Some(decl), None) => decl,
                    (None, Some(inf)) => inf,
                    (None, None) => {
                        self.errors
                            .push(CompileError::CannotInferType { span: *span });
                        Ty::Error
                    }
                };

                self.bind_pattern(pattern, &final_ty, *is_mut);
            }
            ast::Stmt::Expr(expr) => {
                self.infer_expr(expr);
            }
            ast::Stmt::Return { value, span } => {
                let ret_ty = match value {
                    Some(v) => self.infer_expr(v),
                    None => Ty::Unit,
                };
                if let Some(expected) = &self.current_return_type.clone() {
                    self.unify(expected, &ret_ty, *span);
                }
            }
            ast::Stmt::Assignment {
                target,
                value,
                span,
                ..
            } => {
                let target_ty = self.infer_expr(target);
                let value_ty = self.infer_expr(value);
                self.unify(&target_ty, &value_ty, *span);
            }
            ast::Stmt::For {
                pattern,
                iterable,
                body,
                span: _,
            } => {
                let iter_ty = self.infer_expr(iterable);
                // The element type is extracted from the iterable
                let elem_ty = self.element_type_of(&iter_ty);
                self.push_scope();
                self.bind_pattern(pattern, &elem_ty, false);
                self.check_block(body);
                self.pop_scope();
            }
            ast::Stmt::While {
                condition,
                body,
                span: _,
            } => {
                let cond_ty = self.infer_expr(condition);
                self.unify(&Ty::Bool, &cond_ty, condition.span());
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            ast::Stmt::Loop { body, .. } => {
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            ast::Stmt::Break { .. } | ast::Stmt::Continue { .. } => { /* no type work */ }
            ast::Stmt::TryCatch {
                try_block,
                handlers,
                finally_block,
                ..
            } => {
                self.push_scope();
                self.check_block(try_block);
                self.pop_scope();
                for handler in handlers {
                    self.push_scope();
                    if let Some(ref exc_type) = handler.exception_type {
                        let ty = self.resolve_type(exc_type);
                        if let Some(ref binding) = handler.binding {
                            self.define_local(&binding.node, ty, false);
                        }
                    }
                    self.check_block(&handler.body);
                    self.pop_scope();
                }
                if let Some(ref finally) = finally_block {
                    self.push_scope();
                    self.check_block(finally);
                    self.pop_scope();
                }
            }
            ast::Stmt::Raise { exception, .. } => {
                self.infer_expr(exception);
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Expression type inference
    // ─────────────────────────────────────────────────────────────────────

    fn infer_expr(&mut self, expr: &ast::Expr) -> Ty {
        match expr {
            // ── Literals ────────────────────────────────────────────────
            ast::Expr::IntLiteral { suffix, .. } => match suffix.as_deref() {
                Some("i8") => Ty::Int8,
                Some("i16") => Ty::Int16,
                Some("i64") => Ty::Int64,
                Some("u8") => Ty::UInt8,
                Some("u16") => Ty::UInt16,
                Some("u32") => Ty::UInt32,
                Some("u64") => Ty::UInt64,
                _ => Ty::Int32, // default integer type
            },
            ast::Expr::FloatLiteral { suffix, .. } => match suffix.as_deref() {
                Some("f32") => Ty::Float32,
                _ => Ty::Float64, // default float type
            },
            ast::Expr::StringLiteral { .. } | ast::Expr::InterpolatedString { .. } => Ty::String,
            ast::Expr::BoolLiteral { .. } => Ty::Bool,
            ast::Expr::CharLiteral { .. } => Ty::Char,

            // ── Variables ───────────────────────────────────────────────
            ast::Expr::Identifier { name, span } => self.lookup_var(name, *span),

            ast::Expr::Path { segments, span } => {
                let name = segments
                    .iter()
                    .map(|s| s.node.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                self.lookup_var(&name, *span)
            }

            // ── Compound ────────────────────────────────────────────────
            ast::Expr::Tuple { elements, .. } => {
                let tys: Vec<Ty> = elements.iter().map(|e| self.infer_expr(e)).collect();
                if tys.is_empty() {
                    Ty::Unit
                } else {
                    Ty::Tuple(tys)
                }
            }
            ast::Expr::Array { elements, span: _ } => {
                if elements.is_empty() {
                    let elem = self.registry.fresh_type_var();
                    Ty::Array(Box::new(elem), Some(0))
                } else {
                    let first = self.infer_expr(&elements[0]);
                    for elem in &elements[1..] {
                        let ty = self.infer_expr(elem);
                        self.unify(&first, &ty, elem.span());
                    }
                    Ty::Array(Box::new(first), Some(elements.len()))
                }
            }
            ast::Expr::StructInit { path, fields, span } => {
                let name = path
                    .segments
                    .iter()
                    .map(|s| s.node.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                if let Some(type_id) = self.registry.resolve_name(&name) {
                    // Verify fields
                    if let Some(struct_def) = self.registry.lookup_struct(type_id).cloned() {
                        for field_init in fields {
                            if let Some(value) = &field_init.value {
                                let val_ty = self.infer_expr(value);
                                if let Some(field_def) = struct_def
                                    .fields
                                    .iter()
                                    .find(|f| f.name == field_init.name.node)
                                {
                                    self.unify(&field_def.ty, &val_ty, field_init.span);
                                } else {
                                    self.errors.push(CompileError::UndefinedField {
                                        struct_name: name.clone(),
                                        field: field_init.name.node.clone(),
                                        span: field_init.span,
                                    });
                                }
                            }
                        }
                        Ty::Adt(type_id, Vec::new())
                    } else {
                        self.errors.push(CompileError::UndefinedType {
                            name: name.clone(),
                            span: *span,
                        });
                        Ty::Error
                    }
                } else {
                    self.errors
                        .push(CompileError::UndefinedType { name, span: *span });
                    Ty::Error
                }
            }

            // ── Operators ───────────────────────────────────────────────
            ast::Expr::Binary {
                left,
                op,
                right,
                span,
            } => {
                let left_ty = self.infer_expr(left);
                let right_ty = self.infer_expr(right);
                self.check_binary_op(*op, &left_ty, &right_ty, *span)
            }
            ast::Expr::Unary { op, operand, span } => {
                let operand_ty = self.infer_expr(operand);
                self.check_unary_op(*op, &operand_ty, *span)
            }

            // ── Access ──────────────────────────────────────────────────
            ast::Expr::FieldAccess {
                object,
                field,
                span,
            } => {
                let obj_ty = self.infer_expr(object);
                self.resolve_field_type(&obj_ty, &field.node, *span)
            }
            ast::Expr::MethodCall {
                object,
                method,
                args,
                span,
                ..
            } => {
                let obj_ty = self.infer_expr(object);
                let arg_tys: Vec<Ty> = args.iter().map(|a| self.infer_expr(a)).collect();
                self.resolve_method_call(&obj_ty, &method.node, &arg_tys, *span)
            }
            ast::Expr::Index {
                object,
                index,
                span,
            } => {
                let obj_ty = self.infer_expr(object);
                let idx_ty = self.infer_expr(index);
                self.check_index_op(&obj_ty, &idx_ty, *span)
            }

            // ── Calls ───────────────────────────────────────────────────
            ast::Expr::Call {
                callee, args, span, ..
            } => {
                let callee_ty = self.infer_expr(callee);
                let arg_tys: Vec<Ty> = args.iter().map(|a| self.infer_expr(a)).collect();
                self.check_call(&callee_ty, &arg_tys, *span)
            }

            // ── Control flow ────────────────────────────────────────────
            ast::Expr::If {
                condition,
                then_block,
                elif_clauses,
                else_block,
                span,
            } => {
                let cond_ty = self.infer_expr(condition);
                self.unify(&Ty::Bool, &cond_ty, condition.span());

                self.push_scope();
                let then_ty = self.infer_block_ty(then_block);
                self.pop_scope();

                for (elif_cond, elif_block) in elif_clauses {
                    let elif_cond_ty = self.infer_expr(elif_cond);
                    self.unify(&Ty::Bool, &elif_cond_ty, elif_cond.span());
                    self.push_scope();
                    let elif_ty = self.infer_block_ty(elif_block);
                    self.pop_scope();
                    self.unify(&then_ty, &elif_ty, *span);
                }

                if let Some(else_blk) = else_block {
                    self.push_scope();
                    let else_ty = self.infer_block_ty(else_blk);
                    self.pop_scope();
                    self.unify(&then_ty, &else_ty, *span);
                }

                then_ty
            }
            ast::Expr::Match {
                scrutinee,
                arms,
                span,
            } => {
                let scrut_ty = self.infer_expr(scrutinee);

                // Check exhaustiveness
                self.check_exhaustiveness(&scrut_ty, arms, *span);

                let result_ty = self.registry.fresh_type_var();

                for arm in arms {
                    self.push_scope();
                    self.check_pattern(&arm.pattern, &scrut_ty);
                    if let Some(ref guard) = arm.guard {
                        let guard_ty = self.infer_expr(guard);
                        self.unify(&Ty::Bool, &guard_ty, guard.span());
                    }
                    let body_ty = self.infer_expr(&arm.body);
                    self.unify(&result_ty, &body_ty, arm.span);
                    self.pop_scope();
                }

                self.apply_subst(&result_ty)
            }

            // ── References ──────────────────────────────────────────────
            ast::Expr::Reference { is_mut, inner, .. } => {
                let inner_ty = self.infer_expr(inner);
                Ty::Reference {
                    is_mut: *is_mut,
                    inner: Box::new(inner_ty),
                }
            }
            ast::Expr::Deref { inner, span } => {
                let inner_ty = self.infer_expr(inner);
                match inner_ty {
                    Ty::Reference { inner, .. } => *inner,
                    _ => {
                        self.errors.push(CompileError::TypeMismatch {
                            expected: "reference type".into(),
                            found: format!("{inner_ty}"),
                            span: *span,
                        });
                        Ty::Error
                    }
                }
            }

            // ── Ranges ──────────────────────────────────────────────────
            ast::Expr::Range { start, end, .. } => {
                if let Some(s) = start {
                    let s_ty = self.infer_expr(s);
                    if let Some(e) = end {
                        let e_ty = self.infer_expr(e);
                        self.unify(&s_ty, &e_ty, s.span());
                    }
                } else if let Some(e) = end {
                    self.infer_expr(e);
                }
                // Range type — simplified for now
                Ty::Adt(TypeId(u32::MAX), Vec::new())
            }

            // ── Lambda ──────────────────────────────────────────────────
            ast::Expr::Lambda {
                params,
                return_type,
                body,
                ..
            } => {
                self.push_scope();
                let param_tys: Vec<Ty> = params
                    .iter()
                    .map(|p| {
                        let ty =
                            p.ty.as_ref()
                                .map(|t| self.resolve_type(t))
                                .unwrap_or_else(|| self.registry.fresh_type_var());
                        self.define_local(&p.name.node, ty.clone(), false);
                        ty
                    })
                    .collect();
                let body_ty = self.infer_expr(body);
                self.pop_scope();

                let ret = return_type
                    .as_ref()
                    .map(|t| self.resolve_type(t))
                    .unwrap_or(body_ty);

                Ty::Function {
                    params: param_tys,
                    ret: Box::new(ret),
                }
            }

            // ── Async/await ─────────────────────────────────────────────
            ast::Expr::Await { inner, .. } => {
                // For now, await unwraps the future type
                self.infer_expr(inner)
            }

            // ── Try ─────────────────────────────────────────────────────
            ast::Expr::Try { inner, .. } => self.infer_expr(inner),

            // ── Cast ────────────────────────────────────────────────────
            ast::Expr::Cast { expr, ty, span } => {
                let expr_ty = self.infer_expr(expr);
                let target_ty = self.resolve_type(ty);
                // Validate cast compatibility
                if !self.can_cast(&expr_ty, &target_ty) {
                    self.errors.push(CompileError::InvalidCast {
                        from: format!("{expr_ty}"),
                        to: format!("{target_ty}"),
                        span: *span,
                    });
                }
                target_ty
            }

            // ── Block ───────────────────────────────────────────────────
            ast::Expr::Block { block, .. } => {
                self.push_scope();
                let ty = self.infer_block_ty(block);
                self.pop_scope();
                ty
            }
        }
    }

    /// Infer the type of a block (type of the last expression, or Unit).
    fn infer_block_ty(&mut self, block: &ast::Block) -> Ty {
        let mut last_ty = Ty::Unit;
        for stmt in &block.stmts {
            match stmt {
                ast::Stmt::Expr(expr) => {
                    last_ty = self.infer_expr(expr);
                }
                other => {
                    self.check_stmt(other);
                    last_ty = Ty::Unit;
                }
            }
        }
        last_ty
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pattern checking
    // ─────────────────────────────────────────────────────────────────────

    fn check_pattern(&mut self, pattern: &ast::Pattern, expected: &Ty) {
        match pattern {
            ast::Pattern::Wildcard { .. } => { /* matches anything */ }
            ast::Pattern::Identifier { is_mut, name, .. } => {
                self.define_local(&name.node, expected.clone(), *is_mut);
            }
            ast::Pattern::Literal { value, span } => {
                let lit_ty = self.infer_expr(value);
                self.unify(expected, &lit_ty, *span);
            }
            ast::Pattern::Tuple { elements, span } => {
                if let Ty::Tuple(tys) = expected {
                    if elements.len() != tys.len() {
                        self.errors.push(CompileError::TypeMismatch {
                            expected: format!("tuple of {} elements", tys.len()),
                            found: format!("tuple of {} elements", elements.len()),
                            span: *span,
                        });
                    } else {
                        for (pat, ty) in elements.iter().zip(tys.iter()) {
                            self.check_pattern(pat, ty);
                        }
                    }
                }
            }
            ast::Pattern::Struct { path, fields, .. } => {
                let name = path
                    .segments
                    .iter()
                    .map(|s| s.node.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                if let Some(type_id) = self.registry.resolve_name(&name) {
                    if let Some(struct_def) = self.registry.lookup_struct(type_id).cloned() {
                        for pf in fields {
                            if let Some(fd) =
                                struct_def.fields.iter().find(|f| f.name == pf.name.node)
                            {
                                if let Some(ref inner_pat) = pf.pattern {
                                    self.check_pattern(inner_pat, &fd.ty);
                                } else {
                                    self.define_local(&pf.name.node, fd.ty.clone(), false);
                                }
                            }
                        }
                    }
                }
            }
            ast::Pattern::Variant { path, data, .. } => {
                let name = path
                    .segments
                    .iter()
                    .map(|s| s.node.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                // Try to look up as an enum variant
                if let Some(ref inner) = data {
                    // We'd need to resolve the variant's associated type
                    let var_ty = self.registry.fresh_type_var();
                    self.check_pattern(inner, &var_ty);
                }
                let _ = name;
            }
            ast::Pattern::Or { patterns, .. } => {
                for pat in patterns {
                    self.check_pattern(pat, expected);
                }
            }
            ast::Pattern::Binding { name, pattern, .. } => {
                self.define_local(&name.node, expected.clone(), false);
                self.check_pattern(pattern, expected);
            }
            ast::Pattern::Slice {
                elements,
                rest,
                span,
            } => {
                let elem_ty = match expected {
                    Ty::Array(elem, _) | Ty::Slice(elem) => (**elem).clone(),
                    _ => {
                        self.errors.push(CompileError::TypeMismatch {
                            expected: "array or slice".into(),
                            found: format!("{expected}"),
                            span: *span,
                        });
                        Ty::Error
                    }
                };
                for pat in elements {
                    self.check_pattern(pat, &elem_ty);
                }
                if let Some(rest_pat) = rest {
                    self.check_pattern(rest_pat, expected);
                }
            }
            ast::Pattern::Range { .. } => { /* range patterns match numeric types */ }
        }
    }

    fn bind_pattern(&mut self, pattern: &ast::Pattern, ty: &Ty, is_mut: bool) {
        match pattern {
            ast::Pattern::Identifier { name, .. } => {
                self.define_local(&name.node, ty.clone(), is_mut);
            }
            ast::Pattern::Wildcard { .. } => {}
            ast::Pattern::Tuple { elements, .. } => {
                if let Ty::Tuple(tys) = ty {
                    for (pat, ty) in elements.iter().zip(tys.iter()) {
                        self.bind_pattern(pat, ty, is_mut);
                    }
                } else {
                    // Bind to Error to avoid cascading errors
                    for pat in elements {
                        self.bind_pattern(pat, &Ty::Error, is_mut);
                    }
                }
            }
            _ => {
                self.check_pattern(pattern, ty);
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Operator type checking
    // ─────────────────────────────────────────────────────────────────────

    fn check_binary_op(&mut self, op: ast::BinaryOp, left: &Ty, right: &Ty, span: Span) -> Ty {
        use ast::BinaryOp::*;
        match op {
            // Arithmetic: both sides numeric, same type
            Add | Sub | Mul | Div | Mod => {
                self.unify(left, right, span);
                if left.is_numeric() || matches!(left, Ty::Error | Ty::TypeVar(_)) {
                    left.clone()
                } else if op == Add && matches!(left, Ty::String) {
                    Ty::String
                } else {
                    self.errors.push(CompileError::TypeMismatch {
                        expected: "numeric type".into(),
                        found: format!("{left}"),
                        span,
                    });
                    Ty::Error
                }
            }
            // Comparison: both sides same type, result Bool
            Eq | Ne | Lt | Le | Gt | Ge => {
                self.unify(left, right, span);
                Ty::Bool
            }
            // Logical: both Bool
            And | Or => {
                self.unify(&Ty::Bool, left, span);
                self.unify(&Ty::Bool, right, span);
                Ty::Bool
            }
            // Bitwise: both integer
            BitAnd | BitOr | BitXor | Shl | Shr => {
                self.unify(left, right, span);
                if left.is_integer() || matches!(left, Ty::Error | Ty::TypeVar(_)) {
                    left.clone()
                } else {
                    self.errors.push(CompileError::TypeMismatch {
                        expected: "integer type".into(),
                        found: format!("{left}"),
                        span,
                    });
                    Ty::Error
                }
            }
        }
    }

    fn check_unary_op(&mut self, op: ast::UnaryOp, operand: &Ty, span: Span) -> Ty {
        use ast::UnaryOp::*;
        match op {
            Not => {
                if matches!(operand, Ty::Bool | Ty::Error | Ty::TypeVar(_)) {
                    Ty::Bool
                } else {
                    self.errors.push(CompileError::TypeMismatch {
                        expected: "Bool".into(),
                        found: format!("{operand}"),
                        span,
                    });
                    Ty::Error
                }
            }
            Neg => {
                if operand.is_signed()
                    || operand.is_float()
                    || matches!(operand, Ty::Error | Ty::TypeVar(_))
                {
                    operand.clone()
                } else {
                    self.errors.push(CompileError::TypeMismatch {
                        expected: "signed numeric type".into(),
                        found: format!("{operand}"),
                        span,
                    });
                    Ty::Error
                }
            }
            BitNot => {
                if operand.is_integer() || matches!(operand, Ty::Error | Ty::TypeVar(_)) {
                    operand.clone()
                } else {
                    self.errors.push(CompileError::TypeMismatch {
                        expected: "integer type".into(),
                        found: format!("{operand}"),
                        span,
                    });
                    Ty::Error
                }
            }
            Ref => Ty::Reference {
                is_mut: false,
                inner: Box::new(operand.clone()),
            },
            MutRef => Ty::Reference {
                is_mut: true,
                inner: Box::new(operand.clone()),
            },
            Deref => match operand {
                Ty::Reference { inner, .. } => (**inner).clone(),
                _ => {
                    self.errors.push(CompileError::TypeMismatch {
                        expected: "reference type".into(),
                        found: format!("{operand}"),
                        span,
                    });
                    Ty::Error
                }
            },
        }
    }

    fn check_call(&mut self, callee: &Ty, args: &[Ty], span: Span) -> Ty {
        match callee {
            Ty::Function { params, ret } => {
                if params.len() != args.len() {
                    self.errors.push(CompileError::ArgCountMismatch {
                        expected: params.len(),
                        found: args.len(),
                        span,
                    });
                } else {
                    for (param, arg) in params.iter().zip(args.iter()) {
                        self.unify(param, arg, span);
                    }
                }
                (**ret).clone()
            }
            Ty::Error => Ty::Error,
            Ty::TypeVar(_) => {
                // Can't resolve call on type variable yet — return a fresh variable
                self.registry.fresh_type_var()
            }
            _ => {
                self.errors.push(CompileError::NotCallable {
                    ty: format!("{callee}"),
                    span,
                });
                Ty::Error
            }
        }
    }

    fn check_index_op(&mut self, obj: &Ty, idx: &Ty, span: Span) -> Ty {
        match obj {
            Ty::Array(elem, _) | Ty::Slice(elem) => {
                if !idx.is_integer() && !matches!(idx, Ty::Error | Ty::TypeVar(_)) {
                    self.errors.push(CompileError::TypeMismatch {
                        expected: "integer index".into(),
                        found: format!("{idx}"),
                        span,
                    });
                }
                (**elem).clone()
            }
            Ty::Error => Ty::Error,
            Ty::TypeVar(_) => self.registry.fresh_type_var(),
            _ => {
                self.errors.push(CompileError::TypeMismatch {
                    expected: "indexable type".into(),
                    found: format!("{obj}"),
                    span,
                });
                Ty::Error
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Field / method resolution
    // ─────────────────────────────────────────────────────────────────────

    fn resolve_field_type(&mut self, obj: &Ty, field: &str, span: Span) -> Ty {
        match obj {
            Ty::Adt(id, _) => {
                if let Some(def) = self.registry.lookup_struct(*id).cloned() {
                    if let Some(fd) = def.fields.iter().find(|f| f.name == field) {
                        return fd.ty.clone();
                    }
                    self.errors.push(CompileError::UndefinedField {
                        struct_name: def.name.clone(),
                        field: field.to_string(),
                        span,
                    });
                }
                Ty::Error
            }
            Ty::Tuple(tys) => {
                // Allow tuple.0, tuple.1, etc.
                if let Ok(idx) = field.parse::<usize>() {
                    if idx < tys.len() {
                        return tys[idx].clone();
                    }
                }
                self.errors.push(CompileError::TypeMismatch {
                    expected: "valid tuple index".into(),
                    found: field.to_string(),
                    span,
                });
                Ty::Error
            }
            Ty::Error => Ty::Error,
            Ty::TypeVar(_) => self.registry.fresh_type_var(),
            _ => {
                self.errors.push(CompileError::TypeMismatch {
                    expected: "struct or tuple type".into(),
                    found: format!("{obj}"),
                    span,
                });
                Ty::Error
            }
        }
    }

    fn resolve_method_call(&mut self, obj: &Ty, method: &str, _args: &[Ty], _span: Span) -> Ty {
        // Look up method via the qualified name pattern: Type.method
        match obj {
            Ty::Adt(id, _) => {
                // Try struct name first, then enum name
                let type_name = self
                    .registry
                    .lookup_struct(*id)
                    .map(|d| d.name.clone())
                    .or_else(|| self.registry.lookup_enum(*id).map(|d| d.name.clone()));
                if let Some(name) = type_name {
                    let qualified = format!("{}.{}", name, method);
                    if let Some(sig) = self.functions.get(&qualified).cloned() {
                        return sig.ret;
                    }
                }
                // Fall through: return a fresh var to avoid cascading errors
                self.registry.fresh_type_var()
            }
            Ty::String => {
                // Built-in String methods
                match method {
                    "len" => Ty::UInt64,
                    "is_empty" => Ty::Bool,
                    "to_string" | "clone" => Ty::String,
                    _ => self.registry.fresh_type_var(),
                }
            }
            Ty::Error => Ty::Error,
            _ => self.registry.fresh_type_var(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────

    fn resolve_type(&mut self, ast_type: &ast::Type) -> Ty {
        if let ast::Type::Infer = ast_type {
            return self.registry.fresh_type_var();
        }
        self.registry.resolve_ast_type(ast_type).unwrap_or_else(|| {
            // If we can't resolve, check if it's a type param in scope
            if let ast::Type::Named(path) = ast_type {
                let name = path
                    .segments
                    .iter()
                    .map(|s| s.node.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                if let Some(binding) = self.lookup_binding(&name) {
                    return binding.ty.clone();
                }
            }
            self.registry.fresh_type_var()
        })
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_local(&mut self, name: &str, ty: Ty, is_mut: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope
                .bindings
                .insert(name.to_string(), Binding { ty, is_mut });
        }
    }

    fn lookup_binding(&self, name: &str) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(b) = scope.bindings.get(name) {
                return Some(b);
            }
        }
        None
    }

    fn lookup_var(&mut self, name: &str, span: Span) -> Ty {
        // Check local scopes
        if let Some(b) = self.lookup_binding(name) {
            return b.ty.clone();
        }

        // Check function signatures
        if let Some(sig) = self.functions.get(name).cloned() {
            return Ty::Function {
                params: sig.params,
                ret: Box::new(sig.ret),
            };
        }

        // Check if it's a type constructor
        if let Some(_id) = self.registry.resolve_name(name) {
            // Could be an enum variant used as an identifier
            return self.registry.fresh_type_var();
        }

        // Built-in identifiers
        match name {
            "true" | "false" => return Ty::Bool,
            "Ok" | "Err" | "Some" | "None" => return self.registry.fresh_type_var(),
            _ => {}
        }

        self.errors.push(CompileError::UndefinedVariable {
            name: name.to_string(),
            span,
        });
        Ty::Error
    }

    fn element_type_of(&mut self, ty: &Ty) -> Ty {
        match ty {
            Ty::Array(elem, _) | Ty::Slice(elem) => (**elem).clone(),
            Ty::Error => Ty::Error,
            _ => self.registry.fresh_type_var(),
        }
    }

    fn can_cast(&self, from: &Ty, to: &Ty) -> bool {
        if from == to || matches!(from, Ty::Error) || matches!(to, Ty::Error) {
            return true;
        }
        // Allow numeric casts
        if from.is_numeric() && to.is_numeric() {
            return true;
        }
        // Allow type var casts
        if matches!(from, Ty::TypeVar(_)) || matches!(to, Ty::TypeVar(_)) {
            return true;
        }
        false
    }

    fn unify(&mut self, expected: &Ty, found: &Ty, span: Span) {
        if let Err(err) = self.infer.unify(expected, found) {
            match err {
                infer::UnifyError::Mismatch(e, f) => {
                    self.errors.push(CompileError::TypeMismatch {
                        expected: format!("{e}"),
                        found: format!("{f}"),
                        span,
                    });
                }
                infer::UnifyError::OccursCheck => {
                    self.errors.push(CompileError::TypeMismatch {
                        expected: format!("{expected}"),
                        found: "recursive type".into(),
                        span,
                    });
                }
            }
        }
    }

    fn apply_subst(&self, ty: &Ty) -> Ty {
        self.infer.apply(ty)
    }

    fn check_exhaustiveness(&mut self, scrutinee_ty: &Ty, arms: &[ast::MatchArm], span: Span) {
        let result = exhaustiveness::check(scrutinee_ty, arms, &self.registry);
        if let Some(missing) = result {
            self.errors.push(CompileError::NonExhaustiveMatch {
                missing_patterns: missing,
                span,
            });
        }
    }
}
