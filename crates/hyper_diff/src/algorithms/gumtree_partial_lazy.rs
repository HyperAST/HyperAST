use super::{DiffResult, PreparedMappingDurations};
use crate::algorithms::MappingDurations;
use crate::{
    actions::script_generator2::{ScriptGenerator, SimpleAction},
    decompressed_tree_store::{
        bfs_wrapper::SimpleBfsMapper, lazy_post_order::LazyPostOrder, CompletePostOrder,
    },
    matchers::{
        heuristic::gt::{
            greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
        },
        mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
        Decompressible, Mapper,
    },
    tree::tree_path::CompressedTreePath,
};
use hyperast::types::{self, HyperAST, HyperASTShared, NodeId};
use std::{fmt::Debug, time::Instant};

type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;

pub fn diff<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> DiffResult<
    SimpleAction<HAST::Label, CompressedTreePath<HAST::Idx>, HAST::IdN>,
    Mapper<HAST, CDS<HAST>, CDS<HAST>, VecStore<u32>>,
    PreparedMappingDurations<2>,
>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Label: Clone + Copy + Eq + Debug,
    HAST::Idx: hyperast::PrimInt,
    <HAST::TS as types::TypeStore>::Ty: Eq + Debug,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let now = Instant::now();
    let mapper: (HAST, (DS<HAST>, DS<HAST>)) = hyperast.decompress_pair2(src, dst);
    let mut mapper_owned: Mapper<_, DS<HAST>, DS<HAST>, VecStore<_>> = mapper.into();
    // TODO find better way, at least make a shorthand
    let mapper = Mapper {
        hyperast,
        mapping: crate::matchers::Mapping {
            src_arena: mapper_owned.mapping.src_arena.as_mut(),
            dst_arena: mapper_owned.mapping.dst_arena.as_mut(),
            mappings: mapper_owned.mapping.mappings,
        },
    };
    let subtree_prepare_t = now.elapsed().as_secs_f64();
    let now = Instant::now();
    let mapper =
        LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
    let subtree_matcher_t = now.elapsed().as_secs_f64();
    let subtree_mappings_s = mapper.mappings().len();
    dbg!(&subtree_matcher_t, &subtree_mappings_s);
    let now = Instant::now();
    let mapper = Mapper {
        hyperast,
        mapping: crate::matchers::Mapping {
            mappings: mapper.mapping.mappings,
            src_arena: mapper_owned.mapping.src_arena,
            dst_arena: mapper_owned.mapping.dst_arena,
        },
    };
    let mapper = mapper.map(
        |src_arena| {
            Decompressible::<HAST, CompletePostOrder<HAST::IdN, _>>::from(
                src_arena.map(|x| x.complete(hyperast)),
            )
        },
        |dst_arena| {
            Decompressible::<HAST, CompletePostOrder<HAST::IdN, _>>::from(
                dst_arena.map(|x| x.complete(hyperast)),
            )
        },
    );
    // let mapper = mapper.map(
    //     |src_arena| CompletePostOrder::from(src_arena.complete(node_store)),
    //     |dst_arena| CompletePostOrder::from(dst_arena.complete(node_store)),
    // );
    let bottomup_prepare_t = now.elapsed().as_secs_f64();
    let now = Instant::now();
    let mapper = GreedyBottomUpMatcher::<_, _, _, VecStore<_>>::match_it(mapper);
    dbg!(&now.elapsed().as_secs_f64());
    let bottomup_matcher_t = now.elapsed().as_secs_f64();
    let bottomup_mappings_s = mapper.mappings().len();
    dbg!(&bottomup_matcher_t, &bottomup_mappings_s);
    let now = Instant::now();

    let mapper = mapper.map(
        |x| x,
        |dst_arena| SimpleBfsMapper::with_store(hyperast, dst_arena),
    );
    let prepare_gen_t = now.elapsed().as_secs_f64();
    let now = Instant::now();
    let actions = ScriptGenerator::compute_actions(mapper.hyperast, &mapper.mapping).ok();
    let gen_t = now.elapsed().as_secs_f64();
    dbg!(gen_t);
    let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
    DiffResult {
        mapping_durations: PreparedMappingDurations {
            mappings: MappingDurations([subtree_matcher_t, bottomup_matcher_t]),
            preparation: [subtree_prepare_t, bottomup_prepare_t],
        },
        mapper,
        actions,
        prepare_gen_t,
        gen_t,
    }
}
