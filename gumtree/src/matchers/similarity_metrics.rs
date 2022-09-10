use num_traits::{cast, PrimInt, ToPrimitive};

use super::mapping_store::MonoMappingStore;

pub fn chawathe_similarity<Id: PrimInt, Store: MonoMappingStore<Ele = Id>>(
    src: &[Id],
    dst: &[Id],
    mappings: &Store,
) -> f64 {
    let max = f64::max(src.len() as f64, dst.len() as f64);
    number_of_common_descendants(src, dst, mappings) as f64 / max
}

pub fn overlap_similarity<Id: PrimInt, Store: MonoMappingStore<Ele = Id>>(
    src: &[Id],
    dst: &[Id],
    mappings: &Store,
) -> f64 {
    let min = f64::min(src.len() as f64, dst.len() as f64);
    number_of_common_descendants(src, dst, mappings) as f64 / min
}

pub fn dice_similarity<Id: PrimInt, Store: MonoMappingStore<Ele = Id>>(
    src: &[Id],
    dst: &[Id],
    mappings: &Store,
) -> f64 {
    let common_descendants = number_of_common_descendants(src, dst, mappings) as f64;
    (2.0_f64 * common_descendants) / (src.len() as f64 + dst.len() as f64)
}

pub fn jaccard_similarity<Id: PrimInt, Store: MonoMappingStore<Ele = Id>>(
    src: &[Id],
    dst: &[Id],
    mappings: &Store,
) -> f64 {
    let num = number_of_common_descendants(src, dst, mappings) as f64;
    let den = src.len() as f64 + dst.len() as f64 - num;
    num / den
}

fn number_of_common_descendants<Id: PrimInt, Store: MonoMappingStore<Ele = Id>>(
    src: &[Id],
    dst: &[Id],
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
            let m = mappings.get_dst(t).to_usize().unwrap();
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

    return common;
}
