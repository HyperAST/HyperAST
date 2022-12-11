use std::{
    fmt::Debug,
    fs::{self, File},
    path::Path,
    time::{Duration, Instant},
};

use hyper_ast::{
    position::Position,
    store::{defaults::NodeIdentifier, SimpleStores},
};
use hyper_gumtree::decompressed_tree_store::{
    complete_post_order::{DisplayCompletePostOrder, RecCachedProcessor},
    pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper},
    CompletePostOrder, ShallowDecompressedTreeStore,
};
use hyper_gumtree::matchers::mapping_store::VecStore;
use num_traits::{PrimInt, ToPrimitive};
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

// fn compare<'a>(
//     gt_out: &diff_output::F,
//     stores: &SimpleStores,
//     src: &NodeIdentifier,
//     dst: &NodeIdentifier,
// ) -> CompResult {
//     let DiffResult {
//         mapping_durations: [subtree_matcher_t, bottomup_matcher_t],
//         src_arena,
//         dst_arena,
//         mappings,
//         actions,
//         gen_t,
//     } = algorithms::gumtree::diff(stores, src, dst);

//     let hast_mappings: Vec<diff_output::Match> = mappings
//         // .iter()
//         .src_to_dst
//         .par_iter()
//         .enumerate()
//         .filter(|x| *x.1 != 0)
//         .map(|(src, dst)| (num_traits::cast(src).unwrap(), *dst - 1))
//         .map(|(a, b)| diff_output::Match {
//             src: (src, &*stores, &src_arena.path(&src_arena.root(), &a)).into(),
//             dest: (dst, &*stores, &dst_arena.path(&dst_arena.root(), &b)).into(),
//         })
//         .collect();

//     let hast_mappings: HashSet<&diff_output::Match> = hast_mappings.iter().collect();
//     let gt_mappings: HashSet<&diff_output::Match> = gt_out.matches.iter().collect();

//     let missings_mappings: Vec<_> = gt_mappings.difference(&hast_mappings).collect();
//     let additional_mappings: Vec<_> = hast_mappings.difference(&gt_mappings).collect();

//     let timings = vec![subtree_matcher_t, bottomup_matcher_t, gen_t];
//     let gt_len = gt_out.actions.len();
//     let hast_len = actions.len();

//     // let gt_actions: HashSet<&diff_output::Act> = gt_out.actions.iter().collect();
//     // let hast_actions: Vec<diff_output::Act> = actions
//     //     .iter()
//     //     .map(|x| (&src, &*java_tree_gen.stores, x).into())
//     //     .collect();
//     // let hast_actions: HashSet<&diff_output::Act> = hast_actions.iter().collect();
//     // let missings: Vec<_> = gt_actions.difference(&hast_actions).collect();
//     // let additional: Vec<_> = hast_actions.difference(&gt_actions).collect();
//     // if !(missings_mappings
//     //     .iter()
//     //     .find(|x| x.src.r#type != "spaces")
//     //     .is_none()
//     //     && additional_mappings
//     //         .iter()
//     //         .find(|x| x.src.r#type != "spaces")
//     //         .is_none())
//     // {
//     //     dbg!(&missings_mappings, &hast_mappings);
//     //     CompResult::Failure {
//     //         timings,
//     //         actions: actions.len(),
//     //         stage: "mappings".to_string(),
//     //         reason: format!(
//     //             "{:?};{:?}",
//     //             missings_mappings.len(),
//     //             additional_mappings.len()
//     //         ),
//     //     }
//     // } else
//     if !(missings_mappings.is_empty() && additional_mappings.is_empty()) {
//         if missings_mappings.len() < 110 {
//             dbg!(&missings_mappings);
//         }
//         if additional_mappings.len() < 110 {
//             dbg!(&additional_mappings);
//         } else if src_arena.len() < 20000 && dst_arena.len() < 20000 {
//             print_mappings(&dst_arena, &src_arena, stores, &mappings);
//         } else {
//             dbg!(&missings_mappings
//                 .iter()
//                 .filter(|x| x.src.file.starts_with("10"))
//                 .collect::<Vec<_>>());
//             dbg!(&additional_mappings
//                 .iter()
//                 .filter(|x| x.src.file.starts_with("10"))
//                 .collect::<Vec<_>>());
//         }
//         // print_mappings(&dst_arena, &src_arena, stores, &mappings);
//         CompResult::Failure {
//             timings,
//             actions_hast: hast_len,
//             actions_other: gt_len,
//             stage: "mapping".to_string(),
//             reason: format!(
//                 "baseline={} missing={} additional={}",
//                 gt_mappings.len(),
//                 missings_mappings.len(),
//                 additional_mappings.len()
//             ),
//             mappings_hast: todo!(),
//             mappings_other: todo!(),
//         }
//     } else if gt_len != hast_len {
//         CompResult::Failure {
//             timings,
//             actions_hast: hast_len,
//             actions_other: gt_len,
//             stage: "gen".to_string(),
//             reason: format!("different sizes gt={} hast={}", gt_len, hast_len),
//             mappings_hast: todo!(),
//             mappings_other: todo!(),
//         }
//     } else {
//         CompResult::Success {
//             timings,
//             actions: actions.len(),
//             mappings: todo!(),
//         }
//         // TODO problem comparing actions, (related to considered position of elements (before, during, after))
//         // dbg!(gt_len, hast_len, missings.len(), additional.len());
//         // dbg!(&missings, &hast_actions);
//         // panic!();
//         // CompResult::Failure {
//         //     timings,
//         //     actions: actions.len(),
//         //     stage: "gen".to_string(),
//         //     reason: format!("{:?};{:?}", missings, additional),
//         // }
//     }
// }

