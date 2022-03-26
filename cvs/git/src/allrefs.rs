// use std::{
//     collections::{BTreeMap, HashMap},
//     env,
//     fmt::{self, Debug},
//     fs,
//     io::{stdout, Write},
//     iter::Peekable,
//     ops::AddAssign,
//     path::{Components, Path, PathBuf},
//     time::{Duration, Instant, SystemTime, UNIX_EPOCH},
// };

// use git2::{ObjectType, Oid, RemoteCallbacks, Repository, Revwalk, TreeEntry};
// use hyper_ast::{
//     filter::{Bloom, BF},
//     hashed::{self, SyntaxNodeHashs},
//     store::{
//         labels::{DefaultLabelValue, LabelStore},
//         nodes::legion::{compo, EntryRef, NodeStore, CS},
//         nodes::{legion, DefaultNodeIdentifier as NodeIdentifier},
//         TypeStore,
//     },
//     tree_gen::SubTreeMetrics,
//     types::{LabelStore as _, Labeled, Tree, Type, Typed, WithChildren}, position::{Position, extract_position},
// };
// use log::info;
// use rusted_gumtree_core::tree::tree::LabelStore as _;
// use rusted_gumtree_gen_ts_java::{
//     filter::BloomSize,
//     impact::{
//         declaration::ExplorableDecl,
//         element::{ExplorableRef, IdentifierFormat, LabelPtr, RefPtr, RefsEnum},
//         partial_analysis::PartialAnalysis,
//         usage::{self, remake_pkg_ref, IterDeclarations},
//     },
//     java_tree_gen_full_compress_legion_ref::{self, hash32},
//     tree_gen::TreeGen,
// };

use std::{collections::HashMap, fmt::Display, time::Instant};

use hyper_ast::{
    position::{
        extract_position, ExploreStructuralPositions, Position, Scout, StructuralPosition,
        StructuralPositionStore,
    },
    store::defaults::NodeIdentifier,
    types::{LabelStore, Labeled, Type, Typed, WithChildren},
};
use rusted_gumtree_gen_ts_java::impact::{
    element::{IdentifierFormat, LabelPtr, RefsEnum},
    partial_analysis::PartialAnalysis,
    usage::{self, remake_pkg_ref, IterDeclarations, IterDeclarations2},
};

use crate::{maven::IterMavenModules, preprocessed::PreProcessedRepository};

/// find all referencial relations in a commit of a preprocessed repository
pub struct AllRefsFinder<'a> {
    prepro: &'a PreProcessedRepository,
    ana: PartialAnalysis,
    structural_positions: StructuralPositionStore,
    relations: HashMap<usize, Vec<usize>>,
}

impl<'a> AllRefsFinder<'a> {
    pub fn new(prepro: &'a PreProcessedRepository) -> Self {
        Self {
            prepro,
            ana: PartialAnalysis::default(),
            structural_positions: Default::default(),
            relations: Default::default(),
        }
    }

