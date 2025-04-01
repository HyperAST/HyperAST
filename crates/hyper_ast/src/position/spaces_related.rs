use std::marker::PhantomData;
use std::path::PathBuf;

use num::{one, zero, ToPrimitive};

use super::Position;
use super::WithHyperAstPositionConverter;
use crate::position::building;
use crate::types::{
    self, Children, Childrn, HyperAST, HyperType, LabelStore, Labeled, NodeStore, TypeStore,
    WithChildren, WithSerialization,
};
use crate::PrimInt;

pub fn path_with_spaces<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    no_spaces: &mut It,
    stores: &'store HAST,
) -> (Vec<It::Item>, HAST::IdN)
where
    It::Item: Clone + PrimInt,
    HAST::IdN: Clone,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
    HAST: HyperAST,
    for<'t> <HAST as crate::types::AstLending<'t>>::RT:
        WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let mut x = root;
    let mut path_ids = vec![];
    let mut with_spaces = vec![];
    let mut path = vec![];
    for mut o in &mut *no_spaces {
        // dbg!(offset);
        let b = stores.node_store().resolve(&x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());

        let t = stores.resolve_type(&x);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }
        let mut with_s_idx = zero();
        if let Some(cs) = b.children() {
            if !t.is_directory() {
                for y in cs.iter_children() {
                    let b = stores.node_store().resolve(&y);
                    if !stores.resolve_type(&y).is_spaces() {
                        if o == zero() {
                            break;
                        }
                        o = o - one();
                    }
                    with_s_idx = with_s_idx + one();
                }
            } else {
                with_s_idx = o;
                // for y in 0..o.to_usize().unwrap() {
                //     let b = stores.node_store().resolve(cs[y]);
                //     println!("{:?}",b.get_type());
                // }
            }
            // if o.to_usize().unwrap() >= cs.len() {
            //     // dbg!("fail");
            // }
            if let Some(a) = cs.get(with_s_idx) {
                x = a.clone();
                with_spaces.push(with_s_idx);
                path_ids.push(x.clone());
            } else {
                dbg!();
                break;
            }
        } else {
            dbg!();
            break;
        };
    }
    if let Some(x) = no_spaces.next() {
        // assert!(no_spaces.next().is_none());
        dbg!(x);
        panic!()
    }
    let b = stores.node_store().resolve(&x);
    let t = stores.resolve_type(&x);
    if t.is_directory() || t.is_file() {
        let l = stores.label_store().resolve(b.get_label_unchecked());
        path.push(l);
    }
    path_ids.reverse();
    (with_spaces, x)
}

impl<'store, 'src, 'a, Idx: PrimInt, HAST>
    WithHyperAstPositionConverter<
        'store,
        'src,
        Filtered<super::offsets::Offsets<Idx>, node_filters::NoSpace>,
        HAST,
    >
{
    pub fn path_with_spaces<It: Iterator>(
        _root: HAST::IdN,
        _no_spaces: &mut It,
        _stores: &'store HAST,
    ) -> Filtered<Vec<It::Item>, node_filters::Full>
    where
        It::Item: Clone + PrimInt,
        HAST::IdN: Clone,
        HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
        HAST: HyperAST,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT:
            WithSerialization + WithChildren<ChildIdx = It::Item>,
    {
        todo!()
    }
}

pub fn global_pos_with_spaces<'store, T, NS, It: Iterator>(
    _root: T::TreeId,
    // increasing order
    _no_spaces: &mut It,
    _node_store: &'store NS,
) -> (Vec<It::Item>,)
where
    It::Item: Clone + PrimInt,
    T::TreeId: Clone,
    // NS: types::NodeStore<T::TreeId, N = T>,
    T: types::Tree<ChildIdx = It::Item> + types::WithStats,
{
    todo!()
    // let mut offset_with_spaces = zero();
    // let mut offset_without_spaces = zero();
    // // let mut x = root;
    // let mut res = vec![];
    // let (cs, size_no_s) = {
    //     let b = stores.node_store().resolve(&root);
    //     (b.children().unwrap().iter_children().collect::<Vec<_>>(),b.get_size())
    // };
    // let mut stack = vec![(root, size_no_s, 0, cs)];
    // while let Some(curr_no_space) = no_spaces.next() {
    //     loop {

    //         if curr_no_space == offset_without_spaces {
    //             res.push(offset_with_spaces);
    //             break;
    //         }
    //     }
    // }

    // (
    //     res,
    // )
}

