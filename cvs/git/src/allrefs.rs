use std::{borrow::Borrow, collections::HashMap, io::Write, path::Path, time::Instant};

use hyper_ast::{
    position::{
        ExploreStructuralPositions, Position, Scout, StructuralPosition, StructuralPositionStore,
        TreePath,
    },
    store::defaults::NodeIdentifier,
    types::{LabelStore, Labeled, Type, Typed, WithChildren},
};
use rusted_gumtree_gen_ts_java::{
    impact::{
        element::{IdentifierFormat, LabelPtr, RefsEnum},
        partial_analysis::PartialAnalysis,
        reference::DisplayRef,
        usage::{self, remake_pkg_ref},
    },
    usage::declarations::{self, IterDeclarations},
};

use crate::{
    maven::{IterMavenModules, IterMavenModules2},
    preprocessed::PreProcessedRepository,
};

pub fn write_referencial_relations<W: Write>(
    prepro: &PreProcessedRepository,
    root: NodeIdentifier,
    out: &mut W,
) {
    let modules = IterMavenModules::new(&prepro.main_stores, StructuralPosition::new(root), root);
    let declarations = iter_declarations(prepro, modules);

    for (decl, m, of) in declarations {
        let of = of
            .into_iter()
            .map(|x| (x.to_position(&prepro.main_stores).file().to_owned(), x));
        let p_in_of = find_package_in_other_folders(
            prepro,
            decl.to_position(&prepro.main_stores).file(),
            decl.to_position(&prepro.main_stores).file(),
            of,
        );
        let p_in_of: Vec<Scout> = p_in_of.into_iter().map(|x| (x, 0).into()).collect();

        let b = prepro
            .main_stores
            .node_store
            .resolve(decl.node().unwrap().to_owned());
        let t = b.get_type();
        if t == Type::ClassDeclaration
            || t == Type::InterfaceDeclaration
            || t == Type::AnnotationTypeDeclaration
        {
            let mut structural_positions = StructuralPosition::new(root).into();
            let references = RefsFinder::new(prepro, &mut structural_positions)
                .find_type_declaration_references(
                    (decl.clone(), 0).into(),
                    m.node().unwrap(),
                    &p_in_of,
                );
            let decl = decl.to_position(&prepro.main_stores);
            let references = structural_positions.ends_positions(&prepro.main_stores, &references);
            write_positions_of_referencial_relations(decl, references, out);
        }
    }
}

fn write_positions_of_referencial_relations<W: Write>(
    decl: Position,
    references: Vec<Position>,
    out: &mut W,
) {
    write!(out, r#"{{"decl":"#).unwrap();
    write!(out, "{}", decl).unwrap();
    write!(out, r#","refs":["#).unwrap();
    let mut first = true;
    for x in references {
        if first {
            first = false;
        } else {
            write!(out, ",").unwrap();
        }
        write!(out, "{}", x).unwrap();
    }
    write!(out, "]}}").unwrap();
    writeln!(out, ",").unwrap();
}

type ExtendedDeclaration = (
    StructuralPosition,
    StructuralPosition,
    Vec<StructuralPosition>,
);

pub fn iter_declarations<'a>(
    prepro: &'a PreProcessedRepository,
    modules: IterMavenModules<'a, StructuralPosition>,
) -> impl Iterator<Item = ExtendedDeclaration> + 'a {
    modules
        .flat_map(|m| {
            let src = goto_by_name(prepro, m.clone(), "src");
            let source_tests = src
                .clone()
                .and_then(|x| goto_by_name(prepro, x, "test"))
                .and_then(|x| goto_by_name(prepro, x, "java"));
            let source = src
                .and_then(|x| goto_by_name(prepro, x, "main"))
                .and_then(|x| goto_by_name(prepro, x, "java"));
            let mut r = vec![];
            let mut test_folders = vec![];
            if let Some(source_tests) = source_tests {
                test_folders.push(source_tests.clone());
                r.push((source_tests, m.clone(), vec![]))
            }
            if let Some(source) = source {
                r.push((source, m, test_folders))
            }
            r
        })
        .flat_map(|(f, m, of)| {
            let n = *f.node().unwrap();
            IterDeclarations::new(&prepro.main_stores, f, n)
                .map(move |x| (x, m.clone(), of.clone()))
        })
}

pub struct RefsFinder<'a> {
    prepro: &'a PreProcessedRepository,
    ana: PartialAnalysis,
    structural_positions: &'a mut StructuralPositionStore,
}

impl<'a> RefsFinder<'a> {
    pub fn new(
        prepro: &'a PreProcessedRepository,
        structural_positions: &'a mut StructuralPositionStore,
    ) -> Self {
        Self {
            prepro,
            ana: PartialAnalysis::default(),
            structural_positions,
        }
    }

    fn find_type_declaration_references(
        &mut self,
        decl: Scout,
        maven_module: &NodeIdentifier,
        mirror_packages: &[Scout],
    ) -> Vec<usize> {
        let now = Instant::now();
        self.structural_positions
            .check(&self.prepro.main_stores)
            .unwrap();
        println!("{:?}", decl);
        decl.check(&self.prepro.main_stores).unwrap();
        self.structural_positions
            .check(&self.prepro.main_stores)
            .unwrap();
        let mut r = vec![];

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
            .resolve(decl.node_always(&self.structural_positions));
        let t = b.get_type();
        println!(
            "now search for {:?} at {:?}",
            &t,
            decl.to_position(&self.structural_positions, &self.prepro.main_stores)
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
            return r;
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
                            return r;
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
                    return r;
                } else if t == Type::Block {
                    return r; // TODO check if really done
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
                    .resolve(scout.node_always(&self.structural_positions));
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
        r
    }
}

pub fn goto_by_name<T: TreePath<NodeIdentifier>>(
    prepro: &PreProcessedRepository,
    mut p: T,
    name: &str,
) -> Option<T> {
    p.node()
        .and_then(|x| prepro.child_by_name_with_idx(*x, name))
        .and_then(|(n, i)| {
            p.goto(n, i);
            Some(p)
        })
}

pub fn find_package_in_other_folders<
    'a,
    T: TreePath<NodeIdentifier> + Clone,
    U: Borrow<Path>,
    V: Iterator<Item = (U, T)>,
>(
    prepro: &PreProcessedRepository,
    package: &Path,
    root_package_file_path: &Path,
    other_root_packages: V,
) -> Vec<T> {
    let rel = package
        .strip_prefix(root_package_file_path)
        .expect("a relative path");
    other_root_packages
        .filter_map(|(p, x)| {
            let path = p.borrow().join(rel);
            let mut r = x.clone();
            for n in path.components() {
                let x = *r.node().unwrap();
                let n = std::os::unix::prelude::OsStrExt::as_bytes(n.as_os_str());
                let n = std::str::from_utf8(n).unwrap();
                let aaa = prepro.child_by_name_with_idx(x, n);
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
