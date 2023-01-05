use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use hyper_ast::types::{Stored, WithChildren};
use num_traits::{PrimInt, ToPrimitive};

use crate::decompressed_tree_store::{
    DecompressedTreeStore, DecompressedWithParent, PostOrderIterable, ShallowDecompressedTreeStore,
};
use crate::mapping::CmBuilder;
use crate::matchers::mapping_store::{MappingStore, MonoMappingStore};
use crate::tree::tree_path::{CompressedTreePath, TreePath};
use crate::{
    decompressed_tree_store::simple_post_order::SimplePostOrder, matchers::mapping_store::VecStore,
};

use super::{ArenaMStore, CompressedMappingStore, Mree, SimpleCompressedMapping};

// type IdM = usize;

type M<IdM, Idx> = SimpleCompressedMapping<IdM, Idx>;

#[derive(Debug)]
struct Acc<IdM, IdD, Idx> {
    has_mapped: bool,
    // src_parent: IdD,
    direct: Vec<Option<Child<IdM, IdD, Idx>>>,
    additional: Vec<(Idx, Child<IdM, IdD, Idx>)>,
}

#[derive(Debug)]
struct Child<IdM, IdD, Idx> {
    compressed: IdM,
    src_parent: Option<IdD>,
    mapping: Option<ChildMapping<IdD, Idx>>,
}

#[derive(Debug)]
struct ChildMapping<IdD, Idx> {
    src: IdD,
    pos: Idx,
}

pub struct MappedHelper<'a, T: Stored, IdD> {
    dsrc: &'a SimplePostOrder<T, IdD>,
    ddst: &'a SimplePostOrder<T, IdD>,
    mappings: &'a VecStore<IdD>,
}

impl<'m, 'a, T: WithChildren, IdD: PrimInt> MappedHelper<'a, T, IdD>
where
    T::TreeId: Clone + Debug,
    T::ChildIdx: Debug,
    IdD: Hash + Debug,
{
    fn process_direct_children<IdM, B: CmBuilder<IdM, T::ChildIdx>>(
        &self,
        direct: Vec<Option<Child<IdM, IdD, T::ChildIdx>>>,
        builder: &mut B,
        additional: &mut Vec<Vec<Child<IdM, IdD, T::ChildIdx>>>,
        src: Option<IdD>,
    ) {
        for (i, c) in direct.into_iter().enumerate() {
            let i = num_traits::cast(i).unwrap();
            // builder.push(vec![]);
            additional.push(vec![]);
            let Some(c) = c else {continue;};
            self.process_aux(c, src, i, builder, additional);
        }
    }

    fn process_additional_children<IdM, B: CmBuilder<IdM, T::ChildIdx>>(
        &self,
        curr_additional: Vec<(T::ChildIdx, Child<IdM, IdD, T::ChildIdx>)>,
        builder: &mut B,
        additional: &mut Vec<Vec<Child<IdM, IdD, T::ChildIdx>>>,
        src: Option<IdD>,
    ) {
        for (i, c) in curr_additional {
            self.process_aux(c, src, i, builder, additional);
        }
    }

    fn process_aux<IdM, B: CmBuilder<IdM, T::ChildIdx>>(
        &self,
        c: Child<IdM, IdD, T::ChildIdx>,
        src: Option<IdD>,
        i: T::ChildIdx,
        builder: &mut B,
        additional: &mut Vec<Vec<Child<IdM, IdD, T::ChildIdx>>>,
    ) {
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
                // builer[i].push((compressed, vec![pos].into()));
                builder.push(i, compressed, vec![pos].into());
            }
            (
                Child {
                    compressed,
                    src_parent: Some(src_parent),
                    mapping: None,
                },
                Some(src),
            ) if Some(src) == self.dsrc.parent(&src_parent) => {
                // TODO ?  || self.helper.dsrc.is_descendant(&src_parent, &src)
                // builer[i].push((compressed, vec![pos].into()));
                let pos = self.dsrc.position_in_parent(&src_parent).unwrap();
                builder.push(i, compressed, vec![pos].into());
            }
            (c, _) => additional[i.to_usize().unwrap()].push(c),
        }
    }
}

