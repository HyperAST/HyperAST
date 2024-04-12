use hyper_ast::{
    position::{position_accessors::WithPreOrderOffsets, StructuralPosition, TreePath},
    store::{defaults::NodeIdentifier, nodes::legion::NodeStore, SimpleStores},
    types::{IterableChildren, Labeled, Typed, WithChildren},
};
use hyper_ast_gen_ts_cpp::legion::CppTreeGen;

const C0: &str = r#"int f() {
    return 21 + 21;
}"#;

const C1: &str = r#"int f() {
    return 21 - 21;
}"#;

const C2: &str = r#"int f() {
    int a = 21;
    return a + a;
}"#;

const C3: &str = r#"int f() {
    int a = 21;
    int b = 21;
    return a + b;
}"#;

const C4: &str = r#"int f() {
    int b = 21;
    return b + b;
}"#;

// Possible useful stuff:
// - test if subtree is conforming to ts query
//   - initially for each node in subtree, do the test
//     - terminate on wrong root type as fast as possible
//   - after that find different oracles
//     - type oracle
//     - structure hash oracle
//     - filtered structure hash oracle
//     - other convolutions (including prev hashes)
//     - labels through bags of words and defered bloom filters computing
// - edit distance between query and subtree
// - acceleration related to extracting entropy from basic constructs

#[test]
fn gen_match_simple() {
    let (code_store, code) = cpp_tree(C0.as_bytes());
    let pat = code;
    let pat = code_store.node_store.resolve(pat).child(&0).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&4).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&2).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&2).unwrap();
    println!();
    println!(
        "{}",
        hyper_ast::nodes::SyntaxSerializer::<_, _, true>::new(&code_store, pat)
    );
    println!();
    println!(
        "{}",
        hyper_ast::nodes::TextSerializer::new(&code_store, pat)
    );
    println!();
    let q0 =
        tsq_ser::TreeToQuery::<_, _, true, true, false, false>::new(&code_store, pat).to_string();
    println!("{}", q0);
    let (query_store, query) = crate::search::ts_query(q0.as_bytes());

    {
        let path = hyper_ast::position::StructuralPosition::new(code);
        let prepared_matcher =
            crate::search::PreparedMatcher::<_, hyper_ast_gen_ts_cpp::types::Type>::new(
                &query_store,
                query,
            );
        let mut matched = false;
        for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store, path, code) {
            if prepared_matcher.is_matching::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(
                &code_store,
                *e.node().unwrap(),
            ) {
                type T =
                    hyper_ast_gen_ts_cpp::types::TIdN<hyper_ast::store::defaults::NodeIdentifier>;
                let n = code_store
                    .node_store
                    .try_resolve_typed::<T>(e.node().unwrap())
                    .unwrap()
                    .0;
                let t = n.get_type();
                dbg!(t);
                matched = true;
            }
        }
        assert!(matched);
    }
    {
        let (code_store1, code1) = cpp_tree(C1.as_bytes());
        let path = hyper_ast::position::StructuralPosition::new(code1);
        let prepared_matcher =
            crate::search::PreparedMatcher::<_, hyper_ast_gen_ts_cpp::types::Type>::new(
                &query_store,
                query,
            );
        for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store1, path, code1) {
            if prepared_matcher.is_matching::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(
                &code_store1,
                *e.node().unwrap(),
            ) {
                type T =
                    hyper_ast_gen_ts_cpp::types::TIdN<hyper_ast::store::defaults::NodeIdentifier>;
                let n = code_store1
                    .node_store
                    .try_resolve_typed::<T>(e.node().unwrap())
                    .unwrap()
                    .0;
                let t = n.get_type();
                dbg!(t);
                panic!("should not match")
            }
        }
    }
}

