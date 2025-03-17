use hyperast::types::{NodeId, NodeStore, WithChildren, Childrn};

pub mod bottom_up_matcher;
pub mod greedy_bottom_up_matcher;
pub mod greedy_subtree_matcher;
#[allow(unused)] // TODO finish simple bottom up matcher
pub mod simple_bottom_up_matcher;

// lazy versions, that do not decompress directly subtrees
pub mod lazy2_greedy_bottom_up_matcher;
pub mod lazy2_greedy_subtree_matcher;
pub mod lazy_bottom_up_matcher;
pub mod lazy_greedy_bottom_up_matcher;
pub mod lazy_greedy_subtree_matcher;
// pub mod simple_bottom_up_matcher2;

pub fn size<'a, IdC: Clone + NodeId<IdN = IdC>, S>(store: &'a S, x: &IdC) -> usize
where
    S: NodeStore<IdC>,
    for<'t> <S as hyperast::types::NLending<'t, IdC>>::N: WithChildren<TreeId = IdC>,
{
    let node = store.resolve(&x);
    let cs = node.children().unwrap();
    let mut z = 0;
    for x in cs.iter_children() {
        z = z + size(store, &x);
    }
    z + 1
}

/// TODO specilize with WithStats when specilization is stabilized
pub fn height<IdC: Clone + NodeId<IdN = IdC>, S>(store: &S, x: &IdC) -> usize
where
    S: NodeStore<IdC>,
    for<'t> <S as hyperast::types::NLending<'t, IdC>>::N: WithChildren<TreeId = IdC>,
{
    let node = store.resolve(&x);
    let cs = node.children();
    let Some(cs) = cs else {
        return 0;
    };
    if cs.is_empty() {
        return 0;
    }
    let mut z = 0;
    for c in cs.iter_children() {
        z = z.max(height(store, &c));
    }
    z + 1
}
