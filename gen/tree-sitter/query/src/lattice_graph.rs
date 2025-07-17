use std::ops::Deref;

use crate::code2query::QueryLattice;
use hashbrown::{HashMap, HashSet};
use hyperast::position::StructuralPosition;
use petgraph::graph::NodeIndex;

use petgraph::visit::{EdgeRef, IntoNodeReferences};

type IdN = hyperast::store::defaults::NodeIdentifier;
type Idx = u16;
type Lattice<'a> = &'a QueryLattice<&'a StructuralPosition<IdN, Idx>>;
// use crate::utils::Q;

pub fn make_lattice_graph<N, E: 'static>(
    lattice: Lattice,
    f: impl Fn(IdN) -> N,
    g: impl Fn(petgraph::graph::NodeIndex, petgraph::graph::NodeIndex, &'static str) -> E,
) -> petgraph::acyclic::Acyclic<petgraph::Graph<N, E>> {
    use petgraph::prelude::*;
    let mut graph: DiGraph<_, _> = Default::default();

    let mut q_map = HashMap::<IdN, NodeIndex>::with_capacity(lattice.queries.len());
    for i in 0..lattice.queries.len() {
        let k = lattice.queries[i].0;
        let t = f(k);
        let index = graph.add_node(t);
        let _ = q_map.insert(k, index);
    }
    let mut graph = petgraph::acyclic::Acyclic::try_from_graph(graph).unwrap();

    lattice
        .raw_rels
        .iter()
        .filter_map(|(k, v)| q_map.get(k).map(|source| (*source, v)))
        .flat_map(|(source, v)| v.iter().map(move |v| (source, v)))
        .for_each(|(source, v)| {
            v.each(
                |_, _| (),
                |kind, target| {
                    if let Some(&target) = q_map.get(target) {
                        let e = g(source, target, kind);
                        graph.try_add_edge(source, target, e).expect(kind);
                    }
                },
            );
        });
    graph
}