pub fn compute_position_with_no_spaces<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, HAST::IdN, Vec<It::Item>)
where
    It::Item: Clone + PrimInt,
    HAST::IdN: Clone,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
    HAST: HyperAST,
    for<'t> <HAST as crate::types::AstLending<'t>>::RT:
        WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let (pos, mut path_ids, no_spaces) =
        compute_position_and_nodes_with_no_spaces(root, offsets, stores);
    (pos, path_ids.remove(path_ids.len() - 1), no_spaces)
}

pub fn compute_position_and_nodes_with_no_spaces<'store, HAST, It>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, Vec<HAST::IdN>, Vec<It::Item>)
where
    HAST::IdN: Clone,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
    HAST: HyperAST,
    for<'t> <HAST as crate::types::AstLending<'t>>::RT:
        WithSerialization + WithChildren<ChildIdx = It::Item>,
    It: Iterator,
    It::Item: Clone + PrimInt,
{
    let mut offset = 0;
    let mut x = root;
    let mut path_ids = vec![];
    let mut no_spaces = vec![];
    let mut path = vec![];
    for o in &mut *offsets {
        // dbg!(offset);
        let b = stores.node_store().resolve(&x);
        // dbg!(o.to_usize().unwrap());

        let t = stores.resolve_type(&x);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }
        let mut no_s_idx = zero();
        if let Some(cs) = b.children() {
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = stores.node_store().resolve(&y);
                    if !stores.resolve_type(&y).is_spaces() {
                        no_s_idx = no_s_idx + one();
                    }
                    offset += b.try_bytes_len().unwrap().to_usize().unwrap();
                }
            } else {
                no_s_idx = o;
                // for y in 0..o.to_usize().unwrap() {
                //     let b = stores.node_store().resolve(cs[y]);
                //     println!("{:?}",b.get_type());
                // }
            }
            // if o.to_usize().unwrap() >= cs.len() {
            //     // dbg!("fail");
            // }
            if let Some(a) = cs.get(o) {
                x = a.clone();
                no_spaces.push(no_s_idx);
                path_ids.push(x.clone());
            } else {
                dbg!();
                break;
            }
        } else {
            dbg!();
            break;
        };
    }
    assert!(offsets.next().is_none());
    let b = stores.node_store().resolve(&x);
    let t = stores.resolve_type(&x);
    if t.is_directory() || t.is_file() {
        let l = stores.label_store().resolve(b.get_label_unchecked());
        path.push(l);
    }

    let len = if !t.is_directory() {
        b.try_bytes_len().unwrap().to_usize().unwrap()
    } else {
        0
    };
    let file = PathBuf::from_iter(path.iter());
    path_ids.reverse();
    (Position::new(file, offset, len), path_ids, no_spaces)
}

// It should be possible to do the same with other filters if the size is precomputed.
// Computing a file_and_offsets without spaces is pretty useless outside of it being a compression of a naive path,
// indeed there is node point in serializing a filtered CST as it should produce in post cases invalide code,
// except with additional transformations such as with minification or obfuscation.
// On the other hand computing offsets without some nodes is very useful when doing a static analysis on an AST with another tool or a restrictive API

// struct NoSpaces<T>(Filtered<T,node_filters::NoSpace>);
mod node_filters {
    pub struct NoSpace;
    pub struct Full;
}

