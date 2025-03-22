use super::{building, tags, Position, TreePath, WithHyperAstPositionConverter};
use std::{fmt::Debug, path::PathBuf};

use num::one;

use crate::{
    store::defaults::NodeIdentifier,
    types::{
        self, AnyType, Children, Childrn, HyperAST, HyperType, LabelStore, Labeled, NodeId,
        NodeStore, TypeStore, Typed, TypedNodeId, WithChildren, WithSerialization, WithStats,
    },
    PrimInt,
};

pub use super::offsets_and_nodes::StructuralPosition;

mod path_store;

mod scouting;
pub use scouting::*;

mod typed_scouting;
pub use typed_scouting::*;

#[derive(Clone)]
pub struct ExploreStructuralPositions<'a, IdN, Idx = usize, Config = tags::BottomUpFull> {
    sps: &'a StructuralPositionStore<IdN, Idx>,
    i: usize,
    _phantom: std::marker::PhantomData<Config>,
}
impl<'a, IdN, Idx> super::node_filter_traits::Full for ExploreStructuralPositions<'a, IdN, Idx> {}
impl<'a, IdN, Idx> super::node_filter_traits::NoSpace
    for ExploreStructuralPositions<'a, IdN, Idx, tags::BottomUpNoSpace>
{
}

mod esp_impl {
    use super::super::position_accessors::*;
    use super::*;
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> SolvedPosition<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        fn node(&self) -> IdN {
            self.sps.nodes[self.i]
        }
    }
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> RootedPosition<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        fn root(&self) -> IdN {
            todo!("value must be computed")
        }
    }
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> WithOffsets
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        type Idx = Idx;
    }
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> WithPostOrderOffsets
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        fn iter(&self) -> impl Iterator<Item = Self::Idx> {
            IterOffsets(self.clone())
        }
    }

    pub struct IterOffsets<'a, IdN, Idx = usize>(ExploreStructuralPositions<'a, IdN, Idx>);

    impl<'a, IdN, Idx: PrimInt> Iterator for IterOffsets<'a, IdN, Idx> {
        type Item = Idx;

        fn next(&mut self) -> Option<Self::Item> {
            let o = self.0.sps.offsets[self.0.i];
            self.0.try_go_up().map(|_| o - one())
        }
    }

    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> WithPath<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
    }
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> WithPostOrderPath<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        fn iter_offsets_and_parents(&self) -> impl Iterator<Item = (Self::Idx, IdN)> {
            IterOffsetsNodes(self.clone())
        }
    }
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> WithFullPostOrderPath<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        fn iter_with_nodes(&self) -> (IdN, impl Iterator<Item = (Self::Idx, IdN)>) {
            (self.node(), IterOffsetsNodes(self.clone()))
        }
    }

    pub struct IterOffsetsNodes<'a, IdN, Idx = usize>(ExploreStructuralPositions<'a, IdN, Idx>);

    impl<'a, IdN: Copy, Idx: PrimInt> Iterator for IterOffsetsNodes<'a, IdN, Idx> {
        type Item = (Idx, IdN);

        fn next(&mut self) -> Option<Self::Item> {
            let o = self.0.sps.offsets[self.0.i];
            self.0
                .try_go_up()
                .map(|h| (o - one(), self.0.sps.nodes[h.0]))
            // let o = self.0.sps.offsets[self.0.i];
            // let n = self.0.sps.nodes[self.0.i];
            // self.0.try_go_up().map(|h| (o, n))
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpHandle(pub(super) usize);

pub struct StructuralPositionStore<IdN = NodeIdentifier, Idx = u16> {
    pub nodes: Vec<IdN>,
    parents: Vec<usize>,
    offsets: Vec<Idx>,
}

impl<IdN, Idx> Debug for StructuralPositionStore<IdN, Idx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StructuralPositionStore")
            .field("nodes", &self.nodes.len())
            .field("parents", &self.parents.len())
            .field("offsets", &self.offsets.len())
            .finish()
    }
}

// #[derive(Clone, Debug)]
// pub struct StructuralPositionWithIndentation {
//     pub(crate) nodes: Vec<NodeIdentifier>,
//     pub(crate) offsets: Vec<usize>,
//     pub(crate) indentations: Vec<Box<[Space]>>,
// }
impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> ExploreStructuralPositions<'a, IdN, Idx> {
    pub(super) fn peek_parent_node(&self) -> Option<IdN> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.nodes[self.sps.parents[i]];
        Some(r)
    }
    pub(super) fn peek_offset(&self) -> Option<Idx> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.offsets[i] - one();
        Some(r)
    }
    pub(super) fn peek_node(&self) -> Option<IdN> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.nodes[i];
        Some(r)
    }
}

