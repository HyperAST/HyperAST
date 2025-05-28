use std::{
    fmt::Debug,
    fs::{self, File},
    path::Path,
    time::{Duration, Instant},
};

use hyper_diff::{
    decompressed_tree_store::{
        DecompressedWithSiblings, FullyDecompressedTreeStore, PostOrder,
        ShallowDecompressedTreeStore,
        complete_post_order::{DisplayCompletePostOrder, RecCachedProcessor},
        pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper},
    },
    matchers::{Mapper, mapping_store::MonoMappingStore},
    tree::tree_path::CompressedTreePath,
};
use hyper_diff::{matchers::mapping_store::VecStore, tree::tree_path::TreePath};
use hyperast::{
    PrimInt,
    position::Position,
    types::{self, HyperAST, HyperASTShared, HyperType, LabelStore, NodeStore, WithSerialization},
};
use num_traits::ToPrimitive;
use rayon::prelude::ParallelIterator;

use crate::diff_output;

pub enum CompResult {
    Success {
        timings: Vec<f64>,
        mappings: usize,
        actions: usize,
    },
    Error(),
    Failure {
        timings: Vec<f64>,
        mappings_hast: usize,
        mappings_other: usize,
        actions_hast: usize,
        actions_other: usize,
        stage: String,
        reason: String,
    },
}

/// "Approximate" comparison of mappings using bloom filters
///
/// Quick for large codebases .ie 2-3s on something like Spoon.
/// The collision factor is set to 0.001 , so in practice it will detect issues.
/// On the downside it does not help much finding the cause of the difference,
/// still a good way of narrowing the bug is to redo the diff and comparison on a subdirectory of the codebase.
pub struct CompressedBfPostProcess;

impl CompressedBfPostProcess {
    pub fn create(file: &Path) -> compressed_bf_post_process::PP0 {
        use byteorder::{BigEndian, ReadBytesExt};
        let mut cursor = std::io::Cursor::new(fs::read(&file).unwrap());
        assert_eq!(424242, cursor.read_u32::<BigEndian>().unwrap());
        compressed_bf_post_process::PP0 { file: cursor }
    }
}

pub mod compressed_bf_post_process {
    use hyper_diff::matchers::Mapper;
    use hyperast::types::{self, HyperAST};

    use super::*;
    pub struct PP0 {
        pub(super) file: std::io::Cursor<Vec<u8>>,
    }

    impl PP0 {
        pub fn counts(mut self) -> (compressed_bf_post_process::PP1, Counts) {
            use byteorder::{BigEndian, ReadBytesExt};
            let actions = self
                .file
                .read_i32::<BigEndian>()
                .unwrap()
                .to_isize()
                .unwrap();
            let src_heap = self
                .file
                .read_u64::<BigEndian>()
                .unwrap()
                .to_usize()
                .unwrap();
            let dst_heap = self
                .file
                .read_u64::<BigEndian>()
                .unwrap()
                .to_usize()
                .unwrap();
            let mappings = self
                .file
                .read_u32::<BigEndian>()
                .unwrap()
                .to_usize()
                .unwrap();
            (
                compressed_bf_post_process::PP1 { file: self.file },
                Counts {
                    mappings,
                    actions,
                    src_heap,
                    dst_heap,
                },
            )
        }
    }
    pub struct PP1 {
        pub(super) file: std::io::Cursor<Vec<u8>>,
    }

    impl PP1 {
        pub fn performances(mut self) -> (PP2, Vec<f64>) {
            use byteorder::{BigEndian, ReadBytesExt};
            let t_len = self.file.read_u32::<BigEndian>().unwrap() as usize;
            let timings: Vec<_> = (0..t_len)
                .map(|_| self.file.read_u64::<BigEndian>().unwrap())
                .map(|x| Duration::from_nanos(x as u64).as_secs_f64())
                .collect();
            (PP2 { file: self.file }, timings)
        }
    }

    pub struct PP2 {
        file: std::io::Cursor<Vec<u8>>,
    }

