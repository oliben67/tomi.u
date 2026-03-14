//! Unification-based type inference for tomi.u.
//!
//! Implements a Hindley-Milner–style inference engine with:
//! - Type variable allocation
//! - Unification with occurs check
//! - Substitution application
//!
//! The inference context maintains a union-find–like substitution map
//! that grows as constraints are solved during type checking.

use std::collections::HashMap;

use crate::types::{Ty, TypeVarId};

/// Errors that can occur during unification.
#[derive(Debug)]
pub enum UnifyError {
    /// Two concrete types could not be unified.
    Mismatch(Ty, Ty),
    /// A type variable would need to be equal to a type containing itself.
    OccursCheck,
}

/// Inference context holding the current substitution.
pub struct InferCtx {
    /// Substitution map: TypeVarId → resolved Ty.
    subst: HashMap<TypeVarId, Ty>,
}

impl Default for InferCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl InferCtx {
    pub fn new() -> Self {
        Self { subst: HashMap::new() }
    }

    /// Unify two types, extending the substitution.
    pub fn unify(&mut self, a: &Ty, b: &Ty) -> Result<(), UnifyError> {
        let a = self.shallow_resolve(a);
        let b = self.shallow_resolve(b);

        // Identical types unify trivially
        if a == b {
            return Ok(());
        }

        // Error types unify with anything (to suppress cascading errors)
        if matches!(&a, Ty::Error) || matches!(&b, Ty::Error) {
            return Ok(());
        }

        match (&a, &b) {
            // Type variable on the left: bind it
            (Ty::TypeVar(id), _) => {
                if self.occurs(*id, &b) {
                    return Err(UnifyError::OccursCheck);
                }
                self.subst.insert(*id, b);
                Ok(())
            }
            // Type variable on the right: bind it
            (_, Ty::TypeVar(id)) => {
                if self.occurs(*id, &a) {
                    return Err(UnifyError::OccursCheck);
                }
                self.subst.insert(*id, a);
                Ok(())
            }

            // Never unifies with anything (it's a subtype of everything)
            (Ty::Never, _) | (_, Ty::Never) => Ok(()),

            // Structural matching
            (Ty::Tuple(as_), Ty::Tuple(bs)) => {
                if as_.len() != bs.len() {
                    return Err(UnifyError::Mismatch(a, b));
                }
                for (a, b) in as_.iter().zip(bs.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            }
            (Ty::Array(a_elem, a_sz), Ty::Array(b_elem, b_sz)) => {
                if a_sz != b_sz {
                    return Err(UnifyError::Mismatch(a.clone(), b.clone()));
                }
                self.unify(a_elem, b_elem)
            }
            (Ty::Slice(a_elem), Ty::Slice(b_elem)) => self.unify(a_elem, b_elem),
            (Ty::Optional(a_inner), Ty::Optional(b_inner)) => self.unify(a_inner, b_inner),
            (
                Ty::Reference { is_mut: a_mut, inner: a_inner },
                Ty::Reference { is_mut: b_mut, inner: b_inner },
            ) => {
                if a_mut != b_mut {
                    return Err(UnifyError::Mismatch(a, b));
                }
                self.unify(a_inner, b_inner)
            }
            (Ty::Function { params: ap, ret: ar }, Ty::Function { params: bp, ret: br }) => {
                if ap.len() != bp.len() {
                    return Err(UnifyError::Mismatch(a, b));
                }
                for (a, b) in ap.iter().zip(bp.iter()) {
                    self.unify(a, b)?;
                }
                self.unify(ar, br)
            }
            (Ty::Adt(a_id, a_args), Ty::Adt(b_id, b_args)) => {
                if a_id != b_id {
                    return Err(UnifyError::Mismatch(a, b));
                }
                if a_args.len() != b_args.len() {
                    return Err(UnifyError::Mismatch(a, b));
                }
                for (a, b) in a_args.iter().zip(b_args.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            }

            // Anything else is a mismatch
            _ => Err(UnifyError::Mismatch(a, b)),
        }
    }

    /// Resolve a type by following the substitution chain for type variables.
    fn shallow_resolve(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::TypeVar(id) => {
                if let Some(resolved) = self.subst.get(id) {
                    self.shallow_resolve(resolved)
                } else {
                    ty.clone()
                }
            }
            _ => ty.clone(),
        }
    }

    /// Fully apply the current substitution to a type, resolving all variables.
    pub fn apply(&self, ty: &Ty) -> Ty {
        match ty {
            Ty::TypeVar(id) => {
                if let Some(resolved) = self.subst.get(id) {
                    self.apply(resolved)
                } else {
                    ty.clone()
                }
            }
            Ty::Tuple(elems) => Ty::Tuple(elems.iter().map(|t| self.apply(t)).collect()),
            Ty::Array(elem, sz) => Ty::Array(Box::new(self.apply(elem)), *sz),
            Ty::Slice(elem) => Ty::Slice(Box::new(self.apply(elem))),
            Ty::Optional(inner) => Ty::Optional(Box::new(self.apply(inner))),
            Ty::Reference { is_mut, inner } => {
                Ty::Reference { is_mut: *is_mut, inner: Box::new(self.apply(inner)) }
            }
            Ty::Function { params, ret } => Ty::Function {
                params: params.iter().map(|t| self.apply(t)).collect(),
                ret: Box::new(self.apply(ret)),
            },
            Ty::Adt(id, args) => Ty::Adt(*id, args.iter().map(|t| self.apply(t)).collect()),
            _ => ty.clone(),
        }
    }

    /// Occurs check: does the type variable `var` occur in `ty`?
    fn occurs(&self, var: TypeVarId, ty: &Ty) -> bool {
        let ty = self.shallow_resolve(ty);
        match &ty {
            Ty::TypeVar(id) => *id == var,
            Ty::Tuple(elems) => elems.iter().any(|t| self.occurs(var, t)),
            Ty::Array(elem, _) | Ty::Slice(elem) | Ty::Optional(elem) => self.occurs(var, elem),
            Ty::Reference { inner, .. } => self.occurs(var, inner),
            Ty::Function { params, ret } => {
                params.iter().any(|t| self.occurs(var, t)) || self.occurs(var, ret)
            }
            Ty::Adt(_, args) => args.iter().any(|t| self.occurs(var, t)),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeVarId;

    #[test]
    fn unify_identical() {
        let mut ctx = InferCtx::new();
        assert!(ctx.unify(&Ty::Int32, &Ty::Int32).is_ok());
    }

    #[test]
    fn unify_type_var_left() {
        let mut ctx = InferCtx::new();
        let var = Ty::TypeVar(TypeVarId(0));
        assert!(ctx.unify(&var, &Ty::String).is_ok());
        assert_eq!(ctx.apply(&var), Ty::String);
    }

    #[test]
    fn unify_type_var_right() {
        let mut ctx = InferCtx::new();
        let var = Ty::TypeVar(TypeVarId(0));
        assert!(ctx.unify(&Ty::Bool, &var).is_ok());
        assert_eq!(ctx.apply(&var), Ty::Bool);
    }

    #[test]
    fn unify_mismatch() {
        let mut ctx = InferCtx::new();
        assert!(ctx.unify(&Ty::Int32, &Ty::String).is_err());
    }

    #[test]
    fn unify_tuples() {
        let mut ctx = InferCtx::new();
        let a = Ty::Tuple(vec![Ty::Int32, Ty::String]);
        let var = Ty::TypeVar(TypeVarId(0));
        let b = Ty::Tuple(vec![Ty::Int32, var.clone()]);
        assert!(ctx.unify(&a, &b).is_ok());
        assert_eq!(ctx.apply(&var), Ty::String);
    }

    #[test]
    fn unify_tuple_length_mismatch() {
        let mut ctx = InferCtx::new();
        let a = Ty::Tuple(vec![Ty::Int32]);
        let b = Ty::Tuple(vec![Ty::Int32, Ty::String]);
        assert!(ctx.unify(&a, &b).is_err());
    }

    #[test]
    fn unify_functions() {
        let mut ctx = InferCtx::new();
        let var = Ty::TypeVar(TypeVarId(0));
        let a = Ty::Function { params: vec![Ty::Int32], ret: Box::new(Ty::String) };
        let b = Ty::Function { params: vec![Ty::Int32], ret: Box::new(var.clone()) };
        assert!(ctx.unify(&a, &b).is_ok());
        assert_eq!(ctx.apply(&var), Ty::String);
    }

    #[test]
    fn occurs_check() {
        let mut ctx = InferCtx::new();
        let var = Ty::TypeVar(TypeVarId(0));
        let recursive = Ty::Tuple(vec![var.clone()]);
        assert!(ctx.unify(&var, &recursive).is_err());
    }

    #[test]
    fn never_unifies_with_anything() {
        let mut ctx = InferCtx::new();
        assert!(ctx.unify(&Ty::Never, &Ty::Int32).is_ok());
        assert!(ctx.unify(&Ty::String, &Ty::Never).is_ok());
    }

    #[test]
    fn error_unifies_with_anything() {
        let mut ctx = InferCtx::new();
        assert!(ctx.unify(&Ty::Error, &Ty::Int32).is_ok());
        assert!(ctx.unify(&Ty::String, &Ty::Error).is_ok());
    }

    #[test]
    fn chained_substitution() {
        let mut ctx = InferCtx::new();
        let v0 = Ty::TypeVar(TypeVarId(0));
        let v1 = Ty::TypeVar(TypeVarId(1));
        assert!(ctx.unify(&v0, &v1).is_ok());
        assert!(ctx.unify(&v1, &Ty::Float64).is_ok());
        assert_eq!(ctx.apply(&v0), Ty::Float64);
    }
}
