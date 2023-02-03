use std::ops::Range;

use num_traits::{cast, PrimInt, ToPrimitive};

use crate::matchers::mapping_store::MappingStore;

use super::mapping_store::{MonoMappingStore, VecStore};

pub struct SimilarityMeasure {
    ncd: u32,
    src_l: usize,
    dst_l: usize,
}

impl SimilarityMeasure {
    pub fn new<Id: PrimInt, Store: MonoMappingStore<Src = Id, Dst = Id>>(
        src: &[Id],
        dst: &[Id],
        mappings: &Store,
    ) -> Self {
        Self {
            ncd: number_of_common_descendants(src, dst, mappings),
            src_l: src.len(),
            dst_l: dst.len(),
        }
    }

    pub fn range<Id1: PrimInt, Id2: PrimInt, Store: MonoMappingStore<Src = Id1, Dst = Id2>>(
        src: &Range<Id1>,
        dst: &Range<Id2>,
        mappings: &Store,
    ) -> Self {
        Self {
            ncd: number_of_common_descendants_ranges(src, dst, mappings),
            src_l: (src.end - src.start).to_usize().unwrap(),
            dst_l: (dst.end - dst.start).to_usize().unwrap(),
        }
    }

    pub fn chawathe(&self) -> f64 {
        let max = f64::max(self.src_l as f64, self.dst_l as f64);
        self.ncd as f64 / max
    }

    pub fn overlap(&self) -> f64 {
        let min = f64::min(self.src_l as f64, self.dst_l as f64);
        self.ncd as f64 / min
    }

    pub fn dice(&self) -> f64 {
        (2.0_f64 * (self.ncd as f64)) / (self.src_l as f64 + self.dst_l as f64)
    }

    pub fn jaccard(&self) -> f64 {
        let num = self.ncd as f64;
        let den = self.src_l as f64 + self.dst_l as f64 - num;
        self.ncd as f64 / den
    }
}

pub fn chawathe_similarity<
    Id1: PrimInt,
    Id2: PrimInt,
    Store: MonoMappingStore<Src = Id1, Dst = Id2>,
>(
    src: &[Id1],
    dst: &[Id2],
    mappings: &Store,
) -> f64 {
    let max = f64::max(src.len() as f64, dst.len() as f64);
    number_of_common_descendants(src, dst, mappings) as f64 / max
}

pub fn overlap_similarity<
    Id1: PrimInt,
    Id2: PrimInt,
    Store: MonoMappingStore<Src = Id1, Dst = Id2>,
>(
    src: &[Id1],
    dst: &[Id2],
    mappings: &Store,
) -> f64 {
    let min = f64::min(src.len() as f64, dst.len() as f64);
    number_of_common_descendants(src, dst, mappings) as f64 / min
}

pub fn dice_similarity<
    Id1: PrimInt,
    Id2: PrimInt,
    Store: MonoMappingStore<Src = Id1, Dst = Id2>,
>(
    src: &[Id1],
    dst: &[Id2],
    mappings: &Store,
) -> f64 {
    let common_descendants = number_of_common_descendants(src, dst, mappings) as f64;
    (2.0_f64 * common_descendants) / (src.len() as f64 + dst.len() as f64)
}

pub fn jaccard_similarity<
    Id1: PrimInt,
    Id2: PrimInt,
    Store: MonoMappingStore<Src = Id1, Dst = Id2>,
>(
    src: &[Id1],
    dst: &[Id2],
    mappings: &Store,
) -> f64 {
    let num = number_of_common_descendants(src, dst, mappings) as f64;
    let den = src.len() as f64 + dst.len() as f64 - num;
    num / den
}

pub fn number_of_common_descendants<
    Id1: PrimInt,
    Id2: PrimInt,
    Store: MonoMappingStore<Src = Id1, Dst = Id2>,
>(
    src: &[Id1],
    dst: &[Id2],
    mappings: &Store,
) -> u32 {
    let min = dst[0].to_usize().unwrap();
    let max = dst[dst.len() - 1].to_usize().unwrap() + 1;
    let mut a = bitvec::bitvec![0;max-min];
    dst.iter()
        .for_each(|x| a.set(x.to_usize().unwrap() - min, true));
    let dst_descendants: bitvec::boxed::BitBox = a.into_boxed_bitslice();
    let mut common = 0;

    for t in src {
        if mappings.is_src(t) {
            let m = mappings.get_dst_unchecked(t).to_usize().unwrap();
            if dst_descendants.len() != 0
                && min <= m
                && m - min < dst_descendants.len()
                && dst_descendants[m - min]
            {
                common += 1;
            }
        }
    }

    // println!("{}", src.len());

    // assert_eq!(common, {
    //     let s: HashSet<_, RandomState> =
    //         HashSet::from_iter(dst.iter().map(|x| x.to_usize().unwrap()));
    //     let mut c = 0;
    //     for t in src {
    //         if mappings.is_src(t) {
    //             let m = mappings.get_dst(t).to_usize().unwrap();
    //             if s.contains(&m.to_usize().unwrap()) {
    //                 c += 1;
    //             }
    //         }
    //     }
    //     c
    // });
    return common;
}

pub fn number_of_common_descendants_ranges<
    Id1: PrimInt,
    Id2: PrimInt,
    Store: MonoMappingStore<Src = Id1, Dst = Id2>,
>(
    src: &Range<Id1>,
    dst: &Range<Id2>,
    mappings: &Store,
) -> u32 {
    (src.start.to_usize().unwrap()..src.end.to_usize().unwrap())
        .into_iter()
        .filter(|t| mappings.is_src(&cast(*t).unwrap()))
        .filter(|t| dst.contains(&mappings.get_dst_unchecked(&cast(*t).unwrap())))
        .count()
        .try_into()
        .unwrap()
}

pub fn number_of_common_descendants_ranges_par(
    src: &Range<u32>,
    dst: &Range<u32>,
    mappings: &VecStore<u32>,
) -> u32 {
    use specs::prelude::ParallelIterator;
    use specs::rayon::prelude::IntoParallelIterator;
    (src.start.to_usize().unwrap()..src.end.to_usize().unwrap())
        .into_par_iter()
        .filter(|t| mappings.is_src(&(*t).try_into().unwrap()))
        .filter(|t| dst.contains(&mappings.get_dst_unchecked(&(*t).try_into().unwrap())))
        .count()
        .try_into()
        .unwrap()
}
