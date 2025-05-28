use hyperast::{
    PrimInt,
    store::{SimpleStores, defaults::NodeIdentifier},
    types::{Childrn, Labeled},
};

use crate::types::TIdN;

#[derive(Debug)]
pub enum Action<Idx, IdN = NodeIdentifier> {
    Delete { path: Vec<Idx> },
    Replace { path: Vec<Idx>, new: IdN },
}
type Idx = u16;
type IdN = NodeIdentifier;
pub fn regen_query(
    ast: &mut SimpleStores<crate::types::TStore>,
    root: NodeIdentifier,
    actions: Vec<Action<Idx, IdN>>,
) -> Option<NodeIdentifier> {
    let mut md_cache = Default::default();
    let mut query_tree_gen = crate::legion::TsQueryTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: ast,
        md_cache: &mut md_cache,
    };
    #[derive(PartialEq, Debug)]
    enum ActionTree<Idx, IdN = NodeIdentifier> {
        Delete,
        New(IdN),
        Children(Vec<(Idx, ActionTree<Idx, IdN>)>),
    }
    impl<Idx: std::cmp::PartialOrd + Clone + PrimInt> From<Vec<Action<Idx, IdN>>> for ActionTree<Idx> {
        fn from(value: Vec<Action<Idx, IdN>>) -> Self {
            let mut res = ActionTree::Children(vec![]);
            fn insert<Idx: std::cmp::PartialOrd + Clone + PrimInt>(
                s: &mut ActionTree<Idx>,
                a: Action<Idx>,
            ) {
                match a {
                    Action::Delete { path } if path.is_empty() => {
                        *s = ActionTree::Delete;
                    }
                    Action::Delete { mut path } => {
                        let ActionTree::Children(cs) = s else {
                            panic!()
                        };
                        // dbg!(&cs);
                        let p = path.pop().unwrap();
                        let mut low = 0;
                        let mut high = cs.len();
                        loop {
                            if low == high {
                                let mut c = ActionTree::Children(vec![]);
                                insert(&mut c, Action::Delete { path });
                                cs.insert(low, (p, c));
                                break;
                            }
                            let mid = low + (high - low) / 2;
                            if cs[mid].0 == p {
                                insert(&mut cs[mid].1, Action::Delete { path });
                                break;
                            } else if p < cs[mid].0 {
                                high = mid.saturating_sub(1);
                            } else {
                                low = mid + 1;
                            }
                        }
                    }
                    Action::Replace { mut path, new } => {
                        let ActionTree::Children(cs) = s else {
                            panic!()
                        };
                        // dbg!(&cs);
                        let p = path.pop().unwrap();
                        let mut low = 0;
                        let mut high = cs.len();
                        loop {
                            if low == high {
                                if path.is_empty() {
                                    // dbg!(cs.len());
                                    let c = ActionTree::New(new);
                                    cs.push((p, c));
                                    break;
                                }
                                let mut c = ActionTree::Children(vec![]);
                                insert(&mut c, Action::Replace { path, new });
                                cs.insert(low, (p, c));
                                break;
                            }
                            assert!(high >= low);
                            let mid = low + (high - low) / 2;
                            if cs[mid].0 == p {
                                if path.is_empty() {
                                    let c = ActionTree::New(new);
                                    cs[mid] = (p, c); // TODO check behavior
                                    break;
                                }
                                insert(&mut cs[mid].1, Action::Replace { path, new });
                                break;
                            } else if p < cs[mid].0 {
                                high = mid.saturating_sub(1);
                            } else {
                                low = mid + 1;
                            }
                        }
                    }
                }
            }
            for a in value {
                let a = match a {
                    Action::Delete { mut path } => {
                        path.reverse();
                        Action::Delete { path }
                    }
                    Action::Replace { mut path, new } => {
                        path.reverse();
                        Action::Replace { path, new }
                    }
                };
                insert(&mut res, a);
            }
            fn offsetify<Idx: PrimInt>(s: &mut ActionTree<Idx>) {
                let mut i = num::zero();
                if let ActionTree::Children(cs) = s {
                    for (j, c) in cs {
                        let tmp = i;
                        i = *j + num::one();
                        *j -= tmp;
                        offsetify(c);
                    }
                }
            }
            // dbg!(&res);
            offsetify(&mut res);
            res
        }
    }
    let actions = ActionTree::from(actions);
    fn apply(
        ast: &mut crate::legion::TsQueryTreeGen<'_, '_, crate::types::TStore>,
        a: ActionTree<Idx>,
        c: NodeIdentifier,
    ) -> Option<NodeIdentifier> {
        // dbg!(c);
        let (t, n) = ast
            .stores
            .node_store
            .resolve_with_type::<TIdN<NodeIdentifier>>(&c);
        // dbg!(t);
        // println!(
        //     "{}",
        //     hyperast::nodes::SyntaxSerializer::<_, _, false>::new(ast.stores, c) // hyperast::nodes::TextSerializer::new(ast.stores, c)
        // );
        let l = n.try_get_label().copied();
        let mut cs: Vec<NodeIdentifier> = vec![];
        use hyperast::types::WithChildren;

        let cs_nodes = n.children()?.iter_children().collect::<Vec<_>>();
        let mut cs_nodes = cs_nodes.iter();
        drop(n);

        let ActionTree::Children(child_actions) = a else {
            panic!()
        };
        for (mut o, a) in child_actions {
            // dbg!(&a);
            while o > 0 {
                cs.push(cs_nodes.next().unwrap().to_owned());
                o -= 1;
            }
            match a {
                ActionTree::Delete => {
                    cs_nodes.next().unwrap();
                }
                ActionTree::New(new) => {
                    cs_nodes.next().unwrap();
                    cs.push(new);
                }
                a => cs.push(apply(ast, a, *cs_nodes.next().unwrap())?),
            }
        }
        cs.extend(cs_nodes);
        Some(ast.build_then_insert(c, t, l, cs))
    }
    assert_ne!(
        actions,
        ActionTree::Delete,
        "it makes no sense to remove the entire tree"
    );
    // dbg!(&actions);
    apply(&mut query_tree_gen, actions, root)
}