// pub fn compare<'a>(
//     gt_out: &diff_output::F,
//     stores: &SimpleStores,
//     src: &NodeIdentifier,
//     dst: &NodeIdentifier,
// ) -> CompResult {
// }
// fn comparing_outputs_compressed(
//     gt_out: &Path,
//     stores: &SimpleStores,
//     src_arena: CompletePostOrder<NodeIdentifier, u32>,
//     src_tr: NodeIdentifier,
//     dst_arena: CompletePostOrder<NodeIdentifier, u32>,
//     dst_tr: NodeIdentifier,
//     mappings: VecStore<u32>,
//     actions: ActionsVec<SimpleAction<LabelIdentifier, u16, NodeIdentifier>>,
//     timings: Vec<f64>,
// ) -> CompResult {
//     pub struct G {
//         pub times: Vec<f64>,
//         pub matched_m: usize,
//         pub unmatched_m: usize,
//         pub actions: usize,
//     }

//     use hyper_ast::types::LabelStore;
//     use hyper_ast::types::Labeled;
//     use hyper_ast::types::Typed;
//     use hyper_ast::types::WithSerialization;
//     let gt_out = {
//         let now = Instant::now();
//         use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
//         let mut aaaa = std::io::Cursor::new(fs::read(&gt_out).unwrap());
//         assert_eq!(424242, aaaa.read_u32::<BigEndian>().unwrap());
//         let actions = aaaa.read_u32::<BigEndian>().unwrap().to_usize().unwrap();
//         let mappings_l = aaaa.read_u32::<BigEndian>().unwrap();
//         let t_len = aaaa.read_u32::<BigEndian>().unwrap() as usize;
//         let timings: Vec<_> = (0..t_len)
//             .map(|_| aaaa.read_u64::<BigEndian>().unwrap())
//             .collect();
//         let bf_f = aaaa.read_u32::<BigEndian>().unwrap() as usize;
//         let bf_l = aaaa.read_u32::<BigEndian>().unwrap() as usize;
//         dbg!(&actions, &mappings_l, &t_len, &timings, &bf_f, &bf_l);

//         let mut gt_bf = bitvec::bitvec![bitvec::order::Lsb0, u64; 0;bf_l];
//         dbg!(gt_bf.len());
//         dbg!(gt_bf.as_raw_slice().len());
//         dbg!(gt_bf.as_raw_slice().len() * 8);
//         dbg!(gt_bf.as_raw_slice().len() * 8 * 8);
//         aaaa.read_u64_into::<LittleEndian>(gt_bf.as_mut_raw_slice())
//             .unwrap();
//         dbg!(&gt_bf.as_raw_slice()[0].to_le_bytes());
//         let gt_compressed_output_load_t = now.elapsed().as_secs_f64();
//         dbg!(gt_compressed_output_load_t);
//         let gt_bf = gt_bf;

//         let mut hast_bf = bitvec::bitvec![bitvec::order::Lsb0, u64; 0;bf_l];

//         #[derive(Clone)]
//         struct DPos {
//             file: md5::Digest,
//             path: PathBuf,
//             start: u32,
//             len: u32,
//         }

//         impl DPos {
//             fn digest(&self, c: &mut md5::Context) {
//                 c.consume(self.file.0);
//                 c.consume(self.start.to_be_bytes());
//                 c.consume((self.start + self.len).to_be_bytes());

//                 // d.update(t.getType().toString().getBytes());
//                 // if (t.hasLabel()) {
//                 //     d.update(t.getLabel().getBytes());
//                 // }
//                 // d.update(computeFile2(t));
//                 // d.update(Ints.toByteArray(t.getPos()));
//                 // d.update(Ints.toByteArray(t.getEndPos()));
//             }
//         }

//         impl Default for DPos {
//             fn default() -> Self {
//                 Self {
//                     file: md5::Digest(Default::default()),
//                     path: Default::default(),
//                     start: 0,
//                     len: 0,
//                 }
//             }
//         }