impl<'a, IdN: Copy, Idx> Iterator for ExploreStructuralPositions<'a, IdN, Idx> {
    type Item = IdN;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_go_up().map(|i| self.sps.nodes[i.0])
        // if self.i == 0 {
        //     return None;
        // } //println!("next: {} {}", self.i, self.sps.parents[self.i - 1]);
        // let i = self.i - 1;
        // let r = self.sps.nodes[i];
        // if i > 0 {
        //     self.i = self.sps.parents[i] + 1;
        // } else {
        //     self.i = i;
        // }
        // Some(r)
    }
}
impl<'a, IdN, Idx> ExploreStructuralPositions<'a, IdN, Idx> {
    /// return previous index
    #[inline]
    fn try_go_up(&mut self) -> Option<SpHandle> {
        if self.i == 0 {
            return None;
        } //println!("next: {} {}", self.i, self.sps.parents[self.i - 1]);
        let i = self.i - 1;
        let r = i;
        if i > 0 {
            self.i = self.sps.parents[i] + 1;
        } else {
            self.i = i;
        }
        Some(SpHandle(r))
    }
}

// impl<'store, 'src, 'a, IdN: NodeId + Eq + Copy, Idx: PrimInt, HAST>
//     WithHyperAstPositionConverter<'store, 'src, ExploreStructuralPositions<'a, IdN, Idx>, HAST>
// {
//     // TODO rename to compute_file_and_offset ?
//     // pub fn make_file_and_offset(&self) -> Position
//     // where
//     //     'a: 'store,
//     //     HAST: HyperAST<'store, IdN = IdN::IdN>,
//     //     for<'t> <HAST as crate::types::AstLending<'t>>::RT: Typed<Type = AnyType> + WithSerialization + WithChildren,
//     //     <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
//     //     IdN: Debug + NodeId,
//     //     IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
//     // {
//     //     self.src.clone().make_position(self.stores)
//     // }
//     // fn make_position2(&self) -> Position
//     // where
//     //     'a: 'store,
//     //     HAST: HyperAST<'store, IdN = IdN::IdN>,
//     //     for<'t> <HAST as crate::types::AstLending<'t>>::RT: Typed<Type = AnyType> + WithSerialization + WithChildren,
//     //     <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
//     //     IdN: Debug + NodeId,
//     //     IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
//     // {
//     //     let mut from_file = false;
//     //     let sss: ExploreStructuralPositions<'_, IdN, Idx> = self.src.clone();
//     //     let len = if let Some(x) = sss.peek_node() {
//     //         let b = self.stores.node_store().resolve(x.as_id());
//     //         let t = self.stores.resolve_type(x.as_id());
//     //         if let Some(y) = b.try_bytes_len() {
//     //             if t.is_file() {
//     //                 from_file = true;
//     //             }
//     //             y as usize
//     //         } else {
//     //             0
//     //         }
//     //     } else {
//     //         0
//     //     };
//     //     let offset = 0;
//     //     let path = vec![];
//     //     Self::make_position2_aux(sss, self.stores, from_file, len, offset, path)
//     // }

//     // fn make_position2_aux(
//     //     mut sss: ExploreStructuralPositions<'a, IdN, Idx>,
//     //     stores: &'store HAST,
//     //     from_file: bool,
//     //     len: usize,
//     //     mut offset: usize,
//     //     mut path: Vec<&'store str>,
//     // ) -> Position
//     // where
//     //     'a: 'store,
//     //     HAST: HyperAST<'store, IdN = IdN::IdN>,
//     //     for<'t> <HAST as crate::types::AstLending<'t>>::RT: Typed<Type = AnyType> + WithSerialization + WithChildren,
//     //     IdN: Copy + Debug + NodeId,
//     //     IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
//     // {
//     //     if from_file {
//     //         while let Some(p) = sss.peek_parent_node() {
//     //             let b = stores.node_store().resolve(p.as_id());
//     //             let t = stores.resolve_type(p.as_id());
//     //             let o = sss.peek_offset().unwrap();
//     //             let o: <HAST::T as WithChildren>::ChildIdx =
//     //                 num::cast(o).expect("failed to cast, cannot put value of Idx in ChildIdx");
//     //             let c: usize = {
//     //                 let v: Vec<_> = b
//     //                     .children()
//     //                     .unwrap()
//     //                     .before(o - one())
//     //                     .iter_children()
//     //                     .collect();
//     //                 v.iter()
//     //                     .map(|x| stores.node_store().resolve(x).try_bytes_len().unwrap())
//     //                     .sum()
//     //             };
//     //             offset += c;
//     //             if t.is_file() {
//     //                 sss.next();
//     //                 break;
//     //             } else {
//     //                 sss.next();
//     //             }
//     //         }
//     //     }
//     //     for p in sss {
//     //         let b = stores.node_store().resolve(p.as_id());
//     //         let l = stores.label_store().resolve(b.get_label_unchecked());
//     //         path.push(l)
//     //     }
//     //     let file = PathBuf::from_iter(path.iter().rev());
//     //     Position::new(file, offset, len)
//     // }
// }

