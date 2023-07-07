use std::path::PathBuf;

use num::{one, zero, ToPrimitive};

use super::Position;
use super::PrimInt;
use crate::types::{
    self, Children, HyperAST, HyperType, IterableChildren, LabelStore, Labeled, NodeStore,
    TypeStore, WithChildren, WithSerialization,
};

pub fn compute_position_with_no_spaces<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, HAST::IdN, Vec<It::Item>)
where
    It::Item: Clone + PrimInt,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let (pos, mut path_ids, no_spaces) =
        compute_position_and_nodes_with_no_spaces(root, offsets, stores);
    (pos, path_ids.remove(path_ids.len() - 1), no_spaces)
}

pub fn path_with_spaces<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    no_spaces: &mut It,
    stores: &'store HAST,
) -> (Vec<It::Item>,)
where
    It::Item: Clone + PrimInt,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren<ChildIdx = It::Item>,
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

        let t = stores.type_store().resolve_type(&b);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }
        let mut with_s_idx = zero();
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            if !t.is_directory() {
                for y in cs.iter_children() {
                    let b = stores.node_store().resolve(y);
                    if !stores.type_store().resolve_type(&b).is_spaces() {
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
        }
    }
    if let Some(x) = no_spaces.next() {
        // assert!(no_spaces.next().is_none());
        dbg!(x);
        panic!()
    }
    let b = stores.node_store().resolve(&x);
    let t = stores.type_store().resolve_type(&b);
    if t.is_directory() || t.is_file() {
        let l = stores.label_store().resolve(b.get_label_unchecked());
        path.push(l);
    }
    path_ids.reverse();
    (with_spaces,)
}

pub fn global_pos_with_spaces<'store, T, NS, It: Iterator>(
    root: T::TreeId,
    // increasing order
    no_spaces: &mut It,
    node_store: &'store NS,
) -> (Vec<It::Item>,)
where
    It::Item: Clone + PrimInt,
    T::TreeId: Clone,
    NS: 'store + types::NodeStore<T::TreeId, R<'store> = T>,
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
pub fn compute_position_and_nodes_with_no_spaces<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, Vec<HAST::IdN>, Vec<It::Item>)
where
    It::Item: Clone + PrimInt,
    HAST::IdN: Clone,
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path_ids = vec![];
    let mut no_spaces = vec![];
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
        let mut no_s_idx = zero();
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = stores.node_store().resolve(y);
                    if !stores.type_store().resolve_type(&b).is_spaces() {
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
    (
        Position::new(file, offset, len),
        path_ids,
        no_spaces,
    )
}