//         let with_p = |mut pos: DPos, ori| {
//             let r = stores.node_store.resolve(ori);
//             let t = r.get_type();
//             if t.is_directory() || t.is_file() {
//                 let label = stores.label_store.resolve(&r.get_label());
//                 // if pos.path.to_string_lossy().len() == 0 {
//                 //     dbg!(&label);
//                 //     dbg!(&pos.file.0);
//                 // }
//                 if label != "" {
//                     let mut c = md5::Context::new();
//                     // let bo = pos.file.0;
//                     c.consume(pos.file.0);
//                     c.consume(label);
//                     pos.path.push(label);
//                     pos.file = c.compute();
//                     // if label == "src" {
//                     //     dbg!(&pos.file.0);
//                     //     // [-128, -25, 54, -113, -44, 107, -11, 49, 60, -13, -45, -116, -68, 38, -107, -30]
//                     // }
//                 }
//             }
//             pos.len = r.try_bytes_len().unwrap_or(0).to_u32().unwrap();
//             pos
//         };
//         let with_lsib = |mut pos: DPos, lsib| {
//             // assert!(pos.path.to_string_lossy().len()>0, "{:?} {} {}", pos.file.0, pos.start, pos.len);
//             pos.start = pos.start + pos.len;
//             let r = stores.node_store.resolve(lsib);
//             assert!(!r.get_type().is_directory() && !r.get_type().is_file());
//             pos.len = r.try_bytes_len().unwrap().to_u32().unwrap();
//             pos
//         };

//         let mut formator_src = FormatCached::from((stores, &src_arena, src_tr, with_p, with_lsib));
//         let mut formator_dst = FormatCached::from((stores, &dst_arena, dst_tr, with_p, with_lsib));

//         let mut is_not_here = |x| {
//             hast_bf.set((x % bf_l as u32) as usize, true);
//             !gt_bf[(x % bf_l as u32) as usize]
//         };
//         assert!(!is_not_here(0));
//         assert!(!is_not_here(42));
//         let mut g = |h: &[u8; 16]| {
//             let [l1, l2, l3, l4] = h
//                 .array_chunks::<4>()
//                 .map(|x| u32::from_be_bytes(*x))
//                 .array_chunks::<4>()
//                 .next()
//                 .unwrap();

//             if bf_f >= 1
//                 && is_not_here(u32::rotate_left(l1 ^ l2, 2) ^ u32::rotate_right(l3 ^ l4, 2))
//             {
//                 return Err(format!("1"));
//             }
//             if bf_f >= 2
//                 && is_not_here(u32::rotate_left(l1 ^ l3, 2) ^ u32::rotate_right(l2 ^ l4, 2))
//             {
//                 return Err(format!("1"));
//             }
//             if bf_f >= 3
//                 && is_not_here(u32::rotate_left(l1 ^ l4, 2) ^ u32::rotate_right(l2 ^ l3, 2))
//             {
//                 return Err(format!("3"));
//             }
//             if bf_f >= 4 && is_not_here(l1) {
//                 return Err(format!("l1"));
//             }
//             if bf_f >= 5 && is_not_here(l2) {
//                 return Err(format!("l2"));
//             }
//             if bf_f >= 6 && is_not_here(l3) {
//                 return Err(format!("l3"));
//             }
//             if bf_f >= 7 && is_not_here(l4) {
//                 return Err(format!("l4"));
//             }
//             if bf_f >= 8 && is_not_here(l2 ^ l1) {
//                 return Err(format!("l2 ^ l1"));
//             }
//             if bf_f >= 9 && is_not_here(l3 ^ l4) {
//                 return Err(format!("l3 ^ l4"));
//             }
//             if bf_f >= 10 && is_not_here(l2 ^ l3) {
//                 return Err(format!("l2 ^ l3 = {}", l2 ^ l3));
//             }
//             if bf_f >= 11 && is_not_here(l1 ^ l4) {
//                 return Err(format!("l1 ^ l4"));
//             }
//             if bf_f >= 12 && is_not_here(l1 ^ l2 ^ l3) {
//                 return Err(format!("l1 ^ l2 ^ l3"));
//             }
//             if bf_f >= 13 && is_not_here(l1 ^ l2 ^ l4) {
//                 return Err(format!("l1 ^ l2 ^ l4"));
//             }
//             if bf_f > 13 {
//                 return Err(format!("need more hashs l, hf = {},{}", bf_l, bf_f));
//             }
//             Ok(())
//         };