impl<'store, 'src, 'a, HAST, S> WithHyperAstPositionConverter<'store, 'src, S, HAST>
where
    S: super::node_filter_traits::Full,
{
    pub fn compute_pos_post_order<O, B>(&self) -> O
    where
        HAST: HyperAST,
        S: super::position_accessors::WithFullPostOrderPath<HAST::IdN, Idx = HAST::Idx>
            + super::position_accessors::SolvedPosition<HAST::IdN>,
        // IdN: Debug + NodeId + Clone,
        HAST::IdN: NodeId<IdN = HAST::IdN> + Eq + Debug,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT:
            WithSerialization + WithChildren + WithStats,
        HAST::Idx: Debug,
        B: building::bottom_up::ReceiveInFile<HAST::IdN, HAST::Idx, usize, O>
            + building::bottom_up::CreateBuilder,
        B::SB1<O>: building::bottom_up::ReceiveDir<HAST::IdN, HAST::Idx, O>,
    {
        use building::bottom_up;
        let builder: B = building::bottom_up::CreateBuilder::create();
        let stores = self.stores;
        let mut prev_x;
        let (mut x, mut iter) = self.src.iter_with_nodes();
        let mut o;
        let len = {
            let b = stores.node_store().resolve(x.as_id());
            let t = stores.resolve_type(x.as_id());
            // dbg!(t);
            let len = b.try_bytes_len();
            assert!(len.is_some() || t.is_directory());
            prev_x = x;
            len
        };
        use bottom_up::ReceiveNode;
        let mut builder: B::SB1<O> = if let Some(len) = len {
            let mut builder = builder.set(len);
            let builder = loop {
                let Some(aaa) = iter.next() else {
                    use bottom_up::SetRoot;
                    return builder.set_root(prev_x);
                };
                x = aaa.1;
                o = aaa.0;
                let b = stores.node_store().resolve(x.as_id());
                let t = stores.resolve_type(x.as_id());
                // dbg!(&prev_x);
                // dbg!(&x);
                // dbg!(o);
                // dbg!(t);
                let v = &b.children().unwrap();
                // dbg!(v
                //     .iter_children()
                //     .map(|x| stores
                //         .type_store()
                //         .resolve_type(&stores.node_store().resolve(x.as_id())))
                //     .collect::<Vec<_>>());

                // dbg!(aaa.0);
                assert_eq!(Some(&prev_x), v.get(o));
                let v = v.before(o);
                let v: Vec<_> = v.iter_children().collect();
                fn compute<'store, HAST: HyperAST>(
                    stores: &'store HAST,
                    x: &HAST::IdN,
                    col: &mut usize,
                ) -> usize
                where
                    HAST::IdN: NodeId<IdN = HAST::IdN> + Eq + Debug,
                    for<'t> <HAST as crate::types::AstLending<'t>>::RT:
                        WithStats + WithSerialization + WithChildren,
                {
                    let b = stores.node_store().resolve(&x);
                    let l = b.line_count();
                    if l == 0 {
                        *col += b.try_bytes_len().unwrap_or_default() as usize;
                    } else if let Some(cs) = b.children() {
                        for x in cs.iter_children() {
                            if compute(stores, &x, col) > 0 {
                                break;
                            }
                        }
                    } else {
                        *col += b.try_bytes_len().unwrap_or_default() as usize - b.line_count();
                    }
                    l
                }
                let mut row = 0;
                let mut col = 0;
                for x in v.iter().rev() {
                    if row == 0 {
                        row += compute(stores, x, &mut col);
                    } else {
                        let b = stores.node_store().resolve(x);
                        row += b.line_count();
                    }
                }
                let c = v
                    .into_iter()
                    .map(|x| {
                        let b = stores.node_store().resolve(&x);
                        // println!("{:?}", b.get_type());
                        // println!("T1:{:?}", b.get_type());
                        b.try_bytes_len().unwrap_or_default() as usize
                    })
                    .sum();

                use bottom_up::{ReceiveIdx, ReceiveOffset};
                use building::{ReceiveColumns, ReceiveRows};
                builder = builder.push(prev_x).push(c).push(row).push(col).push(o);
                prev_x = x;

                if t.is_file() {
                    let l = stores.label_store().resolve(b.get_label_unchecked());
                    break bottom_up::ReceiveDirName::push(builder, l);
                }
            };
            builder
        } else {
            builder.transit()
        };

        loop {
            let Some(aaa) = iter.next() else {
                use bottom_up::SetRoot;
                return builder.set_root(prev_x);
            };
            x = aaa.1;
            o = aaa.0;
            let b = stores.node_store().resolve(x.as_id());
            let _t = stores.resolve_type(x.as_id());

            // dbg!(t);
            // let v = &b.children().unwrap();
            // dbg!(v
            //     .iter_children()
            //     .map(|x| stores
            //         .type_store()
            //         .resolve_type(&stores.node_store().resolve(x.as_id())))
            //     .collect::<Vec<_>>());

            use bottom_up::ReceiveIdx;
            builder = builder.push(prev_x).push(o);
            prev_x = x;
        }
    }
}

