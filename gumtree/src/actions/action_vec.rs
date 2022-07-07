use std::fmt::Debug;

use num_traits::ToPrimitive;

use crate::tree::{
    tree::{Labeled, NodeStoreMut, Stored, Typed, WithChildren},
    tree_path::TreePath,
};

use super::{
    script_generator2::{Act, SimpleAction},
    Actions,
};

#[derive(Debug)]
pub struct ActionsVec<A: Debug>(Vec<A>);

impl<A: Debug> Actions for ActionsVec<A> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<A: Debug> ActionsVec<A> {
    pub fn iter(&self) -> impl Iterator<Item = &A> + '_ {
        self.0.iter()
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

pub trait ApplicableActions<
    'a,
    T: 'a + Stored + Typed + Labeled + WithChildren + std::cmp::PartialEq,
> where
    T::Label: Clone + Debug,
    T::ChildIdx: Clone + Debug,
    T::TreeId: Debug + Clone,
{
    /// WARN for now it is very inneficient because it completly apply actions every times,
    /// it would need a temporary structure.
    /// actions are applied in order, thus there is a sinple way of applying actions.
    /// might not have enough info to more correctly, action_tree could definetly be more flexible.
    // pub fn apply_actions<S: for<'b> NodeStoreMut<'b, <T as Stored>::TreeId, &'b T>>(
    fn apply_actions<
        S: for<'b> NodeStoreMut<'b, T, &'b T>,
        It: Iterator<Item = &'a SimpleAction<T>>,
    >(
        actions: It,
        mut r: T::TreeId,
        s: &mut S,
    ) -> <T as Stored>::TreeId {
        let mut roots = vec![r.clone()];
        for a in actions {
            Self::apply_action(a, &mut roots, s)
        }
        r
    }

    fn apply_action<S: for<'b> NodeStoreMut<'b, T, &'b T>>(
        a: &'a SimpleAction<T>,
        roots: &mut Vec<T::TreeId>,
        s: &mut S,
    ) {
        log::trace!("{:?}", a);
        let SimpleAction { path, action } = a;

        let from = match action {
            Act::Move { from } => Some(from),
            Act::MovUpd { from, new } => Some(from),
            _ => None,
        };

        let sub = if let Some(from) = from {
            // apply remove
            log::trace!("sub path {:?}", from.mid.iter().collect::<Vec<_>>());
            let mut path = from.mid.iter();
            let r = &mut roots[path.next().unwrap().to_usize().unwrap()];
            let mut x = r.clone();
            let mut parents = vec![];
            while let Some(p) = path.next() {
                let node = s.resolve(&x);
                let cs = node.get_children().to_vec();
                parents.push((x, p, cs.clone()));
                let i = p.to_usize().unwrap();
                if i < cs.len() {
                    x = cs[i].clone();
                } else {
                    assert!(path.next().is_none());
                    break;
                }
            }
            log::trace!("parents {:?}", parents);
            let (node, sub) = if let Some((x, i, cs)) = parents.pop() {
                let mut children = Vec::with_capacity(cs.len() - 1);
                children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
                let node = s.resolve(&x);
                let node = Self::build(node.get_type(), node.get_label().clone(), children);
                (s.get_or_insert(node), cs[i.to_usize().unwrap()].clone())
            } else {
                // let mut children = Vec::with_capacity(cs.len() - 1);
                // children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                // children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
                // let node = s.resolve(&x);
                // let node = Self::build(node.get_type(), node.get_label().clone(), children);
                // s.get_or_insert(node)
                (r.clone(), r.clone())
            };
            let mut node = node;
            for (x, i, cs) in parents.into_iter().rev() {
                let mut children = Vec::with_capacity(cs.len() - 1);
                children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                children.push(node.clone());
                children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
                let n = s.resolve(&x);
                let n = Self::build(n.get_type(), n.get_label().clone(), children);
                node = s.get_or_insert(n);
            }
            *r = node;
            Some(sub)
        } else {
            None
        };

        let mut parents = vec![];
        log::trace!("{:?}", path.mid.iter().collect::<Vec<_>>());
        let mut path = path.mid.iter();
        let fp = path.next().unwrap().to_usize().unwrap();
        let r = if roots.len() > fp {
            &mut roots[fp]
        } else if roots.len() == fp {
            roots.push(roots[fp - 1].clone());
            &mut roots[fp]
        } else {
            panic!()
        };
        let mut x = r.clone();
        while let Some(p) = path.next() {
            let node = s.resolve(&x);
            let cs = node.get_children().to_vec();
            parents.push((x, p, cs.clone()));
            let i = p.to_usize().unwrap();
            if i < cs.len() {
                x = cs[i].clone();
            } else {
                log::error!("{:?} > {:?}", i, cs.len());
                assert_eq!(path.next(), None);
                break;
            }
        }

        let node = match action {
            Act::Delete {} => {
                let (x, i, cs) = parents.pop().unwrap();
                let mut children = Vec::with_capacity(cs.len() - 1);
                children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
                let node = s.resolve(&x);
                let node = Self::build(node.get_type(), node.get_label().clone(), children);
                s.get_or_insert(node)
            }
            Act::Insert { sub } => {
                if let Some((x, i, cs)) = parents.pop() {
                    let mut children = Vec::with_capacity(cs.len());
                    children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                    let sub = {
                        let node = s.resolve(sub);
                        let node = Self::build(node.get_type(), node.get_label().clone(), vec![]);
                        s.get_or_insert(node)
                    };
                    children.push(sub);
                    if i.to_usize().unwrap() < cs.len() {
                        children.extend_from_slice(&cs[i.to_usize().unwrap()..]);
                    }
                    let node = s.resolve(&x);
                    let node = Self::build(node.get_type(), node.get_label().clone(), children);
                    s.get_or_insert(node)
                } else {
                    let sub = {
                        let node = s.resolve(sub);
                        let node = Self::build(node.get_type(), node.get_label().clone(), vec![]);
                        s.get_or_insert(node)
                    };
                    // *r = sub.clone();
                    sub
                }
            }
            Act::Update { new } => {
                if let Some((x, i, cs)) = parents.pop() {
                    let mut children = Vec::with_capacity(cs.len());
                    children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                    let sub = {
                        let x = cs[i.to_usize().unwrap()].clone();
                        let node = s.resolve(&x);
                        let node =
                            Self::build(node.get_type(), new.clone(), node.get_children().to_vec());
                        s.get_or_insert(node)
                    };
                    children.push(sub);
                    children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
                    let node = s.resolve(&x);
                    let node = Self::build(node.get_type(), node.get_label().clone(), children);
                    s.get_or_insert(node)
                } else {
                    let node = s.resolve(&r);
                    let cs = node.get_children().to_vec();
                    let mut children = Vec::with_capacity(cs.len());
                    children.extend_from_slice(&cs[..]);
                    let node = Self::build(node.get_type(), new.clone(), children);
                    s.get_or_insert(node)
                }
            }

            Act::Move { .. } => {
                // apply insert
                let (x, i, cs) = parents.pop().unwrap();
                let mut children = Vec::with_capacity(cs.len());
                children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                let sub = {
                    // let node = s.resolve(&sub.unwrap());
                    // let node = Self::build(node.get_type(), node.get_label().clone(), vec![]);
                    // s.get_or_insert(node)
                    sub.unwrap()
                };
                children.push(sub);
                if i.to_usize().unwrap() < cs.len() {
                    children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
                }
                let node = s.resolve(&x);
                let node = Self::build(node.get_type(), node.get_label().clone(), children);
                s.get_or_insert(node)
            }
            Act::MovUpd { new, .. } => {
                // apply insert
                let (x, i, cs) = parents.pop().unwrap();
                let mut children = Vec::with_capacity(cs.len());
                children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                let sub = {
                    // let node = s.resolve(&sub.unwrap());
                    // let node = Self::build(node.get_type(), node.get_label().clone(), vec![]);
                    // s.get_or_insert(node)
                    sub.unwrap()
                };
                children.push(sub);
                if i.to_usize().unwrap() < cs.len() {
                    children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
                }
                let node = s.resolve(&x);
                let node = Self::build(node.get_type(), new.clone(), children);
                s.get_or_insert(node)
            }
        };
        let mut node = node;
        for (x, i, cs) in parents.into_iter().rev() {
            let mut children = Vec::with_capacity(cs.len() - 1);
            children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
            children.push(node.clone());
            children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
            let n = s.resolve(&x);
            let n = Self::build(n.get_type(), n.get_label().clone(), children);
            node = s.get_or_insert(n);
        }
        *r = node;
    }

    fn build(t: T::Type, l: T::Label, cs: Vec<T::TreeId>) -> T
    where
        T: Stored + Labeled + Typed;
}