    impl PP2 {
        pub fn validity_mappings<'store: 'a, 'a, HAST, SD, DD>(
            self,
            mapper: &'a Mapper<HAST, DD, SD, VecStore<u32>>,
        ) -> ValidityRes<usize>
        where
            HAST: HyperAST + Copy,
            HAST::IdN: Clone + Debug + Eq,
            // for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::Tree,
            SD: ShallowDecompressedTreeStore<HAST, u32>
                + PostOrder<HAST, u32>
                + DecompressedWithSiblings<HAST, u32>,
            DD: ShallowDecompressedTreeStore<HAST, u32>
                + PostOrder<HAST, u32>
                + DecompressedWithSiblings<HAST, u32>,
        {
            let hyperast = mapper.hyperast;
            let mapping = &mapper.mapping;
            let src_arena = &mapping.src_arena;
            let dst_arena = &mapping.dst_arena;
            let src_tr = src_arena.original(&src_arena.root());
            let dst_tr = dst_arena.original(&dst_arena.root());
            self._validity_mappings(
                hyperast,
                src_arena,
                src_tr,
                dst_arena,
                dst_tr,
                &mapping.mappings,
            )
        }
        pub(crate) fn _validity_mappings<'store: 'a, 'a, HAST, SD, DD>(
            mut self,
            stores: HAST,
            src_arena: &'a SD,
            src_tr: HAST::IdN,
            dst_arena: &'a DD,
            dst_tr: HAST::IdN,
            mappings: &VecStore<u32>,
        ) -> ValidityRes<usize>
        where
            HAST: HyperAST + Copy,
            HAST::IdN: Clone + Debug + Eq,
            for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::Tree,
            SD: ShallowDecompressedTreeStore<HAST, u32>
                + PostOrder<HAST, u32>
                + DecompressedWithSiblings<HAST, u32>,
            DD: ShallowDecompressedTreeStore<HAST, u32>
                + PostOrder<HAST, u32>
                + DecompressedWithSiblings<HAST, u32>,
        {
            use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
            let bf_f = self.file.read_u32::<BigEndian>().unwrap() as usize;
            let bf_l = self.file.read_u32::<BigEndian>().unwrap() as usize;

            let now = Instant::now();

            let mut gt_bf = bitvec::bitvec![u64,bitvec::order::Lsb0; 0;bf_l];
            dbg!(gt_bf.len());
            // dbg!(gt_bf.as_raw_slice().len());
            // dbg!(gt_bf.as_raw_slice().len() * 8);
            // dbg!(gt_bf.as_raw_slice().len() * 8 * 8);
            self.file
                .read_u64_into::<LittleEndian>(gt_bf.as_raw_mut_slice())
                .unwrap();
            // dbg!(&gt_bf.as_raw_slice()[0].to_le_bytes());
            let gt_compressed_output_load_t = now.elapsed().as_secs_f64();
            dbg!(gt_compressed_output_load_t);
            let gt_bf = gt_bf;

            let mut hast_bf = bitvec::bitvec![u64,bitvec::order::Lsb0; 0;bf_l];

            type V<Idx> = Option<(md5::Digest, Idx)>;

            let with_p = |pos: V<HAST::Idx>, _ori: HAST::IdN| -> V<HAST::Idx> {
                Some((
                    if let Some((x, i)) = pos {
                        let mut c = md5::Context::new();
                        c.consume(x.0);
                        c.consume(i.to_u32().unwrap().to_be_bytes());
                        c.compute()
                    } else {
                        md5::Digest(Default::default())
                    },
                    num_traits::zero(),
                ))
            };
            let with_lsib = |pos: V<HAST::Idx>, _lsib: HAST::IdN| -> V<HAST::Idx> {
                let mut pos = pos.unwrap();
                pos.1 = pos.1 + num_traits::one();
                Some(pos)
            };

            // let mut formator_src =
            //     FormatCached::from((node_store, src_arena, src_tr, with_p, with_lsib));
            // let mut formator_dst =
            //     FormatCached::from((node_store, dst_arena, dst_tr, with_p, with_lsib));

            let mut formator_src = PathCached::from((src_arena, src_tr, with_p, with_lsib));
            let mut formator_dst = PathCached::from((dst_arena, dst_tr, with_p, with_lsib));

            let mut is_not_here = |x| {
                hast_bf.set((x % bf_l as u32) as usize, true);
                !gt_bf[(x % bf_l as u32) as usize]
            };
            use const_chunks::IteratorConstChunks;
            assert!(!is_not_here(0));
            assert!(!is_not_here(42));
            let mut g = |h: &[u8; 16]| {
                let [l1, l2, l3, l4] = h
                    .into_iter()
                    .cloned()
                    .const_chunks::<4>()
                    .map(|x| u32::from_be_bytes(x))
                    .const_chunks::<4>()
                    .next()
                    .unwrap();

                if bf_f >= 1
                    && is_not_here(u32::rotate_left(l1 ^ l2, 2) ^ u32::rotate_right(l3 ^ l4, 2))
                {
                    return Err(format!("1"));
                }
                if bf_f >= 2
                    && is_not_here(u32::rotate_left(l1 ^ l3, 2) ^ u32::rotate_right(l2 ^ l4, 2))
                {
                    return Err(format!("1"));
                }
                if bf_f >= 3
                    && is_not_here(u32::rotate_left(l1 ^ l4, 2) ^ u32::rotate_right(l2 ^ l3, 2))
                {
                    return Err(format!("3"));
                }
                if bf_f >= 4 && is_not_here(l1) {
                    return Err(format!("l1"));
                }
                if bf_f >= 5 && is_not_here(l2) {
                    return Err(format!("l2"));
                }
                if bf_f >= 6 && is_not_here(l3) {
                    return Err(format!("l3"));
                }
                if bf_f >= 7 && is_not_here(l4) {
                    return Err(format!("l4"));
                }
                if bf_f >= 8 && is_not_here(l2 ^ l1) {
                    return Err(format!("l2 ^ l1"));
                }
                if bf_f >= 9 && is_not_here(l3 ^ l4) {
                    return Err(format!("l3 ^ l4"));
                }
                if bf_f >= 10 && is_not_here(l2 ^ l3) {
                    return Err(format!("l2 ^ l3 = {}", l2 ^ l3));
                }
                if bf_f >= 11 && is_not_here(l1 ^ l4) {
                    return Err(format!("l1 ^ l4"));
                }
                if bf_f >= 12 && is_not_here(l1 ^ l2 ^ l3) {
                    return Err(format!("l1 ^ l2 ^ l3"));
                }
                if bf_f >= 13 && is_not_here(l1 ^ l2 ^ l4) {
                    return Err(format!("l1 ^ l2 ^ l4"));
                }
                if bf_f > 13 {
                    return Err(format!("need more hashs l, hf = {},{}", bf_l, bf_f));
                }
                Ok(())
            };

            g(&[
                0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111,
                0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111,
                0b1111111, 0b1111111,
            ])
            .unwrap();
            g(&[
                0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b0000000, 0b0000000, 0b0000000,
                0b0000000, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b0000000, 0b0000000,
                0b0000000, 0b0000000,
            ])
            .unwrap();
            g(&[
                0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b1111111, 0b1111111, 0b1111111,
                0b1111111, 0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b1111111, 0b1111111,
                0b1111111, 0b1111111,
            ])
            .unwrap();

            {
                let mut c = md5::Context::new();
                c.consume(42_u32.to_be_bytes());
                let d = c.compute().0;
                g(&d).unwrap();

                let mut c2 = md5::Context::new();
                c2.consume(d);
                c2.consume("/file.txt");
                let d = c2.compute().0;
                // dbg!(d);
                g(&d).unwrap();
            }

            // {
            //     let mut c = md5::Context::new();
            //     c.consume(0_u32.to_be_bytes());
            //     dbg!(0_u32.to_be_bytes());
            //     let d = c.compute().0;
            //     dbg!(d);
            //     g(&d).unwrap();
            // }
            let now = Instant::now();
            let mut matched_m = 0;
            let mut unmatched_m = 0;
            for (src, dst) in mappings.iter() {
                let f = |src: V<HAST::Idx>| {
                    if let Some(src) = src {
                        let mut c = md5::Context::new();
                        c.consume(src.0.0);
                        c.consume(src.1.to_u32().unwrap().to_be_bytes());
                        c.compute().0
                    } else {
                        panic!()
                        // let mut c = md5::Context::new();
                        // c.consume(0.to_u32().unwrap().to_be_bytes());
                        // c.compute().0
                    }
                };
                let mut c = md5::Context::new();
                let src = formator_src.format(src);
                let d = f(src.0);
                c.consume(d);
                // src.0.digest(&mut c);
                let dst = formator_dst.format(dst);
                let d = f(dst.0);
                c.consume(d);
                // dst.0.digest(&mut c);
                match g(&c.compute().0) {
                    Err(e) => {
                        unmatched_m += 1;
                        let t = stores.resolve_type(&src.1);
                        log::debug!("{} {:?}", e, t);
                    }
                    Ok(_) => {
                        matched_m += 1;
                    }
                }
            }
            dbg!(matched_m, unmatched_m);
            let missing_mappings = gt_bf
                .as_raw_slice()
                .iter()
                .zip(hast_bf.as_raw_slice().iter())
                .map(|(a, b)| u64::count_ones((a ^ b) & a) as usize)
                .sum();

            let bf_mappings_compare_t = now.elapsed().as_secs_f64();
            dbg!(bf_mappings_compare_t);
            let additional_mappings = unmatched_m;

            ValidityRes {
                missing_mappings,
                additional_mappings,
            }
        }
    }
}

