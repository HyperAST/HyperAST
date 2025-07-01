use crate::preprocess::parse_string_pair;
use criterion::black_box;
use hyper_diff::algorithms;
use hyper_diff::algorithms::{PreparedMappingDurations, ResultsSummary};
use hyperast::store::SimpleStores;
use hyperast::types;
use hyperast::types::{HyperAST, NodeId};
use std::fmt::{Debug, Display};

const DEFAULT_SIM_THRESHOLD: f64 = 0.5f64;

#[derive(Clone, Copy)]
pub enum Algorithm {
    Hybrid,
    LazyHybrid,
    Simple,
    Greedy,
    LazyGreedy,
}

impl AsRef<Algorithm> for Algorithm {
    fn as_ref(&self) -> &Algorithm {
        self
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Algorithm::Hybrid => write!(f, "hybrid"),
            Algorithm::LazyHybrid => write!(f, "lazy_hybrid"),
            Algorithm::Simple => write!(f, "simple"),
            Algorithm::Greedy => write!(f, "greedy"),
            Algorithm::LazyGreedy => write!(f, "lazy_greedy"),
        }
    }
}

pub fn run_diff(
    src: &str,
    dst: &str,
    algorithm: impl AsRef<Algorithm>,
    max_size: usize,
) -> ResultsSummary<PreparedMappingDurations<2>> {
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    let (src_tr, dst_tr) =
        parse_string_pair(&mut stores, &mut md_cache, black_box(src), black_box(dst));

    run_diff_trees(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
        algorithm,
        max_size,
    )
}

pub fn run_diff_trees<HAST: HyperAST + Copy>(
    stores: HAST,
    src_tr: &HAST::IdN,
    dst_tr: &HAST::IdN,
    algorithm: impl AsRef<Algorithm>,
    max_size: usize,
) -> ResultsSummary<PreparedMappingDurations<2>>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let stores = black_box(stores);
    let src_tr = black_box(src_tr);
    let dst_tr = black_box(dst_tr);
    use Algorithm::*;
    use algorithms::*;
    let diff_result = match algorithm.as_ref() {
        Hybrid => gumtree_hybrid::diff_hybrid(stores, src_tr, dst_tr, max_size),
        LazyHybrid => gumtree_hybrid_lazy::diff_hybrid_lazy(stores, src_tr, dst_tr, max_size),
        Simple => gumtree_simple::diff_simple(stores, src_tr, dst_tr),
        Greedy => gumtree::diff(stores, src_tr, dst_tr, max_size, DEFAULT_SIM_THRESHOLD),
        LazyGreedy => gumtree_lazy::diff(stores, src_tr, dst_tr, max_size, DEFAULT_SIM_THRESHOLD),
    };

    black_box(diff_result).summarize()
}
