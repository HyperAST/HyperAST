use std::{fmt::Debug, marker::PhantomData};

use num_traits::{cast, zero, PrimInt};

use crate::{
    matchers::decompressed_tree_store::{
        BreathFirstIterable, DecompressedTreeStore, DecompressedWithParent, Initializable,
        PostOrder, ShallowDecompressedTreeStore,
    },
    tree::tree_path::CompressedTreePath,
};
use hyper_ast::types::{GenericItem, NodeStore, Tree, WithChildren};

pub struct SD<'a, IdC, IdD, D: DecompressedTreeStore<'a, IdC, IdD>> {
    map: Vec<IdD>,
    // fc: Vec<IdD>,
    rev: Vec<IdD>,
    back: &'a D,
    phantom: PhantomData<*const IdC>,
}
impl<'a, IdC, IdD: Debug, D: Debug + DecompressedTreeStore<'a, IdC, IdD>> Debug
    for SD<'a, IdC, IdD, D>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SD")
            .field("map", &self.map)
            .field("rev", &self.rev)
            .field("back", &self.back)
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<'a, IdC, IdD: PrimInt, D: PostOrder<'a, IdC, IdD>> SD<'a, IdC, IdD, D> {
    // pub(crate) fn parent(&self, x: &IdD) -> Option<IdD> {
    //     todo!()
    // }

    // pub(crate) fn label(&self, x: &IdD) -> Label {
    //     todo!()
    // }

    // pub(crate) fn children(&self, x: &IdD) -> Vec<IdD> {
    //     todo!()
    // }

    // fn original(&self, x: &IdD) -> IdC {
    //     todo!()
    // }
    pub fn from<S>(s: &'a S, x: &'a D) -> Self
    where
        S: NodeStore<IdC>,
        // for<'c> < <S as NodeStore2<IdC>>::R  as GenericItem<'c>>::Item:WithChildren<TreeId = IdC>,
        S::R<'a>: WithChildren<TreeId = IdC>,
    {
        let mut map = Vec::with_capacity(x.len());
        // let mut fc = vec![num_traits::zero();x.len()];
        let mut rev = vec![num_traits::zero(); x.len()];
        let mut i = 0;
        rev[cast::<_, usize>(x.root()).unwrap()] = cast(i).unwrap();
        map.push(x.root());

        while map.len() < x.len() {
            let curr = &map[i];
            eprintln!("curr={:?}", curr.to_usize().unwrap());
            let cs = x.children(s, curr);
            // if cs.is_empty() {
            //     fc.push(cast(map.len()).unwrap());
            // }
            eprintln!(
                "{:?}",
                cs.iter().map(|x| x.to_usize().unwrap()).collect::<Vec<_>>()
            );
            rev[cast::<_, usize>(*curr).unwrap()] = cast(i).unwrap();
            map.extend(cs);
            i += 1;
        }

        map.shrink_to_fit();
        Self {
            map,
            // fc,
            rev,
            back: x,
            phantom: PhantomData,
        }
    }
}

