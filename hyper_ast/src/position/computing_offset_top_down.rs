//! Gather most of the common behaviors used to compute positions in an HyperAST

use num::ToPrimitive;

use super::{Position, PrimInt, StructuralPosition, TreePath};
use std::path::PathBuf;

use crate::{
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::HashedNodeRef,
    },
    types::{
        AnyType, Children, HyperAST, HyperType, IterableChildren, LabelStore, Labeled, NodeStore,
        TypeStore, WithChildren, WithSerialization,
    },
};
/// precondition: root node do not contain a File node
/// TODO make whole thing more specific to a path in a tree
pub fn compute_range<'store, It, HAST>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (usize, usize, HAST::IdN)
where
    HAST: HyperAST<'store>,
    HAST::IdN: Copy,
    HAST::T: WithSerialization,
    It: Iterator,
    It::Item: PrimInt,
{
    let mut offset = 0;
    let mut x = root;
    for o in offsets {
        let b = stores.node_store().resolve(&x);
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            for y in 0..o.to_usize().unwrap() {
                let id = &cs[num::cast(y).unwrap()];
                let b = stores.node_store().resolve(id);

                offset += b.try_bytes_len().unwrap_or(0).to_usize().unwrap();
            }
            if let Some(a) = cs.get(num::cast(o).unwrap()) {
                x = *a;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    let b = stores.node_store().resolve(&x);

    let len = b.try_bytes_len().unwrap_or(0).to_usize().unwrap();
    (offset, offset + len, x)
}

pub fn compute_position<'store, HAST, It>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, HAST::IdN)
where
    It::Item: Clone,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren,
    It: Iterator<Item = HAST::Idx>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path = vec![];
    for o in &mut *offsets {
        // dbg!(offset);
        let b = stores.node_store().resolve(&x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());

        let t = stores.type_store().resolve_type(&b);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }

        if let Some(cs) = b.children() {
            let cs = cs.clone();
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = stores.node_store().resolve(y);
                    offset += b.try_bytes_len().unwrap().to_usize().unwrap();
                }
            } else {
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
            } else {
                break;
            }
        } else {
            break;
        }
    }
    assert!(offsets.next().is_none());
    let b = stores.node_store().resolve(&x);
    let t = stores.type_store().resolve_type(&b);
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
    (Position::new(file, offset, len), x)
}

pub fn compute_position_and_nodes<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, Vec<HAST::IdN>)
where
    It::Item: Clone,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path_ids = vec![];
    let mut path = vec![];
    for o in &mut *offsets {
        // dbg!(offset);
        let b = stores.node_store().resolve(&x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());

        let t = stores.type_store().resolve_type(&b);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }

        if let Some(cs) = b.children() {
            let cs = cs.clone();
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = stores.node_store().resolve(y);
                    offset += b.try_bytes_len().unwrap().to_usize().unwrap();
                }
            } else {
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
                path_ids.push(x.clone());
            } else {
                break;
            }
        } else {
            break;
        }
    }
    assert!(offsets.next().is_none());
    let b = stores.node_store().resolve(&x);
    let t = stores.type_store().resolve_type(&b);
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
    (Position::new(file, offset, len), path_ids)
}

impl StructuralPosition<NodeIdentifier, u16> {
    pub fn make_position<'store, HAST>(&self, stores: &'store HAST) -> Position
    where
        HAST: HyperAST<
            'store,
            T = HashedNodeRef<'store>,
            IdN = NodeIdentifier,
            Label = LabelIdentifier,
        >,
        HAST::TS: TypeStore<HashedNodeRef<'store>, Ty = AnyType>,
        // HAST::Types: 'static + TypeTrait + Debug,
    {
        self.check(stores).unwrap();
        // let parents = self.parents.iter().peekable();
        let mut from_file = false;
        // let mut len = 0;
        let x = *self.node().unwrap();
        let b = stores.node_store().resolve(&x);

        let t = stores.type_store().resolve_type(&b);
        // println!("t0:{:?}", t);
        let len = if let Some(y) = b.try_bytes_len() {
            if !t.is_file() {
                from_file = true;
            }
            y as usize
            // Some(x)
        } else {
            0
            // None
        };
        let mut offset = 0;
        let mut path = vec![];
        if self.parents.is_empty() {
            let file = PathBuf::from_iter(path.iter().rev());
            return Position::new(file, offset, len);
        }
        let mut i = self.parents.len() - 1;
        if from_file {
            while i > 0 {
                let p = self.parents[i - 1];
                let b = stores.node_store().resolve(&p);

                let t = stores.type_store().resolve_type(&b);
                // println!("t1:{:?}", t);
                let o = self.offsets[i];
                let c: usize = {
                    let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
                    v.iter()
                        .map(|x| {
                            let b = stores.node_store().resolve(x);

                            // println!("{:?}", b.get_type());
                            b.try_bytes_len().unwrap() as usize
                        })
                        .sum()
                };
                offset += c;
                if t.is_file() {
                    from_file = false;
                    i -= 1;
                    break;
                } else {
                    i -= 1;
                }
            }
        }
        if self.parents.is_empty() {
        } else if !from_file
        // || (i == 0 && stores.node_store().resolve(self.nodes[i]).get_type() == Type::Program)
        {
            loop {
                let n = self.parents[i];
                let b = stores.node_store().resolve(&n);
                // println!("t2:{:?}", b.get_type());
                let l = stores.label_store().resolve(b.get_label_unchecked());
                path.push(l);
                if i == 0 {
                    break;
                } else {
                    i -= 1;
                }
            }
        } else {
            let p = self.parents[i - 1];
            let b = stores.node_store().resolve(&p);
            let o = self.offsets[i];
            let c: usize = {
                let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
                v.iter()
                    .map(|x| {
                        let b = stores.node_store().resolve(x);

                        // println!("{:?}", b.get_type());
                        b.try_bytes_len().unwrap() as usize
                    })
                    .sum()
            };
            offset += c;
        }

        let file = PathBuf::from_iter(path.iter().rev());
        Position::new(file, offset, len)
    }
}