pub struct CompressorHelper<'m, 'a, T: WithChildren, IdD, CM: CompressedMappingStore> {
    cm: &'m mut CM,
    ctx: MappedHelper<'a, T, IdD>,
}

impl<'m, 'a, T: WithChildren, IdD: PrimInt, CM: CompressedMappingStore<Idx = T::ChildIdx>>
    CompressorHelper<'m, 'a, T, IdD, CM>
where
    T::TreeId: Clone + Debug,
    IdD: Hash + Debug,
{
    fn compress_additional_children(
        &mut self,
        additional: Vec<Vec<Child<CM::Id, IdD, <T as WithChildren>::ChildIdx>>>,
        additional_p: &mut Vec<(
            <T as WithChildren>::ChildIdx,
            Child<CM::Id, IdD, <T as WithChildren>::ChildIdx>,
        )>,
        src: Option<IdD>,
        dst: IdD,
    ) {
        let mut grouped = HashMap::<IdD, CM::Builder>::new();
        let dst_pos = self.ctx.ddst.position_in_parent(&dst).unwrap();
        for (i, c) in additional
            .into_iter()
            .enumerate()
            .flat_map(|(i, c)| c.into_iter().map(move |c| (i, c)))
        {
            let Some(m) = &c.mapping else {continue};
            match (src, c.src_parent) {
                (Some(src), Some(src_parent)) if self.ctx.dsrc.parent(&src) == Some(src_parent) => {
                    let mut builder = CM::Builder::default();
                    builder.push(num_traits::cast(i).unwrap(), c.compressed, vec![].into());
                    let compressed = self.cm.insert(builder);
                    let c = Child {
                        compressed,
                        src_parent: Some(src_parent),
                        mapping: Some(ChildMapping {
                            src: num_traits::zero(),
                            pos: m.pos, // TODO not sur it is ok, also need a test where intermediary pos changes
                                        // pos: self.ctx.dsrc.position_in_parent(&src).unwrap(),
                        }),
                    };
                    additional_p.push((dst_pos, c));
                }
                (Some(src), Some(src_parent)) if self.ctx.dsrc.is_descendant(&src, &src_parent) => {
                    dbg!(self.ctx.dsrc.parent(&src));
                    dbg!(self.ctx.dsrc.parent(&src_parent));
                    dbg!(self.ctx.dsrc.is_descendant(&src_parent, &src));
                    assert!(src != src_parent);
                    let mut builder = CM::Builder::default();
                    builder.push(num_traits::cast(i).unwrap(), c.compressed, vec![].into());
                    let compressed = self.cm.insert(builder);
                    let c = Child {
                        compressed,
                        src_parent: Some(src_parent),
                        mapping: None,
                    };
                    additional_p.push((dst_pos, c));
                }
                (Some(src), Some(src_parent)) if self.ctx.dsrc.is_descendant(&src_parent, &src) => {
                    let p = self.ctx.dsrc.path(&src, &src_parent);
                    let p = p.extend(&[m.pos]);
                    grouped.entry(src).or_insert(Default::default()).push(
                        num_traits::cast(i).unwrap(),
                        c.compressed,
                        p,
                    )
                }
                (_, Some(src_parent)) => grouped
                    .entry(src_parent)
                    .or_insert(Default::default())
                    .push(
                        num_traits::cast(i).unwrap(),
                        c.compressed,
                        vec![m.pos].into(),
                    ),
                _ => additional_p.push((num_traits::cast(i).unwrap(), c)),
            };
        }
        if let Some(src) = src {
            if let Some(builder) = grouped.remove(&src) {
                let compressed = self.cm.insert(builder);
                let c = Child {
                    compressed,
                    src_parent: self.ctx.dsrc.parent(&src),
                    mapping: Some(ChildMapping {
                        src: num_traits::zero(),
                        pos: self.ctx.dsrc.position_in_parent(&src).unwrap(),
                    }),
                };
                additional_p.push((dst_pos, c));
            }
        }
        for (src_parent, builder) in grouped {
            let compressed = self.cm.insert(builder);
            let c = Child {
                compressed,
                src_parent: self.ctx.dsrc.parent(&src_parent),
                mapping: Some(ChildMapping {
                    src: num_traits::zero(),
                    pos: self.ctx.dsrc.position_in_parent(&src_parent).unwrap(),
                }),
            };
            additional_p.push((dst_pos, c));
        }
    }
}

