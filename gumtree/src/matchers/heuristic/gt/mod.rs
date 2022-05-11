use num_traits::PrimInt;

use crate::tree::tree::{NodeStore, WithChildren};

pub mod bottom_up_matcher;
pub mod greedy_bottom_up_matcher;
pub mod greedy_subtree_matcher;
pub mod simple_bottom_up_matcher;
pub mod simple_bottom_up_matcher2;

fn size<T: WithChildren, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(store: &S, x: &T::TreeId) -> usize
where
    T::TreeId: PrimInt,
{
    let cs = store.resolve(&x).get_children().to_owned();
    let mut z = 0;
    for x in &cs {
        z = z + size(store, x);
    }
    z + 1
}

/// todo specilize if T impl [WithStats]
fn height<T: WithChildren, S: for<'a> NodeStore<'a, T::TreeId, &'a T>>(store: &S, x: &T::TreeId) -> usize
where
    T::TreeId: PrimInt,
{
    let cs = store.resolve(&x).get_children().to_owned();
    if cs.is_empty() {
        return 0;
    }
    let mut z = 0;
    for c in &cs {
        z = z.max(height(store, c));
    }
    z + 1
}
