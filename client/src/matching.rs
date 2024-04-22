use std::fmt::Debug;

use hyper_ast::types::{self, HyperAST};
use hyper_diff::{decompressed_tree_store::ShallowDecompressedTreeStore, matchers::Mapper};

use hyper_diff::decompressed_tree_store::hidding_wrapper;
use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use hyper_diff::matchers::heuristic::gt::lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher;
pub use hyper_diff::matchers::heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
use hyper_diff::matchers::mapping_store::DefaultMultiMappingStore;
use hyper_diff::matchers::mapping_store::MappingStore;
use hyper_diff::matchers::mapping_store::VecStore;
use hyper_diff::matchers::Mapping;

// pub trait AAA {
//     fn aaa<B, A, R, F: Fn(&Self, &mut B, &mut A) -> R>(&self, f: F, b: &mut B, a: &mut A) -> R;
// }

// impl<'store, HAST> AAA for HAST
// where
//     HAST: HyperAST<'store>,
// {
//     fn aaa<B, A, R, F: Fn(&Self, &mut B, &mut A) -> R>(&self, f: F, b: &mut B, a: &mut A) -> R {
//         f(self, b, a)
//     }
// }

// fn t<'store, HAST: HyperAST<'store>>(
//     hyperast: &'store HAST,
//     src_arena: &mut LazyPostOrder<HAST::T, u32>,
//     dst_arena: &mut LazyPostOrder<HAST::T, u32>,
// ) -> DefaultMultiMappingStore<u32>
// where
//     HAST::IdN: Clone + Debug + Eq,
//     HAST::Label: Clone + Copy + Eq + Debug,
//     <HAST::T as types::Typed>::Type: Debug,
//     <HAST::T as types::WithChildren>::ChildIdx: Debug,
//     HAST::T: 'store + types::WithHashs + types::WithStats,
// {
//     hyperast.aaa(top_down, src_arena, dst_arena)
// }

pub fn top_down<'store, HAST: HyperAST<'store>>(
    hyperast: &'store HAST,
    src_arena: &mut LazyPostOrder<HAST::T, u32>,
    dst_arena: &mut LazyPostOrder<HAST::T, u32>,
) -> DefaultMultiMappingStore<u32>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::Label: Clone + Copy + Eq + Debug,
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    HAST::T: 'store + types::WithHashs + types::WithStats, 
{
    let mut mm: DefaultMultiMappingStore<_> = Default::default();
    mm.topit(src_arena.len(), dst_arena.len());
    Mapper::<_, _, _, VecStore<u32>>::compute_multimapping::<_, 1>(
        hyperast, src_arena, dst_arena, &mut mm,
    );
    mm
}

pub fn full<'store, HAST: HyperAST<'store>>(
    hyperast: &'store HAST,
    mapper: &mut Mapper<
        'store,
        HAST,
        &mut LazyPostOrder<HAST::T, u32>,
        &mut LazyPostOrder<HAST::T, u32>,
        VecStore<u32>,
    >,
) where
    HAST::IdN: Clone + Debug + Eq,
    HAST::Label: Clone + Copy + Eq + Debug,
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    HAST::T: 'store + types::WithHashs + types::WithStats,
{
    let mm = LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::compute_multi_mapping::<
        DefaultMultiMappingStore<_>,
    >(mapper);
    LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::filter_mappings(mapper, &mm);
    GreedyBottomUpMatcher::<_, _, _, _, VecStore<_>>::execute(mapper, hyperast.label_store());
}

pub fn bottom_up_hiding<'store, 'a, 'b, HAST: HyperAST<'store>>(
    hyperast: &'store HAST,
    mm: &hyper_diff::matchers::mapping_store::MultiVecStore<u32>,
    mapper: &'b mut Mapper<
        'store,
        HAST,
        &'a mut LazyPostOrder<HAST::T, u32>,
        &'a mut LazyPostOrder<HAST::T, u32>,
        VecStore<u32>,
    >,
)
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::Label: Clone + Copy + Eq + Debug,
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    HAST::T: 'store + types::WithHashs + types::WithStats,
{
    LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::filter_mappings(mapper, mm);
    use hidding_wrapper::*;

    // # hide matched subtrees
    // from right to left map unmatched nodes in a simple vec,
    let (map_src, rev_src) = hiding_map(
        &mapper.mapping.src_arena,
        &mapper.mapping.mappings.src_to_dst,
    );
    let (map_dst, rev_dst) = hiding_map(
        &mapper.mapping.dst_arena,
        &mapper.mapping.mappings.dst_to_src,
    );
    // a simple arithmetic op allow to still have nodes in post order where root() == len() - 1
    {
        let (src_arena, dst_arena, mappings) = hide(
            &mut *mapper.mapping.src_arena,
            &map_src,
            &rev_src,
            &mut *mapper.mapping.dst_arena,
            &map_dst,
            &rev_dst,
            &mut mapper.mapping.mappings,
        );
        // also wrap mappings (needed because bottom up matcher reads it)
        // then do the bottomup mapping (need another mapper)
        let mut mapper = Mapper {
            hyperast: mapper.hyperast,
            mapping: Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
        };
        GreedyBottomUpMatcher::<_, _, _, _, VecStore<_>>::execute(
            &mut mapper,
            hyperast.label_store(),
        );
    }
}


