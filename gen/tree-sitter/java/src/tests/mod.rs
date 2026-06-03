#[cfg(feature = "impact")]
pub mod tests_legion_with_refs;

mod tree_sitter_types_test;

#[cfg(feature = "tsg")]
mod tsg_hyperast;

// #[cfg(all(feature = "tsg", debug_assertions))]
mod tsqueries_java;

#[cfg(all(feature = "impl", feature = "bevy"))]
mod test_bevy;

mod test_cases;

#[test]
fn type_test_generic_eq() {
    use hyperast::types::HyperType;

    let k = crate::types::Type::EnhancedForStatement;
    let k0 = crate::types::Type::EnhancedForStatement;
    let k1 = crate::types::Type::ClassDeclaration;
    assert!(k.eq(&k));
    assert!(k.eq(&k0));
    assert!(k0.eq(&k));
    assert!(k1.eq(&k1));
    assert!(k.ne(&k1));
    assert!(k1.ne(&k));

    assert!(k.generic_eq(&k));
    assert!(k.generic_eq(&k0));
    assert!(k0.generic_eq(&k));
    assert!(k1.generic_eq(&k1));
    assert!(!k.generic_eq(&k1));
    assert!(!k1.generic_eq(&k));

    let ak = crate::types::as_any(&crate::types::Type::EnhancedForStatement);
    let ak0 = crate::types::as_any(&crate::types::Type::EnhancedForStatement);
    let ak1 = crate::types::as_any(&crate::types::Type::ClassDeclaration);

    assert!(ak.generic_eq(&ak));
    assert!(ak.generic_eq(&ak0));
    assert!(ak0.generic_eq(&ak));
    assert!(ak1.generic_eq(&ak1));
    assert!(!ak.generic_eq(&ak1));
    assert!(!ak1.generic_eq(&ak));

    assert!(k.generic_eq(&ak));
    assert!(k.generic_eq(&ak0));
    assert!(k0.generic_eq(&ak));
    assert!(k1.generic_eq(&ak1));
    assert!(!k.generic_eq(&ak1));
    assert!(!k1.generic_eq(&ak));

    assert!(ak.generic_eq(&k));
    assert!(ak.generic_eq(&k0));
    assert!(ak0.generic_eq(&k));
    assert!(ak1.generic_eq(&k1));
    assert!(!ak.generic_eq(&k1));
    assert!(!ak1.generic_eq(&k));

    assert!(ak.eq(&ak));
    assert!(ak.eq(&ak0));
    assert!(ak0.eq(&ak));
    assert!(ak1.eq(&ak1));
    assert!(!ak.eq(&ak1));
    assert!(!ak1.eq(&ak));
}
