//! # Positioning elements in HyperAST
//!
//! Because the HyperAST is a Direct Acyclic Graph (DAG),
//! any given sub-tree possibly has multiple global positions.
//! The global position, path, or offset of a subtree is global/contextual information, thus it cannot be stored efficiently on subtrees of a DAG.
//!
//! You can look at this module as an example of computing global metrics out of local ones.
//!
//! This module specifically helps with tasks related to positioning nodes globally,
//! - it helps maintain positional states during traversals
//! - it helps convert between positional representations
//!     - topological
//!     - path (list of offsets)
//!     - a file path, an offset and a length
//!     - with/out hidden nodes
//!
//! ## Incremental position storing
//! [structural_pos]
//! 
//! ## topological index
//! [topological_offset]
//!     - post-order
//! ##  path
//! [offsets_and_nodes]
//!     - list of offsets
//! ## collection of path
//!     Optimisation related, sometimes necessary to have acceptable perfs.
//!     - list of paths
//!     - it of paths
//!     - topo ordered list of paths
//!       incremental compute
//!     - reversed dag of paths
//!       mem optimization,
//! ## with hidden nodes (spaces, abtract nodes,....)

use std::{fmt::Debug, path::PathBuf};

use num::traits::NumAssign;

use crate::{
    store::defaults::NodeIdentifier,
    types::{HyperAST, NodeId, TypedNodeId, WithChildren},
};

pub trait PrimInt: num::PrimInt + NumAssign + Debug {}

impl<T> PrimInt for T where T: num::PrimInt + NumAssign + Debug {}

pub trait TreePath<IdN = NodeIdentifier, Idx = u16> {
    fn node(&self) -> Option<&IdN>;
    fn offset(&self) -> Option<&Idx>;
    fn check<'store, HAST>(&self, stores: &'store HAST) -> Result<(), ()>
    where
        HAST: HyperAST<'store, IdN = IdN::IdN>,
        HAST::T: WithChildren<ChildIdx = Idx>,
        HAST::IdN: Eq,
        IdN: NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN>;
}

pub trait TreePathMut<IdN, Idx>: TreePath<IdN, Idx> {
    fn pop(&mut self) -> Option<(IdN, Idx)>;
    fn goto(&mut self, node: IdN, i: Idx);
    fn inc(&mut self, node: IdN);
    fn dec(&mut self, node: IdN);
}

pub trait TypedTreePath<TIdN: TypedNodeId, Idx>: TreePath<TIdN::IdN, Idx> {
    fn node_typed(&self) -> Option<&TIdN>;
    fn pop_typed(&mut self) -> Option<(TIdN, Idx)>;
    fn goto_typed(&mut self, node: TIdN, i: Idx);
}

mod offsets {
    use super::PrimInt;

    pub struct Position<T: PrimInt>(Vec<T>);
}

mod file_and_offset;

pub type Position = file_and_offset::Position<PathBuf, usize>;

mod bottom_up;
pub use bottom_up::*;

mod offsets_and_nodes;
pub use offsets_and_nodes::*;

mod topological_offset;
pub use topological_offset::*;

mod scouting;
pub use scouting::*;

mod computing_path;
pub use computing_path::*;

mod spaces_related;
pub use spaces_related::*;

pub use structural_pos::*;
mod structural_pos;

pub use computing_offset::*;
mod computing_offset;
