use super::Position;
use crate::types::{
    Children, Childrn, HyperAST, HyperType, LabelStore, Labeled, NodeStore, WithChildren,
    WithSerialization,
};

///
///
/// precondition: slices are read from right to left eg.
/// [dir, file, class, method, statement] ~> dir/file:20:40
pub fn extract_file_postion<HAST: HyperAST>(stores: &HAST, parents: &[HAST::IdN]) -> Position {
    if parents.is_empty() {
        Position::default()
    } else {
        let p = &parents[parents.len() - 1];
        let b = stores.node_store().resolve(p);
        let l = stores.label_store().resolve(b.get_label_unchecked());

        let mut r = extract_file_postion(stores, &parents[..parents.len() - 1]);
        r.inc_path(l);
        r
    }
}

///
///
/// precondition: slices are read from right to left eg.
/// [dir, file, class, method, statement] ~> dir/file:20:40
pub fn extract_position<'store, HAST>(
    stores: &'store HAST,
    parents: &[HAST::IdN],
    offsets: &[usize],
) -> Position
where
    HAST: HyperAST,
    for<'t> crate::types::LendT<'t, HAST>: WithSerialization,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
    HAST::IdN: Copy,
{
    if parents.is_empty() {
        return Position::default();
    }
    let p = parents[parents.len() - 1];
    let o = offsets[offsets.len() - 1];
    let o: HAST::Idx = num::cast(o).unwrap();

    let b = stores.resolve(&p);

    let c = b
        .children()
        .unwrap()
        .before(o - num::one())
        .iter_children()
        .map(|x| stores.resolve(&x).try_bytes_len().unwrap())
        .sum();
    if stores.resolve_type(&p).is_file() {
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

pub fn extract_file_postion_it_rec<'store, HAST, It>(
    stores: &'store HAST,
    mut nodes: It,
) -> Position
where
    HAST: HyperAST,
    It: Iterator<Item = HAST::IdN>,
{
    let Some(p) = nodes.next() else {
        return Position::default();
    };
    let b = stores.node_store().resolve(&p);
    let l = stores.label_store().resolve(b.get_label_unchecked());

    let mut r = extract_file_postion_it_rec(stores, nodes);
    r.inc_path(l);
    r
}

pub fn extract_position_it_rec<'store, HAST, It, It2>(stores: &'store HAST, mut it: It) -> Position
where
    HAST: HyperAST,
    for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithSerialization,
    It: Iterator<Item = (HAST::IdN, usize)> + Into<It2>,
    It2: Iterator<Item = HAST::IdN>,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
{
    let Some((p, o)) = it.next() else {
        return Position::default();
    };
    let o: HAST::Idx = num::cast(o).unwrap();

    let b = stores.node_store().resolve(&p);

    let c = b
        .children()
        .unwrap()
        .before(o - num::one())
        .iter_children()
        .map(|x| stores.resolve(&x).try_bytes_len().unwrap())
        .sum();
    if stores.resolve_type(&p).is_file() {
        let mut r = extract_file_postion_it_rec(stores, it.into());
        let l = stores.label_store().resolve(b.get_label_unchecked());
        r.inc_path(l);
        r.inc_offset(c);
        r
    } else {
        let mut r = extract_position_it_rec(stores, it);
        r.inc_offset(c);
        r
    }
}

pub fn extract_file_postion_it<'store, HAST, It>(stores: &'store HAST, nodes: It) -> Position
where
    HAST: HyperAST,
    It: Iterator<Item = HAST::IdN>,
{
    // TODO better to collect into a position ?
    let ls: Vec<_> = nodes
        .map(|p| {
            let b = stores.node_store().resolve(&p);
            stores.label_store().resolve(b.get_label_unchecked())
        })
        .collect();
    let mut r = Position::default();
    for l in ls.into_iter().rev() {
        r.inc_path(l);
    }
    r
}

pub fn extract_position_it<'store, HAST, It, It2>(stores: &'store HAST, mut it: It) -> Position
where
    HAST: HyperAST,
    for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithSerialization,
    It: Iterator<Item = (HAST::IdN, HAST::Idx)> + Into<It2>,
    It2: Iterator<Item = HAST::IdN>,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
{
    let x = Some(0);
    let Some(a) = x else { panic!() };
    println!("{}", a);

    let mut offset: usize = num::zero();
    while let Some((p, o)) = it.next() {
        let b = stores.resolve(&p);
        let c: usize = b
            .children()
            .unwrap()
            .before(o - num::one())
            .iter_children()
            .map(|x| stores.resolve(&x).try_bytes_len().unwrap())
            .sum();
        offset += c;
        if stores.resolve_type(&p).is_file() {
            let mut r = extract_file_postion_it(stores, it.into());
            let l = stores.label_store().resolve(b.get_label_unchecked());
            r.inc_path(l);
            r.inc_offset(offset);
            return r;
        }
    }
    let mut r = Position::default();
    r.inc_offset(offset);
    r
}