//         g(&[
//             0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111,
//             0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b1111111,
//         ])
//         .unwrap();
//         g(&[
//             0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b0000000, 0b0000000, 0b0000000, 0b0000000,
//             0b1111111, 0b1111111, 0b1111111, 0b1111111, 0b0000000, 0b0000000, 0b0000000, 0b0000000,
//         ])
//         .unwrap();
//         g(&[
//             0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b1111111, 0b1111111, 0b1111111, 0b1111111,
//             0b0000000, 0b0000000, 0b0000000, 0b0000000, 0b1111111, 0b1111111, 0b1111111, 0b1111111,
//         ])
//         .unwrap();

//         {
//             let mut c = md5::Context::new();
//             dbg!(42_u32.to_be_bytes());
//             dbg!(42_u32.to_le_bytes());
//             c.consume(42_u32.to_be_bytes());
//             let d = c.compute().0;
//             g(&d).unwrap();
//             dbg!(&d);

//             let mut c2 = md5::Context::new();
//             c2.consume(d);
//             c2.consume("/file.txt");
//             let d = c2.compute().0;
//             g(&d).unwrap();
//             dbg!(&d);
//         }
//         let now = Instant::now();
//         let mut matched_m = 0;
//         let mut unmatched_m = 0;
//         for (src, dst) in mappings.iter() {
//             let mut c = md5::Context::new();
//             let src = formator_src.format(src);
//             src.0.digest(&mut c);
//             let dst = formator_dst.format(dst);
//             dst.0.digest(&mut c);
//             // ControlFlowBuilder.java
//             // 21, 160, 57, 118, 200, 180, 51, 50, 131, 132, 153, 95, 5, 128, 107, 85
//             match g(&c.compute().0) {
//                 Err(e) => {
//                     unmatched_m += 1;
//                     dbg!(src.0.path, src.0.start, src.0.len);
//                     dbg!(dst.0.path);
//                     dbg!(e);
//                     let r = stores.node_store.resolve(src.1);
//                     let t = r.get_type();
//                     dbg!(t);
//                 }
//                 Ok(_) => {
//                     matched_m += 1;
//                     // dbg!(src.0.path,src.0.start,src.0.len);
//                     // dbg!(dst.0.path);
//                     // dbg!(e);
//                 }
//             }
//         }
//         let a = gt_bf ^ hast_bf;
//         dbg!(a.count_ones());
//         assert!(a.not_any(), "{}", a);