pub fn group_lattices<N: Clone, E: Clone + 'static>(
    graph: petgraph::acyclic::Acyclic<petgraph::Graph<N, E>>,
) -> Vec<petgraph::Graph<N, E, petgraph::Directed, petgraph::csr::DefaultIx>> {
    use petgraph::prelude::*;

    use petgraph::visit::NodeIndexable;
    let mut vertex_sets = petgraph::unionfind::UnionFind::new(graph.node_bound());
    for edge in graph.edge_references() {
        let (a, b) = (edge.source(), edge.target());

        // union the two vertices of the edge
        vertex_sets.union(graph.to_index(a), graph.to_index(b));
    }
    let labels = vertex_sets.into_labeling();
    // dbg!(&labels);
    let mut sorting: Vec<_> = (0..labels.len()).collect();
    sorting.sort_unstable_by_key(|i| labels[*i]);
    // dbg!(&sorting.iter().map(|i| labels[*i]).collect::<Vec<_>>());
    sorting.dedup_by_key(|i| labels[*i]);

    let scc = sorting
        .into_iter()
        .map(|i| labels[i])
        .map(|i| {
            labels
                .iter()
                .enumerate()
                .filter_map(|(l, j)| (i == *j).then_some(NodeIndex::new(l)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // let scc = kosaraju_scc(&graph);

    // dbg!(scc.len());
    // dbg!(
    //     scc.iter()
    //         .map(|x| x.len())
    //         .fold(HashMap::<usize, usize>::new(), |mut acc, x| {
    //             *acc.entry(x).or_default() += 1;
    //             acc
    //         })
    // );
    dbg!(scc.len());
    // panic!();

    let uninit = NodeIndex::new(usize::MAX);
    let mut map: Vec<NodeIndex> = vec![uninit; graph.node_count()];

    let mut singles = Graph::new();
    let mut graphs: Vec<_> = scc
        .into_iter()
        .filter_map(|x| {
            if x.len() == 1 {
                singles.add_node(graph.node_weight(x[0]).unwrap().clone());
                return None;
            }
            let mut g = Graph::<_, _>::with_capacity(x.len(), x.len());

            x.iter().rev().for_each(|x| {
                map[x.index()] = g.add_node(graph.node_weight(*x).unwrap().clone());
            });

            x.iter().for_each(|a| {
                graph.edges(*a).for_each(|b| {
                    assert_ne!(a, &uninit);
                    assert_eq!(b.source(), *a);
                    assert_ne!(b.target(), uninit);
                    g.add_edge(map[a.index()], map[b.target().index()], b.weight().clone());
                });
            });

            let graph = g;
            let graph = petgraph::acyclic::Acyclic::try_from_graph(graph).unwrap();

            let toposort = petgraph::algo::toposort(&graph, None).unwrap();

            let (intermediate, revmap) =
                petgraph::algo::tred::dag_to_toposorted_adjacency_list(&graph, &toposort);
            let (reduction, _closure) = petgraph::algo::tred::dag_transitive_reduction_closure::<
                _,
                NodeIndex,
            >(&intermediate);

            let mut graph = graph.into_inner();

            graph.retain_edges(|x, y| {
                if let Some((f, t)) = x.edge_endpoints(y) {
                    reduction.contains_edge(revmap[f.index()], revmap[t.index()])
                } else {
                    false
                }
            });
            let g = graph;
            // let graph = graph.into();
            // let g = egui_graphs::to_graph(&graph);
            Some(g)
        })
        .collect();

    graphs.push(singles);
    // graphs.push(egui_graphs::to_graph(&singles.into()));
    graphs
}

pub struct GroupedLattices<Q> {
    pub graphs: Vec<(LatticeStats, petgraph::Graph<Q, &'static str>)>,
}

impl<Q: Deref<Target = IdN> + Clone + From<IdN>> GroupedLattices<Q> {
    pub fn new(lattice: Lattice) -> Self {
        // preps
        let graph =
            make_lattice_graph(lattice, |query| query.into(), |_source, _target, kind| kind);
        let graphs = group_lattices(graph);
        let mut graphs: Vec<_> = graphs
            .into_iter()
            .map(|graph| (lattice_stats(lattice, &graph), graph))
            .collect();
        graphs.sort_by(|a, b| a.0.cmp(&b.0).reverse());
        GroupedLattices { graphs }
    }
}

pub struct LatticeStats {
    leaf_count: usize,
    node_count: usize,
    edge_count: usize,
    pub complete_tops: Vec<(NodeIndex, TopStats)>,
    pub incompletes_tops: Vec<(NodeIndex, TopStats)>,
    pub uniqs: hashbrown::HashSet<NodeIndex>,
}

impl Eq for LatticeStats {}
impl PartialEq for LatticeStats {
    fn eq(&self, other: &Self) -> bool {
        self.leaf_count.eq(&other.leaf_count)
            && self.node_count.eq(&other.node_count)
            && self.edge_count.eq(&other.edge_count)
    }
}

impl Ord for LatticeStats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl PartialOrd for LatticeStats {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.complete_tops
                .cmp(&other.complete_tops)
                .then(self.leaf_count.cmp(&other.leaf_count))
                .then(self.node_count.cmp(&other.node_count))
                .then(self.edge_count.cmp(&other.edge_count)),
        )
    }
}
impl std::fmt::Display for LatticeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | {} | {} | {}",
            self.node_count,
            self.edge_count,
            self.leaf_count,
            self.complete_tops.len() + self.incompletes_tops.len(),
            self.incompletes_tops.len(),
        )
    }
}

impl LatticeStats {
    pub fn header() -> &'static str {
        "nodes | edges | leafs | tops | incomplete tops"
    }
}

