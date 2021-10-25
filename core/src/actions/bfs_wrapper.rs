use std::marker::PhantomData;

use num_traits::{PrimInt, cast};

use crate::{matchers::decompressed_tree_store::{BreathFirstContigousSiblings, BreathFirstIterable, DecompressedTreeStore, DecompressedWithParent, Initializable, PostOrder, ShallowDecompressedTreeStore}, tree::{tree::{self, NodeStore, Tree, WithChildren}, tree_path::CompressedTreePath}};

pub(crate) struct SD<'a, IdC, IdD, D: DecompressedTreeStore<IdC, IdD>> {
    map: Vec<IdD>,
    // fc: Vec<IdD>,
    rev: Vec<IdD>,
    back: &'a D,
    phantom: PhantomData<*const IdC>,
}

impl<'a, IdC, IdD:PrimInt, D: PostOrder<IdC, IdD>> SD<'a, IdC, IdD, D> {
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
    pub fn from<T: WithChildren<TreeId=IdC>, S: NodeStore<T>>(
        s: &'a S,
        x: &'a D,
    ) -> Self {
        let mut map = Vec::with_capacity(x.len());
        // let mut fc = vec![num_traits::zero();x.len()];
        let mut rev = vec![num_traits::zero();x.len()];
        let mut i = 0;
        rev[cast::<_,usize>(x.root()).unwrap()] = cast(i).unwrap();
        map.push(x.root());

        while map.len() < x.len() {
            let curr = &map[i];
            let cs = x.children(s, curr);
            // if cs.is_empty() {
            //     fc.push(cast(map.len()).unwrap());
            // }
            rev[cast::<_,usize>(*curr).unwrap()] = cast(i).unwrap();
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

impl<'a, IdC, IdD, D: DecompressedTreeStore<IdC, IdD>> Initializable<IdC, IdD>
    for SD<'a, IdC, IdD, D>
{
    fn new<
        T: Tree<TreeId = IdC>, // + WithHashs<HK = HK, HP = HP>,
        // HK: HashKind,
        // HP: PrimInt,
        S: NodeStore<T>,
    >(
        store: &S,
        root: &IdC,
    ) -> Self {
        panic!()
    }
}
// TODO back should be owned to disallow mutability from elsewhere
impl<'a, IdC, IdD: PrimInt, D: DecompressedTreeStore<IdC, IdD>> ShallowDecompressedTreeStore<IdC, IdD>
    for SD<'a, IdC, IdD, D>
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
        num_traits::zero()
    }

    fn path(&self, parent: &IdD, descendant: &IdD) -> CompressedTreePath<u32> {
        todo!()
    }

    fn child<
        T: WithChildren<TreeId = IdC>,
        S: NodeStore<T>,
    >(
        &self,
        store: &S,
        x: &IdD,
        p: &[T::ChildIdx],
    ) -> IdD {
        todo!()
    }

    fn children<
        T: WithChildren<TreeId = IdC>,
        S: NodeStore<T>,
    >(
        &self,
        store: &S,
        x: &IdD,
    ) -> Vec<IdD> {
        // self.back.children(store,&self.map[cast::<_,usize>(*x).unwrap()])
        self.back.children(store,x)
    }
}

impl<'a, IdC, IdD:PrimInt, D: DecompressedTreeStore<IdC, IdD>> DecompressedTreeStore<IdC, IdD>
    for SD<'a, IdC, IdD, D>
{
    fn descendants<T: Tree<TreeId = IdC>, S: NodeStore<T>>(
        &self,
        store: &S,
        x: &IdD,
    ) -> Vec<IdD> {
        todo!()
    }

    fn descendants_count<
        T: Tree<TreeId = IdC>,
        S: NodeStore<T>,
    >(
        &self,
        store: &S,
        x: &IdD,
    ) -> usize {
        todo!()
    }

    fn first_descendant(&self, i: &IdD) -> IdD {
        todo!()
    }
}
impl<'a, IdC, IdD:PrimInt, D: DecompressedTreeStore<IdC, IdD> + DecompressedWithParent<IdD>> DecompressedWithParent<IdD>
    for SD<'a, IdC, IdD, D>
{
    fn has_parent(&self, id: &IdD) -> bool {
        todo!()
    }

    fn parent(&self, id: &IdD) -> Option<IdD> {
        self.back.parent(id)
    }

    fn position_in_parent<
        T: WithChildren,
        S: NodeStore<T>,
    >(
        &self,
        store: &S,
        c: &IdD,
    ) -> T::ChildIdx {
        todo!()
    }
}

impl<'a,IdC, IdD:'static+PrimInt, D: DecompressedTreeStore<IdC, IdD>> BreathFirstIterable<'a, IdC, IdD>
    for SD<'a, IdC, IdD, D>
{
    type It = Iter<'a, IdD>;

    fn iter_bf(&'a self) -> Iter<'a, IdD> {
        Iter {
            curr: 0, 
            len: self.map.len(),
            map: &self.map,
        }
    }
}



pub struct Iter<'a, IdD> {
    curr:usize,
    len:usize,
    map:&'a [IdD],
}

impl<'a, IdD:PrimInt> Iterator for Iter<'a,IdD> {
    type Item = IdD;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.len {
            None
        } else {
            let r = self.curr;
            self.curr = r + 1;
            Some(self.map[cast::<_,usize>(r).unwrap()])
        }
    }


}