#[derive(Debug)]
pub struct Counts {
    pub src_heap: usize,
    pub dst_heap: usize,
    pub mappings: usize,
    pub actions: isize,
}

#[derive(Debug)]
pub struct ValidityRes<T> {
    pub missing_mappings: T,
    pub additional_mappings: T,
}

impl<T> ValidityRes<T> {
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> ValidityRes<U> {
        ValidityRes {
            missing_mappings: f(self.missing_mappings),
            additional_mappings: f(self.additional_mappings),
        }
    }
}

/// Exact comparison of mappings
///
/// Slow for large codebases ie. minutes on something like Spoon.
/// The main slowing factor is io because the subcommand serialize mappings to a json file then we parse it.
/// It could be improved using another intermediate representation.
/// Note. that it would also be more efficient to compare edit scripts,
/// but the exact positions taken to represent evolutions is different
/// between gumtree and our implementation.
/// WARN does not work well with the no space wrapper.
/// TODO compute the byte length of subtree independently of spaces
pub struct SimpleJsonPostProcess {
    file: diff_output::F<diff_output::Tree>,
}

impl SimpleJsonPostProcess {
    pub fn new(file: &Path) -> Self {
        let now = Instant::now();
        let gt_out = serde_json::from_reader::<_, diff_output::F<diff_output::Tree>>(
            File::open(file).expect("should be a file"),
        )
        .unwrap();
        let gt_out_parsing_t = now.elapsed().as_secs_f64();
        dbg!(gt_out_parsing_t);
        Self { file: gt_out }
    }
    pub fn performances(&self) -> Vec<f64> {
        self.file
            .times
            .iter()
            .map(|x| Duration::from_nanos(*x as u64).as_secs_f64())
            .collect::<Vec<_>>()
    }
    pub fn counts(&self) -> Counts {
        let mappings = self.file.matches.len();
        let actions = self.file.actions.as_ref().map_or(-1, |x| x.len() as isize);
        // TODO first need some work on the java side, but anyway not used for eval
        Counts {
            mappings,
            actions,
            src_heap: 42,
            dst_heap: 42,
        }
    }
    pub fn validity_mappings<'store: 'a, 'a, HAST, SD, DD>(
        self,
        mapper: &'a Mapper<HAST, DD, SD, VecStore<u32>>,
    ) -> ValidityRes<Vec<diff_output::Match<diff_output::Tree>>>
    where
        HAST: HyperAST + Copy,
        HAST::IdN: types::NodeId<IdN = HAST::IdN>,
        //  + NodeStore<HAST::IdN, R<'store> = HAST::T>,
        HAST::IdN: Clone + Debug + Eq,
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::Tree + WithSerialization,
        SD: ShallowDecompressedTreeStore<HAST, u32>
            + PostOrder<HAST, u32>
            + DecompressedWithSiblings<HAST, u32>,
        DD: ShallowDecompressedTreeStore<HAST, u32>
            + PostOrder<HAST, u32>
            + DecompressedWithSiblings<HAST, u32>,
    {
        let hyperast = mapper.hyperast;
        let mapping = &mapper.mapping;
        let src_arena = &mapping.src_arena;
        let dst_arena = &mapping.dst_arena;
        let src_tr = src_arena.original(&src_arena.root());
        let dst_tr = dst_arena.original(&dst_arena.root());
        self._validity_mappings(
            hyperast,
            src_arena,
            src_tr,
            dst_arena,
            dst_tr,
            &mapping.mappings,
        )
    }
    pub fn _validity_mappings<'store: 'a, 'a, HAST, SD, DD>(
        self,
        stores: HAST,
        src_arena: &'a SD,
        src_tr: HAST::IdN,
        dst_arena: &'a DD,
        dst_tr: HAST::IdN,
        mappings: &VecStore<u32>,
    ) -> ValidityRes<Vec<diff_output::Match<diff_output::Tree>>>
    where
        HAST: HyperAST + Copy,
        HAST::IdN: types::NodeId<IdN = HAST::IdN>,
        //  + NodeStore<HAST::IdN, R<'store> = HAST::T>,
        HAST::IdN: Clone + Debug,
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithSerialization,
        SD: ShallowDecompressedTreeStore<HAST, u32>
            + PostOrder<HAST, u32>
            + DecompressedWithSiblings<HAST, u32>,
        DD: ShallowDecompressedTreeStore<HAST, u32>
            + PostOrder<HAST, u32>
            + DecompressedWithSiblings<HAST, u32>,
    {
        use hyperast::types::Labeled;
        let with_p = |mut pos: Position, ori| {
            let r = stores.node_store().resolve(&ori);
            let t = stores.resolve_type(&ori);
            if t.is_directory() || t.is_file() {
                pos.inc_path(stores.label_store().resolve(&r.get_label_unchecked()));
            }
            pos.set_len(r.try_bytes_len().unwrap_or(0));
            pos
        };
        let with_lsib = |mut pos: Position, lsib| {
            pos.inc_offset(pos.range().end - pos.range().start);
            let r = stores.node_store().resolve(&lsib);
            pos.set_len(r.try_bytes_len().unwrap());
            pos
        };
        let mut formator_src = FormatCached::from((stores, src_arena, src_tr, with_p, with_lsib));
        let mut formator_dst = FormatCached::from((stores, dst_arena, dst_tr, with_p, with_lsib));
        let mut formator = |a, b| diff_output::Match {
            src: diff_output::Tree::from_pos(stores, formator_src.format(a)),
            dest: diff_output::Tree::from_pos(stores, formator_dst.format(b)),
        };
        use hashbrown::HashSet;
        let now = Instant::now();
        let hast_mappings: Vec<diff_output::Match<diff_output::Tree>> = mappings
            .iter()
            // .src_to_dst.par_iter().enumerate().filter(|x| *x.1 != 0).map(|(src, dst)| (num_traits::cast(src).unwrap(), *dst - 1))
            .map(|(a, b)| formator(a, b))
            .collect();
        let hast_m_formating_t = now.elapsed().as_secs_f64();
        dbg!(hast_m_formating_t);
        let now = Instant::now();
        dbg!(hast_mappings.len());
        let hast_mappings: HashSet<&diff_output::Match<diff_output::Tree>> =
            hast_mappings.iter().collect();
        let gt_mappings: HashSet<&diff_output::Match<diff_output::Tree>> =
            self.file.matches.iter().collect();
        let mappings_formating_t = now.elapsed().as_secs_f64();
        dbg!(mappings_formating_t);
        let now = Instant::now();
        let missings_mappings: Vec<_> = gt_mappings.par_difference(&hast_mappings).collect();
        let additional_mappings: Vec<_> = hast_mappings.par_difference(&gt_mappings).collect();
        let mappings_compare_t = now.elapsed().as_secs_f64();
        dbg!(mappings_compare_t);
        ValidityRes {
            missing_mappings: missings_mappings.into_iter().cloned().cloned().collect(),
            additional_mappings: additional_mappings.into_iter().cloned().cloned().collect(),
        }
    }
}
pub struct PathJsonPostProcess {
    file: diff_output::F<diff_output::Path>,
}