pub fn try_regen_query(
    stores: &SimpleStores<crate::types::TStore>,
    root: NodeIdentifier,
    actions: Vec<Action<Idx, IdN>>,
) -> Option<NodeIdentifier> {
    #[derive(PartialEq, Debug)]
    enum ActionTree<Idx, IdN = NodeIdentifier> {
        Delete,
        New(IdN),
        Children(Vec<(Idx, ActionTree<Idx, IdN>)>),
    }
    impl<Idx: std::cmp::PartialOrd + Clone + PrimInt> From<Vec<Action<Idx, IdN>>> for ActionTree<Idx> {
        fn from(value: Vec<Action<Idx, IdN>>) -> Self {
            let mut res = ActionTree::Children(vec![]);
            fn insert<Idx: std::cmp::PartialOrd + Clone + PrimInt>(
                s: &mut ActionTree<Idx>,
                a: Action<Idx>,
            ) {
                match a {
                    Action::Delete { path } if path.is_empty() => {
                        *s = ActionTree::Delete;
                    }
                    Action::Delete { mut path } => {
                        let ActionTree::Children(cs) = s else {
                            panic!()
                        };
                        // dbg!(&cs);
                        let p = path.pop().unwrap();
                        let mut low = 0;
                        let mut high = cs.len();
                        loop {
                            if low == high {
                                let mut c = ActionTree::Children(vec![]);
                                insert(&mut c, Action::Delete { path });
                                cs.insert(low, (p, c));
                                break;
                            }
                            let mid = low + (high - low) / 2;
                            if cs[mid].0 == p {
                                insert(&mut cs[mid].1, Action::Delete { path });
                                break;
                            } else if p < cs[mid].0 {
                                high = mid.saturating_sub(1);
                            } else {
                                low = mid + 1;
                            }
                        }
                    }
                    Action::Replace { mut path, new } => {
                        let ActionTree::Children(cs) = s else {
                            panic!()
                        };
                        // dbg!(&cs);
                        let p = path.pop().unwrap();
                        let mut low = 0;
                        let mut high = cs.len();
                        loop {
                            if low == high {
                                if path.is_empty() {
                                    // dbg!(cs.len());
                                    let c = ActionTree::New(new);
                                    cs.push((p, c));
                                    break;
                                }
                                let mut c = ActionTree::Children(vec![]);
                                insert(&mut c, Action::Replace { path, new });
                                cs.insert(low, (p, c));
                                break;
                            }
                            assert!(high >= low);
                            let mid = low + (high - low) / 2;
                            if cs[mid].0 == p {
                                if path.is_empty() {
                                    let c = ActionTree::New(new);
                                    cs[mid] = (p, c); // TODO check behavior
                                    break;
                                }
                                insert(&mut cs[mid].1, Action::Replace { path, new });
                                break;
                            } else if p < cs[mid].0 {
                                high = mid.saturating_sub(1);
                            } else {
                                low = mid + 1;
                            }
                        }
                    }
                }
            }
            for a in value {
                let a = match a {
                    Action::Delete { mut path } => {
                        path.reverse();
                        Action::Delete { path }
                    }
                    Action::Replace { mut path, new } => {
                        path.reverse();
                        Action::Replace { path, new }
                    }
                };
                insert(&mut res, a);
            }
            fn offsetify<Idx: PrimInt>(s: &mut ActionTree<Idx>) {
                let mut i = num::zero();
                if let ActionTree::Children(cs) = s {
                    for (j, c) in cs {
                        let tmp = i;
                        i = *j + num::one();
                        *j -= tmp;
                        offsetify(c);
                    }
                }
            }
            // dbg!(&res);
            offsetify(&mut res);
            res
        }
    }
    let actions = ActionTree::from(actions);
    fn apply(
        stores: &SimpleStores<crate::types::TStore>,
        a: ActionTree<Idx>,
        c: NodeIdentifier,
    ) -> Option<NodeIdentifier> {
        // dbg!(c);
        let (t, n) = stores
            .node_store
            .resolve_with_type::<TIdN<NodeIdentifier>>(&c);
        // dbg!(t);
        // println!(
        //     "{}",
        //     hyperast::nodes::SyntaxSerializer::<_, _, false>::new(ast.stores, c) // hyperast::nodes::TextSerializer::new(ast.stores, c)
        // );
        let l = n.try_get_label().copied();
        let mut cs: Vec<NodeIdentifier> = vec![];
        use hyperast::types::WithChildren;

        let cs_nodes = n.children()?.iter_children().collect::<Vec<_>>();
        let mut cs_nodes = cs_nodes.iter();
        drop(n);

        let ActionTree::Children(child_actions) = a else {
            panic!()
        };
        for (mut o, a) in child_actions {
            // dbg!(&a);
            while o > 0 {
                cs.push(cs_nodes.next().unwrap().to_owned());
                o -= 1;
            }
            match a {
                ActionTree::Delete => {
                    cs_nodes.next().unwrap();
                }
                ActionTree::New(new) => {
                    cs_nodes.next().unwrap();
                    cs.push(new);
                }
                a => cs.push(apply(stores, a, *cs_nodes.next().unwrap())?),
            }
        }
        cs.extend(cs_nodes);
        crate::legion::TsQueryTreeGen::try_build(stores, c, t, l, cs)
    }
    assert_ne!(
        actions,
        ActionTree::Delete,
        "it makes no sense to remove the entire tree"
    );
    // dbg!(&actions);
    apply(stores, actions, root)
}
