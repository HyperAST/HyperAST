use crate::types::Type;

pub fn is_cyclomatic_persisted(t: &Type) -> bool {
    t == &Type::ClassDeclaration
        || t == &Type::InterfaceDeclaration
        || t == &Type::EnumDeclaration
        || t == &Type::AnnotationTypeDeclaration
        || t == &Type::MethodDeclaration
        || t == &Type::ConstructorDeclaration
        || t == &Type::Program
}

struct McCabe {
    value: u32,
}
