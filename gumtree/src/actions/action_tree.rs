use std::fmt::Debug;

use num_traits::ToPrimitive;

use crate::tree::{tree::{Labeled, NodeStore, Stored, WithChildren}, tree_path::TreePath};

use super::{script_generator2::{Act, SimpleAction}, Actions};

#[derive(Debug)]
pub struct ActionsTree<A: Debug>(Vec<A>);

impl<IdD: Debug> Actions for ActionsTree<IdD> {
    fn len(&self) -> usize {
        todo!()
    }
}

impl<T: Stored + Labeled + WithChildren + std::cmp::PartialEq> ActionsTree<SimpleAction<T>>
where
    T::Label: Debug,
    T::TreeId: Debug,
    T::TreeId: Clone,
{
    /// WARN should be more efficient than vec variant
    /// and even more consise if made well
    fn apply_actions<S: for<'b> NodeStore<'b, <T as Stored>::TreeId, &'b T>>(
        &self,
        r: T::TreeId,
        s: &mut S,
    ) -> <T as Stored>::TreeId {
        todo!()
    }
}