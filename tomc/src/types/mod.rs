//! Internal type representation for the tomi.u compiler.
//!
//! This module defines the compiler's internal type system, distinct from
//! the AST [`Type`] nodes produced by the parser. While AST types are
//! syntactic (what the user wrote), [`Ty`] represents resolved, semantic types.
//!
//! ## Key concepts
//!
//! - [`Ty`]: The resolved type representation used during type checking.
//! - [`TypeId`]: An interned identifier for user-defined types.
//! - [`TypeVarId`]: A placeholder for not-yet-inferred types.
//! - [`TypeRegistry`]: Stores struct/enum/trait definitions and tracks type variables.

use std::collections::HashMap;
use std::fmt;

use crate::ast;
use crate::span::Span;

// ═══════════════════════════════════════════════════════════════════════════════
// Type IDs
// ═══════════════════════════════════════════════════════════════════════════════

/// Unique identifier for a user-defined type (struct, enum, or trait).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// Unique identifier for a type variable (used during inference).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVarId(pub u32);

// ═══════════════════════════════════════════════════════════════════════════════
// Core Type
// ═══════════════════════════════════════════════════════════════════════════════

/// The compiler's internal type representation.
///
/// Unlike the AST `Type`, this is fully resolved: named types are replaced
/// by their `TypeId`, generics carry resolved arguments, and inference
/// variables are explicit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    // ── Primitives ──────────────────────────────────────────────────────
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Char,
    String,
    Unit,
    Never,

    // ── Compound ────────────────────────────────────────────────────────
    Tuple(Vec<Ty>),
    Array(Box<Ty>, Option<usize>),
    Slice(Box<Ty>),
    Reference {
        is_mut: bool,
        inner: Box<Ty>,
    },
    Optional(Box<Ty>),
    Function {
        params: Vec<Ty>,
        ret: Box<Ty>,
    },

    // ── User-defined ────────────────────────────────────────────────────
    /// A struct or enum identified by its TypeId, with applied generic args.
    Adt(TypeId, Vec<Ty>),

    // ── Inference ───────────────────────────────────────────────────────
    /// An unresolved type variable produced during inference.
    TypeVar(TypeVarId),

    // ── Error sentinel ──────────────────────────────────────────────────
    /// A poisoned type used to suppress cascading errors.
    Error,
}