fn lattice_stats<Q: Deref<Target = IdN>>(
    lattice: Lattice,
    graph: &petgraph::Graph<Q, &'static str>,
) -> LatticeStats {
    let uniqs: hashbrown::HashSet<_> = graph
        .externals(petgraph::Direction::Outgoing)
        .flat_map(|x| {
            let mut uniqs = graph
                .edges_directed(x, petgraph::Direction::Incoming)
                .filter(|x| x.weight() == &"Uniqs")
                .map(|x| x.source())
                .peekable();
            if uniqs.peek().is_none() {
                uniqs.chain(vec![x].into_iter())
            } else {
                uniqs.chain(vec![].into_iter())
            }
        })
        .collect();

    let tops = graph.externals(petgraph::Direction::Incoming);

    let (mut incompletes, mut tops_ranked): (Vec<_>, Vec<_>) = tops
        .map(|node_id| {
            (
                node_id,
                top_stats(graph, &lattice.query_store, &uniqs, node_id),
            )
        })
        .partition(|x| x.1.incomplete);
    incompletes.sort_by(|a, b| a.1.cmp(&b.1).reverse());
    tops_ranked.sort_by(|a, b| a.1.cmp(&b.1).reverse());

    LatticeStats {
        leaf_count: graph
            .node_references()
            .flat_map(|(_, n)| lattice.raw_rels.get(&n.deref()).unwrap())
            .filter_map(|x| {
                let crate::code2query::TR::Init(x) = x else {
                    return None;
                };
                Some(x)
            })
            .count(),
        node_count: graph.node_count(),
        edge_count: graph.edge_count(),
        complete_tops: tops_ranked,
        incompletes_tops: incompletes,
        uniqs,
    }
}

pub struct TopStats {
    pub paths: usize,
    pub reachable_uniqs: hashbrown::HashSet<NodeIndex>,
    pub patt_stats: PatternStats,
    pub incomplete: bool,
}
impl Eq for TopStats {}
impl PartialEq for TopStats {
    fn eq(&self, other: &Self) -> bool {
        self.paths.eq(&other.paths)
            && self.incomplete.eq(&other.incomplete)
            && self.reachable_uniqs.len().eq(&other.reachable_uniqs.len())
            && self.patt_stats.eq(&other.patt_stats)
    }
}

impl Ord for TopStats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl PartialOrd for TopStats {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.paths
                .cmp(&other.paths)
                .then(self.incomplete.cmp(&other.incomplete))
                .then(self.reachable_uniqs.len().cmp(&other.reachable_uniqs.len()))
                .then(self.patt_stats.cmp(&other.patt_stats)),
        )
    }
}
impl std::fmt::Display for TopStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "* paths: {}", self.paths)?;
            writeln!(f, "* reachable uniqs: {}", self.reachable_uniqs.len())?;
            Ok(())
        } else {
            write!(
                f,
                "{} | {} | {} | {}",
                self.paths,
                self.reachable_uniqs.len(),
                self.patt_stats,
                self.incomplete
            )
        }
    }
}

impl TopStats {
    pub fn iter_covering_all<'a, It: Iterator<Item = (NodeIndex, &'a TopStats)>>(
        it: It,
        uniqs: &'a mut HashSet<NodeIndex>,
    ) -> impl Iterator<Item = (NodeIndex, &'a TopStats)> {
        struct Iter<'a, It> {
            inner: It,
            uniqs: &'a mut HashSet<NodeIndex>,
        }

        impl<'a, It> Iterator for Iter<'a, It>
        where
            It: Iterator<Item = (NodeIndex, &'a TopStats)>,
        {
            type Item = (NodeIndex, &'a TopStats);

            fn next(&mut self) -> Option<Self::Item> {
                if self.uniqs.is_empty() {
                    return None;
                }
                let (node_id, stats) = self.inner.next()?;
                let mut useful = false;
                for uniq in &stats.reachable_uniqs {
                    useful |= self.uniqs.remove(uniq);
                }
                if !useful {
                    return None;
                }
                Some((node_id, stats))
            }
        }
        Iter { inner: it, uniqs }
    }
}

