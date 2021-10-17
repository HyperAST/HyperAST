// use std::error::Error;

// use super::tree::{Type, Typed};
// use crate::tree::compressed_tree::CompressedTree;

// type SimpleName = String; // max 256 char
// enum Label {
//     SimpleLabel(SimpleName),
//     LongLabel(String),
// }

// type Parent = Box<DeCompressedTreeSimple>;

// struct DeCompressedTreeSimple {
//     parent:Option<Parent>,
//     compressed:CompressedTree,
// }

// impl Typed for DeCompressedTreeSimple {
//     fn getType(&self) -> &Type {
//         todo!()
//     }
// }
