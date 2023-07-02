use num::ToPrimitive;

use crate::{
    store::{defaults::NodeIdentifier, nodes::HashedNodeRef},
    types::{AnyType, HyperAST, NodeStore, TypeStore, LabelStore, Labeled, WithChildren, WithSerialization, HyperType, Children},
};

use super::Position;

pub fn extract_file_postion<'store, HAST: HyperAST<'store>>(
    stores: &'store HAST,
    parents: &[HAST::IdN],
) -> Position {
    if parents.is_empty() {
        Position::default()
    } else {
        let p = &parents[parents.len() - 1];
        let b = stores.node_store().resolve(p);
        // println!("type {:?}", b.get_type());
        // if !b.has_label() {
        //     panic!("{:?} should have a label", b.get_type());
        // }
        let l = stores.label_store().resolve(b.get_label_unchecked());

        let mut r = extract_file_postion(stores, &parents[..parents.len() - 1]);
        r.inc_path(l);
        r
    }
}

pub fn extract_position<'store, HAST>(
    stores: &'store HAST,
    parents: &[HAST::IdN],
    offsets: &[usize],
) -> Position
where
    HAST: HyperAST<'store, IdN = NodeIdentifier, T = HashedNodeRef<'store>>,
    HAST::TS: TypeStore<HashedNodeRef<'store>, Ty = AnyType>,
{
    if parents.is_empty() {
        return Position::default();
    }
    let p = parents[parents.len() - 1];
    let o = offsets[offsets.len() - 1];

    let b = stores.node_store().resolve(&p);

    let c = {
        let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
        v.iter()
            .map(|x| {
                let b = stores.node_store().resolve(x);

                // println!("{:?}", b.get_type());
                b.try_bytes_len().unwrap() as usize
            })
            .sum()
    };
    if stores.type_store().resolve_type(&b).is_file() {
        let mut r = extract_file_postion(stores, parents);
        r.inc_offset(c);
        r
    } else {
        let mut r = extract_position(
            stores,
            &parents[..parents.len() - 1],
            &offsets[..offsets.len() - 1],
        );
        r.inc_offset(c);
        r
    }
}