impl PathJsonPostProcess {
    pub fn new(file: &Path) -> Self {
        let now = Instant::now();
        let gt_out = serde_json::from_reader::<_, diff_output::F<diff_output::Path>>(
            File::open(file).expect("should be a file"),
        )
        .unwrap();
        let gt_out_parsing_t = now.elapsed().as_secs_f64();
        dbg!(gt_out_parsing_t);
        Self { file: gt_out }
    }
    pub fn performances(&self) -> Vec<f64> {
        self.file
            .times
            .iter()
            .map(|x| Duration::from_nanos(*x as u64).as_secs_f64())
            .collect::<Vec<_>>()
    }
    pub fn counts(&self) -> Counts {
        let mappings = self.file.matches.len();
        let actions = self.file.actions.as_ref().map_or(-1, |x| x.len() as isize);
        // TODO first need some work on the java side, but anyway not used for eval
        Counts {
            mappings,
            actions,
            src_heap: 42,
            dst_heap: 42,
        }
    }
    pub fn validity_mappings<'store: 'a, 'a, HAST, SD, DD>(
        self,
        mapper: &'a Mapper<HAST, DD, SD, VecStore<u32>>,
    ) -> ValidityRes<Vec<diff_output::Match<diff_output::Path>>>
    where
        HAST: HyperAST + Copy,
        HAST::IdN: Clone + Debug + Eq,
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::Tree,
        SD: ShallowDecompressedTreeStore<HAST, u32>
            + PostOrder<HAST, u32>
            + DecompressedWithSiblings<HAST, u32>,
        DD: ShallowDecompressedTreeStore<HAST, u32>
            + PostOrder<HAST, u32>
            + DecompressedWithSiblings<HAST, u32>,
    {
        let mapping = &mapper.mapping;
        let src_arena = &mapping.src_arena;
        let dst_arena = &mapping.dst_arena;
        let src_tr = src_arena.original(&src_arena.root());
        let dst_tr = dst_arena.original(&dst_arena.root());
        self._validity_mappings(src_arena, src_tr, dst_arena, dst_tr, &mapping.mappings)
    }

    pub(crate) fn _validity_mappings<'store: 'a, 'a, HAST, SD, DD>(
        self,
        src_arena: &'a SD,
        src_tr: HAST::IdN,
        dst_arena: &'a DD,
        dst_tr: HAST::IdN,
        mappings: &VecStore<u32>,
    ) -> ValidityRes<Vec<diff_output::Match<diff_output::Path>>>
    where
        HAST: HyperAST + Copy,
        HAST::IdN: Clone + Debug,
        SD: ShallowDecompressedTreeStore<HAST, u32>
            + PostOrder<HAST, u32>
            + DecompressedWithSiblings<HAST, u32>,
        DD: ShallowDecompressedTreeStore<HAST, u32>
            + PostOrder<HAST, u32>
            + DecompressedWithSiblings<HAST, u32>,
    {
        type CP<Idx> = Option<(CompressedTreePath<Idx>, Idx)>;
        let with_p = |pos: CP<HAST::Idx>, _ori: HAST::IdN| -> CP<HAST::Idx> {
            Some((
                if let Some((ctp, i)) = pos {
                    ctp.extend(&[i])
                } else {
                    vec![].into()
                },
                num_traits::zero(),
            ))
        };
        let with_lsib = |pos: CP<HAST::Idx>, _lsib: HAST::IdN| -> CP<HAST::Idx> {
            let mut pos = pos.unwrap();
            pos.1 = pos.1 + num_traits::one();
            Some(pos)
        };
        let mut formator_src = PathCached::from((src_arena, src_tr, with_p, with_lsib));
        let mut formator_dst = PathCached::from((dst_arena, dst_tr, with_p, with_lsib));
        let mut formator = |a, b| diff_output::Match::<diff_output::Path> {
            src: {
                if let Some(a) = formator_src.format(a).0 {
                    let mut v: Vec<_> = a.0.iter().map(|x| x.to_u32().unwrap()).collect();
                    v.push(a.1.to_u32().unwrap());
                    diff_output::Path(v)
                } else {
                    diff_output::Path(vec![])
                }
            },
            dest: {
                if let Some(a) = formator_dst.format(b).0 {
                    let mut v: Vec<_> = a.0.iter().map(|x| x.to_u32().unwrap()).collect();
                    v.push(a.1.to_u32().unwrap());
                    diff_output::Path(v)
                } else {
                    diff_output::Path(vec![])
                }
            },
        };
        use hashbrown::HashSet;
        let now = Instant::now();
        let hast_mappings: Vec<diff_output::Match<diff_output::Path>> = mappings
            .iter()
            // .src_to_dst.par_iter().enumerate().filter(|x| *x.1 != 0).map(|(src, dst)| (num_traits::cast(src).unwrap(), *dst - 1))
            .map(|(a, b)| formator(a, b))
            .collect();
        let hast_m_formating_t = now.elapsed().as_secs_f64();
        dbg!(hast_m_formating_t);
        let now = Instant::now();
        dbg!(hast_mappings.len());
        let hast_mappings: HashSet<&diff_output::Match<diff_output::Path>> =
            hast_mappings.iter().collect();
        let gt_mappings: HashSet<&diff_output::Match<diff_output::Path>> =
            self.file.matches.iter().collect();
        let mappings_formating_t = now.elapsed().as_secs_f64();
        dbg!(mappings_formating_t);
        let now = Instant::now();
        let missings_mappings: Vec<_> = gt_mappings.par_difference(&hast_mappings).collect();
        let additional_mappings: Vec<_> = hast_mappings.par_difference(&gt_mappings).collect();
        let mappings_compare_t = now.elapsed().as_secs_f64();
        dbg!(mappings_compare_t);
        ValidityRes {
            missing_mappings: missings_mappings.into_iter().cloned().cloned().collect(),
            additional_mappings: additional_mappings.into_iter().cloned().cloned().collect(),
        }
    }
}