// impl<'store, 'src, 'a, IdN: NodeId + Eq + Copy, Idx: PrimInt, HAST>
//     From<
//         WithHyperAstPositionConverter<'store, 'src, ExploreStructuralPositions<'a, IdN, Idx>, HAST>,
//     > for Position
// where
//     'a: 'store,
//     HAST: HyperAST<'store, IdN = IdN::IdN>,
//     for<'t> <HAST as crate::types::AstLending<'t>>::RT: Typed<Type = AnyType> + WithSerialization + WithChildren,
//     <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
//     IdN: Debug + NodeId,
//     IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
// {
//     fn from(
//         value: WithHyperAstPositionConverter<
//             'store,
//             'src,
//             ExploreStructuralPositions<'a, IdN, Idx>,
//             HAST,
//         >,
//     ) -> Self {
//         WithHyperAstPositionConverter::make_file_and_offset(&value)
//     }
// }

// TODO separate concerns
// TODO make_position should be a From<ExploreStructuralPositions> for FileAndOffsetPostionT and moved to relevant place
// TODO here the remaining logic should be about giving an iterator through the structural position
impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> ExploreStructuralPositions<'a, IdN, Idx> {
    pub fn make_position<'store, HAST>(self, stores: &'store HAST) -> Position
    where
        'a: 'store,
        HAST: HyperAST<IdN = IdN::IdN>,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT:
            Typed<Type = AnyType> + WithSerialization + WithChildren,
        HAST::Idx: Debug,
        IdN: Debug + NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
    {
        self.sps.check(stores).unwrap();
        // let parents = self.parents.iter().peekable();
        let mut from_file = false;
        // let mut len = 0;
        let len = if let Some(x) = self.peek_node() {
            let b = stores.node_store().resolve(x.as_id());
            let t = stores.resolve_type(x.as_id());
            if let Some(y) = b.try_bytes_len() {
                if t.is_file() {
                    from_file = true;
                }
                y as usize
                // Some(x)
            } else {
                0
                // None
            }
        } else {
            0
            // None
        };
        let offset = 0;
        let path = vec![];
        self.make_position_aux(stores, from_file, len, offset, path)
    }

    fn make_position_aux<'store: 'a, HAST>(
        mut self,
        stores: &'store HAST,
        from_file: bool,
        len: usize,
        mut offset: usize,
        mut path: Vec<&'a str>,
    ) -> Position
    where
        HAST: HyperAST<IdN = IdN::IdN>,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT:
            Typed<Type = AnyType> + WithSerialization + WithChildren,
        IdN: Copy + Debug + NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
    {
        // println!(
        //     "it: {:?},{:?},{:?}",
        //     &it.sps.nodes, &it.sps.offsets, &it.sps.parents
        // );
        if from_file {
            while let Some(p) = self.peek_parent_node() {
                // println!("i: {}", it.i);
                assert_ne!(p, self.peek_node().unwrap());
                assert_eq!(p, self.sps.nodes[self.sps.parents[self.i - 1]]);
                assert_eq!(self.peek_node().unwrap(), self.sps.nodes[self.i - 1]);
                // println!("nodes: {}, parents:{}, offsets:{}",it.sps.nodes.len(),it.sps.parents.len(),it.sps.offsets.len());
                let b = stores.node_store().resolve(p.as_id());
                let t = stores.resolve_type(p.as_id());
                // println!("T0:{:?}", t);
                // let o = it.sps.offsets[it]
                // println!("nodes: ({})", it.sps.nodes.len());
                // println!("offsets: ({}) {:?}", it.sps.offsets.len(), &it.sps.offsets);
                // println!("parents: ({}) {:?}", it.sps.parents.len(), &it.sps.parents);
                // println!(
                //     "o: {}, o p: {}",
                //     it.peek_offset().unwrap(),
                //     it.sps.offsets[it.sps.parents[it.i - 1]]
                // );
                let o = self.peek_offset().unwrap();
                let o: HAST::Idx =
                    num::cast(o).expect("failed to cast, cannot put value of Idx in ChildIdx");
                if self.peek_node().unwrap().as_id() != &b.children().unwrap()[o] {
                    if self.peek_node().unwrap().as_id() != &b.children().unwrap()[o] {
                        log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                    }
                    assert_eq!(
                        self.peek_node().unwrap().as_id(),
                        &b.children().unwrap()[o],
                        "p:{:?} b.cs:{:?} o:{:?} o p:{:?} i p:{}",
                        p,
                        b.children().unwrap().iter_children().collect::<Vec<_>>(),
                        self.peek_offset().unwrap(),
                        self.sps.offsets[self.sps.parents[self.i - 1]] - one(),
                        self.sps.parents[self.i - 1],
                    );
                }
                let c: usize = {
                    let v: Vec<_> = b.children().unwrap().before(o).iter_children().collect();
                    v.iter()
                        .map(|x| {
                            let b = stores.node_store().resolve(x);
                            // println!("{:?}", b.get_type());
                            // println!("T1:{:?}", b.get_type());
                            b.try_bytes_len().unwrap() as usize
                        })
                        .sum()
                };
                offset += c;
                if t.is_file() {
                    self.next();
                    break;
                } else {
                    self.next();
                }
            }
        }
        for p in self {
            let b = stores.node_store().resolve(p.as_id());
            // println!("type {:?}", b.get_type());
            // if !b.has_label() {
            //     panic!("{:?} should have a label", b.get_type());
            // }
            if let Some(l) = b.try_get_label() {
                let l = stores.label_store().resolve(l);
                // println!("value: {}",l);
                // path = path.join(path)
                path.push(l)
            }
        }
        let file = PathBuf::from_iter(path.iter().rev());
        Position::new(file, offset, len)
    }
}

// impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> ExploreStructuralPositions<'a, IdN, Idx> {
//     fn make_position2<'store, HAST>(self, stores: &'store HAST) -> Position
//     where
//         'a: 'store,
//         HAST: HyperAST<'store, IdN = IdN::IdN>,
//         for<'t> <HAST as crate::types::AstLending<'t>>::RT: Typed<Type = AnyType> + WithSerialization + WithChildren,
//         <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
//         IdN: Debug + NodeId,
//         IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
//     {
//         self.sps.check(stores).unwrap();
//         let mut from_file = false;
//         let len = if let Some(x) = self.peek_node() {
//             let b = stores.node_store().resolve(x.as_id());
//             let t = stores.resolve_type(x.as_id());
//             if let Some(y) = b.try_bytes_len() {
//                 if t.is_file() {
//                     from_file = true;
//                 }
//                 y as usize
//             } else {
//                 0
//             }
//         } else {
//             0
//         };
//         let offset = 0;
//         let path = vec![];
//         self.make_position2_aux(stores, from_file, len, offset, path)
//     }

//     fn make_position2_aux<'store: 'a, HAST>(
//         mut self,
//         stores: &'store HAST,
//         from_file: bool,
//         len: usize,
//         mut offset: usize,
//         mut path: Vec<&'a str>,
//     ) -> Position
//     where
//         HAST: HyperAST<'store, IdN = IdN::IdN>,
//         for<'t> <HAST as crate::types::AstLending<'t>>::RT: Typed<Type = AnyType> + WithSerialization + WithChildren,
//         IdN: Copy + Debug + NodeId,
//         IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
//     {
//         if from_file {
//             while let Some(p) = self.peek_parent_node() {
//                 let b = stores.node_store().resolve(p.as_id());
//                 let t = stores.resolve_type(p.as_id());
//                 let o = self
//                     .peek_offset()
//                     .expect("there should be an offset if there is a parent");
//                 let o: <HAST::T as WithChildren>::ChildIdx =
//                     num::cast(o).expect("failed to cast, cannot put value of Idx in ChildIdx");
//                 let c: usize = {
//                     let v: Vec<_> = b
//                         .children()
//                         .unwrap()
//                         .before(o)
//                         .iter_children()
//                         .collect();
//                     v.iter()
//                         .map(|x| stores.node_store().resolve(x).try_bytes_len().unwrap())
//                         .sum()
//                 };
//                 offset += c;
//                 if t.is_file() {
//                     self.next();
//                     break;
//                 } else {
//                     self.next();
//                 }
//             }
//         }
//         for p in self {
//             let b = stores.node_store().resolve(p.as_id());
//             let l = stores.label_store().resolve(b.get_label_unchecked());
//             path.push(l)
//         }
//         let file = PathBuf::from_iter(path.iter().rev());
//         Position::new(file, offset, len)
//     }
// }

impl<TIdN: TypedNodeId, Idx> From<TypedScout<TIdN, Idx>> for Scout<TIdN::IdN, Idx> {
    fn from(value: TypedScout<TIdN, Idx>) -> Self {
        Self {
            ancestors: value.ancestors,
            path: value.path,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Handle(usize);

struct StructuralPositionStore2<IdN = NodeIdentifier, Idx = u16> {
    persisted: Handle,
    nodes: Vec<IdN>,
    parents: Vec<Handle>,
    offsets: Vec<Idx>,
}

impl<IdN, Idx> Debug for StructuralPositionStore2<IdN, Idx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StructuralPositionStore")
            .field("persisted", &self.persisted)
            .field("nodes", &self.nodes.len())
            .field("parents", &self.parents.len())
            .field("offsets", &self.offsets.len())
            .finish()
    }
}

impl<IdN, Idx> StructuralPositionStore2<IdN, Idx> {
    fn persist(&mut self, h: Handle) {
        if self.persisted.0 <= h.0 - 1 {
            self.persisted.0 = h.0;
        }
    }
    fn parent(&self, h: Handle) -> Option<Handle> {
        if h.0 == 0 {
            panic!();
        }
        if h.0 == 1 {
            return None;
        }
        assert!(self.parents[h.0 - 1].0 < h.0);
        Some(self.parents[h.0 - 1])
    }
    fn node(&self, h: Handle) -> IdN
    where
        IdN: Copy,
    {
        if h.0 == 0 {
            panic!();
        }
        self.nodes[h.0 - 1]
    }
    fn offset(&self, h: Handle) -> Idx
    where
        Idx: Copy,
    {
        if h.0 == 0 {
            panic!();
        }
        self.offsets[h.0 - 1]
    }

