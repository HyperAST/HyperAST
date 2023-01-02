use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use hyper_ast::types::{Stored, WithChildren};
use num_traits::{PrimInt, ToPrimitive};

use crate::decompressed_tree_store::{
    DecompressedTreeStore, DecompressedWithParent, PostOrderIterable, ShallowDecompressedTreeStore,
};
use crate::matchers::mapping_store::{MappingStore, MonoMappingStore};
use crate::tree::tree_path::CompressedTreePath;
use crate::{
    decompressed_tree_store::simple_post_order::SimplePostOrder, matchers::mapping_store::VecStore,
};

use super::{ArenaMStore, SimpleCompressedMapping};

type IdM = usize;

type M<IdM, Idx> = SimpleCompressedMapping<IdM, Idx>;

#[derive(Debug)]
struct Acc<IdD, Idx> {
    has_mapped: bool,
    // src_parent: IdD,
    direct: Vec<Option<Child<IdD, Idx>>>,
    additional: Vec<(Idx, Child<IdD, Idx>)>,
}

#[derive(Debug)]
struct Child<IdD, Idx> {
    compressed: IdM,
    src_parent: Option<IdD>,
    mapping: Option<ChildMapping<IdD, Idx>>,
}

#[derive(Debug)]
struct ChildMapping<IdD, Idx> {
    src: IdD,
    pos: Idx,
}

struct MappedHelper<'a, T: Stored, IdD> {
    dsrc: &'a SimplePostOrder<T, IdD>,
    ddst: &'a SimplePostOrder<T, IdD>,
    mappings: &'a VecStore<IdD>,
}

struct Compressor<'m, 'a, T: WithChildren, IdD> {
    cm: &'m mut ArenaMStore<M<IdM,T::ChildIdx>>,
    helper: MappedHelper<'a, T, IdD>,
}

impl<'m, 'a, T: WithChildren, IdD: PrimInt> MappedHelper<'a, T, IdD>
where
    T::TreeId: Clone + Debug,
    T::ChildIdx: Debug,
    IdD: Hash + Debug,
{
    fn process_direct_children(
        &self,
        direct: Vec<Option<Child<IdD, T::ChildIdx>>>,
        mm: &mut Vec<Vec<(IdM, CompressedTreePath<T::ChildIdx>)>>,
        additional: &mut Vec<Vec<Child<IdD, T::ChildIdx>>>,
        src: Option<IdD>,
    ) {
        for c in direct {
            let i = num_traits::cast(mm.len()).unwrap();
            mm.push(vec![]);
            additional.push(vec![]);
            let Some(c) = c else {continue;};
            self.process_aux(c, src, i, mm, additional);
        }
    }

    fn process_aux(
        &self,
        c: Child<IdD, T::ChildIdx>,
        src: Option<IdD>,
        i: T::ChildIdx,
        mm: &mut Vec<Vec<(usize, CompressedTreePath<T::ChildIdx>)>>,
        additional: &mut Vec<Vec<Child<IdD, T::ChildIdx>>>,
    ) {
        let i = i.to_usize().unwrap();
        match (c, src) {
            (
                Child {
                    compressed,
                    src_parent: Some(src_parent),
                    mapping: Some(ChildMapping { src: _, pos }),
                },
                Some(src),
            ) if src == src_parent => {
                // TODO ?  || self.helper.dsrc.is_descendant(&src_parent, &src)
                mm[i].push((compressed, vec![pos].into()));
            }
            (c, _) => additional[i].push(c),
        }
    }

    fn process_additional_children(
        &self,
        curr_additional: Vec<(T::ChildIdx, Child<IdD, T::ChildIdx>)>,
        mm: &mut Vec<Vec<(IdM, CompressedTreePath<T::ChildIdx>)>>,
        additional: &mut Vec<Vec<Child<IdD, T::ChildIdx>>>,
        src: Option<IdD>,
    ) {
        for (i, c) in curr_additional {
            self.process_aux(c, src, i, mm, additional);
        }
    }
}