impl<'a, IdC, IdD, D: DecompressedTreeStore<'a, IdC, IdD>> Initializable<'a, IdC, IdD>
    for SD<'a, IdC, IdD, D>
{
    fn new<
        // HK: HashKind,
        // HP: PrimInt,
        S,
    >(
        _store: &'a S,
        _root: &IdC,
    ) -> Self
    where
        S: 'a + NodeStore<IdC>,
        S::R<'a>: WithChildren<TreeId = IdC>,
    {
        panic!()
    }
}
// TODO back should be owned to disallow mutability from elsewhere
impl<'a, IdC, IdD: PrimInt, D: DecompressedTreeStore<'a, IdC, IdD>>
    ShallowDecompressedTreeStore<'a, IdC, IdD> for SD<'a, IdC, IdD, D>
{
    fn len(&self) -> usize {
        self.map.len()
    }

    fn original(&self, id: &IdD) -> IdC {
        // self.back.original(&self.map[cast::<_,usize>(*id).unwrap()])
        self.back.original(id)
    }

    fn leaf_count(&self) -> IdD {
        self.back.leaf_count()
    }

    fn root(&self) -> IdD {
        self.back.root()
    }

    fn path<Idx: PrimInt>(&self, _parent: &IdD, _descendant: &IdD) -> CompressedTreePath<Idx> {
        todo!()
    }

    fn child<'b, S>(
        &self,
        _store: &'b S,
        _x: &IdD,
        _p: &[<S::R<'b> as WithChildren>::ChildIdx],
    ) -> IdD
    // where
    //     S: NodeStore2<T::TreeId>, //NodeStoreExt<'a, T, R>,
    //     for<'b> S::R<'b>: WithChildren<TreeId = IdC>,
    where
        'a: 'b,
        S: NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        todo!()
    }

    fn children<'b, S>(&self, store: &'b S, x: &IdD) -> Vec<IdD>
    //     S: 'b + NodeStore2<T::TreeId, R<'b> = T>, //NodeStore<'b, T::TreeId, T>
    where
        'a: 'b,
        S: 'b + NodeStore<IdC>,
        // for<'c> < <S as NodeStore2<IdC>>::R  as GenericItem<'c>>::Item:WithChildren<TreeId = IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
    {
        // self.back.children(store,&self.map[cast::<_,usize>(*x).unwrap()])
        self.back.children(store, x)
    }
}

impl<'a, IdC, IdD: PrimInt, D: DecompressedTreeStore<'a, IdC, IdD>>
    DecompressedTreeStore<'a, IdC, IdD> for SD<'a, IdC, IdD, D>
{
    fn descendants<'b, S>(&self, _store: &S, _x: &IdD) -> Vec<IdD>
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
        //     S: 'b + NodeStore2<T::TreeId, R<'b> = T>, //NodeStore<'b, T::TreeId, T>
    {
        todo!()
    }

    fn descendants_count<'b, S>(&self, _store: &'b S, _x: &IdD) -> usize
    where
        S: 'b + NodeStore<IdC>,
        S::R<'b>: WithChildren<TreeId = IdC>,
        // S: 'b + NodeStore2<T::TreeId, R<'b> = T>, //NodeStore<'b, T::TreeId, T>
    {
        todo!()
    }

    fn first_descendant(&self, _i: &IdD) -> IdD {
        todo!()
    }
}
impl<
        'd,
        IdC,
        IdD: PrimInt,
        D: DecompressedTreeStore<'d, IdC, IdD> + DecompressedWithParent<'d, IdC, IdD>,
    > DecompressedWithParent<'d, IdC, IdD> for SD<'d, IdC, IdD, D>
{
    fn has_parent(&self, _id: &IdD) -> bool {
        todo!()
    }

    fn parent(&self, id: &IdD) -> Option<IdD> {
        self.back.parent(id)
    }

    fn position_in_parent<S>(&self, store: &S, c: &IdD) -> <S::R<'d> as WithChildren>::ChildIdx
    where
        S: NodeStore<IdC>,
        // for<'c> < <S as NodeStore2<IdC>>::R  as GenericItem<'c>>::Item:WithChildren<TreeId = IdC>,
        S::R<'d>: WithChildren<TreeId = IdC>,
        // S: 'b + NodeStore2<T::TreeId, R<'b> = T>, //NodeStore<'b, T::TreeId, T>
    {
        self.back.position_in_parent(store, c)
    }
}

impl<'d, IdC, IdD: 'static + PrimInt, D: DecompressedTreeStore<'d, IdC, IdD>>
    BreathFirstIterable<'d, IdC, IdD> for SD<'d, IdC, IdD, D>
{
    type It = Iter<'d, IdD>;

    fn iter_bf(&'_ self) -> Iter<'_, IdD> {
        Iter {
            curr: zero(),
            len: self.map.len(),
            map: &self.map,
        }
    }
}

pub struct Iter<'a, IdD> {
    curr: usize,
    len: usize,
    map: &'a [IdD],
}

impl<'a, IdD: PrimInt> Iterator for Iter<'a, IdD> {
    type Item = IdD;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.len {
            None
        } else {
            let r = self.curr;
            self.curr = r + 1;
            Some(self.map[cast::<_, usize>(r).unwrap()])
        }
    }
}
