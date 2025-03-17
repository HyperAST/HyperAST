use std::cmp::Ordering;
use std::collections::HashSet;

use hyperast::types::{HyperAST, WithSerialization};
use hyperast_gen_ts_tsquery::auto::tsq_ser_meta::Converter;

use hyperast::position::position_accessors::{SolvedPosition, WithPreOrderOffsets};
use hyperast::store::defaults::NodeIdentifier;
use hyperast_gen_ts_tsquery::auto::tsq_transform;
use hyperast_tsquery::{Cursor, Node as _};
use num::integer::Average;

type QStore = hyperast::store::SimpleStores<hyperast_gen_ts_tsquery::types::TStore>;
// TODO use a polyglote code store, snd prio c++ after java
type JStore = hyperast::store::SimpleStores<hyperast_gen_ts_java::types::TStore>;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum TR {
    // WARN different NodeIdentifier, this one is referring to the example
    Init(NodeIdentifier),
    RMs(NodeIdentifier),
    SimpEQ(NodeIdentifier),
}

impl PartialOrd for TR {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TR {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (TR::Init(_), TR::Init(_)) => Ordering::Equal,
            (TR::Init(_), _) => Ordering::Less,
            (TR::RMs(_), TR::RMs(_)) => Ordering::Equal,
            (TR::RMs(_), _) => Ordering::Less,
            (TR::SimpEQ(_), TR::SimpEQ(_)) => Ordering::Equal,
            _ => other.cmp(self).reverse(),
        }
    }
}

pub struct QueryLattice<E> {
    query_store: QStore,
    /// pairs of gen query and source
    relations: Vec<(NodeIdentifier, E)>,
    leaf_query_count: usize,
    raw_rels: std::collections::HashMap<NodeIdentifier, Vec<TR>>,
    queries: Vec<(NodeIdentifier, Vec<E>)>,
}