pub struct Filtered<T, F>(T, std::marker::PhantomData<F>);

impl<T, F> From<T> for Filtered<T, F> {
    fn from(value: T) -> Self {
        Self(value, std::marker::PhantomData)
    }
}

// type NoSpace

// top to bottom
pub type PathNoSpace<IdN, Idx> =
    Filtered<super::offsets::RootedOffsets<IdN, Idx>, node_filters::NoSpace>;

// top to bottom
type SpFull<IdN, Idx> =
    Filtered<super::offsets_and_nodes::StructuralPosition<IdN, Idx>, node_filters::NoSpace>;
type FileAndOffsetFull =
    Filtered<super::file_and_offset::Position<PathBuf, usize>, node_filters::Full>;

impl<'store, 'src, 'a, HAST, S> WithHyperAstPositionConverter<'store, 'src, S, HAST>
// WithHyperAstPositionConverter<'store, 'src, PathNoSpace<HAST::IdN, HAST::Idx>, HAST>
where
    HAST: HyperAST,
    S: super::position_accessors::WithPreOrderOffsets<Idx = HAST::Idx>,
    S: super::position_accessors::RootedPosition<HAST::IdN>,
    S: super::node_filter_traits::Full,
{
    pub fn compute_multi_position_with_no_spaces(
        &self,
    ) -> (FileAndOffsetFull, SpFull<HAST::IdN, HAST::Idx>)
    // ) -> (Position, Vec<HAST::IdN>, Vec<HAST::Idx>)
    where
        HAST::IdN: Clone,
        HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
        HAST: HyperAST,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithSerialization + WithChildren,
    {
        let stores = self.stores;
        // get root
        let mut x = self.src.root();

        let mut offset = 0;
        let mut path_ids = vec![];
        let mut no_spaces = vec![];
        let mut path = vec![];
        // iter offsets
        let mut offsets_iter = self.src.iter_offsets();
        loop {
            let b = stores.node_store().resolve(&x);
            let t = stores.resolve_type(&x);

            // handle name of file or directory
            if t.is_directory() || t.is_file() {
                let l = stores.label_store().resolve(b.get_label_unchecked());
                path.push(l);
            }

            let (cs, o) = match (b.children(), offsets_iter.next()) {
                (Some(cs), Some(o)) => (cs, o),
                (None, Some(_)) => panic!("there is no children remaining"),
                _ => break,
            };

            let mut no_s_idx = zero();
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = stores.node_store().resolve(&y);
                    if !stores.resolve_type(&y).is_spaces() {
                        no_s_idx = no_s_idx + one();
                    }
                    offset += b.try_bytes_len().unwrap().to_usize().unwrap();
                }
            } else {
                no_s_idx = o;
            }
            if let Some(a) = cs.get(o) {
                x = a.clone();
                no_spaces.push(no_s_idx);
                path_ids.push(x.clone());
            } else {
                dbg!();
                break;
            }
        }
        // construct output
        let b = stores.node_store().resolve(&x);
        let t = stores.resolve_type(&x);
        let len = if !t.is_directory() {
            b.try_bytes_len().unwrap().to_usize().unwrap()
        } else {
            0
        };
        let file = PathBuf::from_iter(path.iter());
        path_ids.reverse();
        no_spaces.reverse();
        // path_ids, no_spaces
        let o_and_n = todo!();
        (Position::new(file, offset, len).into(), o_and_n)
    }

    fn compute_multi_position_with_no_spaces2(
        &self,
    ) -> (FileAndOffsetFull, SpFull<HAST::IdN, HAST::Idx>)
    // ) -> (Position, Vec<HAST::IdN>, Vec<HAST::Idx>)
    where
        HAST::IdN: Clone,
        HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
        HAST: HyperAST,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithSerialization + WithChildren,
        // for<'b, 't> <<HAST as crate::types::AstLending<'t>>::RT as WithChildren>::Children: Clone,
    {
        let stores = self.stores;
        let mut x = self.src.root();
        let mut offsets_iter = self.src.iter_offsets();

        let mut aaa = PathBuf::default();
        let mut offset = 0;
        let mut path_ids = vec![];
        let mut no_spaces = vec![];
        let mut path = vec![];
        // iter offsets
        let mut bbb: FileAndOffsetPositionBuilder<_, usize> = {
            let (b, t) = loop {
                let b = stores.node_store().resolve(&x);
                let t = stores.resolve_type(&x);
                // handle name of directory
                if t.is_directory() {
                    let l = stores.label_store().resolve(b.get_label_unchecked());
                    path.push(l);
                    aaa.push(l);
                } else {
                    break (b, t);
                }

                let (cs, o) = match (b.children(), offsets_iter.next()) {
                    (Some(cs), Some(o)) => (cs.iter_children(), o),
                    (None, Some(_)) => panic!("there is no children remaining"),
                    _ => return todo!(),
                };

                let a = cs.get(o).expect("no child at path");
                no_spaces.push(o);
                path_ids.push(a.clone());
                x = a.clone();
            };
            if t.is_file() {
                assert!(t.is_file());
                let l = stores.label_store().resolve(b.get_label_unchecked());
                path.push(l);
                aaa.push(l);
            }
            aaa.into()
        };
        let (b, t) = loop {
            let b = stores.node_store().resolve(&x);
            let t = stores.resolve_type(&x);
            // handle name of file or directory
            assert!(!t.is_directory());

            let (cs, o) = match (b.children(), offsets_iter.next()) {
                (Some(cs), Some(o)) => (cs.iter_children(), o),
                (None, Some(_)) => panic!("there is no children remaining"),
                _ => break (stores.node_store().resolve(&x), t),
            };

            let mut no_s_idx = zero();
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = stores.node_store().resolve(&y);
                    if !stores.resolve_type(&y).is_spaces() {
                        no_s_idx = no_s_idx + one();
                    }
                    let len = b.try_bytes_len().unwrap().to_usize().unwrap();
                    offset += len;
                    bbb.inc_offset(len);
                }
            } else {
                no_s_idx = o;
            }
            let a = cs.get(o).expect("no child at path");
            no_spaces.push(no_s_idx);
            path_ids.push(a.clone());
            x = a.clone();
        };
        // construct output
        let len = if !t.is_directory() {
            b.try_bytes_len().unwrap().to_usize().unwrap()
        } else {
            0
        };
        let file = PathBuf::from_iter(path.iter());
        path_ids.reverse();
        no_spaces.reverse();
        // path_ids, no_spaces
        let o_and_n = todo!();
        (Position::new(file, offset, len).into(), o_and_n)
    }

    fn compute_multi_position_with_no_spaces3<B>(&self) -> B::Prepared
    // ) -> (Position, Vec<HAST::IdN>, Vec<HAST::Idx>)
    where
        HAST::IdN: Clone,
        HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
        HAST: HyperAST,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithSerialization + WithChildren,
        // for<'t, 'b> <<HAST as crate::types::AstLending<'t>>::RT as WithChildren>::Children: Clone,
        B: TopDownPosBuilder<HAST::IdN, HAST::Idx, usize, NoSpacePrepareParams<HAST::Idx>>
            + Default,
    {
        let mut builder: B = Default::default();
        let stores = self.stores;
        let mut x = self.src.root();
        let mut offsets_iter = self.src.iter_offsets();

        // let mut aaa = PathBuf::default();
        // let mut offset = 0;
        // let mut path_ids = vec![];
        // let mut no_spaces = vec![];
        // let mut path = vec![];
        // iter offsets
        let mut builder = {
            loop {
                let b = stores.node_store().resolve(&x);
                let t = stores.resolve_type(&x);
                // handle name of directory
                let l = if t.is_directory() {
                    stores.label_store().resolve(b.get_label_unchecked())
                    // path.push(l);
                    // aaa.push(l);
                } else if t.is_file() {
                    assert!(t.is_file());
                    let l = stores.label_store().resolve(b.get_label_unchecked());
                    break builder.seal_path(l);
                } else {
                    break builder.seal_without_path();
                };

                let (cs, idx) = match (b.children(), offsets_iter.next()) {
                    (Some(cs), Some(o)) => (cs.iter_children(), o),
                    (None, Some(_)) => panic!("there is no children remaining"),
                    _ => return builder.finish(x),
                };
                builder.push(x, idx, l, ());

                let a = cs.get(idx).expect("no child at path");
                x = a.clone();
            }
        };
        let (b, t) = loop {
            let b = stores.node_store().resolve(&x);
            let t = stores.resolve_type(&x);
            // handle name of file or directory
            assert!(!t.is_directory());

            let (cs, idx) = match (b.children(), offsets_iter.next()) {
                (Some(cs), Some(idx)) => (cs, idx),
                (None, Some(_)) => panic!("there is no children remaining"),
                _ => break (stores.node_store().resolve(&x), t),
            };

            let mut no_s_idx = zero();
            let mut byte_offset = 0;
            for y in cs.before(idx.clone()).iter_children() {
                let b = stores.node_store().resolve(&y);
                if !stores.resolve_type(&y).is_spaces() {
                    no_s_idx = no_s_idx + one();
                }
                let len = b.try_bytes_len().unwrap().to_usize().unwrap();
                byte_offset += len;
            }
            builder.push(x, idx, byte_offset, (no_s_idx,));
            let a = cs.get(idx).expect("no child at path");
            // no_spaces.push(no_s_idx);
            // path_ids.push(a.clone());
            x = a.clone();
        };
        // construct output
        let len = if !t.is_directory() {
            b.try_bytes_len().unwrap()
        } else {
            0
        };
        let len = num::cast(len).unwrap();
        // let file = PathBuf::from_iter(path.iter());
        // path_ids.reverse();
        // no_spaces.reverse();
        // path_ids, no_spaces
        // let o_and_n = todo!();
        // (Position::new(file, offset, len).into(), o_and_n)
        builder.finish(x, len, ())
    }

    pub fn compute_no_spaces<O, B>(&self) -> O
    // ) -> (Position, Vec<HAST::IdN>, Vec<HAST::Idx>)
    where
        HAST::IdN: Clone,
        HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
        HAST: HyperAST,
        for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithSerialization + WithChildren,
        // B: receivers_traits::top_down::ReceiveDir2<HAST::IdN, HAST::Idx, usize, O>
        B: building::top_down::ReceiveDir<HAST::IdN, HAST::Idx, O>
            + building::top_down::CreateBuilder,
        B::SB1<O>: building::top_down::ReceiveInFile<HAST::IdN, HAST::Idx, usize, O>,
    {
        let mut builder: B = building::top_down::CreateBuilder::create();
        let stores = self.stores;
        let mut x = self.src.root();
        let mut offsets_iter = self.src.iter_offsets();

        // let mut aaa = PathBuf::default();
        // let mut offset = 0;
        // let mut path_ids = vec![];
        // let mut no_spaces = vec![];
        // let mut path = vec![];
        // iter offsets
        use building::{top_down::ReceiveIdx, Transition};
        let mut builder: B::SB1<O> = {
            loop {
                let b = stores.node_store().resolve(&x);
                let t = stores.resolve_type(&x);
                // handle name of directory
                let l = if t.is_directory() {
                    stores.label_store().resolve(b.get_label_unchecked())
                    // path.push(l);
                    // aaa.push(l);
                } else if t.is_file() {
                    assert!(t.is_file());
                    let l = stores.label_store().resolve(b.get_label_unchecked());
                    break builder.set_file_name(l);
                } else {
                    break builder.transit();
                };

                let (cs, idx) = match (b.children(), offsets_iter.next()) {
                    (Some(cs), Some(o)) => (cs, o),
                    (None, Some(_)) => panic!("there is no children remaining"),
                    _ => return builder.set_node(x),
                };
                use building::top_down::ReceiveDirName;
                builder = builder.push(x).push(idx).push(l);

                let a = cs.get(idx).expect("no child at path");
                x = a.clone();
            }
        };
        let (b, t) = loop {
            let b = stores.node_store().resolve(&x);
            let t = stores.resolve_type(&x);
            // handle name of file or directory
            assert!(!t.is_directory());

            let (cs, idx) = match (b.children(), offsets_iter.next()) {
                (Some(cs), Some(idx)) => (cs, idx),
                (None, Some(_)) => panic!("there is no children remaining"),
                _ => break (stores.node_store().resolve(&x), t),
            };

            let mut no_s_idx = zero();
            let mut byte_offset = 0;
            let mut rows = zero();
            for y in cs.before(idx.clone()).iter_children() {
                let b = stores.node_store().resolve(&y);
                if !stores.resolve_type(&y).is_spaces() {
                    no_s_idx = no_s_idx + one();
                }
                let len = b.try_bytes_len().unwrap().to_usize().unwrap();
                byte_offset += len;
                // TODO count lines
            }
            use building::top_down::{ReceiveIdxNoSpace, ReceiveOffset, ReceiveParent};
            use building::ReceiveRows;
            builder = builder
                .push(x)
                .push(idx)
                .push(byte_offset)
                .push(no_s_idx)
                .push(rows);
            // builder.push(x, idx, byte_offset, (no_s_idx,));
            let a = cs.get(idx).expect("no child at path");
            // no_spaces.push(no_s_idx);
            // path_ids.push(a.clone());
            x = a.clone();
        };
        // construct output
        let len = if !t.is_directory() {
            b.try_bytes_len().unwrap()
        } else {
            0
        };
        let len = num::cast(len).unwrap();
        use building::top_down::SetNode;
        use building::SetLen;
        use building::SetLineSpan;
        builder.set(len).set(todo!()).set_node(x)
    }
}