    pub fn find_references_to_declarations(&mut self, root: NodeIdentifier) {
        self.structural_positions = StructuralPosition::new(root).into();
        let mut m_it = IterMavenModules::new(&self.prepro.main_stores, root);
        loop {
            let d = if let Some(d) = m_it.next() { d } else { break };
            // m_it.parents();
            {
                assert_eq!(m_it.parents().len() + 1, m_it.offsets().len());
                let mut scout = if d == root {
                    Scout::from((StructuralPosition::from((vec![], vec![])), 0))
                } else {
                    let scout = Scout::from((
                        StructuralPosition::from((
                            m_it.parents()[1..].to_vec(),
                            m_it.offsets()[1..].to_vec(),
                            d,
                        )),
                        0,
                    ));
                    assert!(scout.check(&self.prepro.main_stores));
                    scout
                };
                let src = self.prepro.child_by_name_with_idx(d, "src");

                let d = src.and_then(|(d, i)| {
                    scout.goto(d, i);
                    assert!(scout.check(&self.prepro.main_stores));
                    self.prepro.child_by_name_with_idx(d, "main")
                });
                let d = d.and_then(|(d, i)| {
                    scout.goto(d, i);
                    assert!(scout.check(&self.prepro.main_stores));
                    self.prepro.child_by_name_with_idx(d, "java")
                });
                // let s = s.and_then(|d| self.child_by_type(d, &Type::Directory));
                if let Some((d, i)) = d {
                    scout.goto(d, i);
                    assert!(scout.check(&self.prepro.main_stores));
                    // let n = self.prepro.hyper_ast.main_stores().node_store.resolve(d);
                    // println!(
                    //     "search in module/src/main/java {}",
                    //     self.prepro.hyper_ast
                    //         .main_stores
                    //         .label_store
                    //         .resolve(n.get_label())
                    // );
                    // usage::find_all_decls(&self.prepro.hyper_ast.main_stores, &mut self.ana, s);
                    self.find_references_to_declarations_aux(scout)
                }
            }
            // let s = src.and_then(|d| self.prepro.child_by_name(d, "test"));
            // let s = s.and_then(|d| self.prepro.child_by_name(d, "java"));
            // // let s = s.and_then(|d| self.child_by_type(d, &Type::Directory));
            // if let Some(s) = s {
            //     // let n = self.prepro.hyper_ast.main_stores().node_store.resolve(d);
            //     // println!(
            //     //     "search in module/src/test/java {}",
            //     //     self.prepro.hyper_ast
            //     //         .main_stores
            //     //         .label_store
            //     //         .resolve(n.get_label())
            //     // );
            //     // let mut d_it = IterDeclarations::new(&self.prepro.hyper_ast.main_stores, s);
            //     self.find_references_to_declarations_aux(s)
            // }
        }
    }

