use hyper_ast::{
    store::{defaults::NodeIdentifier, SimpleStores},
    types::{IterableChildren, Labeled},
    PrimInt,
};

use crate::types::TIdN;

pub enum Action<Idx> {
    Delete { path: Vec<Idx> },
}
type Idx = u16;
pub fn regen_query(
    ast: &mut SimpleStores<crate::types::TStore>,
    root: NodeIdentifier,
    actions: Vec<Action<Idx>>,
) -> NodeIdentifier {
    let mut md_cache = Default::default();
    let mut query_tree_gen = crate::legion::TsQueryTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: ast,
        md_cache: &mut md_cache,
    };
    #[derive(PartialEq, Debug)]
    enum ActionTree<Idx> {
        Delete,
        Children(Vec<(Idx, ActionTree<Idx>)>),
    }
    impl<Idx: std::cmp::PartialOrd + Clone + PrimInt> From<Vec<Action<Idx>>> for ActionTree<Idx> {
        fn from(value: Vec<Action<Idx>>) -> Self {
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
                }
            }
            for a in value {
                let a = match a {
                    Action::Delete { mut path } => {
                        path.reverse();
                        Action::Delete { path }
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
    ) -> NodeIdentifier {
        // dbg!(c);
        let (t, n) = ast
            .stores
            .node_store
            .resolve_with_type::<TIdN<NodeIdentifier>>(&c);
        dbg!(t);
        println!(
            "{}",
            hyper_ast::nodes::SyntaxSerializer::<_, _, false>::new(ast.stores, c) // hyper_ast::nodes::TextSerializer::new(ast.stores, c)
        );
        let l = n.try_get_label().copied();
        let mut cs: Vec<NodeIdentifier> = vec![];
        use hyper_ast::types::WithChildren;

        let cs_nodes = n
            .children()
            .unwrap()
            .iter_children()
            .copied()
            .collect::<Vec<_>>();
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
                a => cs.push(apply(ast, a, *cs_nodes.next().unwrap())),
            }
        }
        cs.extend(cs_nodes);
        ast.build_then_insert(c, t, l, cs)
    }
    assert_ne!(
        actions,
        ActionTree::Delete,
        "it makes no sense to remove the entire tree"
    );
    dbg!(&actions);
    apply(&mut query_tree_gen, actions, root)
}