    fn inc(&mut self, h: Handle, node: IdN) -> Handle
    where
        Idx: PrimInt,
    {
        if h.0 == 0 {
            panic!();
        }
        if self.persisted.0 < h.0 {
            self.nodes[h.0 - 1] = node;
            self.offsets[h.0 - 1] += num::one();
            h
        } else if let Some(p) = self.parent(h) {
            if self.persisted.0 == self.nodes.len() {
                self.nodes.push(node);
                self.offsets.push(self.offsets[h.0 - 1] + num::one());
                self.parents.push(p);
                let mut h = self.persisted;
                h.0 += 1;
                h
            } else {
                assert!(self.nodes.len() > self.persisted.0);
                self.nodes[self.persisted.0] = node;
                self.offsets[self.persisted.0] = self.offsets[h.0 - 1] + num::one();
                self.parents[self.persisted.0] = p;
                let mut h = self.persisted;
                h.0 += 1;
                h
            }
        } else {
            unreachable!()
        }
    }

    fn down(&mut self, h: Handle, node: IdN, offset: Idx) -> Handle {
        if self.persisted.0 <= h.0 {
            let mut c = h;
            c.0 += 1;
            if self.nodes.len() == c.0 - 1 {
                assert_eq!(self.offsets.len(), c.0 - 1);
                self.nodes.push(node);
                self.offsets.push(offset);
                self.parents.push(h);
            } else if self.nodes.len() < c.0 - 1 {
                dbg!(self.nodes.len());
                dbg!(self.offsets.len());
                dbg!(self.persisted.0);
                dbg!(c.0);
                panic!()
            } else {
                self.nodes[c.0 - 1] = node;
                self.offsets[c.0 - 1] = offset;
                self.parents[c.0 - 1] = h;
            }
            c
        } else {
            let mut h = h;
            h.0 -= 1;
            self.nodes[self.persisted.0] = node;
            self.offsets[self.persisted.0] = offset;
            self.parents[self.persisted.0] = h;
            let mut r = self.persisted;
            r.0 += 1;
            r
        }
    }
}

pub trait AAA<IdN, Idx> {
    fn node(&self) -> IdN;
    fn offset(&self) -> Idx;
    fn parent(&self) -> Option<IdN>;
    fn up(&mut self) -> bool;
}
pub trait BBB<IdN, Idx>: AAA<IdN, Idx> {
    fn inc(&mut self, node: IdN);
    fn down(&mut self, node: IdN, offset: Idx);
}

use std::cell::RefCell;
use std::rc::Rc;

/// Cursor backed by a store, thus allowing to efficiently yield nodes, while sharing the shared sub path between all nodes.
/// As long as a node is not persisted, this cursor reuses and mutate to update itself.
// only tags::BottomUpFull is possible for efficiency
pub struct CursorWithPersistance<IdN, Idx = u16> {
    s: Rc<RefCell<StructuralPositionStore2<IdN, Idx>>>,
    h: Handle,
}

impl<IdN, Idx> PartialEq for CursorWithPersistance<IdN, Idx> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.s, &other.s) && self.h.0 == other.h.0
    }
}

impl<IdN, Idx> Eq for CursorWithPersistance<IdN, Idx> {}

impl<IdN, Idx> CursorWithPersistance<IdN, Idx> {
    pub fn new(node: IdN) -> Self
    where
        Idx: PrimInt,
    {
        let mut n = Self::default();
        n.h = n.s.borrow_mut().down(n.h, node, num::zero());
        n
    }
    pub fn default() -> Self {
        let s = StructuralPositionStore2 {
            persisted: Handle(0),
            nodes: vec![],
            parents: vec![],
            offsets: vec![],
        };
        let s = Rc::new(RefCell::new(s));
        Self { s, h: Handle(0) }
    }
    pub fn persist(&mut self) -> PersistedNode<IdN, Idx> {
        self.s.borrow_mut().persist(self.h);
        PersistedNode {
            s: self.s.clone(),
            h: self.h,
        }
    }
    pub fn persist_parent(&mut self) -> Option<PersistedNode<IdN, Idx>> {
        let p = self.s.borrow().parent(self.h)?;
        self.s.borrow_mut().persist(p);
        Some(PersistedNode {
            s: self.s.clone(),
            h: p,
        })
    }
    pub fn ref_node(&self) -> RefNode<IdN, Idx> {
        let s = self.s.borrow();
        RefNode { s, h: self.h }
    }
    pub fn ref_parent(&self) -> Option<RefNode<IdN, Idx>> {
        let p = self.s.borrow().parent(self.h)?;
        let s = self.s.borrow();
        Some(RefNode { s, h: p })
    }
    pub fn ext(&self) -> ExtRefNode<IdN, Idx> {
        let s = self.s.borrow();
        ExtRefNode::new(s, self.h)
    }
}