//         let bf_mappings_compare_t = now.elapsed().as_secs_f64();
//         dbg!(bf_mappings_compare_t);
//         G {
//             times: timings
//                 .iter()
//                 .map(|x| Duration::from_nanos(*x as u64).as_secs_f64())
//                 .collect(),
//             matched_m,
//             unmatched_m,
//             actions,
//         }
//     };
//     let gt_len = gt_out.actions;
//     let hast_len = actions.len();
//     if !(gt_out.unmatched_m != 0) {
//         dbg!(
//             gt_out.times,
//             &timings,
//             actions.len(),
//             "mapping",
//             format!(
//                 "matched_m={} unmatched_m={}",
//                 gt_out.matched_m, gt_out.unmatched_m,
//             )
//         );
//         CompResult::Failure {
//             timings,
//             actions_hast: hast_len,
//             actions_other: gt_len,
//             stage: "mapping".to_string(),
//             reason: format!(
//                 "matched_m={} unmatched_m={}",
//                 gt_out.matched_m, gt_out.unmatched_m,
//             ),
//             mappings_hast: todo!(),
//             mappings_other: todo!(),
//         }
//     } else if gt_len != hast_len {
//         dbg!(
//             gt_out.times,
//             &timings,
//             actions.len(),
//             "gen",
//             format!("different sizes gt={} hast={}", gt_len, hast_len)
//         );
//         CompResult::Failure {
//             timings,
//             actions_hast: hast_len,
//             actions_other: gt_len,
//             stage: "gen".to_string(),
//             reason: format!("different sizes gt={} hast={}", gt_len, hast_len),
//             mappings_hast: todo!(),
//             mappings_other: todo!(),
//         }
//     } else {
//         dbg!(&timings, gt_out.matched_m, actions.len());
//         CompResult::Success {
//             timings,
//             actions: actions.len(),
//             mappings: todo!(),
//         }
//     }
// }

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
    use super::*;
    pub struct PP0 {
        pub(super) file: std::io::Cursor<Vec<u8>>,
    }

    impl PP0 {
        pub fn counts(mut self) -> (compressed_bf_post_process::PP1, Counts) {
            use byteorder::{BigEndian, ReadBytesExt};
            let actions = self
                .file
                .read_u32::<BigEndian>()
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
                Counts { mappings, actions },
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
        pub fn validity_mappings(
            mut self,
            stores: &SimpleStores,
            src_arena: &CompletePostOrder<NodeIdentifier, u32>,
            src_tr: NodeIdentifier,
            dst_arena: &CompletePostOrder<NodeIdentifier, u32>,
            dst_tr: NodeIdentifier,
            mappings: &VecStore<u32>,
        ) -> ValidityRes<usize> {
            use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
            let bf_f = self.file.read_u32::<BigEndian>().unwrap() as usize;
            let bf_l = self.file.read_u32::<BigEndian>().unwrap() as usize;

            use hyper_ast::types::LabelStore;
            use hyper_ast::types::Labeled;
            use hyper_ast::types::Typed;
            use hyper_ast::types::WithSerialization;
            let now = Instant::now();

            let mut gt_bf = bitvec::bitvec![bitvec::order::Lsb0, u64; 0;bf_l];
            dbg!(gt_bf.len());
            // dbg!(gt_bf.as_raw_slice().len());
            // dbg!(gt_bf.as_raw_slice().len() * 8);
            // dbg!(gt_bf.as_raw_slice().len() * 8 * 8);
            self.file
                .read_u64_into::<LittleEndian>(gt_bf.as_mut_raw_slice())
                .unwrap();
            // dbg!(&gt_bf.as_raw_slice()[0].to_le_bytes());
            let gt_compressed_output_load_t = now.elapsed().as_secs_f64();
            dbg!(gt_compressed_output_load_t);
            let gt_bf = gt_bf;

            let mut hast_bf = bitvec::bitvec![bitvec::order::Lsb0, u64; 0;bf_l];

            #[derive(Clone)]
            struct DPos {
                file: md5::Digest,
                start: u32,
                len: u32,
            }

            impl DPos {
                fn digest(&self, c: &mut md5::Context) {
                    c.consume(self.file.0);
                    c.consume(self.start.to_be_bytes());
                    c.consume((self.start + self.len).to_be_bytes());

                    // d.update(t.getType().toString().getBytes());
                    // if (t.hasLabel()) {
                    //     d.update(t.getLabel().getBytes());
                    // }
                    // d.update(computeFile2(t));
                    // d.update(Ints.toByteArray(t.getPos()));
                    // d.update(Ints.toByteArray(t.getEndPos()));
                }
            }

            impl Default for DPos {
                fn default() -> Self {
                    Self {
                        file: md5::Digest(Default::default()),
                        start: 0,
                        len: 0,
                    }
                }
            }

            let with_p = |mut pos: DPos, ori| {
                let r = stores.node_store.resolve(ori);
                let t = r.get_type();
                if t.is_directory() || t.is_file() {
                    let label = stores.label_store.resolve(&r.get_label());
                    // if pos.path.to_string_lossy().len() == 0 {
                    //     dbg!(&label);
                    //     dbg!(&pos.file.0);
                    // }
                    if label != "" {
                        let mut c = md5::Context::new();
                        // let bo = pos.file.0;
                        c.consume(pos.file.0);
                        c.consume(label);
                        pos.file = c.compute();
                        // if label == "src" {
                        //     dbg!(&pos.file.0);
                        //     // [-128, -25, 54, -113, -44, 107, -11, 49, 60, -13, -45, -116, -68, 38, -107, -30]
                        // }
                    }
                }
                pos.len = r.try_bytes_len().unwrap_or(0).to_u32().unwrap();
                pos
            };
            let with_lsib = |mut pos: DPos, lsib| {
                // assert!(pos.path.to_string_lossy().len()>0, "{:?} {} {}", pos.file.0, pos.start, pos.len);
                pos.start = pos.start + pos.len;
                let r = stores.node_store.resolve(lsib);
                assert!(!r.get_type().is_directory() && !r.get_type().is_file());
                pos.len = r.try_bytes_len().unwrap().to_u32().unwrap();
                pos
            };

            let mut formator_src =
                FormatCached::from((stores, src_arena, src_tr, with_p, with_lsib));
            let mut formator_dst =
                FormatCached::from((stores, dst_arena, dst_tr, with_p, with_lsib));

            let mut is_not_here = |x| {
                hast_bf.set((x % bf_l as u32) as usize, true);
                !gt_bf[(x % bf_l as u32) as usize]
            };
            assert!(!is_not_here(0));
            assert!(!is_not_here(42));
            let mut g = |h: &[u8; 16]| {
                let [l1, l2, l3, l4] = h
                    .array_chunks::<4>()
                    .map(|x| u32::from_be_bytes(*x))
                    .array_chunks::<4>()
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
                g(&d).unwrap();
            }
            let now = Instant::now();
            let mut matched_m = 0;
            let mut unmatched_m = 0;
            for (src, dst) in mappings.iter() {
                let mut c = md5::Context::new();
                let src = formator_src.format(src);
                src.0.digest(&mut c);
                let dst = formator_dst.format(dst);
                dst.0.digest(&mut c);
                // ControlFlowBuilder.java
                // 21, 160, 57, 118, 200, 180, 51, 50, 131, 132, 153, 95, 5, 128, 107, 85
                match g(&c.compute().0) {
                    Err(e) => {
                        unmatched_m += 1;
                        let r = stores.node_store.resolve(src.1);
                        let t = r.get_type();
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
pub struct Counts {
    pub mappings: usize,
    pub actions: usize,
}
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
pub struct SimpleJsonPostProcess {
    file: diff_output::F,
}

impl SimpleJsonPostProcess {
    pub fn new(file: &Path) -> Self {
        let now = Instant::now();
        let gt_out = serde_json::from_reader::<_, diff_output::F>(
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
        let actions = self.file.actions.len();
        Counts { mappings, actions }
    }
    pub fn validity_mappings(
        &self,
        stores: &SimpleStores,
        src_arena: &CompletePostOrder<NodeIdentifier, u32>,
        src_tr: NodeIdentifier,
        dst_arena: &CompletePostOrder<NodeIdentifier, u32>,
        dst_tr: NodeIdentifier,
        mappings: &VecStore<u32>,
    ) -> ValidityRes<Vec<diff_output::Match>> {
        use hyper_ast::types::LabelStore;
        use hyper_ast::types::Labeled;
        use hyper_ast::types::Typed;
        use hyper_ast::types::WithSerialization;
        let with_p = |mut pos: Position, ori| {
            let r = stores.node_store.resolve(ori);
            let t = r.get_type();
            if t.is_directory() || t.is_file() {
                pos.inc_path(stores.label_store.resolve(&r.get_label()));
            }
            pos.set_len(r.try_bytes_len().unwrap_or(0));
            pos
        };
        let with_lsib = |mut pos: Position, lsib| {
            pos.inc_offset(pos.range().end - pos.range().start);
            let r = stores.node_store.resolve(lsib);
            pos.set_len(r.try_bytes_len().unwrap());
            pos
        };
        let mut formator_src = FormatCached::from((stores, src_arena, src_tr, with_p, with_lsib));
        let mut formator_dst = FormatCached::from((stores, dst_arena, dst_tr, with_p, with_lsib));
        let mut formator = |a, b| diff_output::Match {
            src: (stores, formator_src.format(a)).into(),
            dest: (stores, formator_dst.format(b)).into(),
        };
        use hashbrown::HashSet;
        let now = Instant::now();
        let hast_mappings: Vec<diff_output::Match> = mappings
            .iter()
            // .src_to_dst.par_iter().enumerate().filter(|x| *x.1 != 0).map(|(src, dst)| (num_traits::cast(src).unwrap(), *dst - 1))
            .map(|(a, b)| formator(a, b))
            .collect();
        let hast_m_formating_t = now.elapsed().as_secs_f64();
        dbg!(hast_m_formating_t);
        let now = Instant::now();
        dbg!(hast_mappings.len());
        let hast_mappings: HashSet<&diff_output::Match> = hast_mappings.iter().collect();
        let gt_mappings: HashSet<&diff_output::Match> = self.file.matches.iter().collect();
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
// /// Exact comparison of mappings
// ///
// /// Slow for large codebases .ie minutes on something like Spoon.
// /// The main slowing factor is io because the subcommand serialize mappings to a json file then we parse it.
// /// It could be improved using another intermediate representation.
// /// Note. that it would also be more efficient to compare edit scripts,
// /// but the exact positions taken to represent evolutions is different
// /// between gumtree and our implementation.
// fn comparing_outputs(
//     gt_out: &Path,
//     stores: &SimpleStores,
//     src_arena: CompletePostOrder<NodeIdentifier, u32>,
//     src_tr: NodeIdentifier,
//     dst_arena: CompletePostOrder<NodeIdentifier, u32>,
//     dst_tr: NodeIdentifier,
//     mappings: VecStore<u32>,
//     actions: ActionsVec<SimpleAction<LabelIdentifier, u16, NodeIdentifier>>,
//     timings: Vec<f64>,
// ) -> CompResult {
//     let now = Instant::now();
//     let gt_out =
//         serde_json::from_reader::<_, diff_output::F>(File::open(gt_out).expect("should be a file"))
//             .unwrap();
//     let gt_out_parsing_t = now.elapsed().as_secs_f64();
//     dbg!(gt_out_parsing_t);
//     dbg!(&gt_out
//         .times
//         .iter()
//         .map(|x| Duration::from_nanos(*x as u64).as_secs_f64())
//         .collect::<Vec<_>>());
//     use hyper_ast::types::LabelStore;
//     use hyper_ast::types::Labeled;
//     use hyper_ast::types::Typed;
//     use hyper_ast::types::WithSerialization;
//     let with_p = |mut pos: Position, ori| {
//         let r = stores.node_store.resolve(ori);
//         let t = r.get_type();
//         if t.is_directory() || t.is_file() {
//             pos.inc_path(stores.label_store.resolve(&r.get_label()));
//         }
//         pos.set_len(r.try_bytes_len().unwrap_or(0));
//         pos
//     };
//     let with_lsib = |mut pos: Position, lsib| {
//         pos.inc_offset(pos.range().end - pos.range().start);
//         let r = stores.node_store.resolve(lsib);
//         pos.set_len(r.try_bytes_len().unwrap());
//         pos
//     };
//     let mut formator_src = FormatCached::from((stores, &src_arena, src_tr, with_p, with_lsib));
//     let mut formator_dst = FormatCached::from((stores, &dst_arena, dst_tr, with_p, with_lsib));
//     let mut formator = |a, b| diff_output::Match {
//         src: (stores, formator_src.format(a)).into(),
//         dest: (stores, formator_dst.format(b)).into(),
//     };
//     use hashbrown::HashSet;
//     let now = Instant::now();
//     let hast_mappings: Vec<diff_output::Match> = mappings
//         .iter()
//         // .src_to_dst.par_iter().enumerate().filter(|x| *x.1 != 0).map(|(src, dst)| (num_traits::cast(src).unwrap(), *dst - 1))
//         .map(|(a, b)| formator(a, b))
//         .collect();
//     let hast_m_formating_t = now.elapsed().as_secs_f64();
//     dbg!(hast_m_formating_t);
//     let now = Instant::now();
//     dbg!(hast_mappings.len());
//     let hast_mappings: HashSet<&diff_output::Match> = hast_mappings.iter().collect();
//     let gt_mappings: HashSet<&diff_output::Match> = gt_out.matches.iter().collect();
//     let mappings_formating_t = now.elapsed().as_secs_f64();
//     dbg!(mappings_formating_t);
//     let now = Instant::now();
//     let missings_mappings: Vec<_> = gt_mappings.par_difference(&hast_mappings).collect();
//     let additional_mappings: Vec<_> = hast_mappings.par_difference(&gt_mappings).collect();
//     let mappings_compare_t = now.elapsed().as_secs_f64();
//     dbg!(mappings_compare_t);
//     let gt_len = gt_out.actions.len();
//     let hast_len = actions.len();
//     if !(missings_mappings.is_empty() && additional_mappings.is_empty()) {
//         // if missings_mappings.len() < 110 {
//         //     dbg!(&missings_mappings);
//         // }
//         // if additional_mappings.len() < 110 {
//         //     dbg!(&additional_mappings);
//         // } else if src_arena.len() < 20000 && dst_arena.len() < 20000 {
//         //     print_mappings(&dst_arena, &src_arena, &stores, &mappings);
//         // } else {
//         //     // dbg!(&missings_mappings.iter().find(|x|x.src.r#type=="comment"&&x.src.file.contains("src/main/java/spoon/support/compiler/SpoonPom.java")));
//         //     // dbg!(&additional_mappings.iter().find(|x|x.src.r#type=="comment"&&x.src.file.contains("src/main/java/spoon/support/compiler/SpoonPom.java")));
//         //     dbg!(&missings_mappings[..3]);
//         //     dbg!(&additional_mappings[..3]);
//         // }
//         // dbg!(&missings_mappings[..3]);
//         // dbg!(&additional_mappings[..3]);
//         // dbg!(&missings_mappings.iter().find(|x|x.src.r#type=="package_declaration"));
//         // dbg!(&additional_mappings.iter().find(|x|x.src.r#type=="package_declaration"));
//         dbg!(&missings_mappings
//             .iter()
//             .filter(|x| x.src.end < 300)
//             .collect::<Vec<_>>());
//         dbg!(&additional_mappings
//             .iter()
//             .filter(|x| x.src.end < 300)
//             .collect::<Vec<_>>());
//         // print_mappings(&dst_arena, &src_arena, stores, &mappings);
//         dbg!(
//             gt_out
//                 .times
//                 .iter()
//                 .map(|x| Duration::from_nanos(*x as u64).as_secs_f64())
//                 .collect::<Vec<_>>(),
//             &timings,
//             actions.len(),
//             "mapping",
//             format!(
//                 "baseline={} missing={} additional={}",
//                 gt_mappings.len(),
//                 missings_mappings.len(),
//                 additional_mappings.len()
//             )
//         );
//         None
//     } else if gt_len != hast_len {
//         dbg!(
//             gt_out
//                 .times
//                 .iter()
//                 .map(|x| Duration::from_nanos(*x as u64).as_secs_f64())
//                 .collect::<Vec<_>>(),
//             &timings,
//             actions.len(),
//             "gen",
//             format!("different sizes gt={} hast={}", gt_len, hast_len)
//         );
//         None
//     } else {
//         dbg!(&timings, actions.len());
//         Some(format!("{:?},{}", timings, actions.len()))
//         // TODO problem comparing actions, (related to considered position of elements (before, during, after))
//         // dbg!(gt_len, hast_len, missings.len(), additional.len());
//         // dbg!(&missings, &hast_actions);
//         // panic!();
//         // CompResult::Failure {
//         //     timings,
//         //     actions: actions.len(),
//         //     stage: "gen".to_string(),
//         //     reason: format!("{:?};{:?}", missings, additional),
//         // }
//     };
//     todo!()
// }

struct FormatCached<'a, T, F, G> {
    stores: &'a SimpleStores,
    arena: &'a CompletePostOrder<NodeIdentifier, u32>,
    cache: RecCachedProcessor<'a, NodeIdentifier, u32, T, F, G>,
}

impl<'a, T, F: Clone, G: Clone>
    From<(
        &'a SimpleStores,
        &'a CompletePostOrder<NodeIdentifier, u32>,
        NodeIdentifier,
        F,
        G,
    )> for FormatCached<'a, T, F, G>
{
    fn from(
        (stores, arena, tr, with_p, with_lsib): (
            &'a SimpleStores,
            &'a CompletePostOrder<NodeIdentifier, u32>,
            NodeIdentifier,
            F,
            G,
        ),
    ) -> Self {
        Self {
            stores,
            arena,
            cache: RecCachedProcessor::from((arena, tr, with_p, with_lsib)),
        }
    }
}
impl<'a, T: Clone + Default, F, G> FormatCached<'a, T, F, G>
where
    F: Fn(T, NodeIdentifier) -> T,
    G: Fn(T, NodeIdentifier) -> T,
{
    fn format(&mut self, x: u32) -> (T, NodeIdentifier) {
        (
            self.cache.position(&self.stores.node_store, &x).clone(),
            self.arena.original(&x),
        )
    }
}

pub fn print_mappings<
    IdD: PrimInt + Debug,
    M: hyper_gumtree::matchers::mapping_store::MonoMappingStore<Ele = IdD>,
>(
    dst_arena: &CompletePostOrder<NodeIdentifier, IdD>,
    src_arena: &CompletePostOrder<NodeIdentifier, IdD>,
    stores: &SimpleStores,
    mappings: &M,
) {
    let mut mapped = vec![false; dst_arena.len()];
    let src_arena = SimplePreOrderMapper::from(src_arena);
    let dst_arena = DisplayCompletePostOrder {
        inner: dst_arena,
        node_store: &stores.node_store,
        label_store: &stores.label_store,
    }
    .to_string();
    let mappings = src_arena
        .map
        .iter()
        .map(|x| {
            if mappings.is_src(x) {
                let dst = mappings.get_dst(x);
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
        node_store: &stores.node_store,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        algorithms::{self, DiffResult},
        other_tools,
        preprocess::{parse_dir_pair, JavaPreprocessFileSys},
    };
    use hyper_ast::store::{labels::LabelStore, nodes::legion::NodeStore, SimpleStores, TypeStore};

    #[test]
    fn test() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let data_root = root.parent().unwrap().join("gt_datasets/defects4j");
        assert!(data_root.exists());
        let data_root = data_root.as_path();
        std::fs::read_dir(data_root).expect("should be a dir");
        let root_buggy = data_root.join("buggy/Jsoup/92"); // /Jsoup/92
        let root_fixed = data_root.join("fixed/Jsoup/92");
        let src = root_buggy;
        let dst = root_fixed;

        let stores = SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
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
            gt_out_format,
        );

        let DiffResult {
            mapping_durations: [subtree_matcher_t, bottomup_matcher_t],
            src_arena,
            dst_arena,
            mappings,
            actions,
            gen_t,
        } = algorithms::gumtree::diff(
            &java_gen.main_stores,
            &src_tr.compressed_node,
            &dst_tr.compressed_node,
        );

        let hast_timings = vec![subtree_matcher_t, bottomup_matcher_t, gen_t];

        dbg!(&hast_timings);
        let pp = CompressedBfPostProcess::create(&gt_out);
        let (pp, counts) = pp.counts();
        let (pp, gt_timings) = pp.performances();
        let valid = pp.validity_mappings(
            &java_gen.main_stores,
            &src_arena,
            src_tr.compressed_node,
            &dst_arena,
            dst_tr.compressed_node,
            &mappings,
        );
        use hyper_gumtree::actions::Actions as _;
        if valid.additional_mappings > 0 || valid.missing_mappings > 0 {
            dbg!(
                valid.additional_mappings,
                valid.missing_mappings,
                actions.len(),
                counts.actions
            );
            panic!()
        } else if counts.actions != actions.len() {
            dbg!(actions.len(), counts.actions);
            panic!()
        } else {
            println!("gt_tt={:?} evos={}", &gt_timings, counts.actions);
            println!("tt={:?} evos={}", &hast_timings, actions.len())
        }
    }
}