impl Ty {
    /// Return `true` if this is a numeric type.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Ty::Int8
                | Ty::Int16
                | Ty::Int32
                | Ty::Int64
                | Ty::UInt8
                | Ty::UInt16
                | Ty::UInt32
                | Ty::UInt64
                | Ty::Float32
                | Ty::Float64
        )
    }

    /// Return `true` if this is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Ty::Int8
                | Ty::Int16
                | Ty::Int32
                | Ty::Int64
                | Ty::UInt8
                | Ty::UInt16
                | Ty::UInt32
                | Ty::UInt64
        )
    }

    /// Return `true` if this is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(self, Ty::Float32 | Ty::Float64)
    }

    /// Return `true` if this is a signed integer type.
    pub fn is_signed(&self) -> bool {
        matches!(self, Ty::Int8 | Ty::Int16 | Ty::Int32 | Ty::Int64)
    }

    /// Return `true` if this contains any unresolved type variables.
    pub fn has_type_vars(&self) -> bool {
        match self {
            Ty::TypeVar(_) => true,
            Ty::Tuple(elems) => elems.iter().any(Ty::has_type_vars),
            Ty::Array(elem, _) | Ty::Slice(elem) | Ty::Optional(elem) => elem.has_type_vars(),
            Ty::Reference { inner, .. } => inner.has_type_vars(),
            Ty::Function { params, ret } => {
                params.iter().any(Ty::has_type_vars) || ret.has_type_vars()
            }
            Ty::Adt(_, args) => args.iter().any(Ty::has_type_vars),
            _ => false,
        }
    }

    /// Substitute type variables using the given mapping.
    pub fn substitute(&self, subst: &HashMap<TypeVarId, Ty>) -> Ty {
        match self {
            Ty::TypeVar(id) => subst.get(id).cloned().unwrap_or_else(|| self.clone()),
            Ty::Tuple(elems) => Ty::Tuple(elems.iter().map(|t| t.substitute(subst)).collect()),
            Ty::Array(elem, sz) => Ty::Array(Box::new(elem.substitute(subst)), *sz),
            Ty::Slice(elem) => Ty::Slice(Box::new(elem.substitute(subst))),
            Ty::Optional(inner) => Ty::Optional(Box::new(inner.substitute(subst))),
            Ty::Reference { is_mut, inner } => {
                Ty::Reference { is_mut: *is_mut, inner: Box::new(inner.substitute(subst)) }
            }
            Ty::Function { params, ret } => Ty::Function {
                params: params.iter().map(|t| t.substitute(subst)).collect(),
                ret: Box::new(ret.substitute(subst)),
            },
            Ty::Adt(id, args) => Ty::Adt(*id, args.iter().map(|t| t.substitute(subst)).collect()),
            other => other.clone(),
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Bool => write!(f, "Bool"),
            Ty::Int8 => write!(f, "Int8"),
            Ty::Int16 => write!(f, "Int16"),
            Ty::Int32 => write!(f, "Int32"),
            Ty::Int64 => write!(f, "Int64"),
            Ty::UInt8 => write!(f, "UInt8"),
            Ty::UInt16 => write!(f, "UInt16"),
            Ty::UInt32 => write!(f, "UInt32"),
            Ty::UInt64 => write!(f, "UInt64"),
            Ty::Float32 => write!(f, "Float32"),
            Ty::Float64 => write!(f, "Float64"),
            Ty::Char => write!(f, "Char"),
            Ty::String => write!(f, "String"),
            Ty::Unit => write!(f, "()"),
            Ty::Never => write!(f, "!"),
            Ty::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{e}")?;
                }
                write!(f, ")")
            }
            Ty::Array(elem, Some(sz)) => write!(f, "[{elem}; {sz}]"),
            Ty::Array(elem, None) => write!(f, "[{elem}]"),
            Ty::Slice(elem) => write!(f, "[{elem}]"),
            Ty::Optional(inner) => write!(f, "?{inner}"),
            Ty::Reference { is_mut: true, inner } => write!(f, "&mut {inner}"),
            Ty::Reference { is_mut: false, inner } => write!(f, "&{inner}"),
            Ty::Function { params, ret } => {
                write!(f, "def(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ") -> {ret}")
            }
            Ty::Adt(id, args) => {
                write!(f, "Adt({})", id.0)?;
                if !args.is_empty() {
                    write!(f, "[")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{a}")?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            }
            Ty::TypeVar(id) => write!(f, "?T{}", id.0),
            Ty::Error => write!(f, "<error>"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// User-defined type definitions
// ═══════════════════════════════════════════════════════════════════════════════

/// Stored definition for a struct.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<FieldDef>,
    pub span: Span,
}

/// Stored definition for a struct field.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub ty: Ty,
    pub span: Span,
}

/// Stored definition for an enum.
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<VariantDef>,
    pub span: Span,
}

/// Stored definition for an enum variant.
#[derive(Debug, Clone)]
pub struct VariantDef {
    pub name: String,
    pub data: VariantDataDef,
    pub span: Span,
}

/// Data associated with a variant.
#[derive(Debug, Clone)]
pub enum VariantDataDef {
    Unit,
    Tuple(Vec<Ty>),
    Struct(Vec<FieldDef>),
}

/// Stored definition for a trait.
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub super_traits: Vec<TraitRef>,
    pub methods: Vec<TraitMethodDef>,
    pub span: Span,
}

/// A reference to a trait, possibly with generic args.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitRef {
    pub trait_id: TypeId,
    pub args: Vec<Ty>,
}

/// A method declared in a trait.
#[derive(Debug, Clone)]
pub struct TraitMethodDef {
    pub name: String,
    pub params: Vec<Ty>,
    pub ret: Ty,
    pub has_default: bool,
    pub span: Span,
}