impl<IdN, Idx> BBB<IdN, Idx> for CursorWithPersistance<IdN, Idx>
where
    IdN: Copy,
    Idx: PrimInt,
{
    fn inc(&mut self, node: IdN) {
        self.h = self.s.borrow_mut().inc(self.h, node);
    }

    fn down(&mut self, node: IdN, offset: Idx) {
        self.h = self.s.borrow_mut().down(self.h, node, offset);
    }
}

impl<IdN, Idx> AAA<IdN, Idx> for CursorWithPersistance<IdN, Idx>
where
    IdN: Copy,
    Idx: Copy,
{
    fn node(&self) -> IdN {
        self.s.borrow().node(self.h)
    }
    fn offset(&self) -> Idx {
        self.s.borrow().offset(self.h)
    }
    fn up(&mut self) -> bool {
        if let Some(p) = self.s.borrow().parent(self.h) {
            self.h = p;
            return true;
        }
        false
    }
    fn parent(&self) -> Option<IdN> {
        let p = self.s.borrow().parent(self.h)?;
        Some(self.s.borrow().node(p))
    }
}

/// Node that was persited i.e. mutating the cursor guarantee that this node observable values won't change.
#[derive(Clone)]
pub struct PersistedNode<IdN, Idx = u16> {
    s: Rc<RefCell<StructuralPositionStore2<IdN, Idx>>>,
    h: Handle,
}

impl<IdN, Idx> PartialEq for PersistedNode<IdN, Idx> {
    fn eq(&self, other: &Self) -> bool {
        // TODO check
        Rc::ptr_eq(&self.s, &other.s) && self.h.0 == other.h.0
    }
}

impl<IdN, Idx> Eq for PersistedNode<IdN, Idx> {}

impl<IdN, Idx> PartialOrd for PersistedNode<IdN, Idx> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if !Rc::ptr_eq(&self.s, &other.s) {
            return None;
        }
        self.h.0.partial_cmp(&other.h.0)
    }
}

impl<IdN, Idx> Ord for PersistedNode<IdN, Idx> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if !Rc::ptr_eq(&self.s, &other.s) {
            panic!()
        }
        self.h.0.cmp(&other.h.0)
    }
}

impl<IdN, Idx> PersistedNode<IdN, Idx>
where
    IdN: Copy,
    Idx: Copy,
{
    pub fn ext(&self) -> ExtRefNode<IdN, Idx> {
        let s = self.s.borrow();
        ExtRefNode::new(s, self.h)
    }

    pub fn offsets(mut self) -> Vec<Idx> {
        let mut r = vec![];
        loop {
            r.push(self.offset());
            if !self.up() {
                break;
            }
        }
        r
    }
}

impl<IdN, Idx> AAA<IdN, Idx> for PersistedNode<IdN, Idx>
where
    IdN: Copy,
    Idx: Copy,
{
    fn node(&self) -> IdN {
        self.s.borrow().node(self.h)
    }
    fn offset(&self) -> Idx {
        self.s.borrow().offset(self.h)
    }
    fn up(&mut self) -> bool {
        if let Some(p) = self.s.borrow().parent(self.h) {
            self.h = p;
            return true;
        }
        false
    }
    fn parent(&self) -> Option<IdN> {
        let p = self.s.borrow().parent(self.h)?;
        Some(self.s.borrow().node(p))
    }
}
/// Node that is possibly not persited i.e. cannot safely mutate the cursor at the same time.
/// If you need to read a node and modify the cursor at the same time, make a [`PersistedNode`].
pub struct RefNode<'a, IdN, Idx = u16> {
    s: std::cell::Ref<'a, StructuralPositionStore2<IdN, Idx>>,
    h: Handle,
}

impl<'a, IdN, Idx> Clone for RefNode<'a, IdN, Idx> {
    fn clone(&self) -> Self {
        Self {
            s: std::cell::Ref::clone(&self.s),
            h: self.h.clone(),
        }
    }
}

impl<'a, IdN, Idx> PartialOrd for RefNode<'a, IdN, Idx> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if !std::ptr::eq(&self.s, &other.s) {
            return None;
        }
        self.h.0.partial_cmp(&other.h.0)
    }
}

impl<'a, IdN, Idx> Ord for RefNode<'a, IdN, Idx> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if !std::ptr::eq(&self.s, &other.s) {
            panic!()
        }
        self.h.0.cmp(&other.h.0)
    }
}

impl<'a, IdN, Idx> RefNode<'a, IdN, Idx>
where
    IdN: Copy,
    Idx: Copy,
{
    pub fn ext(&self) -> ExtRefNode<'a, IdN, Idx> {
        let s = std::cell::Ref::clone(&self.s);
        ExtRefNode::new(s, self.h)
    }
}

impl<'a, IdN, Idx> AAA<IdN, Idx> for RefNode<'a, IdN, Idx>
where
    IdN: Copy,
    Idx: Copy,
{
    fn node(&self) -> IdN {
        self.s.node(self.h)
    }
    fn offset(&self) -> Idx {
        self.s.offset(self.h)
    }
    fn up(&mut self) -> bool {
        if let Some(p) = self.s.parent(self.h) {
            self.h = p;
            return true;
        }
        false
    }
    fn parent(&self) -> Option<IdN> {
        let p = self.s.parent(self.h)?;
        Some(self.s.node(p))
    }
}

