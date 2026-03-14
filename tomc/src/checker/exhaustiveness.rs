//! Match-expression exhaustiveness analysis.
//!
//! Ensures that a `match` expression covers all possible values of the
//! scrutinee type. For enums this means every variant must be matched
//! (or a wildcard/catch-all arm is present). For other types a wildcard
//! or identifier pattern is sufficient.

use crate::ast;
use crate::types::{Ty, TypeRegistry};

/// Check whether a set of match arms exhaustively covers `scrutinee_ty`.
///
/// Returns `Some(missing_patterns)` if there are uncovered cases,
/// or `None` if exhaustive.
pub fn check(
    scrutinee_ty: &Ty,
    arms: &[ast::MatchArm],
    registry: &TypeRegistry,
) -> Option<Vec<String>> {
    // If any arm is a wildcard or plain identifier, the match is exhaustive
    if arms.iter().any(|arm| is_catch_all(&arm.pattern)) {
        return None;
    }

    match scrutinee_ty {
        Ty::Bool => check_bool(arms),
        Ty::Adt(id, _) => {
            if let Some(enum_def) = registry.lookup_enum(*id) {
                check_enum(
                    arms,
                    &enum_def
                        .variants
                        .iter()
                        .map(|v| v.name.clone())
                        .collect::<Vec<_>>(),
                )
            } else {
                // Struct types are exhaustive with a single arm
                None
            }
        }
        Ty::Unit => None,
        Ty::Never => None,
        _ => {
            // For numeric, string, and other types, we can't easily enumerate
            // all values—require a wildcard. If we got here, there's no wildcard.
            Some(vec!["_".to_string()])
        }
    }
}

/// Check exhaustiveness for Bool match.
fn check_bool(arms: &[ast::MatchArm]) -> Option<Vec<String>> {
    let mut has_true = false;
    let mut has_false = false;

    for arm in arms {
        match &arm.pattern {
            ast::Pattern::Literal {
                value: ast::Expr::BoolLiteral { value: true, .. },
                ..
            } => {
                has_true = true;
            }
            ast::Pattern::Literal {
                value: ast::Expr::BoolLiteral { value: false, .. },
                ..
            } => {
                has_false = true;
            }
            p if is_catch_all(p) => return None,
            _ => {}
        }
    }

    let mut missing = Vec::new();
    if !has_true {
        missing.push("true".to_string());
    }
    if !has_false {
        missing.push("false".to_string());
    }
    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
}

/// Check exhaustiveness for enum match.
fn check_enum(arms: &[ast::MatchArm], variant_names: &[String]) -> Option<Vec<String>> {
    let covered: Vec<String> = arms
        .iter()
        .filter_map(|arm| extract_variant_name(&arm.pattern))
        .collect();

    let missing: Vec<String> = variant_names
        .iter()
        .filter(|v| !covered.contains(v))
        .cloned()
        .collect();

    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
}

/// Check if a pattern is a catch-all (wildcard or plain identifier without guard).
fn is_catch_all(pattern: &ast::Pattern) -> bool {
    match pattern {
        ast::Pattern::Wildcard { .. } => true,
        ast::Pattern::Identifier { .. } => true,
        ast::Pattern::Or { patterns, .. } => patterns.iter().any(is_catch_all),
        _ => false,
    }
}

