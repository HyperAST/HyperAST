use super::{building, tags, Position, TreePath, WithHyperAstPositionConverter};
use std::{fmt::Debug, path::PathBuf};

use num::one;

use crate::{
    store::defaults::NodeIdentifier,
    types::{
        self, AnyType, Children, HyperAST, HyperType, IterableChildren, LabelStore, Labeled,
        NodeId, NodeStore, TypeStore, Typed, TypedNodeId, WithChildren, WithSerialization,
    }, PrimInt,
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
        type It = IterOffsets<'a, IdN, Idx>;

        fn iter(&self) -> Self::It {
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
        type ItPath = IterOffsetsNodes<'a, IdN, Idx>;

        fn iter_offsets_and_parents(&self) -> Self::ItPath {
            IterOffsetsNodes(self.clone())
        }
    }
    impl<'a, IdN: NodeId + Eq + Copy, Idx: PrimInt> WithFullPostOrderPath<IdN>
        for ExploreStructuralPositions<'a, IdN, Idx>
    {
        fn iter_with_nodes(&self) -> (IdN, Self::ItPath) {
            (self.node(), IterOffsetsNodes(self.clone()))
        }
    }

    pub struct IterOffsetsNodes<'a, IdN, Idx = usize>(ExploreStructuralPositions<'a, IdN, Idx>);

    impl<'a, IdN: Copy, Idx: PrimInt> Iterator for IterOffsetsNodes<'a, IdN, Idx> {
        type Item = (Idx, IdN);

        fn next(&mut self) -> Option<Self::Item> {
            let o = self.0.sps.offsets[self.0.i];
            self.0.try_go_up().map(|h| (o - one(), self.0.sps.nodes[h.0]))
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
//     //     HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
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
//     //     HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
//     //     <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
//     //     IdN: Debug + NodeId,
//     //     IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
//     // {
//     //     let mut from_file = false;
//     //     let sss: ExploreStructuralPositions<'_, IdN, Idx> = self.src.clone();
//     //     let len = if let Some(x) = sss.peek_node() {
//     //         let b = self.stores.node_store().resolve(x.as_id());
//     //         let t = self.stores.type_store().resolve_type(&b);
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
//     //     HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
//     //     IdN: Copy + Debug + NodeId,
//     //     IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
//     // {
//     //     if from_file {
//     //         while let Some(p) = sss.peek_parent_node() {
//     //             let b = stores.node_store().resolve(p.as_id());
//     //             let t = stores.type_store().resolve_type(&b);
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
    pub fn compute_pos_post_order<O, B, IdN>(&self) -> O
    where
        HAST: HyperAST<'store, IdN = IdN::IdN>,
        S: super::position_accessors::WithFullPostOrderPath<IdN, Idx = HAST::Idx>
            + super::position_accessors::SolvedPosition<IdN>,
        IdN: Debug + NodeId + Clone,
        IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
        HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
        <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
        B: building::bottom_up::ReceiveInFile<IdN, HAST::Idx, usize, O>
            + building::bottom_up::CreateBuilder,
        B::SB1<O>: building::bottom_up::ReceiveDir<IdN, HAST::Idx, O>,
    {
        use building::bottom_up;
        let builder: B = building::bottom_up::CreateBuilder::create();
        let stores = self.stores;
        let mut prev_x;
        let (mut x, mut iter) = self.src.iter_with_nodes();
        let mut o;
        let len = {
            let b = stores.node_store().resolve(x.as_id());
            let t = stores.type_store().resolve_type(&b);
            dbg!(t);
            let len = b.try_bytes_len();
            assert!(len.is_some() || t.is_directory());
            prev_x = x;
            len
        };
        use crate::position::building::bottom_up::ReceiveNode;
        let mut builder: B::SB1<O> = if let Some(len) = len {
            let mut builder = builder.set(len);
            let builder = loop {
                let Some(aaa) = iter.next() else {
                    use bottom_up::SetRoot;
                    return builder.set_root(prev_x)
                };
                x = aaa.1;
                o = aaa.0;
                let b = stores.node_store().resolve(x.as_id());
                let t = stores.type_store().resolve_type(&b);

                dbg!(o);
                dbg!(t);
                let v = &b.children().unwrap();
                dbg!(v
                    .iter_children()
                    .map(|x| stores
                        .type_store()
                        .resolve_type(&stores.node_store().resolve(x.as_id())))
                    .collect::<Vec<_>>());

                // dbg!(aaa.0);
                assert_eq!(Some(prev_x.as_id()), v.get(o));
                let v = v.before(o);
                let v: Vec<_> = v.iter_children().collect();
                let c = v
                    .into_iter()
                    .map(|x| {
                        let b = stores.node_store().resolve(x);
                        // println!("{:?}", b.get_type());
                        // println!("T1:{:?}", b.get_type());
                        b.try_bytes_len().unwrap() as usize
                    })
                    .sum();

                use bottom_up::{ReceiveIdx, ReceiveOffset};
                builder = builder.push(prev_x).push(c).push(o);
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
                return builder.set_root(prev_x)
            };
            x = aaa.1;
            o = aaa.0;
            let b = stores.node_store().resolve(x.as_id());
            let t = stores.type_store().resolve_type(&b);

            dbg!(t);
            let v = &b.children().unwrap();
            dbg!(v
                .iter_children()
                .map(|x| stores
                    .type_store()
                    .resolve_type(&stores.node_store().resolve(x.as_id())))
                .collect::<Vec<_>>());

            use bottom_up::{ReceiveIdx, ReceiveNode, ReceiveOffset};
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
//     HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
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
    fn make_position<'store, HAST>(self, stores: &'store HAST) -> Position
    where
        'a: 'store,
        HAST: HyperAST<'store, IdN = IdN::IdN>,
        HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
        <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
        IdN: Debug + NodeId,
        IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
    {
        self.sps.check(stores).unwrap();
        // let parents = self.parents.iter().peekable();
        let mut from_file = false;
        // let mut len = 0;
        let len = if let Some(x) = self.peek_node() {
            let b = stores.node_store().resolve(x.as_id());
            let t = stores.type_store().resolve_type(&b);
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
        HAST: HyperAST<'store, IdN = IdN::IdN>,
        HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
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
                let t = stores.type_store().resolve_type(&b);
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
                let o: <HAST::T as WithChildren>::ChildIdx =
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
                    let v: Vec<_> = b
                        .children()
                        .unwrap()
                        .before(o)
                        .iter_children()
                        .collect();
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
//         HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
//         <<HAST as HyperAST<'store>>::T as types::WithChildren>::ChildIdx: Debug,
//         IdN: Debug + NodeId,
//         IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
//     {
//         self.sps.check(stores).unwrap();
//         let mut from_file = false;
//         let len = if let Some(x) = self.peek_node() {
//             let b = stores.node_store().resolve(x.as_id());
//             let t = stores.type_store().resolve_type(&b);
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
//         HAST::T: Typed<Type = AnyType> + WithSerialization + WithChildren,
//         IdN: Copy + Debug + NodeId,
//         IdN::IdN: NodeId<IdN = IdN::IdN> + Eq + Debug,
//     {
//         if from_file {
//             while let Some(p) = self.peek_parent_node() {
//                 let b = stores.node_store().resolve(p.as_id());
//                 let t = stores.type_store().resolve_type(&b);
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