#[test]
fn gen_match_named() {
    let (code_store, code) = cpp_tree(C2.as_bytes());
    println!(
        "\nThe code:\n{}",
        hyper_ast::nodes::TextSerializer::new(&code_store, code)
    );
    let pat = code;
    let pat = code_store.node_store.resolve(pat).child(&0).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&4).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&4).unwrap();
    let pat = code_store.node_store.resolve(pat).child(&2).unwrap();
    println!(
        "\nThe subtree:\n{}",
        hyper_ast::nodes::TextSerializer::new(&code_store, pat)
    );
    let q0 =
        tsq_ser::TreeToQuery::<_, _, true, true, false, false>::new(&code_store, pat).to_string();
    println!("\nThe corresponding query:\n{}", q0);
    let (mut query_store, query) = crate::search::ts_query(q0.as_bytes());

    let path = hyper_ast::position::StructuralPosition::new(code);
    let prepared_matcher =
        crate::search::PreparedMatcher::<_, hyper_ast_gen_ts_cpp::types::Type>::new(
            &query_store,
            query,
        );
    let mut matched = false;
    for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store, path, code) {
        if prepared_matcher.is_matching::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(
            &code_store,
            *e.node().unwrap(),
        ) {
            println!("\nThe matched subtree:");
            println!(
                "{}",
                hyper_ast::nodes::TextSerializer::new(&code_store, *e.node().unwrap())
            );
            matched = true;
        }
    }
    assert!(matched);
    // Negative case
    let (code_store1, code1) = cpp_tree(C3.as_bytes());
    println!(
        "\nThe second code:\n{}",
        hyper_ast::nodes::TextSerializer::new(&code_store1, code1)
    );
    let path = hyper_ast::position::StructuralPosition::new(code1);
    let prepared_matcher =
        crate::search::PreparedMatcher::<_, hyper_ast_gen_ts_cpp::types::Type>::new(
            &query_store,
            query,
        );
    for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store1, path, code1) {
        if prepared_matcher.is_matching::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(
            &code_store1,
            *e.node().unwrap(),
        ) {
            panic!("should not match")
        }
    }

    println!("(Good) The initial query does not match the second code");

    let (code_store2, code2) = cpp_tree(C4.as_bytes());
    println!(
        "\nThe third code (initial code with a rename):\n{}",
        hyper_ast::nodes::TextSerializer::new(&code_store2, code2)
    );

    let path = hyper_ast::position::StructuralPosition::new(code2);
    let prepared_matcher =
        crate::search::PreparedMatcher::<_, hyper_ast_gen_ts_cpp::types::Type>::new(
            &query_store,
            query,
        );
    for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store2, path, code2) {
        if prepared_matcher.is_matching::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(
            &code_store2,
            *e.node().unwrap(),
        ) {
            panic!("should not match")
        }
    }
    println!("(Expected) The initial query does not match the third code");
    println!("Lets make the initial query more generic");

    const M0: &str = r#"(predicate (identifier) @op (#eq? @op "eq") (parameters (capture (identifier) @id ) (string) @label ))"#;
    println!("");
    println!("\nThe meta query:\n{}", M0);
    let (query_store1, query1) = crate::search::ts_query(M0.as_bytes());

    let path = hyper_ast::position::StructuralPosition::new(query);
    let prepared_matcher =
        crate::search::PreparedMatcher::<_, crate::types::Type>::new(&query_store1, query1);
    let mut per_label = std::collections::HashMap::<
        String,
        Vec<(String, StructuralPosition<NodeIdentifier, u16>)>,
    >::default();
    for e in crate::iter::IterAll::new(&query_store, path, query) {
        if let Some(capts) = prepared_matcher
            .is_matching_and_capture::<_, crate::types::TIdN<NodeIdentifier>>(
                &query_store,
                *e.node().unwrap(),
            )
        {
            dbg!(&capts);
            let k = capts.get("label").unwrap().clone().label().unwrap();
            let v = capts.get("id").unwrap().clone().label().unwrap();
            let p = e;
            per_label.entry(k).or_insert(vec![]).push((v, p));
        }
    }
    assert_eq!(1, per_label.len());
    dbg!(&per_label);
    struct PerLabel(
        std::collections::HashMap<String, Vec<(String, StructuralPosition<NodeIdentifier, u16>)>>,
    );

    impl std::fmt::Display for PerLabel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for x in self.0.values() {
                if x.len() == 2 {
                    writeln!(f, "(#eq? @{} @{})", x[0].0, x[1].0)?;
                    dbg!(x[0].1.shared_ancestors(&x[1].1));
                } else if x.len() == 1 {
                    // noop
                } else {
                    todo!("need to do combination")
                }
            }
            Ok(())
        }
    }

    let query_bis = tsq_transform::regen_query(
        &mut query_store,
        query,
        vec![
            tsq_transform::Action::Delete {
                path: per_label.get("\"a\"").unwrap()[0]
                    .1
                    .iter_offsets()
                    .collect(),
            },
            tsq_transform::Action::Delete {
                path: per_label.get("\"a\"").unwrap()[1]
                    .1
                    .iter_offsets()
                    .collect(),
            },
        ],
    );

    println!(
        "\nAgain the third code (initial code with a rename):\n{}",
        hyper_ast::nodes::TextSerializer::new(&code_store2, code2)
    );

    let qbis = hyper_ast::nodes::TextSerializer::<_, _>::new(&query_store, query_bis).to_string();
    let qbis = format!("{} {}", qbis, PerLabel(per_label.clone()));
    println!("\nThe generified query:\n{}", qbis);
    let (query_store, query) = crate::search::ts_query(qbis.as_bytes());

    {
        let path = hyper_ast::position::StructuralPosition::new(code2);
        let prepared_matcher =
            crate::search::PreparedMatcher::<_, hyper_ast_gen_ts_cpp::types::Type>::new(
                &query_store,
                query,
            );
        let mut matched = false;
        for e in hyper_ast_gen_ts_cpp::iter::IterAll::new(&code_store2, path, code2) {
            if prepared_matcher.is_matching::<_, hyper_ast_gen_ts_cpp::types::TIdN<NodeIdentifier>>(
                &code_store2,
                *e.node().unwrap(),
            ) {
                println!("\nThe matched subtree with a rename:");
                println!(
                    "{}",
                    hyper_ast::nodes::TextSerializer::new(&code_store2, *e.node().unwrap())
                );
                matched = true;
            }
        }
        assert!(matched);
    }
}

