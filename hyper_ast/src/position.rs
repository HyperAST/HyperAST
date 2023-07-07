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

pub mod position_accessors {
    // we want to represent the possible combination which produce a global position in the HyperAST
    // 1) root + file path + offset + len
    // 2) root + path (nodes to consider)
    // 3) root + topological index (nodes to consider)
    // 4) parents + offsets + node (nodes to consider)
    //
    // to test position equality roots must be equal,
    // a shared combination of attributes is chosen for the final comparison

    use super::*;

    pub trait RootedPositionT<IdN> {
        fn root(&self) -> IdN;
    }

    pub trait SolvedPositionT<IdN> {
        fn node(&self) -> IdN;
    }

    pub trait PathPositionT<IdN>
    where
        Self::Idx: PrimInt,
    {
        type Idx;
    }

    pub trait PostOrderPathPositionT<IdN>: PathPositionT<IdN> {
        // type Path: Iterator;
        // fn path(&self) -> Self::Path;
    }

    pub trait PreOrderPathPositionT<IdN>: PathPositionT<IdN> {
        // type Path: Iterator;
        // fn path(&self) -> Self::Path;
    }
    pub trait TopoIndexPositionT<IdN>
    where
        Self::IdI: PrimInt,
    {
        type IdI;
        fn index(&self) -> Self::IdI;
    }
    pub trait FileAndOffsetPostionT<IdN>
    where
        Self::IdO: PrimInt,
    {
        /// Offset in characters or bytes
        type IdO;
        fn file(&self) -> std::path::PathBuf;
        fn offset(&self) -> Self::IdO;
        fn len(&self) -> Self::IdO;
        fn start(&self) -> Self::IdO;
        fn end(&self) -> Self::IdO;
    }
}



pub struct PositionConverter<'src, SrcPos> {
    src: &'src SrcPos,
}

impl<'src, SrcPos> PositionConverter<'src, SrcPos> {
    pub fn new(src: &'src SrcPos) -> Self {
        Self { src }
    }
    pub fn with_stores<'store, HAST>(
        self,
        stores: &'store HAST,
    ) -> WithHyperAstPositionConverter<'store, 'src, SrcPos, HAST> {
        WithHyperAstPositionConverter {
            src: self.src,
            stores,
        }
    }
}

pub struct WithHyperAstPositionConverter<'store, 'src, SrcPos, HAST> {
    src: &'src SrcPos,
    stores: &'store HAST,
}

mod offsets {
    use super::PrimInt;

    pub struct Position<T: PrimInt>(Vec<T>);
}

mod file_and_offset;

pub type Position = file_and_offset::Position<PathBuf, usize>;

mod offsets_and_nodes;
pub use offsets_and_nodes::*;

mod topological_offset;
pub use topological_offset::*;

mod spaces_related;
pub use spaces_related::{
    compute_position_and_nodes_with_no_spaces, compute_position_with_no_spaces,
    global_pos_with_spaces, path_with_spaces,
};

mod computing_offset_bottom_up;
// pub use computing_offset_bottom_up::{extract_file_postion, extract_position};

mod computing_offset_top_down;
pub use computing_offset_top_down::{compute_position, compute_position_and_nodes, compute_range};

mod computing_path;
pub use computing_path::resolve_range;

// advanced optimization, uses a dag StructuralPositionStore to share parent paths

mod structural_pos;
pub use structural_pos::{
    ExploreStructuralPositions, Scout, SpHandle, StructuralPositionStore, TypedScout,
};
pub type StructuralPosition<IdN = NodeIdentifier, Idx = u16> =
    structural_pos::StructuralPosition<IdN, Idx>;