fn top_stats<Q: Deref<Target = IdN>, E>(
    lattice: &petgraph::Graph<Q, E>,
    stores: &hyperast::store::SimpleStores<crate::types::TStore>,
    uniqs: &HashSet<NodeIndex>,
    node_id: NodeIndex,
) -> TopStats {
    let patt_id = **lattice.node_weight(node_id).unwrap();
    let patt_stats = pattern_stats(&stores, patt_id);

    let mut visit = vec![node_id];
    let mut paths = 0;
    let mut reachable_uniqs = hashbrown::HashSet::new();
    while let Some(node_id) = visit.pop() {
        if uniqs.contains(&node_id) {
            reachable_uniqs.insert(node_id);
        }
        for e in lattice.edges(node_id) {
            paths += 1;
            visit.push(e.target());
        }
    }

    let meta_simp = hyperast_tsquery::Query::new(
        r#"
        (named_node
            (identifier) (#EQ? "argument_list")
            (named_node
                (identifier) (#EQ? "identifier")
            ) .
            (predicate
                (identifier) (#EQ? "EQ")
                (parameters
                    (string) @label
                )
            ) @incomplete
        )
        (named_node
            (identifier) (#EQ? "field_access") .
            (named_node
                (identifier) (#EQ? "identifier")
            ) .
            (predicate
                (identifier) (#EQ? "EQ")
                (parameters
                    (string) @label
                )
            ) @incomplete
        )
        (named_node
            (identifier) (#EQ? "array_access") .
            (named_node
                (identifier) (#EQ? "identifier")
            ) .
            (predicate
                (identifier) (#EQ? "EQ")
                (parameters
                    (string) @label
                )
            ) @incomplete
        )
        "#,
        crate::language(),
    )
    .unwrap();
    let cid = meta_simp.capture_index_for_name("incomplete").unwrap();
    let m = crate::code2query::find_matches(&stores, patt_id, &meta_simp, cid);
    TopStats {
        paths,
        patt_stats,
        reachable_uniqs,
        incomplete: !m.is_empty(),
    }
}

pub struct PatternStats {
    byte_len: usize,
    node_count: usize,
    pred_count: usize,
    height: usize,
}
impl Eq for PatternStats {}
impl PartialEq for PatternStats {
    fn eq(&self, other: &Self) -> bool {
        self.byte_len.eq(&other.byte_len)
            && self.node_count.eq(&other.node_count)
            && self.pred_count.eq(&other.pred_count)
            && self.height.eq(&other.height)
    }
}

impl Ord for PatternStats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl PartialOrd for PatternStats {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.byte_len
                .cmp(&other.byte_len)
                .then(self.node_count.cmp(&other.node_count))
                .then(self.pred_count.cmp(&other.pred_count))
                .then(self.height.cmp(&other.height)),
        )
    }
}
impl std::fmt::Display for PatternStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "* node count: {}", self.node_count)?;
            writeln!(f, "* byte length: {}", self.byte_len)?;
            Ok(())
        } else {
            write!(
                f,
                "{} | {} | {} | {}",
                self.byte_len, self.node_count, self.pred_count, self.height
            )
        }
    }
}
impl PatternStats {
    pub fn header(&self) -> &'static str {
        "| bytes | size | preds | height |
|------|------|------|------|"
    }
}

pub fn pattern_stats(
    stores: &hyperast::store::SimpleStores<crate::types::TStore>,
    id: IdN,
) -> PatternStats {
    use hyperast::types::{WithSerialization, WithStats};
    let n = stores.node_store.resolve(id);
    PatternStats {
        byte_len: n.try_bytes_len().unwrap(),
        node_count: n.size(),
        pred_count: 0,
        height: n.height(),
    }
}
