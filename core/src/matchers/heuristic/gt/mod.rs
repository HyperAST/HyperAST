#![feature(specialization)]

use num_traits::{cast, zero};

use crate::tree::tree::{NodeStore, Stored, Tree, WithChildren, WithStats};

pub mod bottom_up_matcher;
pub mod greedy_bottom_up_matcher;
pub mod greedy_subtree_matcher;
pub mod optimized_greedy_subtree_matcher;
pub mod simple_bottom_up_matcher;
pub mod simple_bottom_up_matcher2;

pub(crate) struct ComputeStruct {}
pub(crate) struct ComputeStruct2 {}

// pub(crate) trait ComputeTreeStats<T: Node> {
//     fn size<S: NodeStore<T>>(store: &S, x: T::TreeId) -> T::TreeId;
//     fn height<S: NodeStore<T>>(store: &S, x: T::TreeId) -> T::TreeId;
// }

// // impl<T: Node> ComputeTreeStats<T> for ComputeStruct {
// //     fn size<S: NodeStore<T>>(store: &S, x: T::TreeId) -> T::TreeId {
// //         1
// //     }

// //     fn height<S: NodeStore<T>>(store: &S, x: T::TreeId) -> T::TreeId {
// //         1
// //     }
// // }

// impl<IdC,T: WithStats> ComputeTreeStats<T> for ComputeStruct {
//     fn size<S: NodeStore<T>>(store: &S, x: IdC) -> usize {
//         cast(store.get_node_at_id(&x).descendants_count()).unwrap()
//     }

//     fn height<S: NodeStore<T>>(store: &S, x: IdC) -> usize {
//         cast(store.get_node_at_id(&x).height()).unwrap()
//     }
// }

// impl<T: WithChildren> ComputeTreeStats<T> for ComputeStruct2 {
//     fn size<S: NodeStore<T>>(store: &S, x: T::TreeId) -> T::TreeId {
//         let cs = store.get_node_at_id(&x).get_children().to_owned();

//         let mut z:T::TreeId = num_traits::zero();
//         for x in cs {
//             z = z + Self::size(store, x);
//         }
//         z + num_traits::one()
//     }

//     fn height<S: NodeStore<T>>(store: &S, x: T::TreeId) -> T::TreeId {
//         let cs = store.get_node_at_id(&x).get_children().to_owned();

//         let mut z:T::TreeId = num_traits::zero();
//         for x in cs {
//             z = z.max(Self::height(store, x));
//         }
//         z + num_traits::one()
//     }
// }

fn size<T: WithChildren, S: NodeStore<T>>(store: &S, x: &T::TreeId) -> usize {
    let cs = store.get_node_at_id(&x).get_children().to_owned();
    let mut z = 0;
    for x in &cs {
        z = z + size(store, x);
    }
    z + 1
}

/// todo specilize if T impl [WithStats]
fn height<T: WithChildren, S: NodeStore<T>>(store: &S, x: &T::TreeId) -> usize {
    let cs = store.get_node_at_id(&x).get_children().to_owned();
    if cs.is_empty() {
        return 0;
    }
    let mut z = 0;
    for c in &cs {
        z = z.max(height(store, c));
    }
    z + 1
}
