use std::cmp::max;
use std::fmt::Debug;
use criterion::black_box;
use hyper_diff::algorithms;
use hyper_diff::algorithms::{PreparedMappingDurations, ResultsSummary};
use hyperast::store::SimpleStores;
use hyperast::types;
use hyperast::types::{HyperAST, NodeId};
use crate::preprocess::parse_string_pair;

const DEFAULT_SIM_THRESHOLD: f64 = 0.5f64;

pub fn run_diff(src: &str, dst: &str, algorithm: &str, max_size: usize) -> ResultsSummary<PreparedMappingDurations<2>> {
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    let (src_tr, dst_tr) =
        parse_string_pair(&mut stores, &mut md_cache, black_box(src), black_box(dst));
    
    run_diff_trees(&stores, &src_tr.local.compressed_node, &dst_tr.local.compressed_node, algorithm, max_size)
}

pub fn run_diff_trees<HAST: HyperAST + Copy>(stores: HAST, src_tr: &HAST::IdN, dst_tr: &HAST::IdN, algorithm: &str, max_size: usize) -> ResultsSummary<PreparedMappingDurations<2>>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    
    let diff_result= match algorithm {
        "hybrid" => algorithms::gumtree_hybrid::diff_hybrid(
            black_box(stores),
            black_box(src_tr),
            black_box(dst_tr),
            max_size
        ),
        "lazy_hybrid" => algorithms::gumtree_hybrid_lazy::diff_hybrid_lazy(
            black_box(stores),
            black_box(src_tr),
            black_box(dst_tr),
            max_size
        ),
        "simple" => algorithms::gumtree_simple::diff_simple(
            black_box(stores),
            black_box(src_tr),
            black_box(dst_tr),
        ),
        "greedy" => algorithms::gumtree::diff(
            black_box(stores),
            black_box(src_tr),
            black_box(dst_tr),
            max_size,
            DEFAULT_SIM_THRESHOLD
        ),
        "lazy" => algorithms::gumtree_lazy::diff(
            black_box(stores),
            black_box(src_tr),
            black_box(dst_tr),
            max_size,
            DEFAULT_SIM_THRESHOLD
        ),
        _ => panic!("Unknown function")
    };

    black_box(diff_result.summarize())
}