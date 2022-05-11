use std::fmt::Debug;

use num_traits::ToPrimitive;

use crate::tree::{tree::{Labeled, NodeStore, Stored, WithChildren}, tree_path::TreePath};

use super::{script_generator2::{Act, SimpleAction}, Actions};

#[derive(Debug)]
pub struct ActionsVec<A: Debug>(Vec<A>);

impl<IdD: Debug> Actions for ActionsVec<IdD> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub trait TestActions<T: Stored + Labeled + WithChildren> {
    fn has_actions(&self, items: &[SimpleAction<T>]) -> bool;
}

impl<T: Stored + Labeled + WithChildren + std::cmp::PartialEq> TestActions<T>
    for ActionsVec<SimpleAction<T>>
where
    T::Label: Debug,
    T::TreeId: Debug,
{
    fn has_actions(&self, items: &[SimpleAction<T>]) -> bool {
        items.iter().all(|x| self.0.contains(x))
    }
}



impl<T: Stored + Labeled + WithChildren> ActionsVec<SimpleAction<T>>
where
    T::Label: Debug,
    T::TreeId: Debug,
{
    pub(crate) fn push(&mut self, action: SimpleAction<T>) {
        self.0.push(action)
    }

    pub(crate) fn new() -> Self {
        Self(Default::default())
    }
}



impl<T: Stored + Labeled + WithChildren + std::cmp::PartialEq> ActionsVec<SimpleAction<T>>
where
    T::Label: Debug,
    T::TreeId: Debug,
    T::TreeId: Clone,
{
    /// WARN for now it is very inneficient because it completly apply actions every times
    /// would need a temporary structure
    /// might not have enough info to apply correctly, action_tree should definetly be easier
    pub fn apply_actions<S: for<'b> NodeStore<'b, <T as Stored>::TreeId, &'b T>>(
        &self,
        r: T::TreeId,
        s: &mut S,
    ) -> <T as Stored>::TreeId {
        let mut r = r;
        for a in &self.0 {
            let SimpleAction { path, action } = a;
            let mut x = r;
            let mut parents = vec![];
            for p in path.iter() {
                let node = s.resolve(&x);
                let cs = node.get_children().clone();
                x = cs[p.to_usize().unwrap()].clone();
                parents.push((p, cs.clone()));
            }

            let node = match action {
                Act::Delete {} => {
                    let (i,cs) = parents.pop().unwrap();
                    let mut children = Vec::with_capacity(cs.len()-1);
                    children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                    children.extend_from_slice(&cs[i.to_usize().unwrap()+1..]);
                    // x = s.get_or_insert();
                    todo!()
                },
                Act::Update { new } => todo!(),
                Act::Move { from } => todo!(),
                Act::Insert { sub } => todo!(),
            };
            let mut node = node;
            for p in parents.into_iter().rev() {
                let (i,cs) = parents.pop().unwrap();
                let mut children = Vec::with_capacity(cs.len()-1);
                children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                children.push(x);
                children.extend_from_slice(&cs[i.to_usize().unwrap()+1..]);
            }
            r = x;
        }
        todo!()
    }
}