/// A trait implementation: `impl Trait for Type`.
#[derive(Debug, Clone)]
pub struct TraitImpl {
    pub trait_ref: TraitRef,
    pub target_ty: Ty,
    pub methods: Vec<String>,
    pub span: Span,
}

/// A function signature stored in the type environment.
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub type_params: Vec<String>,
    pub params: Vec<Ty>,
    pub ret: Ty,
    pub span: Span,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Type Registry
// ═══════════════════════════════════════════════════════════════════════════════

/// Central storage for all type definitions, trait impls, and type variables.
#[derive(Debug)]
pub struct TypeRegistry {
    structs: HashMap<TypeId, StructDef>,
    enums: HashMap<TypeId, EnumDef>,
    traits: HashMap<TypeId, TraitDef>,
    trait_impls: Vec<TraitImpl>,
    type_aliases: HashMap<String, Ty>,

    /// Maps type names → TypeId.
    name_to_id: HashMap<String, TypeId>,
    /// Maps trait names → TypeId.
    trait_name_to_id: HashMap<String, TypeId>,

    next_type_id: u32,
    next_type_var: u32,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            trait_impls: Vec::new(),
            type_aliases: HashMap::new(),
            name_to_id: HashMap::new(),
            trait_name_to_id: HashMap::new(),
            next_type_id: 0,
            next_type_var: 0,
        }
    }

    /// Allocate a fresh type variable.
    pub fn fresh_type_var(&mut self) -> Ty {
        let id = TypeVarId(self.next_type_var);
        self.next_type_var += 1;
        Ty::TypeVar(id)
    }

    /// Allocate a fresh TypeId.
    fn alloc_id(&mut self) -> TypeId {
        let id = TypeId(self.next_type_id);
        self.next_type_id += 1;
        id
    }

    /// Register a struct definition, returning its TypeId.
    pub fn register_struct(&mut self, def: StructDef) -> TypeId {
        let id = self.alloc_id();
        self.name_to_id.insert(def.name.clone(), id);
        self.structs.insert(id, def);
        id
    }

    /// Register an enum definition, returning its TypeId.
    pub fn register_enum(&mut self, def: EnumDef) -> TypeId {
        let id = self.alloc_id();
        self.name_to_id.insert(def.name.clone(), id);
        self.enums.insert(id, def);
        id
    }

    /// Register a trait definition, returning its TypeId.
    pub fn register_trait(&mut self, def: TraitDef) -> TypeId {
        let id = self.alloc_id();
        self.trait_name_to_id.insert(def.name.clone(), id);
        self.traits.insert(id, def);
        id
    }

    /// Register a trait implementation.
    pub fn register_impl(&mut self, imp: TraitImpl) {
        self.trait_impls.push(imp);
    }

    /// Register a type alias.
    pub fn register_alias(&mut self, name: String, ty: Ty) {
        self.type_aliases.insert(name, ty);
    }

    pub fn lookup_struct(&self, id: TypeId) -> Option<&StructDef> {
        self.structs.get(&id)
    }

    pub fn lookup_enum(&self, id: TypeId) -> Option<&EnumDef> {
        self.enums.get(&id)
    }

    pub fn lookup_trait(&self, id: TypeId) -> Option<&TraitDef> {
        self.traits.get(&id)
    }

    /// Resolve a type name to its `TypeId`.
    pub fn resolve_name(&self, name: &str) -> Option<TypeId> {
        self.name_to_id.get(name).copied()
    }

    /// Resolve a trait name to its `TypeId`.
    pub fn resolve_trait_name(&self, name: &str) -> Option<TypeId> {
        self.trait_name_to_id.get(name).copied()
    }

    /// Look up a type alias.
    pub fn resolve_alias(&self, name: &str) -> Option<&Ty> {
        self.type_aliases.get(name)
    }

    /// Find all trait implementations for a given type.
    pub fn impls_for_type(&self, ty: &Ty) -> Vec<&TraitImpl> {
        self.trait_impls.iter().filter(|imp| &imp.target_ty == ty).collect()
    }

    /// Check if a trait is implemented for a type.
    pub fn has_impl(&self, trait_id: TypeId, ty: &Ty) -> bool {
        self.trait_impls
            .iter()
            .any(|imp| imp.trait_ref.trait_id == trait_id && &imp.target_ty == ty)
    }

    /// List all enum definitions (for exhaustiveness checking etc.)
    pub fn all_enums(&self) -> impl Iterator<Item = (TypeId, &EnumDef)> {
        self.enums.iter().map(|(&id, def)| (id, def))
    }

    /// Resolve an AST type path to a primitive type name.
    pub fn resolve_ast_type(&self, ast_type: &ast::Type) -> Option<Ty> {
        match ast_type {
            ast::Type::Named(path) => {
                let name =
                    path.segments.iter().map(|s| s.node.as_str()).collect::<Vec<_>>().join(".");
                self.resolve_primitive_or_named(&name)
            }
            ast::Type::Generic(path, args) => {
                let name =
                    path.segments.iter().map(|s| s.node.as_str()).collect::<Vec<_>>().join(".");
                let type_id = self.resolve_name(&name)?;
                let resolved_args: Option<Vec<Ty>> =
                    args.iter().map(|a| self.resolve_ast_type(a)).collect();
                Some(Ty::Adt(type_id, resolved_args?))
            }
            ast::Type::Function { params, return_type } => {
                let ps: Option<Vec<Ty>> = params.iter().map(|p| self.resolve_ast_type(p)).collect();
                let ret = self.resolve_ast_type(return_type)?;
                Some(Ty::Function { params: ps?, ret: Box::new(ret) })
            }
            ast::Type::Reference { is_mut, inner } => {
                let inner = self.resolve_ast_type(inner)?;
                Some(Ty::Reference { is_mut: *is_mut, inner: Box::new(inner) })
            }
            ast::Type::Optional(inner) => {
                let inner = self.resolve_ast_type(inner)?;
                Some(Ty::Optional(Box::new(inner)))
            }
            ast::Type::Tuple(elems) => {
                let resolved: Option<Vec<Ty>> =
                    elems.iter().map(|e| self.resolve_ast_type(e)).collect();
                Some(Ty::Tuple(resolved?))
            }
            ast::Type::Array { element, size: _ } => {
                let elem = self.resolve_ast_type(element)?;
                // For now, don't evaluate const expressions for size
                Some(Ty::Array(Box::new(elem), None))
            }
            ast::Type::Slice(inner) => {
                let inner = self.resolve_ast_type(inner)?;
                Some(Ty::Slice(Box::new(inner)))
            }
            ast::Type::Unit => Some(Ty::Unit),
            ast::Type::Never => Some(Ty::Never),
            ast::Type::Infer => None, // caller should create a type var
        }
    }

    /// Resolve a name that may be a primitive type or a registered type.
    fn resolve_primitive_or_named(&self, name: &str) -> Option<Ty> {
        // Check primitives first
        match name {
            "Bool" => return Some(Ty::Bool),
            "Int8" => return Some(Ty::Int8),
            "Int16" => return Some(Ty::Int16),
            "Int32" => return Some(Ty::Int32),
            "Int64" => return Some(Ty::Int64),
            "UInt8" => return Some(Ty::UInt8),
            "UInt16" => return Some(Ty::UInt16),
            "UInt32" => return Some(Ty::UInt32),
            "UInt64" => return Some(Ty::UInt64),
            "Float32" => return Some(Ty::Float32),
            "Float64" => return Some(Ty::Float64),
            "Char" => return Some(Ty::Char),
            "String" => return Some(Ty::String),
            _ => {}
        }

        // Check aliases
        if let Some(ty) = self.type_aliases.get(name) {
            return Some(ty.clone());
        }

        // Check registered types
        if let Some(&id) = self.name_to_id.get(name) {
            return Some(Ty::Adt(id, Vec::new()));
        }

        None
    }
}