impl<'m, 'a, T: WithChildren, IdD: PrimInt> Compressor<'m, 'a, T, IdD>
where
    T::TreeId: Clone + Debug,
    T::ChildIdx: Debug,
    IdD: Hash + Debug,
{
    pub fn compress(&mut self) -> IdM {
        let mut waiting: HashMap<IdD, Acc<IdD, T::ChildIdx>> = Default::default();

        for dst in self.helper.ddst.iter_df_post::<false>() {
            let curr_waiting = waiting.remove(&dst);
            let dst_parent = self.helper.ddst.parent(&dst).unwrap();
            let waiting_p = waiting.entry(dst_parent).or_insert_with(|| Acc {
                has_mapped: false,
                direct: vec![],
                additional: vec![],
            });
            if self.helper.mappings.is_dst(&dst) {
                // is mapped
                let src = self.helper.mappings.get_src(&dst);
                let src_parent = self.helper.dsrc.parent(&src);
                let pos = self.helper.dsrc.position_in_parent(&src).unwrap();
                dbg!(src, dst);
                dbg!(&curr_waiting);

                let Some(curr_waiting) = curr_waiting else {
                        waiting_p.has_mapped = true;
                        let compressed = self.cm.insert(M {
                            is_mapped: true,
                            mm: vec![],
                        });
                        waiting_p.direct.push(Some(Child {
                            compressed,
                            src_parent,
                            mapping: Some(ChildMapping {
                                src,
                                pos,
                            })
                        }));
                        continue;
                    };

                let mut additional = vec![];
                let mut mm = vec![];
                self.helper.process_direct_children(
                    curr_waiting.direct,
                    &mut mm,
                    &mut additional,
                    Some(src),
                );
                self.helper.process_additional_children(
                    curr_waiting.additional,
                    &mut mm,
                    &mut additional,
                    Some(src),
                );

                let node = M {
                    is_mapped: true,
                    mm,
                };
                self.compress_additional_children(
                    additional,
                    &mut waiting_p.additional,
                    Some(src),
                    dst,
                );

                let compressed = self.cm.insert(node);
                waiting_p.direct.push(Some(Child {
                    compressed,
                    src_parent,
                    mapping: Some(ChildMapping { src, pos }),
                }));
                waiting_p.has_mapped = true;
            } else {
                // is not mapped
                dbg!(dst);

                let Some(curr_waiting) = curr_waiting else {
                        waiting_p.direct.push(None);
                        continue;
                    };
                dbg!(&curr_waiting);
                // if !curr_waiting.has_mapped {
                //     continue;
                // }
                let mut mm = vec![];
                let mut additional = vec![];
                self.helper.process_direct_children(
                    curr_waiting.direct,
                    &mut mm,
                    &mut additional,
                    None,
                );
                // TODO necessary ?
                self.helper.process_additional_children(
                    curr_waiting.additional,
                    &mut mm,
                    &mut additional,
                    None,
                );
                self.compress_additional_children(additional, &mut waiting_p.additional, None, dst);
                dbg!(&waiting_p.additional);
                dbg!(&mm);
                assert!(mm.iter().all(|l| l.is_empty()));
                // dbg!(&mm);
                // let node = SimpleCompressedMapping {
                //     is_mapped: false,
                //     mm,
                // };
                // let compressed = self.cm.insert(node);
                waiting_p.direct.push(None);
                // waiting_p.direct.push(Some(Child {
                //     compressed,
                //     src_parent: None,
                //     mapping: None,
                // }));
            }
        }
        // handle the root
        let dst = self.helper.ddst.root();
        dbg!(dst);
        let curr_waiting = waiting.remove(&dst);
        dbg!(&curr_waiting);

        let mut mm = vec![];

        if let Some(curr_waiting) = curr_waiting {
            let mut additional = vec![];
            let src = self
                .helper
                .mappings
                .is_dst(&dst)
                .then(|| self.helper.mappings.get_src(&dst));

            self.helper
                .process_direct_children(curr_waiting.direct, &mut mm, &mut additional, src);
            self.helper.process_additional_children(
                curr_waiting.additional,
                &mut mm,
                &mut additional,
                src,
            );
            // let mut remaining = vec![];
            // self.compress_additional_children(additional, &mut remaining, src, dst);
            for (i, x) in additional.into_iter().enumerate() {
                for x in x {
                    mm[i].push((
                        x.compressed,
                        x.mapping.map_or(vec![], |x| vec![x.pos]).into(),
                    ));
                }
            }
        }
        let root = M {
            is_mapped: self.helper.mappings.is_dst(&dst),
            mm,
        };
        let compressed = self.cm.insert(root);
        compressed
    }

    fn compress_additional_children(
        &mut self,
        additional: Vec<Vec<Child<IdD, <T as WithChildren>::ChildIdx>>>,
        additional_p: &mut Vec<(
            <T as WithChildren>::ChildIdx,
            Child<IdD, <T as WithChildren>::ChildIdx>,
        )>,
        src: Option<IdD>,
        dst: IdD,
    ) {
        type MM<Idx> = Vec<Vec<(IdM, CompressedTreePath<Idx>)>>;
        let mut grouped = HashMap::<IdD, MM<T::ChildIdx>>::new();
        for (i, c) in additional.into_iter().enumerate() {
            for c in c {
                if let Some(m) = &c.mapping {
                    // if m.src_parent
                    //     .map_or(false, |x| self.helper.dsrc.is_descendant(&src.unwrap(), &x))
                    // {
                    //     // TODO ?
                    // }
                    // if m.src_parent
                    //     .map_or(false, |x| self.helper.dsrc.is_descendant(&x, &src.unwrap()))
                    // {
                    //     // TODO ?
                    // }
                    if let Some(src_parent) = c.src_parent {
                        let mm = grouped.entry(src_parent).or_insert(vec![]);
                        if i <= mm.len() {
                            mm.resize(i + 1, vec![])
                        }
                        mm[i].push((c.compressed, vec![m.pos].into()));
                    }
                }
            }
        }
        let pos = self.helper.ddst.position_in_parent(&dst).unwrap();
        for (src_parent, mm) in grouped {
            let node = SimpleCompressedMapping {
                is_mapped: false,
                mm,
            };
            let compressed = self.cm.insert(node);
            let c = Child {
                compressed,
                src_parent: self.helper.dsrc.parent(&src_parent),
                mapping: Some(ChildMapping {
                    src: num_traits::zero(),
                    pos,
                }),
            };
            additional_p.push((pos, c));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        decompressed_tree_store::{CompletePostOrder, Initializable},
        mapping::{
            compress::{Compressor, MappedHelper},
            visualize::print_mappings_no_ranges,
            ArenaMStore, MS,
        },
        matchers::{
            mapping_store::{DefaultMappingStore, MappingStore},
        },
        tests::examples,
        tree::simple_tree::{vpair_to_stores, Tree, TreeRef},
    };
    // TODO remove usage of matchers in those unit tests, better move that to integration tests.

    use crate::decompressed_tree_store::ShallowDecompressedTreeStore;

    #[test]
    fn test_move() {
        let (label_store, node_store, src, dst) = vpair_to_stores(examples::example_move());
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        mappings.link(0, 1);
        mappings.link(1, 2);
        mappings.link(2, 0);
        mappings.link(3, 3);
        mappings.link(4, 4);

        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        let mut cm = ArenaMStore { v: vec![] };
        let compressed_root = Compressor {
            cm: &mut cm,
            helper: MappedHelper {
                dsrc: &src_arena,
                ddst: &dst_arena,
                mappings: &mappings,
            },
        }
        .compress();
        dbg!(compressed_root);
        let r = cm.resolve(compressed_root);
        dbg!(r);
        assert!(r.is_mapped);
        assert_eq!(2, r.mm.len());
        assert_eq!(1, r.mm[0].len());
        {
            let r0 = cm.resolve(r.mm[0][0].0);
            assert!(r0.is_mapped, "{:?}", r0);
            assert!(r0.mm.is_empty(), "{:?}", r0);
        }
        assert_eq!(2, r.mm[1].len());
        {
            let r1 = cm.resolve(r.mm[1][0].0);
            dbg!(r1);
            assert!(r1.is_mapped);
            assert_eq!(2, r1.mm.len());
        }
        let r1 = cm.resolve(r.mm[1][1].0);
        dbg!(r1);
        assert!(!r1.is_mapped);
        assert_eq!(2, r1.mm.len());
        let r2 = cm.resolve(r1.mm[0][0].0);
        dbg!(r2);
        assert!(r2.is_mapped);
        let r3 = cm.resolve(r1.mm[1][0].0);
        dbg!(r3);
        assert!(r3.is_mapped);
    }

    #[test]
    fn test_simple1a() {
        let (label_store, node_store, src, dst) = vpair_to_stores(examples::example_simple1());
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        mappings.link(0, 0);
        mappings.link(1, 1);
        mappings.link(3, 3);

        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        let mut cm = ArenaMStore { v: vec![] };
        let compressed_root = Compressor {
            cm: &mut cm,
            helper: MappedHelper {
                dsrc: &src_arena,
                ddst: &dst_arena,
                mappings: &mappings,
            },
        }
        .compress();
        dbg!(compressed_root);
        let r = cm.resolve(compressed_root);
        dbg!(r);
        assert!(r.is_mapped);
        assert_eq!(1, r.mm.len());
        assert_eq!(1, r.mm[0].len());
        let r1 = cm.resolve(r.mm[0][0].0);
        dbg!(r1);
        assert!(!r1.is_mapped);
        assert_eq!(2, r1.mm.len());
        let r2 = cm.resolve(r1.mm[0][0].0);
        dbg!(r2);
        assert!(r2.is_mapped);
        let r3 = cm.resolve(r1.mm[1][0].0);
        dbg!(r3);
        assert!(r3.is_mapped);
    }

    #[test]
    fn test_simple1() {
        let (label_store, node_store, src, dst) = vpair_to_stores(examples::example_simple1());
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        mappings.link(0, 0);
        mappings.link(1, 1);
        mappings.link(2, 2);
        mappings.link(3, 3);
        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        // let mut mappings = mappings;
        // mappings.link(src, dst);

        let mut cm = ArenaMStore { v: vec![] };
        let compressed_root = Compressor {
            cm: &mut cm,
            helper: MappedHelper {
                dsrc: &src_arena,
                ddst: &dst_arena,
                mappings: &mappings,
            },
        }
        .compress();
        dbg!(compressed_root);
        let r = cm.resolve(compressed_root);
        dbg!(r);
        assert!(r.is_mapped);
        assert_eq!(1, r.mm.len());
        let r1 = cm.resolve(r.mm[0][0].0);
        dbg!(r1);
        assert!(r1.is_mapped);
        assert_eq!(2, r1.mm.len());
    }

    #[test]
    fn test_simple() {
        let (label_store, node_store, src, dst) = vpair_to_stores(examples::example_simple());
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        mappings.link(0, 0);
        mappings.link(1, 1);
        mappings.link(2, 2);
        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        // let mut mappings = mappings;
        // mappings.link(src, dst);

        let mut cm = ArenaMStore { v: vec![] };
        let compressed_root = Compressor {
            cm: &mut cm,
            helper: MappedHelper {
                dsrc: &src_arena,
                ddst: &dst_arena,
                mappings: &mappings,
            },
        }
        .compress();
        dbg!(compressed_root);
        let r = cm.resolve(compressed_root);
        dbg!(r);
        assert!(r.is_mapped);
        assert_eq!(2, r.mm.len());
    }

    #[test]
    fn test_single() {
        let (label_store, node_store, src, dst) = vpair_to_stores(examples::example_single());
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        mappings.link(0, 0);
        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        // for (src,dst) in mappings._iter() {
        //     println!("mappings.link({},{});",src,dst);
        // }

        let mut cm = ArenaMStore { v: vec![] };
        let compressed_root = Compressor {
            cm: &mut cm,
            helper: MappedHelper {
                dsrc: &src_arena,
                ddst: &dst_arena,
                mappings: &mappings,
            },
        }
        .compress();
        dbg!(compressed_root);
        let r = cm.resolve(compressed_root);
        assert!(r.mm.is_empty());
        assert!(r.is_mapped);
    }
}