fn cpp_tree(
    text: &[u8],
) -> (
    SimpleStores<hyper_ast_gen_ts_cpp::types::TStore>,
    legion::Entity,
) {
    use hyper_ast_gen_ts_cpp::types::TStore;
    let tree = match CppTreeGen::<TStore>::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => t,
    };
    // println!("{:#?}", tree.root_node().to_sexp());
    let mut stores: SimpleStores<TStore> = SimpleStores::default();
    let mut md_cache = Default::default();
    let mut tree_gen = CppTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let x = tree_gen.generate_file(b"", text, tree.walk()).local;
    let entity = x.compressed_node;
    // println!(
    //     "{}",
    //     hyper_ast::nodes::SyntaxSerializer::<_, _, true>::new(&stores, entity)
    // );
    (stores, entity)
}

mod tsq_ser {

    use hyper_ast::nodes::Space;
    use hyper_ast::types::HyperType;
    use hyper_ast::types::IterableChildren;
    use hyper_ast::types::{self, NodeId};
    use std::fmt::{Debug, Display, Write};

    pub struct TreeToQuery<
        'a,
        IdN,
        HAST,
        const TY: bool = true,
        const LABELS: bool = false,
        const IDS: bool = false,
        const SPC: bool = false,
    > {
        stores: &'a HAST,
        root: IdN,
    }

    impl<
            'store,
            IdN,
            HAST,
            const TY: bool,
            const LABELS: bool,
            const IDS: bool,
            const SPC: bool,
        > TreeToQuery<'store, IdN, HAST, TY, LABELS, IDS, SPC>
    {
        pub fn new(stores: &'store HAST, root: IdN) -> Self {
            Self { stores, root }
        }
    }

