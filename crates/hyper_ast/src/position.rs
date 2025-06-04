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

use crate::{
    store::defaults::NodeIdentifier,
    types::{HyperAST, NodeId, TypedNodeId},
};

pub trait TreePath<IdN = NodeIdentifier, Idx = u16> {
    fn node(&self) -> Option<&IdN>;
    fn offset(&self) -> Option<&Idx>;
    fn check<'store, HAST>(&self, stores: &'store HAST) -> Result<(), ()>
    where
        HAST: HyperAST<IdN = IdN::IdN>,
        // for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithChildren<ChildIdx = Idx>,
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

    use crate::PrimInt;

    use super::*;

    pub trait RootedPosition<IdN> {
        fn root(&self) -> IdN;
    }

    pub trait AssistedFrom<S, T> {
        fn compute(&self, store: S) -> T;
    }

    pub trait SolvedPosition<IdN> {
        fn node(&self) -> IdN;
    }

    pub trait WithOffsets
    where
        Self::Idx: PrimInt,
    {
        type Idx;
        // type PreOrderIt: Iterator<Item=Self::Idx>;
        // fn iter_pre_order(&self) -> Self::PreOrderIt;
        // fn iter_pre_order(&self) -> Box<dyn Iterator<Item=Self::Idx>> {
        //     todo!()
        // }
        // type PostOrderIt: Iterator<Item=Self::Idx>;
        // fn iter_post_order(&self) -> Self::PostOrderIt;
    }

    pub trait WithPath<IdN>: WithOffsets {
        // fn iter_pre_order(&self) -> Self::PreOrderIt;
        // fn iter_post_order(&self) -> Self::PostOrderIt;
    }

    #[derive(Debug)]
    pub enum SharedPath<P> {
        Exact(P),
        Remain(P),
        Submatch(P),
        Different(P),
    }

    pub trait WithPreOrderOffsets: WithOffsets {
        // type Path: Iterator;
        // fn path(&self) -> Self::Path;
        type It<'a>: Iterator<Item = Self::Idx>
        where
            Self: 'a,
            Self::Idx: 'a;
        fn iter_offsets(&self) -> Self::It<'_>;

        fn shared_ancestors<Other: WithPreOrderOffsets<Idx = Self::Idx>>(
            &self,
            other: &Other,
        ) -> SharedPath<Vec<Self::Idx>> {
            let mut other = other.iter_offsets();
            let mut r = vec![];
            for s in self.iter_offsets() {
                if let Some(other) = other.next() {
                    if s != other {
                        return SharedPath::Different(r);
                    }
                    r.push(s);
                } else {
                    return SharedPath::Submatch(r);
                }
            }
            if other.next().is_some() {
                SharedPath::Remain(r)
            } else {
                SharedPath::Exact(r)
            }
        }
    }

    /// test invariants with [assert_invariants_pre]
    pub trait WithPreOrderPath<IdN>: WithPath<IdN> + WithPreOrderOffsets {
        // type Path: Iterator;
        // fn path(&self) -> Self::Path;
        type ItPath: Iterator<Item = (Self::Idx, IdN)>;
        fn iter_offsets_and_nodes(&self) -> Self::ItPath;
    }

    #[cfg(debug_assertions)]
    pub fn assert_invariants_pre<'store, IdN, P, HAST>(p: &P, store: &'store HAST)
    where
        IdN: std::cmp::Eq + std::hash::Hash + std::fmt::Debug + Clone + NodeId,
        P: WithPreOrderPath<IdN> + RootedPosition<IdN>,
        HAST: HyperAST<IdN = IdN, Idx = P::Idx>,
        <IdN as NodeId>::IdN: PartialEq<<<IdN as NodeId>::IdN as NodeId>::IdN>,
        <IdN as NodeId>::IdN: std::fmt::Debug,
        <<IdN as NodeId>::IdN as NodeId>::IdN: std::fmt::Debug,
    {
        use crate::types::NodeStore;
        use crate::types::WithChildren;
        use std::collections::HashSet;
        let mut set: HashSet<IdN> = HashSet::default();
        let root = p.root();
        let mut prev = root.clone();
        let it = p.iter_offsets_and_nodes();
        let snd_it = p.iter_offsets();
        set.insert(root);
        for ((o0, x), o1) in it.into_iter().zip(snd_it) {
            assert_eq!(o0, o1);
            if !set.insert(x.clone()) {
                panic!("path returns 2 times the same node")
            }
            let b = store.node_store().resolve(&prev);
            assert_eq!(x.as_id(), &b.child(&o0).expect("should have a child"));
            prev = x.clone();
        }
    }

    pub trait WithPostOrderOffsets: WithOffsets {
        // type Path: Iterator;
        // fn path(&self) -> Self::Path;
        fn iter(&self) -> impl Iterator<Item = Self::Idx>;
        // TODO into_iter ?
    }

    /// test invariants with [assert_invariants_post]
    pub trait WithPostOrderPath<IdN>: WithPath<IdN> + WithPostOrderOffsets {
        // type Path: Iterator;
        // fn path(&self) -> Self::Path;
        fn iter_offsets_and_parents(&self) -> impl Iterator<Item = (Self::Idx, IdN)>;
    }

    /// - p should only return each node once
    /// - resolved children should correspond
    #[cfg(debug_assertions)]
    pub fn assert_invariants_post<'store, IdN, P, HAST>(p: &P, store: &'store HAST)
    where
        IdN: std::cmp::Eq + std::hash::Hash + std::fmt::Debug + Clone + NodeId,
        P: WithPostOrderPath<IdN> + SolvedPosition<IdN>,
        HAST: HyperAST<IdN = IdN::IdN, Idx = P::Idx>,
        <IdN as NodeId>::IdN: PartialEq<<<IdN as NodeId>::IdN as NodeId>::IdN>,
        <IdN as NodeId>::IdN: std::fmt::Debug,
        <<IdN as NodeId>::IdN as NodeId>::IdN: std::fmt::Debug,
    {
        use crate::types::NodeStore;
        use crate::types::WithChildren;
        use std::collections::HashSet;
        let mut set: HashSet<IdN> = HashSet::default();
        let node = p.node();
        let mut prev = node.clone();
        let it = p.iter_offsets_and_parents();
        let snd_it = p.iter();
        set.insert(node);
        for ((o0, x), o1) in it.into_iter().zip(snd_it) {
            assert_eq!(o0, o1);
            if !set.insert(x.clone()) {
                panic!("path returns 2 times the same node")
            }
            let b = store.node_store().resolve(x.as_id());
            assert_eq!(prev.as_id(), &b.child(&o0).expect("should have a child"));
            prev = x.clone();
        }
    }

    /// test invariants with [assert_invariants_post_full]
    pub trait WithFullPostOrderPath<IdN>: RootedPosition<IdN> + WithPostOrderPath<IdN> {
        fn iter_with_nodes(&self) -> (IdN, impl Iterator<Item = (Self::Idx, IdN)>);
    }

    /// - p should only return each node once
    /// - resolved children should corespond
    #[cfg(debug_assertions)]
    pub fn assert_invariants_post_full<'store, IdN, P, HAST>(p: &P, store: &'store HAST)
    where
        IdN: std::cmp::Eq + std::hash::Hash + std::fmt::Debug + Clone + NodeId,
        P: WithFullPostOrderPath<IdN>,
        HAST: HyperAST<IdN = IdN::IdN, Idx = P::Idx>,
        <IdN as NodeId>::IdN: PartialEq<<<IdN as NodeId>::IdN as NodeId>::IdN>,
        <IdN as NodeId>::IdN: std::fmt::Debug,
        <<IdN as NodeId>::IdN as NodeId>::IdN: std::fmt::Debug,
    {
        use crate::types::NodeStore;
        use crate::types::WithChildren;
        use std::collections::HashSet;
        let mut set: HashSet<IdN> = HashSet::default();
        let (node, it) = p.iter_with_nodes();
        let mut prev = node.clone();
        let snd_it = p.iter_offsets_and_parents();
        let third_it = p.iter();
        set.insert(node);
        for (((o0, x), (o1, y)), o2) in it.into_iter().zip(snd_it).zip(third_it) {
            dbg!(&prev, o0);
            assert_eq!(x, y);
            assert_eq!(o0, o1);
            assert_eq!(o2, o1);
            if !set.insert(x.clone()) {
                panic!("path returns 2 times the same node")
            }
            let b = store.node_store().resolve(x.as_id());
            assert_eq!(prev.as_id(), &b.child(&(o0)).expect("should have a child"));
            prev = x.clone();
        }
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
    pub trait OffsetPostionT<IdN>
    where
        Self::IdO: PrimInt,
    {
        /// Offset in characters or bytes
        type IdO;
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

pub mod building;

pub mod tags {
    #[derive(Clone, Copy, Debug)]
    pub struct TopDownNoSpace;
    #[derive(Clone, Copy, Debug)]
    pub struct TopDownFull;
    #[derive(Clone, Copy, Debug)]
    pub struct BottomUpNoSpace;
    #[derive(Clone, Copy, Debug)]
    pub struct BottomUpFull;
}

pub mod node_filter_traits {
    pub trait NoSpace {}
    pub trait Full {}
}

pub use building::CompoundPositionPreparer;

pub mod offsets;
pub use offsets::*;

pub mod file_and_offset;

pub type Position = file_and_offset::Position<PathBuf, usize>;

pub mod offsets_and_nodes;
pub use offsets_and_nodes::*;

pub mod topological_offset;

pub mod row_col;

pub mod file_and_range;

#[allow(unused)] // TODO remove all not working function and test the remaining ones
mod spaces_related;
pub use spaces_related::{
    compute_position_and_nodes_with_no_spaces, compute_position_with_no_spaces,
    global_pos_with_spaces, path_with_spaces,
};

pub mod computing_offset_bottom_up;
//pub use computing_offset_bottom_up::{extract_file_postion, extract_position};

mod computing_offset_top_down;
pub use computing_offset_top_down::{
    compute_position,
    compute_position_and_nodes,
    // compute_position_and_nodes2,
    // compute_position_and_nodes3,
    compute_range,
};

mod computing_path;
pub use computing_path::resolve_range;

// advanced optimization, uses a dag StructuralPositionStore to share parent paths

pub mod structural_pos;
pub use structural_pos::{
    ExploreStructuralPositions, Scout, SpHandle, StructuralPositionStore, TypedScout,
};
pub type StructuralPosition<IdN = NodeIdentifier, Idx = u16> =
    structural_pos::StructuralPosition<IdN, Idx>;
