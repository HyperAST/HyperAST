///! WIP this action collection aims to better express all possible order (of application) of actions from edit scripts,
///! as action_vec only allow one order to apply actions.
///! Maybe it can be integrated in the existing script generator or it needs major changes.
///! Maybe an other algorithm similar to the Chawathe that better fits my needs exists in the literature.
use hyperast::PrimInt;

use crate::tree::tree_path::{CompressedTreePath, TreePath};
use std::fmt::{Debug, Display};

use super::{
    script_generator2::{Act, ApplicablePath, SimpleAction},
    Actions,
};

#[derive(Debug)]
pub struct ActionsTree<A> {
    pub atomics: Vec<Node<A>>,
    composed: Vec<A>,
} // TODO use NS ? or a decompressed tree ?

#[derive(Debug)]
pub struct Node<A> {
    pub action: A,
    pub children: Vec<Node<A>>,
}
impl<A> Node<A> {
    fn size(&self) -> usize {
        1 + self.children.iter().map(|x| x.size()).sum::<usize>()
    }
}
pub trait NodeSummary {
    fn pretty(&self) -> impl Display + '_;
}

impl<A> Actions for ActionsTree<A> {
    fn len(&self) -> usize {
        self.atomics.len()
    }
}

impl<A> ActionsTree<A>
where
    Node<A>: NodeSummary,
{
    pub fn inspect(&self) -> impl Debug + '_ {
        struct Summary<'a, A>(&'a ActionsTree<A>);
        fn g<A>(a: &Node<A>, f: &mut std::fmt::Formatter<'_>, d: usize) -> std::fmt::Result
        where
            Node<A>: NodeSummary,
        {
            writeln!(f, "{}{}", " ".repeat(d * 2), NodeSummary::pretty(a))?;
            if d < 9 {
                for a in &a.children {
                    g(a, f, d + 1)?;
                }
            }
            Ok(())
        }
        impl<'a, A> Debug for Summary<'a, A>
        where
            Node<A>: NodeSummary,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                writeln!(f, "Changes:")?;
                for a in &self.0.atomics {
                    g(a, f, 1)?;
                }
                Ok(())
            }
        }
        Summary(self)
    }
}

impl<L: Clone, Idx: PrimInt, I: Clone> ActionsTree<SimpleAction<L, CompressedTreePath<Idx>, I>> {
    pub fn merge_mid(&mut self, action: &SimpleAction<L, CompressedTreePath<Idx>, I>) {
        Self::merge_mid_aux(action, &mut self.atomics);
    }
    fn merge_mid_aux(
        action: &SimpleAction<L, CompressedTreePath<Idx>, I>,
        r: &mut Vec<Node<SimpleAction<L, CompressedTreePath<Idx>, I>>>,
    ) {
        let mut i = 0;
        for x in r.iter_mut() {
            i += 1;
            use hyperast::position::position_accessors::SharedPath;
            dbg!(&x.action.path.mid, &action.path.mid);
            let sh = action.path.mid.shared_ancestors(&x.action.path.mid);
            dbg!(&sh);
            match sh {
                SharedPath::Exact(_) => panic!(),
                SharedPath::Remain(_) => panic!(),
                SharedPath::Submatch(_) => return Self::merge_mid_aux(action, &mut x.children),
                SharedPath::Different(_) => continue,
            }
        }
        let action = match &action.action {
            Act::Delete {} => todo!(),
            Act::Update { new } => todo!(),
            Act::Move { from } => todo!(),
            Act::MovUpd { from, new } => todo!(),
            Act::Insert { sub } => todo!(),
        };
        r.insert(
            i,
            Node {
                action,
                children: vec![],
            },
        )
    }
    pub fn merge_ori(&mut self, action: &SimpleAction<L, CompressedTreePath<Idx>, I>) {
        // dbg!(&action.path.ori);
        Self::merge_aux(
            action.path.ori.iter(),
            &mut self.atomics,
            |p| &mut p.ori,
            |path| match &action.action {
                Act::Delete {} => SimpleAction {
                    path: super::script_generator2::ApplicablePath {
                        ori: path.into(),
                        mid: action.path.mid.clone(),
                    },
                    action: Act::Delete {},
                },
                Act::Update { new } => SimpleAction {
                    path: super::script_generator2::ApplicablePath {
                        ori: path.into(),
                        mid: action.path.mid.clone(),
                    },
                    action: Act::Update { new: new.clone() },
                },
                Act::Move { from } => SimpleAction {
                    path: super::script_generator2::ApplicablePath {
                        ori: path.into(),
                        mid: action.path.mid.clone(),
                    },
                    action: Act::Move { from: from.clone() },
                },
                Act::MovUpd { from, new } => SimpleAction {
                    path: super::script_generator2::ApplicablePath {
                        ori: path.into(),
                        mid: action.path.mid.clone(),
                    },
                    action: Act::MovUpd {
                        from: from.clone(),
                        new: new.clone(),
                    },
                },
                Act::Insert { sub } => SimpleAction {
                    path: super::script_generator2::ApplicablePath {
                        ori: path.into(),
                        mid: action.path.mid.clone(),
                    },
                    action: Act::Insert { sub: sub.clone() },
                },
            },
        );
    }

