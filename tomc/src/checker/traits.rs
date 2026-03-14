//! Trait bound verification and method resolution.
//!
//! This module checks that:
//! - Trait implementations provide all required methods.
//! - Generic type parameters satisfy their declared bounds.
//! - Method calls can be resolved through trait implementations.

use crate::types::{Ty, TypeId, TypeRegistry};

/// Resolves trait bounds and impl completeness.
pub struct TraitResolver<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> TraitResolver<'a> {
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }

    /// Check that an `impl Trait for Type` provides all required methods.
    ///
    /// Returns `Some(missing_names)` if there are unimplemented methods
    /// without defaults, otherwise `None`.
    pub fn check_impl_completeness(
        &self,
        trait_id: TypeId,
        impl_methods: &[&str],
    ) -> Option<Vec<String>> {
        let trait_def = self.registry.lookup_trait(trait_id)?;

        let missing: Vec<String> = trait_def
            .methods
            .iter()
            .filter(|m| !m.has_default && !impl_methods.contains(&m.name.as_str()))
            .map(|m| m.name.clone())
            .collect();

        if missing.is_empty() {
            None
        } else {
            Some(missing)
        }
    }

    /// Check that a concrete type satisfies a trait bound.
    ///
    /// Returns `true` if there is a registered `impl Trait for Type`.
    pub fn satisfies_bound(&self, ty: &Ty, trait_id: TypeId) -> bool {
        // Primitive types satisfy certain built-in traits
        if self.has_builtin_impl(ty, trait_id) {
            return true;
        }

        // Check explicit impls
        self.registry.has_impl(trait_id, ty)
    }

    /// Check that all bounds on a set of type parameters are satisfied.
    ///
    /// `type_params` maps parameter names to their concrete types.
    /// `bounds` maps parameter names to lists of trait IDs they must implement.
    ///
    /// Returns a list of unsatisfied bounds.
    pub fn check_bounds(
        &self,
        type_params: &[(&str, &Ty)],
        bounds: &[(&str, &[TypeId])],
    ) -> Vec<(String, String)> {
        let mut violations = Vec::new();

        for (param_name, trait_ids) in bounds {
            if let Some((_, ty)) = type_params.iter().find(|(n, _)| n == param_name) {
                for trait_id in *trait_ids {
                    if !self.satisfies_bound(ty, *trait_id) {
                        let trait_name = self
                            .registry
                            .lookup_trait(*trait_id)
                            .map(|t| t.name.clone())
                            .unwrap_or_else(|| format!("Trait({})", trait_id.0));
                        violations.push((param_name.to_string(), trait_name));
                    }
                }
            }
        }

        violations
    }

    /// Check if a primitive type has a built-in trait implementation.
    fn has_builtin_impl(&self, ty: &Ty, trait_id: TypeId) -> bool {
        let trait_name = match self.registry.lookup_trait(trait_id) {
            Some(t) => &t.name,
            None => return false,
        };

        match trait_name.as_str() {
            "Display" => matches!(
                ty,
                Ty::Bool
                    | Ty::Int8
                    | Ty::Int16
                    | Ty::Int32
                    | Ty::Int64
                    | Ty::UInt8
                    | Ty::UInt16
                    | Ty::UInt32
                    | Ty::UInt64
                    | Ty::Float32
                    | Ty::Float64
                    | Ty::Char
                    | Ty::String
            ),
            "Eq" | "PartialEq" => matches!(
                ty,
                Ty::Bool
                    | Ty::Int8
                    | Ty::Int16
                    | Ty::Int32
                    | Ty::Int64
                    | Ty::UInt8
                    | Ty::UInt16
                    | Ty::UInt32
                    | Ty::UInt64
                    | Ty::Char
                    | Ty::String
            ),
            "Ord" | "PartialOrd" => matches!(
                ty,
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
                    | Ty::Char
                    | Ty::String
            ),
            "Hash" => matches!(
                ty,
                Ty::Bool
                    | Ty::Int8
                    | Ty::Int16
                    | Ty::Int32
                    | Ty::Int64
                    | Ty::UInt8
                    | Ty::UInt16
                    | Ty::UInt32
                    | Ty::UInt64
                    | Ty::Char
                    | Ty::String
            ),
            "Clone" | "Copy" => matches!(
                ty,
                Ty::Bool
                    | Ty::Int8
                    | Ty::Int16
                    | Ty::Int32
                    | Ty::Int64
                    | Ty::UInt8
                    | Ty::UInt16
                    | Ty::UInt32
                    | Ty::UInt64
                    | Ty::Float32
                    | Ty::Float64
                    | Ty::Char
            ),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn impl_completeness_all_present() {
        let mut reg = TypeRegistry::new();
        let trait_id = reg.register_trait(TraitDef {
            name: "Greet".into(),
            type_params: vec![],
            super_traits: vec![],
            methods: vec![TraitMethodDef {
                name: "greet".into(),
                params: vec![],
                ret: Ty::String,
                has_default: false,
                span: Span::DUMMY,
            }],
            span: Span::DUMMY,
        });

        let resolver = TraitResolver::new(&reg);
        assert!(resolver
            .check_impl_completeness(trait_id, &["greet"])
            .is_none());
    }

    #[test]
    fn impl_completeness_missing_method() {
        let mut reg = TypeRegistry::new();
        let trait_id = reg.register_trait(TraitDef {
            name: "Greet".into(),
            type_params: vec![],
            super_traits: vec![],
            methods: vec![
                TraitMethodDef {
                    name: "greet".into(),
                    params: vec![],
                    ret: Ty::String,
                    has_default: false,
                    span: Span::DUMMY,
                },
                TraitMethodDef {
                    name: "farewell".into(),
                    params: vec![],
                    ret: Ty::String,
                    has_default: false,
                    span: Span::DUMMY,
                },
            ],
            span: Span::DUMMY,
        });

        let resolver = TraitResolver::new(&reg);
        let missing = resolver.check_impl_completeness(trait_id, &["greet"]);
        assert_eq!(missing, Some(vec!["farewell".to_string()]));
    }

    #[test]
    fn impl_default_methods_not_required() {
        let mut reg = TypeRegistry::new();
        let trait_id = reg.register_trait(TraitDef {
            name: "Counter".into(),
            type_params: vec![],
            super_traits: vec![],
            methods: vec![
                TraitMethodDef {
                    name: "next".into(),
                    params: vec![],
                    ret: Ty::Int32,
                    has_default: false,
                    span: Span::DUMMY,
                },
                TraitMethodDef {
                    name: "count".into(),
                    params: vec![],
                    ret: Ty::Int64,
                    has_default: true,
                    span: Span::DUMMY,
                },
            ],
            span: Span::DUMMY,
        });

        let resolver = TraitResolver::new(&reg);
        // Only "next" is required, "count" has a default
        assert!(resolver
            .check_impl_completeness(trait_id, &["next"])
            .is_none());
    }

    #[test]
    fn builtin_display_for_primitives() {
        let mut reg = TypeRegistry::new();
        let trait_id = reg.register_trait(TraitDef {
            name: "Display".into(),
            type_params: vec![],
            super_traits: vec![],
            methods: vec![],
            span: Span::DUMMY,
        });

        let resolver = TraitResolver::new(&reg);
        assert!(resolver.satisfies_bound(&Ty::Int32, trait_id));
        assert!(resolver.satisfies_bound(&Ty::String, trait_id));
        assert!(!resolver.satisfies_bound(&Ty::Unit, trait_id));
    }

    use crate::span::Span;
}
