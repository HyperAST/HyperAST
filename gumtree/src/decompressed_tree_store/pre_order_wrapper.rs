use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use num_traits::{cast, zero, PrimInt, ToPrimitive, Zero};

use crate::decompressed_tree_store::{
    DecompressedTreeStore, PostOrder, ShallowDecompressedTreeStore,
};
use hyper_ast::types::{NodeStore, Typed, WithChildren, WithSerialization};

use super::{CompletePostOrder, SimpleZsTree};

pub struct SimplePreOrderMapper<'a, IdC, IdD, D: DecompressedTreeStore<'a, IdC, IdD>> {
    pub map: Vec<IdD>,
    // fc: Vec<IdD>,
    rev: Vec<IdD>,
    pub(crate) depth: Vec<u16>,
    back: &'a D,
    phantom: PhantomData<*const IdC>,
}

impl<'a, IdC, IdD: Debug, D: Debug + DecompressedTreeStore<'a, IdC, IdD>> Debug
    for SimplePreOrderMapper<'a, IdC, IdD, D>
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

impl<'a, IdC, IdD: PrimInt, D: PostOrder<'a, IdC, IdD>> From<&'a D>
    for SimplePreOrderMapper<'a, IdC, IdD, D>
{
    fn from(x: &'a D) -> Self {
        let mut map: Vec<IdD> = vec![zero(); x.len()];
        let mut rev: Vec<IdD> = vec![zero(); x.len()];
        let mut depth = vec![0; x.len()];
        let mut o_id = x.root();
        map[0] = o_id;
        let mut fd = x.first_descendant(&o_id);
        let mut d_len = (o_id - fd).to_usize().unwrap();
        (0..d_len).for_each(|x| {
            depth[1 + x] = 1;
        });

        let mut n_id = 0;
        o_id = o_id - num_traits::one();

        loop {
            if d_len == 0 {
                while map[n_id] != zero() {
                    n_id = n_id - 1;
                }
            }
            n_id = n_id + d_len;
            fd = x.first_descendant(&o_id);
            d_len = (o_id - fd).to_usize().unwrap();

            n_id = n_id - d_len;

            let dep = depth[n_id] + 1;

            (n_id..n_id + d_len).for_each(|x| {
                depth[1 + x] = dep;
            });

            map[n_id] = o_id;
            rev[o_id.to_usize().unwrap()] = cast(n_id).unwrap();

            if o_id == num_traits::zero() {
                break;
            }
            o_id = o_id - num_traits::one();
            if d_len == 0 {
                n_id = n_id - 1;
            }
        }

        Self {
            map,
            // fc,
            rev,
            depth,
            back: x,
            phantom: PhantomData,
        }
    }
}

pub struct DisplaySimplePreOrderMapper<'a, 'b, IdC, IdD: PrimInt, S, D: PostOrder<'b, IdC, IdD>> {
    pub inner: &'b SimplePreOrderMapper<'b, IdC, IdD, D>,
    pub node_store: &'a S,
}

impl<'a, 'b, IdC: Clone + Debug + Eq, IdD: PrimInt, S> Display
    for DisplaySimplePreOrderMapper<'a, 'b, IdC, IdD, S, CompletePostOrder<IdC, IdD>>
where
    S: NodeStore<IdC>,
    S::R<'a>: WithChildren<TreeId = IdC> + Typed + WithSerialization,
    <S::R<'a> as Typed>::Type: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pos = 0;
        for i in 0..self.inner.map.len() {
            let o = self.inner.map[i];
            let ori = self.inner.back.original(&o);
            let node = self.node_store.resolve(&ori);
            let len = node.try_bytes_len().unwrap_or(0);
            writeln!(
                f,
                "{:>3}:{} {:?}    [{},{}]",
                o.to_usize().unwrap(),
                "  ".repeat(self.inner.depth[i].to_usize().unwrap()),
                node.get_type(),
                pos,
                pos + len,
            )?;
            if node.child_count().is_zero() {
                pos += len;
            }
        }
        Ok(())
    }
}
impl<'a, 'b, IdC: Clone, IdD: PrimInt, S> Display
    for DisplaySimplePreOrderMapper<'a, 'b, IdC, IdD, S, SimpleZsTree<IdC, IdD>>
where
    IdC: Debug + Eq,
    S: NodeStore<IdC>,
    S::R<'a>: WithChildren<TreeId = IdC> + Typed + WithSerialization, //<TreeId = IdC> + Typed,
    <S::R<'a> as Typed>::Type: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pos = 0;
        for i in 0..self.inner.map.len() {
            let o = self.inner.map[i];
            let ori = self.inner.back.original(&o);
            let node = self.node_store.resolve(&ori);
            let len = node.try_bytes_len().unwrap_or(0);
            writeln!(
                f,
                "{:>3}:{} {:?}    [{},{}]",
                o.to_usize().unwrap(),
                "  ".repeat(self.inner.depth[i].to_usize().unwrap()),
                node.get_type(),
                pos,
                pos + len,
            )?;
            if node.child_count().is_zero() {
                pos += len;
            }
        }
        Ok(())
    }
}

// pub struct Iter<'a, IdD> {
//     curr: usize,
//     len: usize,
//     map: &'a [IdD],
// }

// impl<'a, IdD: PrimInt> Iterator for Iter<'a, IdD> {
//     type Item = IdD;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.curr == self.len {
//             None
//         } else {
//             let r = self.curr;
//             self.curr = r + 1;
//             Some(self.map[cast::<_, usize>(r).unwrap()])
//         }
//     }
// }