struct FormatCached<'a, HAST: HyperASTShared, D, U, F, G> {
    store: HAST,
    arena: &'a D,
    cache: RecCachedProcessor<'a, HAST::IdN, D, u32, U, F, G>,
}

impl<'a, S: HyperASTShared, D, U, F: Clone, G: Clone> From<(S, &'a D, S::IdN, F, G)>
    for FormatCached<'a, S, D, U, F, G>
{
    fn from((store, arena, tr, with_p, with_lsib): (S, &'a D, S::IdN, F, G)) -> Self {
        Self {
            store,
            arena,
            cache: RecCachedProcessor::from((arena, tr, with_p, with_lsib)),
        }
    }
}
impl<'a, HAST, D, U: Clone + Default, F, G> FormatCached<'a, HAST, D, U, F, G>
where
    HAST: HyperAST + Copy,
    HAST::IdN: Debug,
    D: ShallowDecompressedTreeStore<HAST, u32>
        + PostOrder<HAST, u32>
        + DecompressedWithSiblings<HAST, u32>,
    F: Fn(U, HAST::IdN) -> U,
    G: Fn(U, HAST::IdN) -> U,
{
    fn format(&mut self, x: u32) -> (U, HAST::IdN) {
        (
            self.cache.position(self.store, &x).clone(),
            self.arena.original(&x),
        )
    }
}
struct PathCached<'a, IdN, D, U, F, G> {
    arena: &'a D,
    cache: RecCachedProcessor<'a, IdN, D, u32, U, F, G>,
}