#[derive(Default)]
pub struct NoSpacePrepareParams<Idx>(PhantomData<Idx>);

impl<Idx> AdditionalPrepareParams for NoSpacePrepareParams<Idx> {
    type C = (Idx,);
    type L = ();
}
impl<Idx> AdditionalPrepareFileParams for NoSpacePrepareParams<Idx> {
    type F = ();
}

pub trait AdditionalPrepareParams {
    type C;
    type L;
}
impl AdditionalPrepareParams for () {
    type C = ();
    type L = ();
}
impl AdditionalPrepareFileParams for () {
    type F = ();
}

pub trait AdditionalPrepareFileParams: AdditionalPrepareParams {
    type F;
}

pub trait TopDownPosBuilder<IdN, Idx, IdO, Additional: AdditionalPrepareFileParams = ()> {
    type Prepared;
    type SealedFile: SealedFileTopDownPosBuilder<
        IdN,
        Idx,
        IdO,
        Additional,
        Prepared = Self::Prepared,
    >;
    fn seal_path(self, file_name: &str) -> Self::SealedFile;
    fn seal_without_path(self) -> Self::SealedFile;
    fn push(&mut self, parent: IdN, idx: Idx, dir_name: &str, additional: Additional::F);
    fn finish(self, node: IdN) -> Self::Prepared;
}

