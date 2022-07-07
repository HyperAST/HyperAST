/// WIP to better express variability in edit scripts actions order,
/// action_vec only allow one order to apply actions.
/// Maybe it can be integrated in the existing script generator or it needs major changes.
/// Maybe an other algorithm similar to the Chawathe that better fits my needs exists in the literature.
use std::fmt::Debug;

use crate::tree::tree::{Labeled, NodeStoreMut, Stored, WithChildren};

use super::{
    script_generator2::{Act, SimpleAction},
    Actions,
};

#[derive(Debug)]
pub struct ActionsTree<A: Debug> {
    atomics: Vec<Node<A>>,
    composed: Vec<A>,
} // TODO use NS ? or a decompressed tree ?

#[derive(Debug)]
pub struct Node<A: Debug> {
    action: A,
    children: Vec<Node<A>>,
}

impl<A: Debug> Actions for ActionsTree<A> {
    fn len(&self) -> usize {
        self.atomics.len()
    }
}

impl<T: Stored + Labeled + WithChildren> ActionsTree<SimpleAction<T>>
where
    T::Label: Debug,
    T::TreeId: Debug,
{
    pub(crate) fn push(&mut self, action: SimpleAction<T>) {
        Self::push_aux(
            Node {
                action,
                children: vec![],
            },
            &mut self.atomics,
        );
    }
    fn push_aux(node: Node<SimpleAction<T>>, r: &mut Vec<Node<SimpleAction<T>>>) {
        let mut i = 0;
        for x in r.iter_mut() {
            i += 1;
            use crate::tree::tree_path::SharedPath;
            match x.action.path.mid.shared_ancestors(&node.action.path.mid) {
                SharedPath::Exact(_) => panic!(),
                SharedPath::Remain(_) => panic!(),
                SharedPath::Submatch(_) => return Self::push_aux(node, &mut x.children),
                SharedPath::Different(_) => break,
            }
        }
        r.insert(i, node)
        // match &action.action {
        //     Act::Delete {} => todo!(),
        //     Act::Update { new } => todo!(),
        //     Act::Move { from } => todo!(),
        //     Act::Insert { sub } => todo!(),
        // }
    }

    fn push_node(&mut self, node: Node<SimpleAction<T>>) {
        Self::push_aux(node, &mut self.atomics);
    }

    pub(crate) fn new() -> Self {
        Self {
            atomics: Default::default(),
            composed: Default::default(),
        }
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
    fn apply_actions<S: for<'b> NodeStoreMut<'b, T, &'b T>>(
        &self,
        r: T::TreeId,
        s: &mut S,
    ) -> <T as Stored>::TreeId {
        todo!()
    }
}
