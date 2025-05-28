use super::MappingDurations;
use super::tr;
use super::{DiffResult, PreparedMappingDurations};
use crate::{
    actions::script_generator2::{ScriptGenerator, SimpleAction},
    decompressed_tree_store::{CompletePostOrder, bfs_wrapper::SimpleBfsMapper},
    matchers::{
        Decompressible, Mapper,
        heuristic::gt::{
            greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            greedy_subtree_matcher::GreedySubtreeMatcher,
        },
        mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
    },
    tree::tree_path::CompressedTreePath,
};
use hyperast::types::{self, HyperAST, HyperASTShared, NodeId};
use std::{fmt::Debug, time::Instant};

#[allow(type_alias_bounds)]
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
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let now = Instant::now();
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    if cfg!(debug_assertions) {
        check_oneshot_decompressed_against_lazy(hyperast, src, dst, &mapper);
    }
    let subtree_prepare_t = now.elapsed().as_secs_f64();
    tr!(subtree_prepare_t);

    let now = Instant::now();
    let mapper =
        GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
    let subtree_matcher_t = now.elapsed().as_secs_f64();
    let subtree_mappings_s = mapper.mappings().len();
    tr!(subtree_matcher_t, subtree_mappings_s);

    let bottomup_prepare_t = 0.; // nothing to prepare

    let now = Instant::now();
    let mapper = GreedyBottomUpMatcher::<_, _, _, _>::match_it(mapper);
    let bottomup_matcher_t = now.elapsed().as_secs_f64();
    let bottomup_mappings_s = mapper.mappings().len();
    tr!(bottomup_matcher_t, bottomup_mappings_s);
    let mapping_durations = PreparedMappingDurations {
        mappings: MappingDurations([subtree_matcher_t, bottomup_matcher_t]),
        preparation: [subtree_prepare_t, bottomup_prepare_t],
    };

    let now = Instant::now();
    let mapper = mapper.map(
        |x| x,
        // the dst side has to be traversed in bfs for chawathe
        |dst_arena| SimpleBfsMapper::with_store(hyperast, dst_arena),
    );
    let prepare_gen_t = now.elapsed().as_secs_f64();
    tr!(prepare_gen_t);
    let now = Instant::now();
    let actions = ScriptGenerator::compute_actions(hyperast, &mapper.mapping).ok();
    let gen_t = now.elapsed().as_secs_f64();
    tr!(gen_t);
    let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
    DiffResult {
        mapping_durations,
        mapper,
        actions,
        prepare_gen_t,
        gen_t,
    }
}

fn check_oneshot_decompressed_against_lazy<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &<HAST as HyperASTShared>::IdN,
    dst: &<HAST as HyperASTShared>::IdN,
    mapper: &Mapper<
        HAST,
        Decompressible<HAST, CompletePostOrder<<HAST as HyperASTShared>::IdN, u32>>,
        Decompressible<HAST, CompletePostOrder<<HAST as HyperASTShared>::IdN, u32>>,
        VecStore<u32>,
    >,
) where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let mapper = mapper.src_arena.decomp.deref();
    let mapper = mapper.deref();
    log::trace!(
        "naive.ids:\t{:?}",
        &mapper.iter().take(20).collect::<Vec<_>>()
    );
    log::trace!(
        "naive:\t{:?}",
        &mapper.llds.iter().take(20).collect::<Vec<_>>()
    );
    #[allow(type_alias_bounds)]
    type DS<HAST: HyperASTShared> = Decompressible<
        HAST,
        crate::decompressed_tree_store::lazy_post_order::LazyPostOrder<HAST::IdN, u32>,
    >;
    let _mapper: (HAST, (DS<HAST>, DS<HAST>)) = hyperast.decompress_pair(src, dst);
    let mut _mapper_owned: Mapper<_, DS<HAST>, DS<HAST>, VecStore<u32>> = _mapper.into();
    let _mapper = Mapper {
        hyperast,
        mapping: crate::matchers::Mapping {
            mappings: _mapper_owned.mapping.mappings,
            src_arena: _mapper_owned.mapping.src_arena,
            dst_arena: _mapper_owned.mapping.dst_arena,
        },
    };
    let _mapper = _mapper.map(
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
    use std::ops::Deref;
    let _mapper = _mapper.src_arena.decomp.deref();
    let _mapper = _mapper.deref();
    log::trace!(
        "lazy:\t{:?}",
        &_mapper.llds.iter().take(20).collect::<Vec<_>>()
    );
    log::trace!(
        "lazy.ids:\t{:?}",
        &_mapper.iter().take(20).collect::<Vec<_>>()
    );
    assert_eq!(_mapper.llds, mapper.llds);
}