pub trait SealedFileTopDownPosBuilder<IdN, Idx, IdO, Params: AdditionalPrepareParams = ()> {
    type Prepared;
    fn push(&mut self, parent: IdN, idx: Idx, offset: IdO, params: Params::C);
    fn finish(self, node: IdN, len: Idx, additional: Params::L) -> Self::Prepared;
}

struct TopDownPositionPreparer<IdN, Idx, IdO> {
    parents: Vec<IdN>,
    offsets: Vec<Idx>,
    filtered_offsets: Vec<Idx>,
    file: PathBuf,
    range: Option<std::ops::Range<IdO>>,
}

impl<IdN, Idx: PrimInt, IdO: PrimInt + Default>
    TopDownPosBuilder<IdN, Idx, IdO, NoSpacePrepareParams<Idx>>
    for TopDownPositionPreparer<IdN, Idx, IdO>
{
    type Prepared = TopDownPositionBuilder<IdN, Idx, IdO>;

    type SealedFile = TopDownPositionPreparer<IdN, Idx, IdO>;

    fn seal_path(mut self, file_name: &str) -> Self::SealedFile {
        self.file.push(file_name);
        self
    }

    fn seal_without_path(self) -> Self::SealedFile {
        self
    }

    fn push(&mut self, parent: IdN, offset: Idx, dir_name: &str, _additional: ()) {
        self.parents.push(parent);
        self.offsets.push(offset);
        self.file.push(dir_name);
    }

    fn finish(self, node: IdN) -> Self::Prepared {
        debug_assert!(self.range.is_none());
        Self::Prepared {
            parents: self.parents,
            offsets: self.offsets,
            file: self.file,
            range: Some(Default::default()),
            node,
        }
    }
}
impl<IdN, Idx: PrimInt, IdO: PrimInt>
    SealedFileTopDownPosBuilder<IdN, Idx, IdO, NoSpacePrepareParams<Idx>>
    for TopDownPositionPreparer<IdN, Idx, IdO>
{
    type Prepared = TopDownPositionBuilder<IdN, Idx, IdO>;

    fn push(&mut self, parent: IdN, idx: Idx, offset: IdO, (no_s_idx,): (Idx,)) {
        self.parents.push(parent);
        self.offsets.push(idx);
        self.range.as_mut().unwrap().start += num::cast(idx).unwrap();
        self.filtered_offsets.push(no_s_idx);
    }

    fn finish(self, node: IdN, len: Idx, _additional: ()) -> Self::Prepared {
        let mut range = self.range.unwrap();
        range.end = num::cast(len).unwrap();
        Self::Prepared {
            parents: self.parents,
            offsets: self.offsets,
            file: self.file,
            range: Some(range),
            node,
        }
    }
}