pub fn full2<'store, 'a, 'b, HAST: HyperAST<'store>>(
    hyperast: &'store HAST,
    mapper: &'b mut Mapper<
        'store,
        HAST,
        &'a mut LazyPostOrder<HAST::T, u32>,
        &'a mut LazyPostOrder<HAST::T, u32>,
        VecStore<u32>,
    >,
)
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::Label: Clone + Copy + Eq + Debug,
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    HAST::T: 'store + types::WithHashs + types::WithStats,
{
    let mut mm: DefaultMultiMappingStore<_> = Default::default();
    mm.topit(mapper.src_arena.len(), mapper.dst_arena.len());
    let now = std::time::Instant::now();
    Mapper::<HAST, _, _, VecStore<u32>>::compute_multimapping::<_, 1>(
        mapper.hyperast,
        &mut mapper.mapping.src_arena,
        &mut mapper.mapping.dst_arena,
        &mut mm,
    );
    let compute_multimapping_t = now.elapsed().as_secs_f64();
    dbg!(compute_multimapping_t);
    let now = std::time::Instant::now();
    bottom_up_hiding(hyperast, &mm, mapper);
    let bottom_up_hiding_t = now.elapsed().as_secs_f64();
    dbg!(bottom_up_hiding_t);

}

// There is, I believe a performance regression after having replaced the get_type by TStore::resolve_type
// TODO handle this perf regression
// [client/src/track.rs:676] src_oid = 0de92576100bba948cae854ebb9cd5a7a9502b43
// [client/src/track.rs:676] dst_oid = b84af67f4c88f3e3f7b61bf2035475f79fb3e62e
// 2024-04-15T13:18:54.600731Z  WARN request{method=GET uri=/track_at_path_with_changes/github/official-stockfish/Stockfish/1e6d21dbb6918a2d5f2f09730b0c30e3a4895d5c/0/33/2/125/4/32?upd=true&child=true&parent=false&exact_child=false&exact_parent=false&sim_child=false&sim_parent=false&meth=false&typ=false&top=false&file=false&pack=false&dependency=false&dependent=false&references=false&declaration=false version=HTTP/1.1}: client::track: done construction of [b84af67f4c88f3e3f7b61bf2035475f79fb3e62e, 7c8b7222f5eea024ab480abb2d9289fd1e42da9c, ec9038b7b4cb2701c3a3b8be56632e7f08e461ac, ab65d3fd0ecf340842408548bc7f3e6c28ad4c85] in official-stockfish    
// [client/src/track.rs:1037] &path_to_target = [
//     0,
//     38,
//     2,
//     132,
//     4,
//     38,
// ]
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:521] match_init_t = 1.125e-5
// [client/src/track/compute.rs:126]
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:242] &ambiguous_mappings.len() = 1484
// [client/src/track/compute.rs:414] bottom_up_hiding_t = 1.041259709
// [client/src/track.rs:643]
// [client/src/changes.rs:80]
// [client/src/changes.rs:87] src_arena.len() = 98757
// [client/src/changes.rs:88] dst_arena.len() = 113866
// [client/src/changes.rs:91] src_size = 98757
// [client/src/changes.rs:92] dst_size = 113866
// [client/src/changes.rs:101]
// [client/src/changes.rs:102] mapper.mapping.src_arena.len() = 98757
// [client/src/changes.rs:103] mapper.mapping.dst_arena.len() = 113866
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:521] match_init_t = 1.8125e-5
// [client/src/matching.rs:169] compute_multimapping_t = 295.667145542
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:242] &ambiguous_mappings.len() = 189189
// [client/src/matching.rs:173] bottom_up_hiding_t = 45.245505583
// [client/src/changes.rs:112]


