use hyper_ast::types::{NodeStore, WithChildren, IterableChildren};

pub mod bottom_up_matcher;
pub mod greedy_bottom_up_matcher;
pub mod greedy_subtree_matcher;
pub mod simple_bottom_up_matcher;

// lazy versions, that do not decompress directly subtrees
pub mod lazy_greedy_subtree_matcher;
pub mod lazy2_greedy_subtree_matcher;
pub mod lazy_bottom_up_matcher;
pub mod lazy_greedy_bottom_up_matcher;
pub mod lazy2_greedy_bottom_up_matcher;
// pub mod simple_bottom_up_matcher2;


pub fn size<'a, IdC: Clone, S>(store: &'a S, x: &IdC) -> usize
where
    S: 'a + NodeStore<IdC>,
    S::R<'a>: WithChildren<TreeId = IdC>,
{
    let node = store.resolve(&x);
    let cs = node.children().unwrap();
    let mut z = 0;
    for x in cs.iter_children() {
        z = z + size(store, x);
    }
    z + 1
}

/// todo specilize if T impl [WithStats]
pub fn height<'a, IdC: Clone, S>(store: &'a S, x: &IdC) -> usize
where
    S: 'a + NodeStore<IdC>,
    S::R<'a>: WithChildren<TreeId = IdC>,
{
    let node = store.resolve(&x);
    let cs = node.children();
    let cs = if let Some(cs) = cs {
        cs
    } else {
        return 0;
    };
    if cs.is_empty() {
        return 0;
    }
    let mut z = 0;
    for c in cs.iter_children() {
        z = z.max(height(store, c));
    }
    z + 1
}