impl<'a, IdN, Idx> PartialEq for RefNode<'a, IdN, Idx> {
    fn eq(&self, other: &Self) -> bool {
        // TODO check
        std::ptr::eq(&self.s, &other.s) && self.h.0 == other.h.0
    }
}

impl<'a, IdN, Idx> Eq for RefNode<'a, IdN, Idx> {}

impl<'a, IdN: std::hash::Hash, Idx: std::hash::Hash> std::hash::Hash for RefNode<'a, IdN, Idx>
where
    IdN: Copy,
    Idx: Copy,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if cfg!(debug_assertions) {
            todo!("make tests to assert it gives the same results as in offsets_and_nodes")
        }
        // TODO make tests to assert it gives the same results as in offsets_and_nodes
        let mut s = self.clone();
        // self.parents.first().hash(state);
        s.node().hash(state);
        loop {
            // self.offsets.hash(state);
            s.offset().hash(state);
            if !s.up() {
                // self.parents.last().hash(state);
                s.node().hash(state);
                break;
            }
        }
    }
}

pub struct ExtRefNode<'a, IdN, Idx = u16> {
    s: std::cell::Ref<'a, StructuralPositionStore2<IdN, Idx>>,
    h: Handle,
    ext_nodes: Vec<IdN>,
    ext_offsets: Vec<Idx>,
}
impl<'a, IdN: Clone, Idx: Clone> Clone for ExtRefNode<'a, IdN, Idx> {
    fn clone(&self) -> Self {
        Self {
            s: std::cell::Ref::clone(&self.s),
            h: self.h.clone(),
            ext_nodes: self.ext_nodes.clone(),
            ext_offsets: self.ext_offsets.clone(),
        }
    }
}
impl<'a, IdN, Idx> ExtRefNode<'a, IdN, Idx> {
    fn new(s: std::cell::Ref<'a, StructuralPositionStore2<IdN, Idx>>, h: Handle) -> Self {
        ExtRefNode {
            s,
            h,
            ext_nodes: vec![],
            ext_offsets: vec![],
        }
    }
}

impl<'a, IdN, Idx> BBB<IdN, Idx> for ExtRefNode<'a, IdN, Idx>
where
    IdN: Copy,
    Idx: PrimInt,
{
    fn inc(&mut self, node: IdN) {
        if self.ext_nodes.is_empty() {
            let o = self.s.offset(self.h);
            if let Some(p) = self.s.parent(self.h) {
                self.h = p;
            } else {
                todo!()
            }
            self.ext_nodes.push(node);
            self.ext_offsets.push(o + num::one());
        } else {
            *self.ext_nodes.last_mut().unwrap() = node;
            *self.ext_offsets.last_mut().unwrap() += num::one();
        }
    }

    fn down(&mut self, node: IdN, offset: Idx) {
        self.ext_nodes.push(node);
        self.ext_offsets.push(offset);
    }
}

impl<'a, IdN, Idx> AAA<IdN, Idx> for ExtRefNode<'a, IdN, Idx>
where
    IdN: Copy,
    Idx: Copy,
{
    fn node(&self) -> IdN {
        if let Some(n) = self.ext_nodes.last() {
            *n
        } else {
            self.s.node(self.h)
        }
    }
    fn offset(&self) -> Idx {
        if let Some(o) = self.ext_offsets.last() {
            *o
        } else {
            self.s.offset(self.h)
        }
    }
    fn up(&mut self) -> bool {
        if self.ext_nodes.pop().is_some() {
            assert!(self.ext_offsets.pop().is_some());
            return true;
        } else if let Some(p) = self.s.parent(self.h) {
            self.h = p;
            return true;
        }
        false
    }
    fn parent(&self) -> Option<IdN> {
        if self.ext_nodes.len() > 1 {
            self.ext_nodes.get(self.ext_nodes.len() - 2).copied()
        } else if self.ext_nodes.len() == 1 {
            Some(self.s.node(self.h))
        } else {
            let p = self.s.parent(self.h)?;
            Some(self.s.node(p))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple2() {
        let mut c = CursorWithPersistance::default();
        c.down(0u32, 0u32);
        assert_eq!(c.node(), 0);
        assert_eq!(c.offset(), 0);
        c.inc(1u32);
        assert_eq!(c.node(), 1);
        assert_eq!(c.offset(), 1);
        let n = c.persist();
        assert_eq!(n.node(), 1);
        assert_eq!(n.offset(), 1);
        c.down(2u32, 0u32);
        assert_eq!(c.node(), 2);
        assert_eq!(c.offset(), 0);
        assert!(c.up());
        assert_eq!(c.node(), 1);
        assert_eq!(c.offset(), 1);
        c.down(2u32, 0u32);
        assert_eq!(c.node(), 2);
        assert_eq!(c.offset(), 0);
        assert!(c.up());
        assert_eq!(c.node(), 1);
        assert_eq!(c.offset(), 1);
    }
}
