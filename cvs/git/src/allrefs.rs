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

use std::{collections::HashMap, fmt::Display, ops::ControlFlow, time::Instant};

use hyper_ast::{
    position::{
        extract_position, ExploreStructuralPositions, Position, Scout, StructuralPosition,
        StructuralPositionStore,
    },
    store::{defaults::NodeIdentifier, nodes::legion::HashedNodeRef},
    types::{LabelStore, Labeled, Type, Typed, WithChildren},
};
use rusted_gumtree_gen_ts_java::impact::{
    element::{IdentifierFormat, LabelPtr, RefsEnum},
    partial_analysis::PartialAnalysis,
    reference::DisplayRef,
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
            let maven_module = if let Some(d) = m_it.next() { d } else { break };
            assert_eq!(m_it.parents().len() + 1, m_it.offsets().len());
            let mut other_root_packages = vec![];
            let mut scout_main = if maven_module == root {
                Scout::from((StructuralPosition::from((vec![], vec![])), 0))
            } else {
                let scout = Scout::from((
                    StructuralPosition::from((
                        m_it.parents()[1..].to_vec(),
                        m_it.offsets()[1..].to_vec(),
                        maven_module,
                    )),
                    0,
                ));
                scout.check(&self.prepro.main_stores).unwrap();
                scout
            };
            let mut scout_test = scout_main.clone();
            let src = self.prepro.child_by_name_with_idx(maven_module, "src");
            println!("src:{:?}", src);
            // first the tests code
            let src = if let Some((d, i)) = src {
                scout_main.goto(d, i);
                scout_test.goto(d, i);
                // scout_main.check(&self.prepro.main_stores).unwrap();
                // scout_test.check(&self.prepro.main_stores).unwrap();
                d
            } else {
                continue;
            };
            let src_test = self.prepro.child_by_name_with_idx(src, "test");
            let src_test_java = src_test.and_then(|(d, i)| {
                scout_test.goto(d, i);
                scout_test.check(&self.prepro.main_stores).unwrap();
                self.prepro.child_by_name_with_idx(d, "java")
            });
            if let Some((d, i)) = src_test_java {
                scout_test.goto(d, i);
                scout_test.check(&self.prepro.main_stores).unwrap();
                other_root_packages.push(scout_test.clone());
                self.find_references_to_declarations_aux(&maven_module, scout_test, &vec![])
            }

            // then the production code

            let src_main = self.prepro.child_by_name_with_idx(src, "main");
            let src_main_java = src_main.and_then(|(d, i)| {
                scout_main.goto(d, i);
                scout_main.check(&self.prepro.main_stores).unwrap();
                self.prepro.child_by_name_with_idx(d, "java")
            });
            // let s = s.and_then(|d| self.child_by_type(d, &Type::Directory));
            if let Some((d, i)) = src_main_java {
                scout_main.goto(d, i);
                scout_main.check(&self.prepro.main_stores).unwrap();
                // let n = self.prepro.hyper_ast.main_stores().node_store.resolve(d);
                // println!(
                //     "search in module/src/main/java {}",
                //     self.prepro.hyper_ast
                //         .main_stores
                //         .label_store
                //         .resolve(n.get_label())
                // );
                // usage::find_all_decls(&self.prepro.hyper_ast.main_stores, &mut self.ana, s);
                self.find_references_to_declarations_aux(
                    &maven_module,
                    scout_main,
                    &other_root_packages,
                )
            }
        }
    }

    fn find_references_to_declarations_aux(
        &mut self,
        mut maven_module: &NodeIdentifier,
        mut root_package: Scout,
        other_root_packages: &[Scout],
    ) {
        // let mut d_it = IterDeclarations::new(&self.prepro.main_stores, s.node(&self.structural_positions));
        println!("scout: {:?}", root_package);
        self.structural_positions
            .check(&self.prepro.main_stores)
            .unwrap();
        root_package.check(&self.prepro.main_stores).unwrap();
        let root_package_p =
            root_package.to_position(&self.structural_positions, &self.prepro.main_stores);
        let root_package_file_path = root_package_p.file();
        let structural_positions_root = self.structural_positions.push(&mut root_package);
        self.structural_positions
            .check(&self.prepro.main_stores)
            .unwrap();
        let it = ExploreStructuralPositions::from((
            &self.structural_positions,
            structural_positions_root,
        ));
        println!("search from {:?}", it.to_position(&self.prepro.main_stores));

        let mut d_it = {
            let n = root_package.node(&self.structural_positions);
            IterDeclarations2::new(&self.prepro.main_stores, root_package, n)
        };
        self.structural_positions
            .check(&self.prepro.main_stores)
            .unwrap();
        loop {
            if let Some(decl) = d_it.next() {
                let b = self
                    .prepro
                    .main_stores
                    .node_store
                    .resolve(decl.node(&self.structural_positions));
                let t = b.get_type();
                println!("could search {:?}", &t);
                if t == Type::ClassDeclaration
                    || t == Type::InterfaceDeclaration
                    || t == Type::AnnotationTypeDeclaration
                {
                    let other_root_packages_paths: Vec<_> = self.find_package_in_other_roots(
                        &decl,
                        root_package_file_path,
                        other_root_packages,
                    );

                    self.find_type_declaration_references(
                        decl,
                        maven_module,
                        &other_root_packages_paths,
                    );
                } else if t == Type::EnumDeclaration {
                    self.find_type_declaration_references(decl, maven_module, &vec![]);
                // TODO
                // TODO go to variants and search references to them then union results
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

    fn find_package_in_other_roots(
        &mut self,
        decl: &Scout,
        root_package_file_path: &std::path::Path,
        other_root_packages: &[Scout],
    ) -> Vec<Scout> {
        let decl_p = decl.to_position(&self.structural_positions, &self.prepro.main_stores);
        let decl_file_path = decl_p.file();
        let decl_package_path = decl_file_path.parent().unwrap();
        let rel = decl_package_path
            .strip_prefix(root_package_file_path)
            .expect("a relative path");
        other_root_packages
            .iter()
            .filter_map(|x| {
                let other_root_p =
                    x.to_position(&self.structural_positions, &self.prepro.main_stores);
                let other_root_path = other_root_p.file();
                let path = other_root_path.join(rel);
                let mut r = x.clone();
                for n in path.components() {
                    let x = r.node(&self.structural_positions);
                    let n = std::os::unix::prelude::OsStrExt::as_bytes(n.as_os_str());
                    let n = std::str::from_utf8(n).unwrap();
                    let aaa = self.prepro.child_by_name_with_idx(x, n);
                    if let Some((x, i)) = aaa {
                        r.goto(x, i);
                    } else {
                        return None;
                    }
                }
                Some(r)
            })
            .collect()
    }

    fn find_type_declaration_references(
        &mut self,
        mut decl: Scout,
        maven_module: &NodeIdentifier,
        mirror_packages: &[Scout],
    ) {
        let now = Instant::now();
        self.structural_positions
            .check(&self.prepro.main_stores)
            .unwrap();
        println!("{:?}", decl);
        decl.check(&self.prepro.main_stores).unwrap();
        let key = self.structural_positions.push(&mut decl);
        self.structural_positions
            .check(&self.prepro.main_stores)
            .unwrap();
        let it = ExploreStructuralPositions::from((&self.structural_positions, key));
        let r = self.relations.entry(key).or_insert(vec![]);

        macro_rules! search {
            ( $p:expr, $i:expr, $s:expr ) => {{
                r.extend(
                    usage::RefsFinder::new(
                        &self.prepro.main_stores,
                        &mut self.ana,
                        &mut self.structural_positions,
                    )
                    .find_all($p, $i, $s.clone()),
                );
            }};
        }

        let b = self
            .prepro
            .main_stores
            .node_store
            .resolve(decl.node(&self.structural_positions));
        let t = b.get_type();
        println!(
            "now search for {:?} at {:?}",
            &t,
            it.to_position(&self.prepro.main_stores)
        );
        {
            let r = self.ana.solver.intern(RefsEnum::MaybeMissing);
            let i = self.ana.solver.intern(RefsEnum::This(r));
            println!("try search this");
            search!(r, i, decl);
        }
        let mut l = None;
        for xx in b.get_children() {
            let bb = self.prepro.main_stores.node_store.resolve(*xx);
            if bb.get_type() == Type::Identifier {
                let i = bb.get_label();
                l = Some(*i);
            }
        }
        let i = if let Some(i) = l {
            i
        } else {
            println!("time taken for refs search: {}", now.elapsed().as_nanos());
            return;
        };

        let f = self.prepro.main_stores.label_store.resolve(&i);
        println!("search uses of {:?}", f);
        let f = IdentifierFormat::from(f);
        let l = LabelPtr::new(i, f);
        let o = self.ana.solver.intern(RefsEnum::MaybeMissing);
        let i = self.ana.solver.intern(RefsEnum::ScopedIdentifier(o, l));
        println!(
            "try search {}",
            DisplayRef::from((
                self.ana.solver.nodes.with(i),
                &self.prepro.main_stores.label_store
            ))
        );

        search!(o, i, decl);
        {
            let i = self.ana.solver.intern(RefsEnum::This(i));
            println!(
                "try search {}",
                DisplayRef::from((
                    self.ana.solver.nodes.with(i),
                    &self.prepro.main_stores.label_store
                ))
            );
            search!(o, i, decl);
        }
        let mut scout = decl.clone();
        let mut curr = scout.up(&self.structural_positions);
        let mut prev = curr;
        let mut before_p_ref = i;
        let mut max_qual_ref = i;
        let mut package_ref = i;
        // go through classes if inner
        loop {
            if let Some(xx) = curr {
                let bb = self.prepro.main_stores.node_store.resolve(xx);
                let t = bb.get_type();
                if t.is_type_body() {
                    println!(
                        "try search {}",
                        DisplayRef::from((
                            self.ana.solver.nodes.with(max_qual_ref),
                            &self.prepro.main_stores.label_store
                        ))
                    );
                    for (i, xx) in bb.get_children().iter().enumerate() {
                        scout.goto(*xx, i);
                        if Some(*xx) != prev {
                            search!(package_ref, max_qual_ref, scout);
                        }
                        scout.up(&self.structural_positions);
                    }
                    // TODO do things to find type of field at the same level than type decl
                    // should loop through siblings
                    prev = curr;
                    curr = scout.up(&self.structural_positions);
                    if let Some(xxx) = curr {
                        let bb = self.prepro.main_stores.node_store.resolve(xxx);
                        let t = bb.get_type();
                        if t == Type::ObjectCreationExpression {
                            return;
                        } else if !t.is_type_declaration() {
                            panic!("{:?}", t);
                        }
                        let mut l2 = None;
                        for xx in bb.get_children() {
                            let bb = self.prepro.main_stores.node_store.resolve(*xx);
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
                            let i = self.ana.solver.intern(RefsEnum::ScopedIdentifier(o, l));
                            max_qual_ref = self
                                .ana
                                .solver
                                .try_solve_node_with(max_qual_ref, i)
                                .unwrap();
                            println!(
                                "try search {}",
                                DisplayRef::from((
                                    self.ana.solver.nodes.with(max_qual_ref),
                                    &self.prepro.main_stores.label_store
                                ))
                            );

                            search!(package_ref, max_qual_ref, scout);
                            // TODO do things to find type of field at the same level than type decl
                            // should loop through members searching for parent qualified
                        }
                        prev = curr;

                        curr = scout.up(&self.structural_positions);
                    }
                } else if t == Type::Program {
                    println!(
                        "go through program {:?} {:?} {}",
                        curr,
                        scout.to_position(&self.structural_positions, &self.prepro.main_stores),
                        b.child_count()
                    );
                    // go through program i.e. package declaration
                    before_p_ref = max_qual_ref;
                    for (i, xx) in bb.get_children().iter().enumerate() {
                        scout.goto(*xx, i);
                        let bb = self.prepro.main_stores.node_store.resolve(*xx);
                        let t = bb.get_type();
                        println!("in program {:?}", t);
                        if t == Type::PackageDeclaration {
                            package_ref =
                                remake_pkg_ref(&self.prepro.main_stores, &mut self.ana, *xx);
                            max_qual_ref = self
                                .ana
                                .solver
                                .try_solve_node_with(max_qual_ref, package_ref)
                                .unwrap();
                            println!(
                                "now have fully qual ref {} with package decl {}",
                                DisplayRef::from((
                                    self.ana.solver.nodes.with(max_qual_ref),
                                    &self.prepro.main_stores.label_store
                                )),
                                DisplayRef::from((
                                    self.ana.solver.nodes.with(package_ref),
                                    &self.prepro.main_stores.label_store
                                ))
                            );
                        } else if t.is_type_declaration() {
                            println!(
                                "try search {}",
                                DisplayRef::from((
                                    self.ana.solver.nodes.with(max_qual_ref),
                                    &self.prepro.main_stores.label_store
                                ))
                            );
                            if Some(*xx) != prev && max_qual_ref != before_p_ref {
                                search!(package_ref, before_p_ref, scout);
                            }

                            search!(package_ref, max_qual_ref, scout);
                        }
                        scout.up(&self.structural_positions);
                    }
                    prev = curr;
                    curr = scout.up(&self.structural_positions);
                    break;
                } else if t == Type::ObjectCreationExpression {
                    return;
                } else if t == Type::Block {
                    return; // TODO check if really done
                } else {
                    todo!("{:?}", t)
                }
            }
        }
        if let Some(xx) = curr {
            let bb = self.prepro.main_stores.node_store.resolve(xx);
            let t = bb.get_type();
            println!(
                "search in package {:?} {:?} {:?}",
                curr,
                t,
                scout.to_position(&self.structural_positions, &self.prepro.main_stores)
            );
            for (i, xx) in bb.get_children().iter().enumerate() {
                scout.goto(*xx, i);
                if Some(*xx) != prev {
                    println!(
                        "search {} in package children {:?} {:?}",
                        DisplayRef::from((
                            self.ana.solver.nodes.with(max_qual_ref),
                            &self.prepro.main_stores.label_store
                        )),
                        curr,
                        scout.to_position(&self.structural_positions, &self.prepro.main_stores)
                    );
                    search!(package_ref, max_qual_ref, scout);

                    // let bb = self.prepro.main_stores.node_store.resolve(*xx);
                    // let t = bb.get_type();
                    // if t == Type::Program {
                    //     search!(before_p_ref, scout);
                    // }
                }
                scout.up(&self.structural_positions);
            }

            for scout in mirror_packages {
                let mut scout = scout.clone();
                let bb = self
                    .prepro
                    .main_stores
                    .node_store
                    .resolve(scout.node(&self.structural_positions));
                for (i, xx) in bb.get_children().iter().enumerate() {
                    let bb = self.prepro.main_stores.node_store.resolve(*xx);
                    let t = bb.get_type();
                    scout.goto(*xx, i);
                    if t == Type::Program {
                        println!(
                            "search {} in other package children {:?}",
                            DisplayRef::from((
                                self.ana.solver.nodes.with(max_qual_ref),
                                &self.prepro.main_stores.label_store
                            )),
                            scout.to_position(&self.structural_positions, &self.prepro.main_stores),
                        );
                        search!(package_ref, max_qual_ref, scout);
                    }
                    scout.up(&self.structural_positions);
                }
            }

            prev = curr;
            curr = scout.up(&self.structural_positions);
        }
        loop {
            println!(
                "search in directory {:?} {:?}",
                curr,
                scout.to_position(&self.structural_positions, &self.prepro.main_stores)
            );
            if let Some(xx) = curr {
                let bb = self.prepro.main_stores.node_store.resolve(xx);
                let t = bb.get_type();
                // println!("search in package {:?} {:?}", curr, t);
                for (i, xx) in bb.get_children().iter().enumerate() {
                    scout.goto(*xx, i);
                    if Some(*xx) != prev {
                        search!(package_ref, max_qual_ref, scout);
                    }
                    scout.up(&self.structural_positions);
                }
                if &xx == maven_module {
                    break;
                }
                prev = curr;
                curr = scout.up(&self.structural_positions);
            } else {
                break;
            }
        }
        println!("time taken for refs search: {}", now.elapsed().as_nanos());
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