    impl<
            'store,
            IdN,
            HAST,
            const TY: bool,
            const LABELS: bool,
            const IDS: bool,
            const SPC: bool,
        > Display for TreeToQuery<'store, IdN, HAST, TY, LABELS, IDS, SPC>
    where
        IdN: NodeId<IdN = IdN> + Debug,
        HAST: types::NodeStore<IdN>,
        HAST: types::LabelStore<str>,
        HAST: types::TypeStore<HAST::R<'store>>,
        HAST::R<'store>: types::Labeled<Label = HAST::I> + types::WithChildren<TreeId = IdN>,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.serialize(&self.root, &mut 0, f).map(|_| ())
        }
    }

    impl<
            'store,
            IdN,
            HAST,
            const TY: bool,
            const LABELS: bool,
            const IDS: bool,
            const SPC: bool,
        > TreeToQuery<'store, IdN, HAST, TY, LABELS, IDS, SPC>
    where
        IdN: NodeId<IdN = IdN> + Debug,
        HAST: types::NodeStore<IdN>,
        HAST: types::LabelStore<str>,
        HAST: types::TypeStore<HAST::R<'store>>,
        HAST::R<'store>: types::Labeled<Label = HAST::I> + types::WithChildren<TreeId = IdN>,
    {
        // pub fn tree_syntax_with_ids(
        fn serialize(
            &self,
            id: &IdN,
            mut count: &mut usize,
            out: &mut std::fmt::Formatter<'_>,
        ) -> Result<(), std::fmt::Error> {
            const LABELS0: bool = false;
            use types::LabelStore;
            use types::Labeled;
            use types::NodeStore;
            use types::WithChildren;
            let b = NodeStore::resolve(self.stores, id);
            // let kind = (self.stores.type_store(), b);
            let kind = self.stores.resolve_type(&b);
            let label = b.try_get_label();
            let children = b.children();

            if kind.is_spaces() {
                if SPC {
                    let s = LabelStore::resolve(self.stores, &label.unwrap());
                    let b: String = Space::format_indentation(s.as_bytes())
                        .iter()
                        .map(|x| x.to_string())
                        .collect();
                    write!(out, "(")?;
                    if IDS { write!(out, "{:?}", id) } else { Ok(()) }.and_then(|x| {
                        if TY {
                            write!(out, "_",)
                        } else {
                            Ok(x)
                        }
                    })?;
                    if LABELS0 {
                        write!(out, " {:?}", Space::format_indentation(b.as_bytes()))?;
                    }
                    write!(out, ")")?;
                }
                return Ok(());
            }

            let w_kind = |out: &mut std::fmt::Formatter<'_>| {
                if IDS { write!(out, "{:?}", id) } else { Ok(()) }.and_then(|x| {
                    if TY {
                        write!(out, "{}", kind.to_string())
                    } else {
                        Ok(x)
                    }
                })
            };

            match (label, children) {
                (None, None) => {
                    // w_kind(out)?;
                    if IDS { write!(out, "{:?}", id) } else { Ok(()) }.and_then(|x| {
                        if TY {
                            write!(out, "\"{}\"", kind.to_string())
                        } else {
                            Ok(x)
                        }
                    })?;
                }
                (label, Some(children)) => {
                    if let Some(label) = label {
                        let s = LabelStore::resolve(self.stores, label);
                        if LABELS0 {
                            write!(out, " {:?}", Space::format_indentation(s.as_bytes()))?;
                        }
                    }
                    if !children.is_empty() {
                        let it = children.iter_children();
                        write!(out, "(")?;
                        w_kind(out)?;
                        for id in it {
                            let b = NodeStore::resolve(self.stores, id);
                            let kind = self.stores.resolve_type(&b);
                            if !kind.is_spaces() {
                                write!(out, " ")?;
                            }
                            self.serialize(&id, count, out)?;
                        }
                        write!(out, ")")?;
                    }
                }
                (Some(label), None) => {
                    write!(out, "(")?;
                    w_kind(out)?;
                    if LABELS0 {
                        let s = LabelStore::resolve(self.stores, label);
                        if s.len() > 20 {
                            write!(out, "='{}...'", &s[..20])?;
                        } else {
                            write!(out, "='{}'", s)?;
                        }
                    }
                    write!(out, ")")?;
                    if LABELS {
                        let s = LabelStore::resolve(self.stores, label);
                        write!(out, " @id{} (#eq? @id{} \"{}\")", count, count, s)?;
                        *count += 1;
                    }
                }
            }
            return Ok(());
        }
    }

    fn escape(src: &str) -> String {
        let mut escaped = String::with_capacity(src.len());
        let mut utf16_buf = [0u16; 2];
        for c in src.chars() {
            match c {
                ' ' => escaped += " ",
                '\x08' => escaped += "\\b",
                '\x0c' => escaped += "\\f",
                '\n' => escaped += "\\n",
                '\r' => escaped += "\\r",
                '\t' => escaped += "\\t",
                '"' => escaped += "\\\"",
                '\\' => escaped += "\\\\",
                c if c.is_ascii_graphic() => escaped.push(c),
                c => {
                    let encoded = c.encode_utf16(&mut utf16_buf);
                    for utf16 in encoded {
                        write!(&mut escaped, "\\u{:04X}", utf16).unwrap();
                    }
                }
            }
        }
        escaped
    }
}

mod tsq_transform {
    use hyper_ast::{
        store::{defaults::NodeIdentifier, SimpleStores},
        types::{IterableChildren, Labeled, Typed},
        PrimInt,
    };

    use crate::types::TIdN;

    pub enum Action<Idx> {
        Delete { path: Vec<Idx> },
    }
    type Idx = u16;
    pub(crate) fn regen_query(
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
            // dbg!(t);
            // println!(
            //     "{}",
            //     hyper_ast::nodes::SyntaxSerializer::<_, _, false>::new(ast.stores, c)
            // );
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
                match a {
                    ActionTree::Delete => {
                        while o > 0 {
                            cs.push(cs_nodes.next().unwrap().to_owned());
                            o -= 1;
                        }
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
        // dbg!(&actions);
        apply(&mut query_tree_gen, actions, root)
    }
}