pub struct Compressor<'m, 'a, T: WithChildren, IdD, CM: CompressedMappingStore> {
    waiting: HashMap<IdD, Acc<CM::Id, IdD, T::ChildIdx>>,
    helper: CompressorHelper<'m, 'a, T, IdD, CM>,
}

impl<'m, 'a, T: WithChildren, IdD: PrimInt, CM: CompressedMappingStore<Idx = T::ChildIdx>>
    Compressor<'m, 'a, T, IdD, CM>
where
    T::TreeId: Clone + Debug,
    T::ChildIdx: Debug,
    IdD: Hash + Debug,
    CM::Id: Clone + Debug,
{
    pub fn compress(&mut self) -> CM::Id {
        for dst in self.helper.ctx.ddst.iter_df_post::<false>() {
            self.next_po(dst);
        }
        self.finalyze()
    }

    fn next_po(&mut self, dst: IdD)
    where
        IdD: Hash + Debug,
    {
        let curr_waiting = self.waiting.remove(&dst);
        let dst_parent = self.helper.ctx.ddst.parent(&dst).unwrap();
        let waiting_p = self.waiting.entry(dst_parent).or_insert_with(|| Acc {
            has_mapped: false,
            direct: vec![],
            additional: vec![],
        });
        if self.helper.ctx.mappings.is_dst(&dst) {
            // is mapped
            let src = self.helper.ctx.mappings.get_src(&dst);
            let src_parent = self.helper.ctx.dsrc.parent(&src);
            let pos = self.helper.ctx.dsrc.position_in_parent(&src).unwrap();
            dbg!(src, dst);
            dbg!(&curr_waiting);

            let Some(curr_waiting) = curr_waiting else {
                    waiting_p.has_mapped = true;
                    let mut builder = CM::Builder::default();
                    builder.mapped();
                    let compressed = self.helper.cm.insert(builder);
                    waiting_p.direct.push(Some(Child {
                        compressed,
                        src_parent,
                        mapping: Some(ChildMapping {
                            src,
                            pos,
                        })
                    }));
                    return;
                };

            let mut additional = vec![];
            let mut builder = CM::Builder::default();
            builder.mapped();

            self.helper.ctx.process_direct_children(
                curr_waiting.direct,
                &mut builder,
                &mut additional,
                Some(src),
            );
            self.helper.ctx.process_additional_children(
                curr_waiting.additional,
                &mut builder,
                &mut additional,
                Some(src),
            );

            // let builder = M {
            //     is_mapped: true,
            //     mm,
            // };
            self.helper.compress_additional_children(
                additional,
                &mut waiting_p.additional,
                Some(src),
                dst,
            );

            let compressed = self.helper.cm.insert(builder);
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
                    return;
                };
            dbg!(&curr_waiting);
            // if !curr_waiting.has_mapped {
            //     continue;
            // }
            // let mut mm = vec![];
            let mut builder = CM::Builder::default();
            let mut additional = vec![];
            self.helper.ctx.process_direct_children(
                curr_waiting.direct,
                &mut builder,
                &mut additional,
                None,
            );
            // TODO necessary ?
            self.helper.ctx.process_additional_children(
                curr_waiting.additional,
                &mut builder,
                &mut additional,
                None,
            );
            self.helper.compress_additional_children(
                additional,
                &mut waiting_p.additional,
                None,
                dst,
            );
            dbg!(&waiting_p.additional);
            // dbg!(&mm);
            // TODO assert!(builder.iter().all(|l| l.is_empty())); ie. builder only has empty children
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

    fn finalyze(&mut self) -> <CM as CompressedMappingStore>::Id {
        // handle the root
        let dst = self.helper.ctx.ddst.root();
        dbg!(dst);
        let curr_waiting = self.waiting.remove(&dst);
        dbg!(&curr_waiting);
        let mut builder = CM::Builder::default();
        if let Some(curr_waiting) = curr_waiting {
            let mut additional = vec![];
            let src = self
                .helper
                .ctx
                .mappings
                .is_dst(&dst)
                .then(|| self.helper.ctx.mappings.get_src(&dst));

            self.helper.ctx.process_direct_children(
                curr_waiting.direct,
                &mut builder,
                &mut additional,
                src,
            );
            self.helper.ctx.process_additional_children(
                curr_waiting.additional,
                &mut builder,
                &mut additional,
                src,
            );
            for (i, x) in additional.into_iter().enumerate() {
                for x in x {
                    builder.push(
                        num_traits::cast(i).unwrap(),
                        x.compressed,
                        x.mapping.map_or(vec![], |x| vec![x.pos]).into(),
                    );
                }
            }
        }
        if self.helper.ctx.mappings.is_dst(&dst) {
            builder.mapped();
        }
        let compressed = self.helper.cm.insert(builder);
        compressed
    }
}

#[cfg(test)]
mod test {
    use crate::{
        decompressed_tree_store::{CompletePostOrder, DecompressedWithParent, Initializable, PostOrderIterable},
        mapping::{
            compress::{Compressor, CompressorHelper, MappedHelper},
            remapping::Remapper,
            visualize::print_mappings_no_ranges,
            ArenaMStore, CompressedMappingStore,
        },
        matchers::mapping_store::{DefaultMappingStore, MappingStore},
        tests::examples,
        tree::{
            simple_tree::{vpair_to_stores, Tree, TreeRef},
            tree_path::TreePath,
        },
    };

    use crate::decompressed_tree_store::ShallowDecompressedTreeStore;

    #[test]
    fn hands_on() {
        let (label_store, node_store, src, dst) =
            vpair_to_stores((examples::example_move1().0, examples::example_move().1));
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();
        mappings.link(0, 1);
        mappings.link(1, 2);
        mappings.link(3, 0);
        mappings.link(4, 3);
        mappings.link(5, 4);
        // |   5: 0; f       | 4 |   4: 0; f     |
        // |   3:   0; g     | 0 |   0:   0; g   |
        // |   2:     0; i   |   |   3:   0; h   |
        // |   0:       0; d | 1 |   1:     0; d |
        // |   1:       0; e | 2 |   2:     0; e |
        // |   4:   0; h     | 3 |               |

        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        let mut cm = ArenaMStore { v: vec![] };
        let mut compressor = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
        };
        let mut it = compressor.helper.ctx.ddst.iter_df_post::<false>();
        compressor.next_po(it.next().unwrap());
        dbg!(&compressor.waiting);
        compressor.next_po(it.next().unwrap());
        dbg!(&compressor.waiting);
        compressor.next_po(it.next().unwrap());
        dbg!(&compressor.waiting);
        compressor.next_po(it.next().unwrap());
        dbg!(&compressor.waiting);
        dbg!(&compressor.helper.cm.resolve(3));
        assert!(it.next().is_none());
        let compressed_root: usize = compressor.finalyze();
        dbg!(compressed_root);
        panic!();
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
            assert_eq!(0, r1.mm.len());
        }
        let r1 = cm.resolve(r.mm[1][1].0);
        dbg!(r1);
        assert!(!r1.is_mapped);
        assert_eq!(1, r1.mm.len());
        let r2 = cm.resolve(r1.mm[0][0].0);
        dbg!(r2);
        assert!(!r2.is_mapped);
        let r3 = cm.resolve(r2.mm[0][0].0);
        dbg!(r3);
        assert!(r3.is_mapped);
        let r4 = cm.resolve(r2.mm[1][0].0);
        dbg!(r4);
        assert!(r4.is_mapped);

        dbg!(dst_arena.path(&dst_arena.root(), &1));
        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &1);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &2);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(1), remapped.next());
            assert_eq!(None, remapped.next());
        }
    }

    #[test]
    fn test_move_mix2b() {
        let (label_store, node_store, src, dst) =
            vpair_to_stores((examples::example_move1().0, examples::example_move().1));
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();
        mappings.link(0, 1);
        mappings.link(1, 2);
        mappings.link(3, 0);
        mappings.link(4, 3);
        mappings.link(5, 4);
        // |   5: 0; f       | 4 |   4: 0; f     |
        // |   3:   0; g     | 0 |   0:   0; g   |
        // |   2:     0; i   |   |   3:   0; h   |
        // |   0:       0; d | 1 |   1:     0; d |
        // |   1:       0; e | 2 |   2:     0; e |
        // |   4:   0; h     | 3 |               |

        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        let mut cm = ArenaMStore { v: vec![] };
        let compressed_root: usize = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
        }.compress();
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
            assert_eq!(0, r1.mm.len());
        }
        let r1 = cm.resolve(r.mm[1][1].0);
        dbg!(r1);
        assert!(!r1.is_mapped);
        assert_eq!(1, r1.mm.len());
        let r2 = cm.resolve(r1.mm[0][0].0);
        dbg!(r2);
        assert!(!r2.is_mapped);
        let r3 = cm.resolve(r2.mm[0][0].0);
        dbg!(r3);
        assert!(r3.is_mapped);
        let r4 = cm.resolve(r2.mm[1][0].0);
        dbg!(r4);
        assert!(r4.is_mapped);

        dbg!(dst_arena.path(&dst_arena.root(), &1));
        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &1);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &2);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(1), remapped.next());
            assert_eq!(None, remapped.next());
        }
    }

    #[test]
    fn test_move_mix2() {
        let (label_store, node_store, src, dst) =
            vpair_to_stores((examples::example_move().0, examples::example_move1().1));
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();
        mappings.link(0, 1);
        mappings.link(1, 2);
        mappings.link(2, 0);
        mappings.link(3, 4);
        mappings.link(4, 5);
        // |   4: 0; f     | 5 |   5: 0; f       |
        // |   2:   0; g   | 0 |   0:   0; g     |
        // |   0:     0; d | 1 |   4:   0; h     |
        // |   1:     0; e | 2 |   3:     0; i   |
        // |   3:   0; h   | 4 |   1:       0; d |
        // |               |   |   2:       0; e |

        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        let mut cm = ArenaMStore { v: vec![] };
        let compressed_root: usize = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
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
            assert_eq!(0, r1.mm.len());
        }
        let r1 = cm.resolve(r.mm[1][1].0);
        dbg!(r1);
        assert!(!r1.is_mapped);
        assert_eq!(1, r1.mm.len());
        let r2 = cm.resolve(r1.mm[0][0].0);
        dbg!(r2);
        assert!(!r2.is_mapped);
        assert_eq!(2, r2.mm.len());
        let r3 = cm.resolve(r2.mm[0][0].0);
        dbg!(r3);
        assert!(r3.is_mapped);
        let r4 = cm.resolve(r2.mm[1][0].0);
        dbg!(r4);
        assert!(r4.is_mapped);
        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &1);
            dbg!(&path); // 1.0.0
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            // TODO should be 0.0
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            // assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &2);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            // assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(1), remapped.next());
            assert_eq!(None, remapped.next());
        }
    }

    #[test]
    fn test_move1b() {
        let (label_store, node_store, src, dst) = vpair_to_stores(examples::example_move1());
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        mappings.link(0, 1);
        mappings.link(1, 2);
        // mappings.link(2, 3);
        mappings.link(3, 0);
        mappings.link(4, 4);
        mappings.link(5, 5);

        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        let mut cm = ArenaMStore { v: vec![] };
        let compressed_root: usize = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
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
            assert_eq!(0, r1.mm.len());
        }
        let r1 = cm.resolve(r.mm[1][1].0);
        dbg!(r1);
        assert!(!r1.is_mapped);
        assert_eq!(1, r1.mm.len());
        let r2 = cm.resolve(r1.mm[0][0].0);
        dbg!(r2);
        assert!(!r2.is_mapped);
        let r3 = cm.resolve(r2.mm[0][0].0);
        dbg!(r3);
        assert!(r3.is_mapped);
        let r4 = cm.resolve(r2.mm[1][0].0);
        dbg!(r4);
        assert!(r4.is_mapped);
        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &1);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &2);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(1), remapped.next());
            assert_eq!(None, remapped.next());
        }
    }

    #[test]
    fn test_move1() {
        let (label_store, node_store, src, dst) = vpair_to_stores(examples::example_move1());
        let mut mappings = DefaultMappingStore::default();
        let src_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<TreeRef<Tree>, u16>::new(&node_store, &dst);
        mappings.topit(src_arena.len(), dst_arena.len());
        mappings.link(0, 1);
        mappings.link(1, 2);
        mappings.link(2, 3);
        mappings.link(3, 0);
        mappings.link(4, 4);
        mappings.link(5, 5);

        print_mappings_no_ranges(&dst_arena, &src_arena, &node_store, &label_store, &mappings);
        println!();

        let mut cm = ArenaMStore { v: vec![] };
        let compressed_root: usize = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
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
            assert_eq!(0, r1.mm.len());
        }
        let r1 = cm.resolve(r.mm[1][1].0);
        dbg!(r1);
        assert!(!r1.is_mapped);
        assert_eq!(1, r1.mm.len());
        let r2 = cm.resolve(r1.mm[0][0].0);
        dbg!(r2);
        assert!(r2.is_mapped);
        let r3 = cm.resolve(r2.mm[0][0].0);
        dbg!(r3);
        assert!(r3.is_mapped);
        let r4 = cm.resolve(r2.mm[1][0].0);
        dbg!(r4);
        assert!(r4.is_mapped);
        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &1);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &2);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(1), remapped.next());
            assert_eq!(None, remapped.next());
        }
    }

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
        let compressed_root: usize = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
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
            assert_eq!(0, r1.mm.len());
            assert_eq!(vec![1], r.mm[1][0].1.iter().collect::<Vec<_>>());
        }
        let r1 = cm.resolve(r.mm[1][1].0);
        dbg!(r1);
        assert!(!r1.is_mapped);
        assert_eq!(2, r1.mm.len());
        assert_eq!(vec![0], r.mm[1][1].1.iter().collect::<Vec<_>>());
        let r2 = cm.resolve(r1.mm[0][0].0);
        dbg!(r2);
        assert!(r2.is_mapped);
        let r3 = cm.resolve(r1.mm[1][0].0);
        dbg!(r3);
        assert!(r3.is_mapped);
        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &1);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &2);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(Some(1), remapped.next());
            assert_eq!(None, remapped.next());
        }
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
        let compressed_root: usize = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
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
        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
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
        let compressed_root: usize = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
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
        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
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
        let compressed_root: usize = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
        }
        .compress();
        dbg!(compressed_root);
        let r = cm.resolve(compressed_root);
        dbg!(r);
        assert!(r.is_mapped);
        assert_eq!(2, r.mm.len());

        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(0), remapped.next());
            assert_eq!(None, remapped.next());
        }
        {
            let path = dst_arena.path(&dst_arena.root(), &1);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            assert_eq!(Some(1), remapped.next());
            assert_eq!(None, remapped.next());
        }
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
        let compressed_root: usize = Compressor {
            helper: CompressorHelper {
                cm: &mut cm,
                ctx: MappedHelper {
                    dsrc: &src_arena,
                    ddst: &dst_arena,
                    mappings: &mappings,
                },
            },
            waiting: Default::default(),
        }
        .compress();
        dbg!(compressed_root);
        let r = cm.resolve(compressed_root);
        assert!(r.mm.is_empty());
        assert!(r.is_mapped);

        {
            let path = dst_arena.path(&dst_arena.root(), &0);
            dbg!(&path);
            let mut remapped = Remapper::new(&cm, compressed_root, path.into_iter());
            while let Some(i) = remapped.next() {
                dbg!(i);
            }
        }
    }
}
