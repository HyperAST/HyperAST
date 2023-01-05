/// WIP to better express variability in edit scripts actions order,
/// action_vec only allow one order to apply actions.
/// Maybe it can be integrated in the existing script generator or it needs major changes.
/// Maybe an other algorithm similar to the Chawathe that better fits my needs exists in the literature.
use std::fmt::Debug;

use num_traits::PrimInt;
use crate::tree::tree_path::CompressedTreePath;

use super::{
    script_generator2::{Act, SimpleAction},
    Actions,
};

pub struct ActionsTree<A> {
    atomics: Vec<Node<A>>,
    composed: Vec<A>,
} // TODO use NS ? or a decompressed tree ?

pub struct Node<A> {
    action: A,
    children: Vec<Node<A>>,
}

impl<A: Debug> Actions for ActionsTree<A> {
    fn len(&self) -> usize {
        self.atomics.len()
    }
}

impl<L,Idx:PrimInt,I> ActionsTree<SimpleAction<L,CompressedTreePath<Idx>,I>>
{
    pub(crate) fn push(&mut self, action: SimpleAction<L,CompressedTreePath<Idx>,I>) {
        Self::push_aux(
            Node {
                action,
                children: vec![],
            },
            &mut self.atomics,
        );
    }
    fn push_aux(node: Node<SimpleAction<L,CompressedTreePath<Idx>,I>>, r: &mut Vec<Node<SimpleAction<L,CompressedTreePath<Idx>,I>>>) {
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

    fn push_node(&mut self, node: Node<SimpleAction<L,CompressedTreePath<Idx>,I>>) {
        Self::push_aux(node, &mut self.atomics);
    }

    pub(crate) fn new() -> Self {
        Self {
            atomics: Default::default(),
            composed: Default::default(),
        }
    }
}

// impl<L,Idx,I> ActionsTree<SimpleAction<L,Idx,I>>
// {
//     /// WARN should be more efficient than vec variant
//     /// and even more consise if made well
//     fn apply_actions<S: for<'b> NodeStoreMut<'b, T, &'b T>>(
//         &self,
//         r: T::TreeId,
//         s: &mut S,
//     ) -> <T as Stored>::TreeId {
//         todo!()
//     }
// }