    fn merge_aux(
        path: impl Iterator<Item = Idx> + Clone,
        mut r: &mut Vec<Node<SimpleAction<L, CompressedTreePath<Idx>, I>>>,
        f: impl Fn(&mut ApplicablePath<CompressedTreePath<Idx>>) -> &mut CompressedTreePath<Idx>,
        g: impl Fn(Vec<Idx>) -> SimpleAction<L, CompressedTreePath<Idx>, I>,
    ) {
        let mut path: Vec<Idx> = path.collect();
        'aaa: loop {
            let mut i = 0;
            loop {
                let Some(x) = r.get_mut(i) else { break };
                use hyperast::position::position_accessors::SharedPath;
                // dbg!(f(&mut x.action.path));
                let sh = crate::tree::tree_path::shared_ancestors(
                    path.iter().copied(),
                    f(&mut x.action.path).iter(),
                );
                // dbg!(&sh);
                match sh {
                    SharedPath::Exact(_) => panic!(),
                    SharedPath::Remain(_s) => {
                        dbg!(&path);
                        let action = g(path);
                        let mut tmp = std::mem::replace(
                            &mut r[i],
                            Node {
                                action,
                                children: vec![],
                            },
                        );
                        let p = f(&mut tmp.action.path);
                        *p = p.iter().skip(_s.len() - 1).collect::<Vec<_>>().into();
                        dbg!(&p);
                        r[i].children.push(tmp);
                        return;
                    }
                    SharedPath::Submatch(s) => {
                        r = &mut r[i].children;
                        path = path[s.len()..].to_vec();
                        // dbg!(&path);
                        continue 'aaa;
                    }
                    SharedPath::Different(_) => (),
                }
                i += 1;
            }
            let action = g(path);
            r.insert(
                i,
                Node {
                    action,
                    children: vec![],
                },
            );
            break;
            // break (r, vec![], i);
        }
    }

    // fn push_node(&mut self, node: Node<SimpleAction<L, CompressedTreePath<Idx>, I>>) {
    //     Self::push_aux(node, &mut self.atomics);
    // }

    pub fn new() -> Self {
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

#[cfg(test)]
mod tests {
    use crate::actions::{
        action_vec::ActionsVec,
        script_generator2::{Act, ApplicablePath},
    };

    use super::*;

    #[test]
    fn test_push() {
        let mut actions = ActionsVec::<SimpleAction<u16, CompressedTreePath<u16>, u32>>::default();
        actions.0.push(SimpleAction {
            path: ApplicablePath {
                ori: CompressedTreePath::from(vec![0, 1, 2]),
                mid: CompressedTreePath::from(vec![0, 1, 2, 3]),
            },
            action: crate::actions::script_generator2::Act::Delete {},
        });
        actions.0.push(SimpleAction {
            path: ApplicablePath {
                ori: CompressedTreePath::from(vec![0, 1, 2, 3]),
                mid: CompressedTreePath::from(vec![0, 1, 2, 3, 4]),
            },
            action: crate::actions::script_generator2::Act::Delete {},
        });
        actions.0.push(SimpleAction {
            path: ApplicablePath {
                ori: CompressedTreePath::from(vec![0, 1, 2, 3, 4]),
                mid: CompressedTreePath::from(vec![0, 1, 2, 3, 4]),
            },
            action: crate::actions::script_generator2::Act::Delete {},
        });
        actions.0.push(SimpleAction {
            path: ApplicablePath {
                ori: CompressedTreePath::from(vec![0, 1, 2, 3, 5]),
                mid: CompressedTreePath::from(vec![0, 1, 2, 3, 4]),
            },
            action: crate::actions::script_generator2::Act::Delete {},
        });
        let actions = actions;
        let mut a_tree = ActionsTree::new();
        for a in &actions.0 {
            if let Act::Delete { .. } = &a.action {
                a_tree.merge_ori(a);
            }
        }
        dbg!(&a_tree);
    }
}