impl<'a, IdN, D, U, F: Clone, G: Clone> From<(&'a D, IdN, F, G)>
    for PathCached<'a, IdN, D, U, F, G>
{
    fn from((arena, tr, with_p, with_lsib): (&'a D, IdN, F, G)) -> Self {
        Self {
            arena,
            cache: RecCachedProcessor::from((arena, tr, with_p, with_lsib)),
        }
    }
}

impl<'a, IdN, D, U: Clone + Default, F, G> PathCached<'a, IdN, D, U, F, G>
where
    F: Fn(U, IdN) -> U,
    G: Fn(U, IdN) -> U,
{
    fn format<HAST: HyperAST<IdN = IdN> + Copy>(&mut self, x: u32) -> (U, IdN)
    where
        D: ShallowDecompressedTreeStore<HAST, u32>
            + PostOrder<HAST, u32>
            + DecompressedWithSiblings<HAST, u32>,
    {
        (self.cache.position2(&x).clone(), self.arena.original(&x))
    }
}

pub fn print_mappings<
    'a,
    IdD: 'a + PrimInt + Debug,
    M: MonoMappingStore<Src = IdD, Dst = IdD> + Debug,
    HAST: HyperAST + Copy,
    // IdN: Clone + Eq + Debug,
    // NS: NodeStore<IdN>,
    // LS: LabelStore<str>,
    SD,
    DD,
>(
    dst_arena: &'a DD, //CompletePostOrder<NS::R<'store>, IdD>,
    src_arena: &'a SD, //CompletePostOrder<NS::R<'store>, IdD>,
    stores: HAST,
    mappings: &M,
) where
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: WithSerialization,
    // <NS as types::NodeStore<IdN>>::R<'store>:
    //     'store + Tree<TreeId = IdN, Label = LS::I> + types::WithSerialization,
    // <<NS as types::NodeStore<IdN>>::R<'store> as types::Typed>::Type: Debug,
    SD: FullyDecompressedTreeStore<HAST, IdD> + PostOrder<HAST, IdD>, // + DecompressedWithParent<HAST, IdD>,
    DD: FullyDecompressedTreeStore<HAST, IdD> + PostOrder<HAST, IdD>, //+ DecompressedWithParent<HAST, IdD>,
{
    let mut mapped = vec![false; dst_arena.len()];
    let src_arena = SimplePreOrderMapper::from(src_arena);
    let disp = DisplayCompletePostOrder::<IdD, _, _>::new(stores, dst_arena);
    let dst_arena = disp.to_string();
    let mappings = src_arena
        .map
        .iter()
        .map(|x| {
            if let Some(dst) = mappings.get_dst(x) {
                if mapped[dst.to_usize().unwrap()] {
                    assert!(false, "GreedySubtreeMatcher {}", dst.to_usize().unwrap())
                }
                mapped[dst.to_usize().unwrap()] = true;
                Some(dst)
            } else {
                None
            }
        })
        .fold("".to_string(), |x, c| {
            if let Some(c) = c {
                let c = c.to_usize().unwrap();
                format!("{x}{c}\n")
            } else {
                format!("{x} \n")
            }
        });
    let src_arena = DisplaySimplePreOrderMapper {
        inner: &src_arena,
        stores: &stores,
    }
    .to_string();
    let cols = vec![src_arena, mappings, dst_arena];
    let sizes: Vec<_> = cols
        .iter()
        .map(|x| x.lines().map(|x| x.len()).max().unwrap_or(0))
        .collect();
    let mut cols: Vec<_> = cols.iter().map(|x| x.lines()).collect();
    loop {
        let mut b = false;
        print!("|");
        for i in 0..cols.len() {
            if let Some(l) = cols[i].next() {
                print!(" {}{} |", l, " ".repeat(sizes[i] - l.len()));
                b = true;
            } else {
                print!(" {} |", " ".repeat(sizes[i]));
            }
        }
        println!();
        if !b {
            break;
        }
    }
}

pub fn print_mappings_no_ranges<
    'a,
    IdD: 'a + PrimInt + Debug,
    M: MonoMappingStore<Src = IdD, Dst = IdD>,
    HAST: HyperAST + Copy,
    // IdN: Clone + Eq + Debug,
    DD: PostOrder<HAST, IdD> + FullyDecompressedTreeStore<HAST, IdD>,
    SD: PostOrder<HAST, IdD> + FullyDecompressedTreeStore<HAST, IdD>,
>(
    dst_arena: &'a DD,
    src_arena: &'a SD,
    stores: HAST,
    mappings: &M,
) where
    for<'t> <HAST as types::AstLending<'t>>::RT: WithSerialization,
    // <NS as types::NodeStore<IdN>>::R<'store>: 'store + Tree<TreeId = IdN, Label = LS::I>,
    // <<NS as types::NodeStore<IdN>>::R<'store> as types::Typed>::Type: Debug,
{
    let mut mapped = vec![false; dst_arena.len()];
    let src_arena = SimplePreOrderMapper::from(src_arena);
    let disp = DisplayCompletePostOrder::<IdD, _, _>::new(stores, dst_arena);
    let dst_arena = format!("{:?}", disp);
    let mappings = src_arena
        .map
        .iter()
        .map(|x| {
            if let Some(dst) = mappings.get_dst(x) {
                if mapped[dst.to_usize().unwrap()] {
                    assert!(false, "GreedySubtreeMatcher {}", dst.to_usize().unwrap())
                }
                mapped[dst.to_usize().unwrap()] = true;
                Some(dst)
            } else {
                None
            }
        })
        .fold("".to_string(), |x, c| {
            if let Some(c) = c {
                let c = c.to_usize().unwrap();
                format!("{x}{c}\n")
            } else {
                format!("{x} \n")
            }
        });
    let src_arena = format!(
        "{:?}",
        DisplaySimplePreOrderMapper {
            inner: &src_arena,
            stores: &stores
        }
    );
    let cols = vec![src_arena, mappings, dst_arena];
    let sizes: Vec<_> = cols
        .iter()
        .map(|x| x.lines().map(|x| x.len()).max().unwrap_or(0))
        .collect();
    let mut cols: Vec<_> = cols.iter().map(|x| x.lines()).collect();
    loop {
        let mut b = false;
        print!("|");
        for i in 0..cols.len() {
            if let Some(l) = cols[i].next() {
                print!(" {}{} |", l, " ".repeat(sizes[i] - l.len()));
                b = true;
            } else {
                print!(" {} |", " ".repeat(sizes[i]));
            }
        }
        println!();
        if !b {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        other_tools,
        preprocess::{JavaPreprocessFileSys, parse_dir_pair},
    };
    use hyperast::store::{SimpleStores, labels::LabelStore, nodes::legion::NodeStore};
    // use hyperast_gen_ts_java::types::TStore;
    use hyper_diff::algorithms::{self, DiffResult, MappingDurations};

    #[test]
    fn test() {
        println!("{:?}", std::env::current_dir());
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let src_dst = crate::buggy_fixed::buggy_fixed_dataset_roots(root);
        let [buggy_path, fixed_path] = src_dst.map(|x| x.join("Jsoup/92"));
        let src = buggy_path;
        let dst = fixed_path;

        let stores = SimpleStores {
            label_store: LabelStore::new(),
            type_store: Default::default(),
            node_store: NodeStore::new(),
        };
        let md_cache = Default::default();
        let mut java_gen = JavaPreprocessFileSys {
            main_stores: stores,
            java_md_cache: md_cache,
        };
        let now = Instant::now();
        let (src_tr, dst_tr) = parse_dir_pair(&mut java_gen, &src, &dst);
        let parse_t = now.elapsed().as_secs_f64();

        dbg!(&parse_t);
        dbg!(&src_tr.metrics.size);
        dbg!(&dst_tr.metrics.size);

        let gt_out_format = "COMPRESSED"; // JSON
        let gt_out = other_tools::gumtree::subprocess(
            &java_gen.main_stores,
            src_tr.compressed_node,
            dst_tr.compressed_node,
            "gumtree",
            "Chawathe",
            60 * 5,
            gt_out_format,
        )
        .unwrap();

        let DiffResult {
            mapping_durations,
            mapper: mapping,
            actions,
            prepare_gen_t,
            gen_t,
        } = algorithms::gumtree::diff(
            &java_gen.main_stores,
            &src_tr.compressed_node,
            &dst_tr.compressed_node,
        );
        let actions = actions.unwrap();
        // let Mapping {
        //     src_arena,
        //     dst_arena,
        //     mappings,
        // } = mapping;
        let MappingDurations([subtree_matcher_t, bottomup_matcher_t]) = mapping_durations.into();

        let hast_timings = vec![subtree_matcher_t, bottomup_matcher_t, prepare_gen_t + gen_t];

        dbg!(&hast_timings);
        let pp = CompressedBfPostProcess::create(&gt_out);
        let (pp, counts) = pp.counts();
        let (pp, gt_timings) = pp.performances();
        let valid = pp.validity_mappings(&mapping);
        use hyper_diff::actions::Actions as _;
        if valid.additional_mappings > 0 || valid.missing_mappings > 0 {
            dbg!(
                valid.additional_mappings,
                valid.missing_mappings,
                actions.len(),
                counts.actions
            );
            panic!()
        } else if counts.actions < 0 {
            panic!("no actions computed")
        } else if counts.actions as usize != actions.len() {
            dbg!(actions.len(), counts.actions);
            panic!()
        } else {
            println!("gt_tt={:?} evos={}", &gt_timings, counts.actions);
            println!("tt={:?} evos={}", &hast_timings, actions.len())
        }
    }
}