/// Extract the variant name from a pattern, if it's a variant pattern.
fn extract_variant_name(pattern: &ast::Pattern) -> Option<String> {
    match pattern {
        ast::Pattern::Variant { path, .. } => Some(path.segments.last()?.node.clone()),
        ast::Pattern::Identifier { name, .. } => {
            // Could be a unit variant used without parens
            let n = &name.node;
            if n.chars().next().map_or(false, |c| c.is_uppercase()) {
                Some(n.clone())
            } else {
                None // catch-all, not a variant name
            }
        }
        ast::Pattern::Or { patterns, .. } => {
            // For or-patterns, extract from first
            patterns.iter().find_map(extract_variant_name)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::span::Span;
    use crate::span::Spanned;

    fn dummy_arm(pattern: Pattern, body: Expr) -> MatchArm {
        MatchArm {
            pattern,
            guard: None,
            body,
            span: Span::DUMMY,
        }
    }

    fn wildcard() -> Pattern {
        Pattern::Wildcard { span: Span::DUMMY }
    }

    fn bool_pat(val: bool) -> Pattern {
        Pattern::Literal {
            value: Expr::BoolLiteral {
                value: val,
                span: Span::DUMMY,
            },
            span: Span::DUMMY,
        }
    }

    fn variant_pat(name: &str) -> Pattern {
        Pattern::Variant {
            path: TypePath {
                segments: vec![Spanned::new(name.to_string(), Span::DUMMY)],
                span: Span::DUMMY,
            },
            data: None,
            span: Span::DUMMY,
        }
    }

    fn unit_body() -> Expr {
        Expr::Tuple {
            elements: vec![],
            span: Span::DUMMY,
        }
    }

    #[test]
    fn wildcard_is_exhaustive() {
        let arms = vec![dummy_arm(wildcard(), unit_body())];
        let result = check(&Ty::Int32, &arms, &TypeRegistry::new());
        assert!(result.is_none());
    }

    #[test]
    fn bool_both_covered() {
        let arms = vec![
            dummy_arm(bool_pat(true), unit_body()),
            dummy_arm(bool_pat(false), unit_body()),
        ];
        let result = check(&Ty::Bool, &arms, &TypeRegistry::new());
        assert!(result.is_none());
    }

    #[test]
    fn bool_missing_false() {
        let arms = vec![dummy_arm(bool_pat(true), unit_body())];
        let result = check(&Ty::Bool, &arms, &TypeRegistry::new());
        assert_eq!(result, Some(vec!["false".to_string()]));
    }

    #[test]
    fn enum_exhaustive() {
        let mut reg = TypeRegistry::new();
        let id = reg.register_enum(crate::types::EnumDef {
            name: "Color".into(),
            type_params: vec![],
            variants: vec![
                crate::types::VariantDef {
                    name: "Red".into(),
                    data: crate::types::VariantDataDef::Unit,
                    span: Span::DUMMY,
                },
                crate::types::VariantDef {
                    name: "Blue".into(),
                    data: crate::types::VariantDataDef::Unit,
                    span: Span::DUMMY,
                },
            ],
            span: Span::DUMMY,
        });

        let arms = vec![
            dummy_arm(variant_pat("Red"), unit_body()),
            dummy_arm(variant_pat("Blue"), unit_body()),
        ];
        let result = check(&Ty::Adt(id, vec![]), &arms, &reg);
        assert!(result.is_none());
    }

    #[test]
    fn enum_missing_variant() {
        let mut reg = TypeRegistry::new();
        let id = reg.register_enum(crate::types::EnumDef {
            name: "Color".into(),
            type_params: vec![],
            variants: vec![
                crate::types::VariantDef {
                    name: "Red".into(),
                    data: crate::types::VariantDataDef::Unit,
                    span: Span::DUMMY,
                },
                crate::types::VariantDef {
                    name: "Blue".into(),
                    data: crate::types::VariantDataDef::Unit,
                    span: Span::DUMMY,
                },
            ],
            span: Span::DUMMY,
        });

        let arms = vec![dummy_arm(variant_pat("Red"), unit_body())];
        let result = check(&Ty::Adt(id, vec![]), &arms, &reg);
        assert_eq!(result, Some(vec!["Blue".to_string()]));
    }

    #[test]
    fn numeric_without_wildcard_not_exhaustive() {
        let arms = vec![dummy_arm(
            Pattern::Literal {
                value: Expr::IntLiteral {
                    value: 0,
                    suffix: None,
                    span: Span::DUMMY,
                },
                span: Span::DUMMY,
            },
            unit_body(),
        )];
        let result = check(&Ty::Int32, &arms, &TypeRegistry::new());
        assert_eq!(result, Some(vec!["_".to_string()]));
    }
}