impl QueryLattice<NodeIdentifier> {
    fn generate_query0<'hast, HAST>(&mut self, stores: &'hast HAST, from: HAST::IdN) -> QueryId
    where
        HAST: hyperast::types::HyperAST,
        HAST::IdN: std::fmt::Debug,
        HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
    {
        use hyperast_gen_ts_tsquery::auto::tsq_ser::TreeToQuery;
        let query = TreeToQuery::<_, _, true>::with_pred(stores, from, |_| true);
        let query = query.to_string();
        // not necessary for simple generation of queries, but needed to be consistent with the rest
        let query =
            hyperast_gen_ts_tsquery::search::ts_query2(&mut self.query_store, query.as_bytes());
        QueryId(query)
    }

    fn generate_query(&mut self, stores: &JStore, from: NodeIdentifier) -> QueryId {
        QueryId(generate_query(&mut self.query_store, stores, from))
    }

    /// meta_simp: `(predicate (identifier) (#EQ? "EQ") (parameters (string) @label )) @pred (named_node (identifier) (#EQ "expression_statement")) @rm`
    fn generate_query2(
        &mut self,
        stores: &JStore,
        from: NodeIdentifier,
        meta_gen: &hyperast_tsquery::Query,
        meta_simp: &hyperast_tsquery::Query,
    ) -> QueryId {
        let q = generate_query2(&mut self.query_store, stores, from, meta_gen, meta_simp).unwrap();
        QueryId(q)
    }

    fn pp(&self, query: QueryId) -> String {
        hyperast::nodes::TextSerializer::<_, _>::new(&self.query_store, query.0).to_string()
    }
    pub fn with_examples(
        stores: &JStore,
        from: impl Iterator<Item = NodeIdentifier>,
        meta_gen: &hyperast_tsquery::Query,
        meta_simp: &hyperast_tsquery::Query,
    ) -> Self {
        let mut s = Self::new();
        macro_rules! sort {
            ($v:expr) => {
                $v.sort_by(|a, b| {
                    let tr = a.1.cmp(&b.1);
                    if tr != Ordering::Equal {
                        return tr;
                    }
                    let a_l = s
                        .query_store
                        .node_store()
                        .resolve(a.0)
                        .try_bytes_len()
                        .unwrap_or_default();
                    let b_l = s
                        .query_store
                        .node_store()
                        .resolve(b.0)
                        .try_bytes_len()
                        .unwrap_or_default();
                    let l_ord = a_l.cmp(&b_l);
                    l_ord
                });
            };
        }
        // TODO do not use u32 but the entry_raw and compute the hash on the fly
        let mut dedup = std::collections::HashMap::<u32, Vec<(NodeIdentifier, TR)>>::new();
        for from in from {
            // TODO add variant with immediates
            let Some((query, label_h)) =
                generate_query2_aux(&mut s.query_store, stores, from, meta_gen)
            else {
                continue;
            };
            // TODO generate multiple initial variants, by adding common meta rules
            s.relations.push((query, from));

            let v = &mut dedup.entry(label_h).or_default();
            let x = (query, TR::Init(from));
            if !v.contains(&x) {
                v.push(x);
                sort!(v);
            }
        }
        s.leaf_query_count = s.relations.len();
        let mut active: Vec<_> = dedup.keys().copied().collect();
        for _ in 0..2 {
            dbg!(dedup.len());
            let rms = std::mem::take(&mut active)
                .into_iter()
                // .filter_map(|x| {
                //     let query = x[0].0;
                //     let (new_q, label_h) =
                //         simp_rms(&mut s.query_store, query, meta_simp, |len| i * 10 % len)?;
                //     Some((label_h, (new_q, TR::RMs(query))))
                // })
                .flat_map(|x| {
                    let Some(x) = dedup.get(&x) else {
                        return vec![];
                    };
                    let query = x[0].0;
                    simp_rms2(&mut s.query_store, query, meta_simp)
                        .map(|(new_q, label_h)| (label_h, (new_q, TR::RMs(query))))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            dbg!(rms.len());
            for (label_h, x) in rms {
                let v = dedup.entry(label_h);
                let v = match v {
                    std::collections::hash_map::Entry::Occupied(x) => x.into_mut(),
                    std::collections::hash_map::Entry::Vacant(x) => {
                        active.push(label_h);
                        x.insert(vec![])
                    }
                };
                if !v.contains(&x) {
                    v.push(x);
                    sort!(v);
                } else {
                    dbg!()
                }
            }
            // TODO add pass to replace some symbols with a wildcard
        }
        dbg!(dedup.len());
        let simp_eq = dedup
            .values()
            .filter_map(|x| {
                let query = x[0].0;
                let (new_q, label_h) = simp_imm_eq(&mut s.query_store, query, meta_simp)?;
                Some((label_h, (new_q, TR::RMs(query))))
            })
            .collect::<Vec<_>>();
        for (label_h, x) in simp_eq {
            let v = dedup.entry(label_h).or_default();
            if !v.contains(&x) {
                v.push(x);
                sort!(v);
            }
        }
        dbg!(dedup.len());
        for v in dedup.values() {
            for v in v {
                let w = s.raw_rels.entry(v.0).or_default();
                w.push(v.1);
            }
        }
        fn extract<'a>(
            map: &'a std::collections::HashMap<NodeIdentifier, Vec<TR>>,
            v: impl Iterator<Item = &'a TR>,
            already: &mut HashSet<NodeIdentifier>,
            r: &mut Vec<NodeIdentifier>,
        ) {
            for v in v {
                match v {
                    TR::Init(x) => r.push(*x),
                    TR::RMs(v) | TR::SimpEQ(v) if !already.contains(v) => {
                        already.insert(*v);
                        extract(map, map.get(v).unwrap().iter(), already, r)
                    }
                    _ => (),
                }
            }
        }
        for v in dedup.values() {
            let mut already = HashSet::default();
            let mut r = vec![];
            extract(
                &s.raw_rels,
                v.into_iter().map(|x| &x.1),
                &mut already,
                &mut r,
            );
            s.queries.push((v[0].0, r));
        }
        s
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (String, &'a [NodeIdentifier])> {
        self.queries.iter().filter_map(|(q, e)| {
            let q = hyperast::nodes::TextSerializer::<_, _>::new(&self.query_store, *q)
                .to_string()
                .trim()
                .to_string();
            if q.is_empty() {
                return None;
            }
            Some((q, &e[..]))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueryId(
    // even if I use just a NodeIdentifier, queries are dedup early
    NodeIdentifier,
);

impl<E> QueryLattice<E> {
    pub fn new() -> Self {
        Self {
            query_store: hyperast_gen_ts_tsquery::search::ts_query_store(),
            relations: vec![],
            leaf_query_count: 0,
            queries: vec![],
            raw_rels: Default::default(),
        }
    }
}

impl<E> Default for QueryLattice<E> {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_query(
    query_store: &mut QStore,
    stores: &JStore,
    from: NodeIdentifier,
) -> NodeIdentifier {
    #[derive(Default)]
    struct Conv;
    impl Converter for Conv {
        type Ty = hyperast_gen_ts_java::types::Type;

        fn conv(s: &str) -> Option<Self::Ty> {
            hyperast_gen_ts_java::types::Type::from_str(s)
        }
    }
    let _query = hyperast_gen_ts_tsquery::auto::tsq_ser_meta::TreeToQuery::<
        _,
        hyperast_gen_ts_java::types::TIdN<_>,
        Conv,
    >::with_pred(stores, from, "(identifier) (type_identifier)");
    let _query = _query.to_string();
    let (mut query_store, query) = hyperast_gen_ts_tsquery::search::ts_query(_query.as_bytes());
    const M0: &str = r#"(predicate (identifier) @op (#eq? @op "eq") (parameters (capture (identifier) @id ) (string) @label ))"#;
    println!("");
    println!("\nThe meta query:\n{}", M0);
    let (query_store1, query1) = hyperast_gen_ts_tsquery::search::ts_query(M0.as_bytes());
    let path = hyperast::position::structural_pos::StructuralPosition::new(query);
    let prepared_matcher = hyperast_gen_ts_tsquery::search::PreparedMatcher::<
        hyperast_gen_ts_tsquery::types::Type,
    >::new(query_store1.with_ts(), query1);
    let mut per_label = std::collections::HashMap::<
        String,
        Vec<(
            String,
            hyperast::position::structural_pos::StructuralPosition<NodeIdentifier, u16>,
        )>,
    >::default();
    for e in hyperast_gen_ts_tsquery::iter::IterAll::new(&query_store, path, query) {
        if let Some(capts) = prepared_matcher
            .is_matching_and_capture::<_, hyperast_gen_ts_tsquery::types::TIdN<NodeIdentifier>>(
                &query_store,
                e.node(),
            )
        {
            dbg!(&capts);
            let l_l = prepared_matcher
                .captures
                .iter()
                .position(|x| &x.name == "label")
                .unwrap() as u32;
            let l_i = prepared_matcher
                .captures
                .iter()
                .position(|x| &x.name == "id")
                .unwrap() as u32;
            let k = capts
                .by_capture_id(l_l)
                .unwrap()
                .clone()
                .try_label(&query_store)
                .unwrap();
            let v = capts
                .by_capture_id(l_i)
                .unwrap()
                .clone()
                .try_label(&query_store)
                .unwrap();
            let p = e;
            per_label
                .entry(k.to_string())
                .or_insert(vec![])
                .push((v.to_string(), p));
        }
    }
    dbg!(&per_label);
    let query_bis = tsq_transform::regen_query(
        &mut query_store,
        query,
        per_label
            .values()
            .filter(|l| l.len() == 2)
            .flatten()
            .map(|x| tsq_transform::Action::Delete {
                path: x.1.iter_offsets().collect(),
            })
            .collect(),
    );
    let query =
        hyperast::nodes::TextSerializer::<_, _>::new(&query_store, query_bis.unwrap()).to_string();
    let query = format!("{} {}", query, PerLabel(per_label.clone()));
    println!("\nThe generified query:\n{}", query);
    let query = hyperast_gen_ts_tsquery::search::ts_query2(&mut query_store, query.as_bytes());
    query
}

fn generate_query2(
    query_store: &mut QStore,
    stores: &JStore,
    from: NodeIdentifier,
    meta_gen: &hyperast_tsquery::Query,
    meta_simp: &hyperast_tsquery::Query,
) -> Option<NodeIdentifier> {
    let query = generate_query2_aux(query_store, stores, from, meta_gen)?.0;

    let query = simp_rms(query_store, query, meta_simp, |len| {
        0.average_floor(&len).clamp(0, len)
    })?
    .0;

    let query = simp_imm_eq(query_store, query, meta_simp)?.0;
    Some(query)
}

type LableH = u32;

fn simp_imm_eq(
    query_store: &mut hyperast::store::SimpleStores<hyperast_gen_ts_tsquery::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> Option<(NodeIdentifier, LableH)> {
    // merge immediate predicates with identical labels
    let mut per_label = simp_search_imm_preds(&query_store, query, meta_simp);
    // dbg!(&per_label);
    let query = replace_preds_with_caps(query_store, query, &mut per_label);
    let query = hyperast::nodes::TextSerializer::<_, _>::new(&*query_store, query?).to_string();
    // TODO pretty print
    // NOTE hyperast::nodes::PrettyPrinter is not specifica enough to do a proper pp
    // print issue after removing something is due to having consecutive space nodes,
    // best would be to keep the one with a newline or else the first.

    let query = format!("{} {}", query, PerLabel(per_label));
    println!("\nThe generified query:\n{}", query);
    hyperast_gen_ts_tsquery::search::ts_query2_with_label_hash(query_store, query.as_bytes())
}

/// remove a matched thing from query
fn simp_rms(
    query_store: &mut hyperast::store::SimpleStores<hyperast_gen_ts_tsquery::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
    f: impl Fn(usize) -> usize,
) -> Option<(NodeIdentifier, LableH)> {
    let rms = simp_search_rm(&query_store, query, meta_simp);
    let query = apply_rms(query_store, query, &rms, f);
    let query =
        hyperast::nodes::TextSerializer::<_, _>::new(&*query_store, query.unwrap()).to_string();
    hyperast_gen_ts_tsquery::search::ts_query2_with_label_hash(query_store, query.as_bytes())
}

fn simp_rms2<'a>(
    query_store: &'a mut hyperast::store::SimpleStores<hyperast_gen_ts_tsquery::types::TStore>,
    query: NodeIdentifier,
    meta_simp: &'a hyperast_tsquery::Query,
) -> impl Iterator<Item = (NodeIdentifier, LableH)> + 'a {
    let rms = simp_search_rm(&query_store, query, meta_simp);
    rms.into_iter().filter_map(move |path| {
        let query = apply_rms_aux(query_store, query, &path);
        let query = hyperast::nodes::TextSerializer::<_, _>::new(&*query_store, query.unwrap())
            .to_string();
        hyperast_gen_ts_tsquery::search::ts_query2_with_label_hash(query_store, query.as_bytes())
    })
}

fn generate_query2_aux(
    query_store: &mut QStore,
    stores: &JStore,
    from: NodeIdentifier,
    meta_gen: &hyperast_tsquery::Query,
) -> Option<(NodeIdentifier, LableH)> {
    let query = hyperast_gen_ts_tsquery::auto::tsq_ser_meta2::TreeToQuery::<
        _,
        hyperast_gen_ts_java::types::TIdN<_>,
    >::new(stores, from, meta_gen.clone());
    let query = format!("{} @_root", query);
    hyperast_gen_ts_tsquery::search::ts_query2_with_label_hash(query_store, query.as_bytes())
}

fn simp_search_rm(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> Vec<P> {
    let mut result = vec![];
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
    let matches = meta_simp.matches(cursor);
    let Some(cid_p) = meta_simp.capture_index_for_name("rm") else {
        return vec![];
    };
    for capts in matches {
        let Some(p) = capts.nodes_for_capture_index(cid_p).next() else {
            continue;
        };
        let v = "";
        let p = p.pos.clone().offsets();
        result.push(p);
    }
    result
}

fn apply_rms(
    query_store: &mut Store,
    query: NodeIdentifier,
    rms: &Vec<P>,
    f: impl Fn(usize) -> usize,
) -> Option<NodeIdentifier> {
    let len = rms.len();
    if len == 0 {
        return Some(query);
    }
    let i = f(len);
    apply_rms_aux(query_store, query, &rms[i])
}

fn apply_rms_aux(
    query_store: &mut Store,
    query: NodeIdentifier,
    path: &Vec<u16>,
) -> Option<NodeIdentifier> {
    let mut path = path.clone();
    path.pop();
    path.reverse();
    // dbg!(&path);
    let action = tsq_transform::Action::Delete { path };
    let actions = vec![action];
    // dbg!(&actions);
    let query_bis = tsq_transform::regen_query(query_store, query, actions);
    query_bis
}
fn replace_preds_with_caps(
    query_store: &mut Store,
    query: NodeIdentifier,
    per_label: &mut std::collections::HashMap<Lab, Vec<(Cap, P)>>,
) -> Option<NodeIdentifier> {
    let mut count = 0;
    let actions = per_label
        .values_mut()
        .filter(|l| l.len() == 2)
        .flatten()
        .filter_map(|x| {
            assert!(x.0.is_empty()); // for now lets not consider other cases than imm. eq
            x.0 = format!("p{}", count);
            count += 1;
            let new = make_cap(query_store, &x.0);
            let mut path = x.1.clone();
            path.pop();
            path.reverse();
            // dbg!(&path);
            Some(tsq_transform::Action::Replace { path, new })
        })
        .collect();
    // dbg!(&actions);
    let query_bis = tsq_transform::regen_query(query_store, query, actions);
    query_bis
}

type Store = hyperast::store::SimpleStores<hyperast_gen_ts_tsquery::types::TStore>;

fn make_cap(query_store: &mut Store, name: &str) -> NodeIdentifier {
    let q = format!("_ @{}", name);
    let q = hyperast_gen_ts_tsquery::search::ts_query2(query_store, q.as_bytes());
    use hyperast::types::WithChildren;
    let q = query_store.node_store.resolve(q).child(&0).unwrap();
    let q = query_store.node_store.resolve(q).child(&2).unwrap();
    q
}

type P = Vec<u16>;
type Lab = String;
type Cap = String;

fn simp_search_imm_preds(
    query_store: &Store,
    query: NodeIdentifier,
    meta_simp: &hyperast_tsquery::Query,
) -> std::collections::HashMap<Lab, Vec<(Cap, P)>> {
    let mut per_label = std::collections::HashMap::default();
    let pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
    let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
    let mut matches = meta_simp.matches(cursor);
    let Some(cid_p) = meta_simp.capture_index_for_name("pred") else {
        return Default::default();
    };
    let Some(cid_l) = meta_simp.capture_index_for_name("label") else {
        return Default::default();
    };
    // let cid_i = meta_simp.capture_index_for_name("id").unwrap();
    loop {
        let Some(capts) = matches.next() else {
            break
        };
        let Some(p) = capts.nodes_for_capture_index(cid_p).next() else {
            continue;
        };
        let k = capts.nodes_for_capture_index(cid_l).next().unwrap();
        let k = k.text(matches.cursor().text_provider());
        // let v = capts.nodes_for_capture_index(cid_i).next().unwrap();
        // let v = v.text(());
        let v = "";
        let p = p.pos.clone().offsets();
        per_label
            .entry(k.to_string())
            .or_insert(vec![])
            .push((v.to_string(), p));
    }
    per_label
}

struct PerLabel<P>(std::collections::HashMap<String, Vec<(String, P)>>);
impl<P> std::fmt::Display for PerLabel<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in self.0.values() {
            if x.len() == 2 {
                writeln!(f, "(#eq? @{} @{})", x[0].0, x[1].0)?;
            } else if x.len() == 1 {
                // noop
            } else {
                todo!("need to do combination")
            }
        }
        Ok(())
    }
}