pub(super) struct TopDownPositionBuilder<IdN, Idx, IdO> {
    pub(super) parents: Vec<IdN>,
    pub(super) offsets: Vec<Idx>,
    pub(super) file: PathBuf,
    pub(super) range: Option<std::ops::Range<IdO>>,
    pub(super) node: IdN,
}

trait BottomUpPosBuilder<IdN, Idx> {
    type F0;
    type S: SealedOffsetBottomUpPosBuilder<IdN, Idx>;
    fn seal_offset(self) -> Self::S;
    fn push(&mut self, node: IdN, offset: Idx);
    fn finish(self, root: IdN) -> Self::F0;
}

trait SealedOffsetBottomUpPosBuilder<IdN, Idx> {
    type F;
    fn push(&mut self, node: IdN, offset: Idx, name: &str);
    fn finish(self, root: IdN) -> Self::F;
}

impl<F, T: num::Zero> From<F> for FileAndOffsetPositionBuilder<F, T> {
    fn from(value: F) -> Self {
        Self {
            path: value,
            offset: num::zero(),
        }
    }
}

struct FileAndOffsetPositionBuilder<F, T> {
    path: F,
    offset: T,
}

impl<F, T: PrimInt> FileAndOffsetPositionBuilder<F, T> {
    fn inc_offset(&mut self, o: T) -> &mut Self {
        self.offset += o;
        self
    }
    fn build(self, len: T) -> super::file_and_offset::Position<F, T> {
        super::file_and_offset::Position::new(self.path, self.offset, len)
    }
}
