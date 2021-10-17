// use super::tree::{Type, Typed};

// type SimpleName = String; // max 256 char
// pub(crate) enum Label {
//     SimpleLabel(SimpleName),
//     LongLabel(String),
// }

// type Children = Vec<Box<CompressedTree>>;

// pub(crate) enum CompressedTree {
//     LanguageSymbol(Type),
//     SimplyLabeled(Type, SimpleName),
//     Literal(Type, Label),
//     Spaces(Label),
//     Tree(Type, Children),
// }

// impl Typed for CompressedTree {
//     fn getType(&self) -> &Type {
//         match self {
//             CompressedTree::LanguageSymbol(t) => t,
//             CompressedTree::SimplyLabeled(t, _) => t,
//             CompressedTree::Literal(t, _) => t,
//             CompressedTree::Spaces(_) => &Type::Spaces,
//             CompressedTree::Tree(t, _) => t,
//         }
//     }
// }

// pub(crate) struct Declaration {

// }
// pub(crate) struct QualifiedName {

// }
