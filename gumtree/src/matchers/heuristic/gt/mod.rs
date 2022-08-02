use num_traits::PrimInt;

use hyper_ast::types::{NodeStore, WithChildren};

pub mod bottom_up_matcher;
pub mod greedy_bottom_up_matcher;
pub mod greedy_subtree_matcher;
pub mod simple_bottom_up_matcher;
pub mod simple_bottom_up_matcher2;

fn size<'a, IdC: Clone, S>(store: &'a S, x: &IdC) -> usize
where
    S: 'a + NodeStore<IdC>,
    // for<'c> <<S as NodeStore2<IdC>>::R as GenericItem<'c>>::Item: WithChildren<TreeId = IdC>,
    S::R<'a>: WithChildren<TreeId = IdC>,
{
    let cs = store.resolve(&x).get_children().to_owned();
    let mut z = 0;
    for x in &cs {
        z = z + size(store, x);
    }
    z + 1
}

/// todo specilize if T impl [WithStats]
fn height<'a, IdC: Clone, S>(store: &'a S, x: &IdC) -> usize
where
    S: 'a + NodeStore<IdC>,
    // for<'c> <<S as NodeStore2<IdC>>::R as GenericItem<'c>>::Item: WithChildren<TreeId = IdC>,
    S::R<'a>: WithChildren<TreeId = IdC>,
{
    let node = store.resolve(&x);
    let cs = node.try_get_children();
    let cs = if let Some(cs) = cs {
        cs.to_owned()
    } else {
        return 0;
    };
    if cs.is_empty() {
        return 0;
    }
    let mut z = 0;
    for c in &cs {
        z = z.max(height(store, c));
    }
    z + 1
}