// after type_eq
// [client/src/track.rs:676] src_oid = 0de92576100bba948cae854ebb9cd5a7a9502b43
// [client/src/track.rs:676] dst_oid = b84af67f4c88f3e3f7b61bf2035475f79fb3e62e
// 2024-04-16T08:08:25.213987Z  WARN request{method=GET uri=/track_at_path_with_changes/github/official-stockfish/Stockfish/5d1644ba696c0a4d81450f922d216bf6479d4929/0/33/2/130/8/26?upd=true&child=true&parent=false&exact_child=false&exact_parent=false&sim_child=false&sim_parent=false&meth=false&typ=false&top=false&file=false&pack=false&dependency=false&dependent=false&references=false&declaration=false version=HTTP/1.1}: client::track: done construction of [b84af67f4c88f3e3f7b61bf2035475f79fb3e62e, 7c8b7222f5eea024ab480abb2d9289fd1e42da9c, ec9038b7b4cb2701c3a3b8be56632e7f08e461ac, ab65d3fd0ecf340842408548bc7f3e6c28ad4c85] in official-stockfish    
// [client/src/track.rs:1037] &path_to_target = [
//     0,
//     38,
//     2,
//     132,
//     4,
//     38,
// ]
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:521] match_init_t = 2.0333e-5
// [client/src/track/compute.rs:126]
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:242] &ambiguous_mappings.len() = 1484
// [client/src/track/compute.rs:414] bottom_up_hiding_t = 1.269342625
// [client/src/track.rs:643]
// [client/src/changes.rs:80]
// [client/src/changes.rs:87] src_arena.len() = 98757
// [client/src/changes.rs:88] dst_arena.len() = 108457
// [client/src/changes.rs:91] src_size = 98757
// [client/src/changes.rs:92] dst_size = 108457
// [client/src/changes.rs:101]
// [client/src/changes.rs:102] mapper.mapping.src_arena.len() = 98757
// [client/src/changes.rs:103] mapper.mapping.dst_arena.len() = 108457
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:521] match_init_t = 2.5583e-5
// [client/src/matching.rs:169] compute_multimapping_t = 179.631986917
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:242] &ambiguous_mappings.len() = 212352
// [client/src/matching.rs:173] bottom_up_hiding_t = 63.056121792
// [client/src/changes.rs:111]

// Concrete type store, using directly cpp type
// [client/src/track.rs:676] src_oid = 0de92576100bba948cae854ebb9cd5a7a9502b43
// [client/src/track.rs:676] dst_oid = b84af67f4c88f3e3f7b61bf2035475f79fb3e62e
// 2024-04-16T10:51:13.038962Z  WARN request{method=GET uri=/track_at_path_with_changes/github/official-stockfish/Stockfish/5d1644ba696c0a4d81450f922d216bf6479d4929/0/33/2/130/8/26?upd=true&child=true&parent=false&exact_child=false&exact_parent=false&sim_child=false&sim_parent=false&meth=false&typ=false&top=false&file=false&pack=false&dependency=false&dependent=false&references=false&declaration=false version=HTTP/1.1}: client::track: done construction of [b84af67f4c88f3e3f7b61bf2035475f79fb3e62e, 7c8b7222f5eea024ab480abb2d9289fd1e42da9c, ec9038b7b4cb2701c3a3b8be56632e7f08e461ac, ab65d3fd0ecf340842408548bc7f3e6c28ad4c85] in official-stockfish    
// [client/src/track.rs:1037] &path_to_target = [
//     0,
//     38,
//     2,
//     132,
//     4,
//     38,
// ]
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:521] match_init_t = 2.9208e-5
// [client/src/track/compute.rs:126]
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:242] &ambiguous_mappings.len() = 1484
// [client/src/track/compute.rs:414] bottom_up_hiding_t = 1.2039654579999999
// [client/src/track.rs:643]
// [client/src/changes.rs:132]
// [client/src/changes.rs:139] src_arena.len() = 98757
// [client/src/changes.rs:140] dst_arena.len() = 108457
// [client/src/changes.rs:143] src_size = 98757
// [client/src/changes.rs:144] dst_size = 108457
// [client/src/changes.rs:153]
// [client/src/changes.rs:154] mapper.mapping.src_arena.len() = 98757
// [client/src/changes.rs:155] mapper.mapping.dst_arena.len() = 108457
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:521] match_init_t = 2.2417e-5
// [client/src/matching.rs:169] compute_multimapping_t = 161.409503917
// [hyper_diff/src/matchers/heuristic/gt/lazy2_greedy_subtree_matcher.rs:242] &ambiguous_mappings.len() = 212352
// [client/src/matching.rs:173] bottom_up_hiding_t = 61.255278167
// [client/src/changes.rs:163]