    fn find_references_to_declarations_aux(&mut self, mut s: Scout) {
        // let mut d_it = IterDeclarations::new(&self.prepro.main_stores, s.node(&self.structural_positions));
        println!("scout: {:?}", s);
        assert!(self.structural_positions.check(&self.prepro.main_stores));
        assert!(s.check(&self.prepro.main_stores));
        let structural_positions_root = self.structural_positions.push(&mut s);
        assert!(self.structural_positions.check(&self.prepro.main_stores));
        let it = ExploreStructuralPositions::from((
            &self.structural_positions,
            structural_positions_root,
        ));
        let mut d_it ={
            let n = s.node(&self.structural_positions);
            IterDeclarations2::new(&self.prepro.main_stores, s,n)
        };
        // self.structural_positions.check(&self.prepro.main_stores);
        println!("search from {:?}", it.to_position(&self.prepro.main_stores));
        loop {
            if let Some(x) = d_it.next() {
                let b = self.prepro.main_stores.node_store.resolve(x.node(&self.structural_positions));
                let t = b.get_type();
                let now = Instant::now();
                if &t == &Type::ClassDeclaration {
                    // assert_eq!(d_it.parents().len() + 1, d_it.offsets().len());
                    // println!("{:?} {:?} {:?} {:?}", s.node(&self.structural_positions), d_it.parents().to_vec(), x, d_it.offsets().to_vec());
                    // {
                    //     let nodes =d_it.parents()[1..].to_vec();
                    //     let offsets = d_it.offsets()[1..].to_vec();
                    //     let mut i = nodes.len() -1;
                    //     while i > 0 {
                    //         let e = nodes[i];
                    //         let o = offsets[i]-1;
                    //         let p = nodes[i-1];
                    //         let b = self.prepro.main_stores.node_store.resolve(p);
                    //         assert_eq!(e,b.get_child(&(o.to_u16().expect("TODO"))));
                    //         i-=1;
                    //     }
                    // }
                    
                    let mut decl_scout = x;
                    // let mut decl_scout = Scout::from((
                    //     StructuralPosition::from((
                    //         d_it.parents()[1..].to_vec(),
                    //         d_it.offsets()[1..].to_vec(),
                    //         x,
                    //     )),
                    //     structural_positions_root,
                    // ));
                    assert!(self.structural_positions.check(&self.prepro.main_stores));
                    println!("{:?}",decl_scout);
                    assert!(decl_scout.check(&self.prepro.main_stores));
                    let key = self.structural_positions.push(&mut decl_scout);
                    assert!(self.structural_positions.check(&self.prepro.main_stores));
                    let it = ExploreStructuralPositions::from((&self.structural_positions, key));
                    let r = self.relations.entry(key).or_insert(vec![]);
                    // let mut position =
                    //     extract_position(&self.prepro.main_stores, d_it.parents(), d_it.offsets());
                    // position.set_len(b.get_bytes_len() as usize);
                    // println!("now search for {:?} at {:?}", &t, position);
                    // self.structural_positions.check(&self.prepro.main_stores);
                    println!(
                        "now search for {:?} at {:?}",
                        &t,
                        it.to_position(&self.prepro.main_stores)
                    );
                    {
                        let i = self.ana.solver.intern(RefsEnum::MaybeMissing);
                        let i = self.ana.solver.intern(RefsEnum::This(i));
                        println!("try search this");
                        r.extend(
                            usage::RefsFinder::new(
                                &self.prepro.main_stores,
                                &mut self.ana,
                                &mut self.structural_positions,
                            )
                            .find_all(i, decl_scout.clone()),
                        );
                        // let v = self.structural_positions.get_positions(stores, v);
                        // usage::find_refs(
                        //     &self.prepro.main_stores,
                        //     &mut self.ana,
                        //     &mut d_it.position(x),
                        //     i,
                        //     x,
                        // );
                    }
                    let mut l = None;

                    for xx in b.get_children() {
                        let bb = self.prepro.main_stores.node_store.resolve(*xx);
                        if bb.get_type() == Type::Identifier {
                            let i = bb.get_label();
                            l = Some(*i);
                        }
                    }
                    if let Some(i) = l {
                        let o = self.ana.solver.intern(RefsEnum::MaybeMissing);
                        let f = self.prepro.main_stores.label_store.resolve(&i);
                        println!("search uses of {:?}", f);
                        let f = IdentifierFormat::from(f);
                        let l = LabelPtr::new(i, f);
                        let i = self.ana.solver.intern(RefsEnum::ScopedIdentifier(o, l));
                        println!("try search {:?}", &mut self.ana.solver.nodes.with(i));

                        r.extend(
                            usage::RefsFinder::new(
                                &self.prepro.main_stores,
                                &mut self.ana,
                                &mut self.structural_positions,
                            )
                            .find_all(i, decl_scout.clone()),
                        );
                        // usage::find_refs(
                        //     &self.prepro.main_stores,
                        //     &mut self.ana,
                        //     &mut d_it.position(x),
                        //     i,
                        //     x,
                        // );
                        {
                            let i = self.ana.solver.intern(RefsEnum::This(i));
                            println!("try search {:?}", &mut self.ana.solver.nodes.with(i));
                            r.extend(
                                usage::RefsFinder::new(
                                    &self.prepro.main_stores,
                                    &mut self.ana,
                                    &mut self.structural_positions,
                                )
                                .find_all(i, decl_scout.clone()),
                            );
                            // usage::find_refs(
                            //     &self.prepro.main_stores,
                            //     &mut self.ana,
                            //     &mut d_it.position(x),
                            //     i,
                            //     x,
                            // );
                        }
                        let mut scout = decl_scout.clone();
                        // let mut parents = d_it.parents().to_vec();
                        // let mut offsets = d_it.offsets().to_vec();
                        // let mut curr = parents.pop();
                        // offsets.pop();
                        let mut curr = if scout.has_parents() {
                            scout.up(&self.structural_positions);
                            Some(scout.node(&self.structural_positions))
                        } else {
                            None
                        };
                        let mut prev = curr;
                        let mut before_p_ref = i;
                        let mut max_qual_ref = i;
                        let mut conti = false;
                        // go through classes if inner
                        loop {
                            if let Some(xx) = curr {
                                let bb = self.prepro.main_stores.node_store.resolve(xx);
                                let t = bb.get_type();
                                if t.is_type_body() {
                                    println!(
                                        "try search {:?}",
                                        &mut self.ana.solver.nodes.with(max_qual_ref)
                                    );
                                    r.extend(
                                        usage::RefsFinder::new(
                                            &self.prepro.main_stores,
                                            &mut self.ana,
                                            &mut self.structural_positions,
                                        )
                                        .find_all(max_qual_ref, scout.clone()),
                                    );
                                    // usage::find_refs(
                                    //     &self.prepro.main_stores,
                                    //     &mut self.ana,
                                    //     &mut (parents.clone(), offsets.clone(), x).into(),
                                    //     max_qual_ref,
                                    //     x,
                                    // );
                                    prev = curr;
                                    // curr = parents.pop();
                                    // offsets.pop();
                                    curr = if scout.has_parents() {
                                        scout.up(&self.structural_positions);
                                        Some(scout.node(&self.structural_positions))
                                    } else {
                                        None
                                    };
                                    if let Some(xxx) = curr {
                                        let bb = self.prepro.main_stores.node_store.resolve(xxx);
                                        let t = bb.get_type();
                                        if t == Type::ObjectCreationExpression {
                                            conti = true;
                                            break;
                                        } else if !t.is_type_declaration() {
                                            panic!("{:?}", t);
                                        }
                                        let mut l2 = None;
                                        for xx in b.get_children() {
                                            let bb =
                                                self.prepro.main_stores.node_store.resolve(*xx);
                                            if bb.get_type() == Type::Identifier {
                                                let i = bb.get_label();
                                                l2 = Some(*i);
                                            }
                                        }
                                        if let Some(i) = l2 {
                                            let o = self.ana.solver.intern(RefsEnum::MaybeMissing);
                                            let f = IdentifierFormat::from(
                                                self.prepro.main_stores.label_store.resolve(&i),
                                            );
                                            let l = LabelPtr::new(i, f);
                                            let i = self
                                                .ana
                                                .solver
                                                .intern(RefsEnum::ScopedIdentifier(o, l));
                                            max_qual_ref = self
                                                .ana
                                                .solver
                                                .try_solve_node_with(max_qual_ref, i)
                                                .unwrap();
                                            println!(
                                                "try search {:?}",
                                                self.ana.solver.nodes.with(max_qual_ref)
                                            );

                                            r.extend(
                                                usage::RefsFinder::new(
                                                    &self.prepro.main_stores,
                                                    &mut self.ana,
                                                    &mut self.structural_positions,
                                                )
                                                .find_all(max_qual_ref, scout.clone()),
                                            );
                                            // usage::find_refs(
                                            //     &self.prepro.main_stores,
                                            //     &mut self.ana,
                                            //     &mut (parents.clone(), offsets.clone(), xx).into(),
                                            //     max_qual_ref,
                                            //     xx,
                                            // );
                                        }
                                        prev = curr;

                                        curr = if scout.has_parents() {
                                            scout.up(&self.structural_positions);
                                            Some(scout.node(&self.structural_positions))
                                        } else {
                                            None
                                        };
                                        // curr = parents.pop();
                                        // offsets.pop();
                                    }
                                } else if t == Type::Program {
                                    // go through program i.e. package declaration
                                    before_p_ref = max_qual_ref;
                                    for (i, xx) in b.get_children().iter().enumerate() {
                                        scout.goto(*xx, i);
                                        let bb = self.prepro.main_stores.node_store.resolve(*xx);
                                        let t = bb.get_type();
                                        if t == Type::PackageDeclaration {
                                            let p = remake_pkg_ref(
                                                &self.prepro.main_stores,
                                                &mut self.ana,
                                                *xx,
                                            );
                                            max_qual_ref = self
                                                .ana
                                                .solver
                                                .try_solve_node_with(max_qual_ref, p)
                                                .unwrap();
                                        } else if t.is_type_declaration() {
                                            println!(
                                                "try search {:?}",
                                                &mut self.ana.solver.nodes.with(max_qual_ref)
                                            );
                                            if Some(*xx) != prev {
                                                r.extend(
                                                    usage::RefsFinder::new(
                                                        &self.prepro.main_stores,
                                                        &mut self.ana,
                                                        &mut self.structural_positions,
                                                    )
                                                    .find_all(before_p_ref, scout.clone()),
                                                );
                                                // usage::find_refs(
                                                //     &self.prepro.main_stores,
                                                //     &mut self.ana,
                                                //     &mut (parents.clone(), offsets.clone(), *xx)
                                                //         .into(),
                                                //     before_p_ref,
                                                //     *xx,
                                                // );
                                            }
                                            r.extend(
                                                usage::RefsFinder::new(
                                                    &self.prepro.main_stores,
                                                    &mut self.ana,
                                                    &mut self.structural_positions,
                                                )
                                                .find_all(max_qual_ref, scout.clone()),
                                            );
                                            // usage::find_refs(
                                            //     &self.prepro.main_stores,
                                            //     &mut self.ana,
                                            //     &mut (parents.clone(), offsets.clone(), *xx).into(),
                                            //     max_qual_ref,
                                            //     *xx,
                                            // );
                                        }
                                        scout.up(&self.structural_positions);
                                    }
                                    prev = curr;

                                    curr = if scout.has_parents() {
                                        scout.up(&self.structural_positions);
                                        Some(scout.node(&self.structural_positions))
                                    } else {
                                        None
                                    };
                                    // curr = parents.pop();
                                    // offsets.pop();
                                    break;
                                } else if t == Type::ObjectCreationExpression {
                                    conti = true;
                                    break;
                                } else if t == Type::Block {
                                    conti = true;
                                    break; // TODO check if really done
                                } else {
                                    todo!("{:?}", t)
                                }
                            }
                        }
                        if conti {
                            continue;
                        }
                        // go through package
                        if let Some(xx) = curr {
                            if Some(xx) != prev {
                                r.extend(
                                    usage::RefsFinder::new(
                                        &self.prepro.main_stores,
                                        &mut self.ana,
                                        &mut self.structural_positions,
                                    )
                                    .find_all(before_p_ref, scout.clone()),
                                );
                                r.extend(
                                    usage::RefsFinder::new(
                                        &self.prepro.main_stores,
                                        &mut self.ana,
                                        &mut self.structural_positions,
                                    )
                                    .find_all(max_qual_ref, scout.clone()),
                                );
                                // usage::find_refs(
                                //     &self.prepro.main_stores,
                                //     &mut self.ana,
                                //     &mut (parents.clone(), offsets.clone(), xx).into(),
                                //     before_p_ref,
                                //     xx,
                                // );
                            }
                            // usage::find_refs(
                            //     &self.prepro.main_stores,
                            //     &mut self.ana,
                            //     &mut (parents.clone(), offsets.clone(), xx).into(),
                            //     max_qual_ref,
                            //     xx,
                            // );
                            prev = curr;

                            curr = if scout.has_parents() {
                                scout.up(&self.structural_positions);
                                Some(scout.node(&self.structural_positions))
                            } else {
                                None
                            };
                            // curr = parents.pop();
                            // offsets.pop();
                        }
                        // go through directories
                        loop {
                            if let Some(xx) = curr {
                                if Some(xx) != prev {
                                    r.extend(
                                        usage::RefsFinder::new(
                                            &self.prepro.main_stores,
                                            &mut self.ana,
                                            &mut self.structural_positions,
                                        )
                                        .find_all(max_qual_ref, scout.clone()),
                                    );
                                    // usage::find_refs(
                                    //     &self.prepro.main_stores,
                                    //     &mut self.ana,
                                    //     &mut (parents.clone(), offsets.clone(), xx).into(),
                                    //     max_qual_ref,
                                    //     xx,
                                    // );
                                }
                                prev = curr;
                                curr = if scout.has_parents() {
                                    scout.up(&self.structural_positions);
                                    Some(scout.node(&self.structural_positions))
                                } else {
                                    None
                                };
                                // curr = parents.pop();
                                // offsets.pop();
                            } else {
                                break;
                            }
                        }
                    }

                    println!("time taken for refs search: {}", now.elapsed().as_nanos());
                } else {
                    // TODO
                    // println!("todo impl search on {:?}", &t);
                }

                // println!("it state {:?}", &d_it);
                // java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                //     &self.prepro.hyper_ast.main_stores().node_store,
                //     &self.prepro.hyper_ast.main_stores().label_store,
                //     &x,
                // );
                // println!();
            } else {
                break;
            }
        }
    }

    pub fn iter_relations(&self) -> impl Iterator<Item = (Position, Vec<Position>)> + '_ {
        IterRefRelations {
            x: &self,
            r: self.relations.iter(),
        }
    }
}

impl<'a> Into<HashMap<Position, Vec<Position>>> for AllRefsFinder<'a> {
    fn into(self) -> HashMap<Position, Vec<Position>> {
        let mut r: HashMap<Position, Vec<Position>> = Default::default();
        for (k, v) in self.relations {
            let v = self
                .structural_positions
                .get_positions(&self.prepro.main_stores, &v);
            // let v = v
            //     .iter()
            //     .map(|x| x.to_position(&self.prepro.main_stores))
            //     .collect();
            let k = self
                .structural_positions
                .get_positions(&self.prepro.main_stores, &[k])
                .pop()
                .expect("should have produced one position");
            // let k = k.to_position(&self.prepro.main_stores);
            r.insert(k, v);
        }
        r
    }
}
struct IterRefRelations<'a, It>
where
    It: Iterator<Item = (&'a usize, &'a Vec<usize>)>,
{
    x: &'a AllRefsFinder<'a>,
    r: It,
}

impl<'a, It> Iterator for IterRefRelations<'a, It>
where
    It: Iterator<Item = (&'a usize, &'a Vec<usize>)>,
{
    type Item = (Position, Vec<Position>);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = self.r.next()?;
        let v = self
            .x
            .structural_positions
            .get_positions(&self.x.prepro.main_stores, &v);
        let k = self
            .x
            .structural_positions
            .get_positions(&self.x.prepro.main_stores, &[*k])
            .pop()
            .expect("should have produced one position");
        Some((k, v))
    }
}

// impl<'a> Display for AllRefsFinder<'a> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

//         for (k, v) in self.relations.iter() {
//             let v = self
//                 .structural_positions
//                 .get_positions(&self.prepro.main_stores, v);
//             // let v = v
//             //     .iter()
//             //     .map(|x| x.to_position(&self.prepro.main_stores))
//             //     .collect();
//             let k = self
//                 .structural_positions
//                 .get_positions(&self.prepro.main_stores, &[*k])
//                 .pop()
//                 .expect("should have produced one position");
//             // let k = k.to_position(&self.prepro.main_stores);
//             write!(f, "{{\"decl\":{},\"refs\":{}}}",k,v)?;//.insert(k, v);
//         }
//         Ok(())
//     }
// }

// TODO everyting to iterators
// struct IterPerDeclarations {

// }
