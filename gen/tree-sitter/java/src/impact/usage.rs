use core::fmt;
use std::{
    fmt::Debug,
    io::stdout,
    ops::{AddAssign, Deref},
};

use hyper_ast::{
    filter::{BloomResult, BloomSize},
    nodes::{print_tree_syntax, RefContainer},
    position::{
        extract_position, ExploreStructuralPositions, Scout, StructuralPosition,
        StructuralPositionStore,
    },
    store::defaults::LabelIdentifier,
    store::nodes::legion::HashedNodeRef,
    store::{
        defaults::{LabelValue, NodeIdentifier},
        SimpleStores,
    },
    types::{LabelStore, Labeled, Tree, Type, Typed, WithChildren},
};
// use rusted_gumtree_core::tree::tree::{WithChildren, Tree, Labeled};

use crate::{
    impact::{
        element::{IdentifierFormat, LabelPtr},
        reference::DisplayRef,
    },
    java_tree_gen_full_compress_legion_ref::{
        self,
        // HashedNodeRef,
        JavaTreeGen,
    },
};

use super::{
    element::ExplorableRef,
    element::{RefPtr, RefsEnum},
    partial_analysis::PartialAnalysis,
};

// TODO use generic node and store

pub fn remake_pkg_ref(
    stores: &SimpleStores,
    ana: &mut PartialAnalysis,
    x: NodeIdentifier,
) -> RefPtr {
    print_tree_syntax(
        |x| {
            stores
                .node_store
                .resolve(*x)
                .into_compressed_node()
                .unwrap()
        },
        |x| stores.label_store.resolve(x).to_string(),
        &x,
    );
    println!();
    let b = stores.node_store.resolve(x);
    let t = b.get_type();
    if t == Type::ScopedAbsoluteIdentifier {
        let x = b.get_child(&0);
        let o = remake_pkg_ref(stores, ana, x);
        let x = b.get_child(&2);
        let b = stores.node_store.resolve(x);
        let i = b.try_get_label().unwrap();
        let f = IdentifierFormat::from(stores.label_store.resolve(i));
        let l = LabelPtr::new(*i, f);
        let i = ana.solver.intern(RefsEnum::ScopedIdentifier(o, l));
        i
    } else if t == Type::Identifier {
        let i = b.try_get_label().unwrap();
        let o = ana.solver.intern(RefsEnum::Root);
        let f = IdentifierFormat::from(stores.label_store.resolve(i));
        let l = LabelPtr::new(*i, f);
        let i = ana.solver.intern(RefsEnum::ScopedIdentifier(o, l));
        i
    } else if t == Type::PackageDeclaration {
        let x = b.get_child(&2);
        remake_pkg_ref(stores, ana, x)
    } else if t == Type::Spaces {
        todo!()
    } else {
        todo!("{:?}", t)
    }
}
pub fn eq_root_scoped(d: ExplorableRef, stores: &SimpleStores, b: HashedNodeRef) -> bool {
    match d.as_ref() {
        RefsEnum::Root => false, // TODO check, not sure
        RefsEnum::MaybeMissing => false,
        RefsEnum::ScopedIdentifier(o, i) => {
            let t = b.get_type();
            if t == Type::ScopedAbsoluteIdentifier {
                let mut bo = false;
                for x in b.get_children().iter().rev() {
                    // log::trace!("d:{:?}",d);
                    let b = stores.node_store.resolve(*x);
                    let t = b.get_type();
                    if t == Type::ScopedAbsoluteIdentifier {
                        if !eq_root_scoped(d.with(*o), stores, b) {
                            return false;
                        }
                    } else if t == Type::Identifier {
                        if bo {
                            return eq_root_scoped(d.with(*o), stores, b);
                        }
                        if let Some(l) = b.try_get_label() {
                            if l != i.as_ref() {
                                return false;
                            } else {
                            }
                        } else {
                            panic!()
                        }
                        bo = true;
                    }
                }
                true
            } else if t == Type::Identifier {
                if let Some(l) = b.try_get_label() {
                    if l != i.as_ref() {
                        false
                    } else {
                        if let RefsEnum::Root = d.with(*o).as_ref() {
                            true
                        } else {
                            false
                        }
                    }
                } else {
                    panic!()
                }
            } else {
                todo!("{:?}", t)
            }
        }
        x => {
            panic!("{:?}", x)
        }
    }
}

// need some kind of visitor pattern
// or some interpreter pattern

// to find elements
// we need to reach these elements and check their validity
// then return their positions

// because we search for references maching a given pattern
// due to the possibility of the pattern locally changing
// we also need to carry the locally valid patterns

// trait Visitor<T: AddAssign + Default> {
//     fn stores(&self) -> &SimpleStores;
//     fn dispatch(&mut self, node: HashedNodeRef) -> T {
//         let t = node.get_type();
//         if t.is_type_declaration() {
//             self.visit_declaration(node)
//         } else {
//             self.visit(node)
//         }
//     }
//     fn visit(&mut self, node: HashedNodeRef) -> T {
//         if node.has_children() {
//             let mut r = Default::default();
//             for x in node.get_children() {
//                 let b = self.stores().node_store.resolve(*x);
//                 r += self.dispatch(b);
//             }
//             r
//         } else {
//             Default::default()
//         }
//     }
//     fn visit_import(&mut self, import: HashedNodeRef) -> T;
//     fn visit_declaration(&mut self, decl: HashedNodeRef) -> T;
//     fn visit_reference(&mut self, reff: HashedNodeRef) -> T;
//     fn visit_body(&mut self, body: HashedNodeRef) -> T;
//     fn visit_block(&mut self, block: HashedNodeRef) -> T;
// }

// trait Folder {
//     fn fold(&self) -> NodeIdentifier;
// }

// struct RefFinder2<'a> {
//     stores: &'a SimpleStores,
// }

// struct RefFinderResult {
//     new_target:Option<RefPtr>,
// }

// impl AddAssign for RefFinderResult {
//     fn add_assign(&mut self, rhs: Self) {
//         todo!()
//     }
// }
// impl Default for RefFinderResult {
//     fn default() -> Self {
//         todo!()
//     }
// }

// impl<'a> Visitor<RefFinderResult> for RefFinder2<'a> {
//     fn visit_import(&mut self, import: HashedNodeRef) -> RefFinderResult {
//         todo!()
//     }

//     fn visit_declaration(&mut self, decl: HashedNodeRef) -> RefFinderResult {
//         todo!()
//     }

//     fn visit_reference(&mut self, reff: HashedNodeRef) -> RefFinderResult {
//         todo!()
//     }

//     fn visit_body(&mut self, body: HashedNodeRef) -> RefFinderResult {
//         todo!()
//     }

//     fn visit_block(&mut self, block: HashedNodeRef) -> RefFinderResult {
//         todo!()
//     }

//     fn stores(&self) -> &SimpleStores {
//         self.stores
//     }
// }

// /// contextual payload to find references in a HyperAST
// struct PayLoad {
//     new_target:Option<RefPtr>,
//     targets: Vec<RefPtr>,
//     // relative position in hyperAST
//     position: (usize, Option<StructuralPosition>),
// }

pub struct RefsFinder<'a> {
    stores: &'a SimpleStores,
    ana: &'a mut PartialAnalysis,
    /// result of search
    sp_store: &'a mut StructuralPositionStore,
    refs: Vec<usize>,
}

impl<'a> RefsFinder<'a> {
    pub fn new(
        stores: &'a SimpleStores,
        ana: &'a mut PartialAnalysis,
        sp_store: &'a mut StructuralPositionStore,
    ) -> Self {
        Self {
            stores,
            ana,
            sp_store,
            refs: Default::default(),
        }
    }

    /// Find all references to `target` that was declared in `package`
    ///
    /// returns the indexes that should be used on self.sp_store the `StructuralPositionStore`
    pub fn find_all(mut self, package: RefPtr, target: RefPtr, mut x: Scout) -> Vec<usize> {
        self.find_refs(package, target, &mut x);
        self.refs
    }

    fn find_refs(&mut self, package: RefPtr, target: RefPtr, scout: &mut Scout) -> Vec<RefPtr> {
        let current = scout.node(&self.sp_store);
        let b = self.stores.node_store.resolve(current);
        let t = b.get_type();
        if &t == &Type::Spaces {
            return vec![];
        } else if &t == &Type::Comment {
            return vec![];
        } else if &t == &Type::PackageDeclaration {
            let root_ref = self.ana.solver.intern(RefsEnum::Root);
            let d = self.ana.solver.nodes.with(package);
            let (b, x) = {
                let mut i = 0;
                let r;
                let x;
                loop {
                    let y = b.get_child(&i);
                    let b = self.stores.node_store.resolve(y);
                    let t = b.get_type();
                    if t == Type::ScopedAbsoluteIdentifier || t == Type::Identifier {
                        r = b;
                        x = y;
                        break;
                    } else {
                        i += 1;
                    }
                }
                (r, x)
            };
            println!(
                "d=1 {} {:?} {:?}",
                DisplayRef::from((self.ana.solver.nodes.with(target), &self.stores.label_store)),
                &t,
                scout.to_position(&self.sp_store, &self.stores)
            );
            java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                &self.stores.node_store,
                &self.stores.label_store,
                &x,
            );
            println!();

            if eq_root_scoped(d, self.stores, b) {
                if let Some(x) = self.ana.solver.try_unsolve_node_with(target, package) {
                    if let Some(y) = self.ana.solver.try_unsolve_node_with(target, root_ref) {
                        return vec![x, y];
                    } else {
                        return vec![x];
                    }
                } else {
                    let d = self.ana.solver.nodes.with(package);
                    let b = self.stores.node_store.resolve(x);
                    eq_root_scoped(d, self.stores, b);
                    self.ana.solver.try_unsolve_node_with(target, package);
                    panic!()
                }
            } else {
                if let Some(x) = self.ana.solver.try_unsolve_node_with(target, root_ref) {
                    return vec![x];
                } else {
                    return vec![];
                }
            }
        } else if &t == &Type::Program {
            println!(
                "d=1 {} {:?} {:?}",
                DisplayRef::from((self.ana.solver.nodes.with(target), &self.stores.label_store)),
                &t,
                scout.to_position(&self.sp_store, &self.stores)
            );
        } else if &t == &Type::Directory {
            println!(
                "d=1 {} {:?} {:?}",
                DisplayRef::from((self.ana.solver.nodes.with(target), &self.stores.label_store)),
                &t,
                scout.to_position(&self.sp_store, &self.stores)
            );
            // TODO if package, get top level declarations then localize if ref.
            // in the end we do not need due to the way we do the impact ana.
            // we should only come from parent of package with canonical id.
        } else if &t == &Type::MavenDirectory {
            println!(
                "d=1 {:?} {:?}",
                &t,
                scout.to_position(&self.sp_store, &self.stores)
            );
            // idem
        } else if &t == &Type::ImportDeclaration {
            println!("d=1 {:?}", &t);
            // TODO move print to maybe contains branch
            java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                &self.stores.node_store,
                &self.stores.label_store,
                &current,
            );
            println!();
            if target == package {
                return vec![];
            }

            let mut curr = target;

            let parse_import = || {
                let b = self.stores.node_store.resolve(current);
                let mut scop = None;
                let mut sstatic = false;
                let mut asterisk = false;
                for c in b.get_children() {
                    let b = self.stores.node_store.resolve(*c);
                    match b.get_type() {
                        Type::TS86 => sstatic = true,
                        Type::Asterisk => asterisk = true,
                        Type::Identifier => scop = Some(*c),
                        Type::ScopedAbsoluteIdentifier => scop = Some(*c),
                        _ => (),
                    }
                }
                (sstatic, scop.unwrap(), asterisk)
            };
            let mut parsed_import: Option<(bool, NodeIdentifier, bool)> = None;
            loop {
                let d = self.ana.solver.nodes.with(curr);
                let c = b.check(Into::<Box<[u8]>>::into(d.clone()).as_ref());
                if let BloomResult::MaybeContain = c {
                    println!("+++import+++++Maybe contains");

                    let (stic, scop, asterisk) = if let Some(x) = &parsed_import {
                        x.clone()
                    } else {
                        parsed_import = Some(parse_import());
                        parsed_import.unwrap().clone()
                    };

                    if eq_root_scoped(d, self.stores, self.stores.node_store.resolve(scop)) {
                        if stic {
                            println!("the import is static");
                        }
                        if asterisk {
                            if target != curr {
                                println!("on-demand import matched ref");
                                if let Some(x) = self.ana.solver.try_unsolve_node_with(target, curr)
                                {
                                    return vec![x];
                                }
                            }
                        } else {
                            let d = self.ana.solver.nodes.with(curr);
                            if let RefsEnum::ScopedIdentifier(o, _) = d.as_ref() {
                                curr = *o;
                            } else {
                                return vec![];
                            };
                            println!("import matched ref");
                            assert_ne!(target, curr);
                            if let Some(x) = self.ana.solver.try_unsolve_node_with(target, curr) {
                                return vec![x];
                            }
                        }
                    }

                    if curr == package {
                        return vec![];
                    }
                } else {
                    println!("Do not contains");
                }

                let d = self.ana.solver.nodes.with(curr);
                if let RefsEnum::ScopedIdentifier(o, _) = d.as_ref() {
                    curr = *o;
                } else {
                    return vec![];
                };
            }
            // let whole_match;
            // let c = {
            //     let d = self.ana.solver.nodes.with(target);
            //     b.check(Into::<Box<[u8]>>::into(d.clone()).as_ref())
            // };
            // let asterisk_match;
            // let c = if let BloomResult::MaybeContain = c {
            //     whole_match = true;
            //     c
            // } else {
            //     whole_match = false;
            //     let c = {
            //         let d = self.ana.solver.nodes.with(package);
            //         b.check(Into::<Box<[u8]>>::into(d.clone()).as_ref())
            //     };
            //     c
            // };
            // let c = if let BloomResult::MaybeContain = c {
            //     let d = self.ana.solver.nodes.with(target);
            //     if let RefsEnum::ScopedIdentifier(o, _) = d.as_ref() {
            //         if *o == package {
            //             asterisk_match = false;
            //         } else {
            //             asterisk_match = true;
            //         }
            //     } else {
            //         asterisk_match = true;
            //     }
            //     c
            // } else {
            //     asterisk_match = false;
            //     let d = self.ana.solver.nodes.with(target);
            //     if let RefsEnum::ScopedIdentifier(o, _) = d.as_ref() {
            //         let d = self.ana.solver.nodes.with(*o);
            //         b.check(Into::<Box<[u8]>>::into(d.clone()).as_ref())
            //     } else {
            //         c
            //     }
            // };
            // println!("d=1 {:?}", &t);
            // if let BloomResult::MaybeContain = c {
            //     println!("+++import+++++Maybe contains");
            //     let parent_match = !whole_match && !asterisk_match;
            //     java_tree_gen_full_compress_legion_ref::print_tree_syntax(
            //         &self.stores.node_store,
            //         &self.stores.label_store,
            //         &current,
            //     );
            //     println!();
            //     let (stic, scop, asterisk) = {
            //         let b = self.stores.node_store.resolve(current);
            //         let mut scop = None;
            //         let mut sstatic = false;
            //         let mut asterisk = false;
            //         for c in b.get_children() {
            //             let b = self.stores.node_store.resolve(*c);
            //             match b.get_type() {
            //                 Type::TS86 => sstatic = true,
            //                 Type::Asterisk => asterisk = true,
            //                 Type::Identifier => scop = Some((*c, b)),
            //                 Type::ScopedAbsoluteIdentifier => scop = Some((*c, b)),
            //                 _ => (),
            //             }
            //         }
            //         (sstatic, scop.unwrap(), asterisk)
            //     };
            //     if stic {
            //         return vec![]; // TODO
            //     } else if asterisk && (parent_match || asterisk_match) {
            //         let d = self.ana.solver.nodes.with(package);
            //         if eq_root_scoped(d, self.stores, scop.1) {
            //             if let Some(x) = self.ana.solver.try_unsolve_node_with(target, package) {
            //                 return vec![x];
            //             };
            //             println!("on-demand import matched ref");
            //         } else {
            //             return vec![];
            //         }
            //     } else if parent_match {
            //         let d = self.ana.solver.nodes.with(target);
            //         let o = if let RefsEnum::ScopedIdentifier(o, _) = d.as_ref() {
            //             let d = self.ana.solver.nodes.with(*o);
            //             if eq_root_scoped(d, self.stores, scop.1) {
            //                 Some(*o)
            //             } else {
            //                 None
            //             }
            //         } else {
            //             None
            //         };
            //         if let Some(o) = o {
            //             println!("import matched inner type ref");
            //             if let Some(x) = &self.ana.solver.try_unsolve_node_with(target, o) {
            //                 return vec![*x];
            //             } else {
            //                 panic!();
            //             }
            //         } else {
            //             return vec![];
            //         }
            //     } else {
            //         let d = self.ana.solver.nodes.with(target);
            //         if eq_root_scoped(d, self.stores, scop.1) {
            //             // TODO use self.ana.solver.try_unsolve_node_with
            //             let d = self.ana.solver.nodes.with(target);
            //             let i = if let RefsEnum::ScopedIdentifier(_, i) = d.as_ref() {
            //                 *i
            //             } else {
            //                 panic!()
            //             };
            //             let o = self.ana.solver.intern(RefsEnum::MaybeMissing);
            //             let i = self.ana.solver.intern(RefsEnum::ScopedIdentifier(o, i));
            //             // let i = handle_import(
            //             //     java_tree_gen,
            //             //     self.ana,
            //             //     self.stores.node_store.resolve(scop.0),
            //             // );
            //             println!("import matched ref");
            //             return vec![i];
            //         } else {
            //             return vec![];
            //         }
            //     }
            // } else {
            //     println!("Do not contains");
            //     return vec![];
            // }
        }
        if !b.has_children() {
            return vec![];
        }
        println!("d=1 {:?}", &t);
        let c = if b.get_component::<BloomSize>().is_ok() {
            let d = self.ana.solver.nodes.with(target);
            b.check(Into::<Box<[u8]>>::into(d.clone()).as_ref())
        } else {
            BloomResult::MaybeContain
        };

        struct IoOut<W: std::io::Write> {
            stream: W,
        }

        impl<W: std::io::Write> std::fmt::Write for IoOut<W> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.stream
                    .write_all(s.as_bytes())
                    .map_err(|_| std::fmt::Error)
            }
        }
        if let BloomResult::MaybeContain = c {
            println!("++++++++++++++Maybe contains");

            if &t == &Type::MethodInvocation // find object
                || &t == &Type::FormalParameter // find simple type
                || &t == &Type::GenericType // find simple type
                || &t == &Type::TypeBound // find simple type
                || &t == &Type::ObjectCreationExpression // find simple object
                || &t == &Type::ScopedIdentifier // find identifier
                || &t == &Type::ScopedAbsoluteIdentifier // find identifier
                || &t == &Type::ScopedTypeIdentifier
                || &t == &Type::CatchType // TODO to check
                || &t == &Type::FieldAccess // TODO to check
                || &t == &Type::FieldDeclaration // TODO to check
                || &t == &Type::Superclass // TODO to check for hierachy
                || &t == &Type::SuperInterfaces // TODO to check for hierachy
                || &t == &Type::ExtendsInterfaces // TODO to check for hierachy
                || &t == &Type::InstanceofExpression // TODO to handle
                || &t == &Type::AnnotatedType // TODO to handle
                || &t == &Type::ClassLiteral // to handle A.class
                || &t == &Type::ArrayType // TODO to handle A[]
                || &t == &Type::MethodReference // TODO to handle A::m
                || &t == &Type::CastExpression // TODO to handle (A)x
                // || &t == &Type::ConstructorDeclaration // TODO to handle constructors
                || &t == &Type::ConstantDeclaration // find simple type
                || &t == &Type::LocalVariableDeclaration // find simple type
                || &t == &Type::EnhancedForVariable
                // || &t == &Type::ForStatement // no need
                // || &t == &Type::TryWithResourcesStatement // no need
                // || &t == &Type::CatchClause // no need
                || &t == &Type::EnhancedForStatement // TODO to handle declarative staements
                || &t == &Type::Resource
            // TODO to check
            // find identifier
            {
                // Here, for now, we try to find Identifiers (not invocations)
                // thus we either search directly for scoped identifiers
                // or we search for simple identifiers because they do not present refs in themself
                println!("!found {:?}", &t);
                java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                    &self.stores.node_store,
                    &self.stores.label_store,
                    &current,
                );
                println!();

                // let d = ExplorableRef {
                //     rf: target,
                //     nodes: &self.ana.solver.nodes,
                // };

                // if eq_node_ref(d, self.stores, current) {
                //     // let mut position = path.to_position(&self.stores);
                //     // position.set_len(b.get_bytes_len() as usize);
                //     // println!("really found {:?}", position);
                //     let r = self.sp_store.push(scout);
                //     self.refs.push(r);
                //     let it = ExploreStructuralPositions::from((&*self.sp_store, r));
                //     println!("really found {:?}", it.to_position(&self.stores));
                //     // println!("really found");
                // }
                self.exact_match(target, scout.clone());
            // } else if &t == &Type::ClassDeclaration || &t == &Type::InterfaceDeclaration {
            //     // TODO to handle refs by heritage
            //     println!(
            //         "!found Class or Interface declaration at {:?}",
            //         scout.to_position(&self.sp_store, &self.stores)
            //     );
            //     // let mut out = IoOut { stream: stdout() };
            //     // java_tree_gen_full_compress_legion_ref::serialize(
            //     //     &self.stores.node_store,
            //     //     &self.stores.label_store,
            //     //     &current,
            //     //     &mut out,
            //     //     "\n",
            //     // );
            //     self.exact_match(target, scout.clone());
            } else if &t == &Type::TypeIdentifier {
                println!("!found TypeIdentifier");
                let mut out = IoOut { stream: stdout() };
                java_tree_gen_full_compress_legion_ref::serialize(
                    &self.stores.node_store,
                    &self.stores.label_store,
                    &current,
                    &mut out,
                    "\n",
                );
                println!();
                self.exact_match(target, scout.clone());
            } else if &t == &Type::MethodDeclaration {
                // java_tree_gen::print_tree_syntax(
                //     &self.stores.node_store,
                //     &self.stores.label_store,
                //     &x,
                // );
                let mut out = IoOut { stream: stdout() };
                java_tree_gen_full_compress_legion_ref::serialize(
                    &self.stores.node_store,
                    &self.stores.label_store,
                    &current,
                    &mut out,
                    "\n",
                );
                println!();
                self.exact_match(target, scout.clone());
            }
        } else {
            println!("Do not contains");
            return vec![];
        }

        let mut v: Vec<usize> = vec![];
        println!("c_count {}", b.child_count());
        // scout.down();
        let mut i = 0;
        for x in b.get_children().clone() {
            // scout.inc(*x);
            scout.goto(*x, i);
            i += 1;
            log::trace!(
                "rec search ref {}",
                DisplayRef::from((self.ana.solver.nodes.with(target), &self.stores.label_store)),
            );
            let z = self.find_refs(package, target, scout);
            for w in v.clone() {
                log::trace!(
                    "rec search ref {}",
                    DisplayRef::from((self.ana.solver.nodes.with(w), &self.stores.label_store)),
                );
                let z = self.find_refs(package, w, scout);
                v.extend(z)
            }
            v.extend(z);
            scout.up(&self.sp_store);
        }
        vec![]
    }

    pub fn exact_match(&mut self, target: RefPtr, mut scout: Scout) {
        let d = ExplorableRef {
            rf: target,
            nodes: &self.ana.solver.nodes,
        };

        match d.as_ref().clone() {
            RefsEnum::Root => todo!(),
            RefsEnum::MaybeMissing => todo!(),
            RefsEnum::ScopedIdentifier(o, i) => self.exact_match_aux(o, i.as_ref(), scout),
            RefsEnum::TypeIdentifier(o, i) => self.exact_match_aux(o, i.as_ref(), scout),
            RefsEnum::MethodReference(_, _) => todo!(),
            RefsEnum::ConstructorReference(_) => todo!(),
            RefsEnum::Invocation(_, _, _) => todo!(),
            RefsEnum::ConstructorInvocation(_, _) => todo!(),
            RefsEnum::Primitive(_) => todo!(),
            RefsEnum::Array(_) => todo!(),
            RefsEnum::This(_) => {
                let b = self.stores.node_store.resolve(scout.node(self.sp_store));
                if b.get_type() != Type::This {
                    // println!("not matched");
                } else {
                    assert!(!b.has_children()); // TODO
                    self.successful_match(&mut scout);
                }
            }
            RefsEnum::Super(_) => todo!(),
            RefsEnum::ArrayAccess(_) => todo!(),
            RefsEnum::Mask(_, _) => todo!(),
            RefsEnum::Or(_) => todo!(),
        }
    }

    pub fn exact_match_aux(&mut self, o: RefPtr, i: &LabelIdentifier, mut scout: Scout) {
        let b = self.stores.node_store.resolve(scout.node(&self.sp_store));
        let t = b.get_type();
        if t == Type::MethodInvocation {
            let x = b.get_child(&0);
            scout.goto(x, 0);
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t == Type::TypeIdentifier {
                if let Some(l) = b.try_get_label() {
                    if l == i {
                        println!("success 1");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::Identifier {
                if let Some(l) = b.try_get_label() {
                    if l == i {
                        println!("success 2");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            // } else if t == Type::FieldAccess {
            //     // if b.has_label() {
            //     //     let l = b.get_label();
            //     //     if l != i {
            //     //         // println!("not matched"); // TODO
            //     //     } else {println!("success 2.1");
            //     //         self.successful_match(&mut scout); // TODO
            //     //     }
            //     // } else {
            //     //     todo!()
            //     // }
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else if t == Type::This {
                // TODO if scoped might be handled after
            } else if t == Type::Super {
                // TODO if scoped might be handled after
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::ObjectCreationExpression {
            let (b, t) = {
                let mut i = 0;
                let mut r;
                let mut t;
                loop {
                    let x = b.get_child(&i);
                    r = self.stores.node_store.resolve(x);
                    t = r.get_type();
                    if t == Type::TS74 {
                        // find new
                        // TODO but should alse construct the fully qualified name in the mean time
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                loop {
                    let x = b.get_child(&i);
                    r = self.stores.node_store.resolve(x);
                    t = r.get_type();
                    if t != Type::Spaces
                        && t != Type::Comment
                        && t != Type::MarkerAnnotation
                        && t != Type::Annotation
                    {
                        // scout.goto(x, i as usize);
                        break;
                    }
                    i += 1;
                }
                (r, t)
            };
            if t == Type::TypeIdentifier {
                if let Some(l) = b.try_get_label() {
                    if l == i {
                        println!("success 3");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::GenericType {
                // println!("not matched"); // should be handled after
            } else if t == Type::ScopedTypeIdentifier {
                // println!("not matched"); // should be handled after
            } else if t == Type::TypeArguments {
                // println!("not matched"); // should be handled after
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::MethodDeclaration {
            let (r, t) = {
                let mut i = 0;
                let r;
                let t;
                loop {
                    let x = b.get_child(&i);
                    let b = self.stores.node_store.resolve(x);
                    let tt = b.get_type();
                    if tt == Type::Modifiers {
                        i += 1;
                    } else if tt == Type::TypeParameters {
                        i += 1;
                    } else if tt == Type::Annotation {
                        i += 1;
                    } else if tt == Type::Spaces {
                        i += 1;
                    } else {
                        r = b;
                        t = tt;
                        break;
                    }
                }
                (r, t)
            };
            if t == Type::TypeIdentifier {
                if let Some(l) = r.try_get_label() {
                    if l == i {
                        self.successful_match(&mut scout);
                    }
                } else {
                    todo!()
                }
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::FormalParameter
            || t == Type::LocalVariableDeclaration
            || t == Type::EnhancedForVariable
            || t == Type::ConstantDeclaration
        {
            let (r, t) = {
                let x = b.get_child(&0);
                let r = self.stores.node_store.resolve(x);
                let t = r.get_type();
                if t == Type::Modifiers {
                    let x = b.get_child(&2);
                    let r = self.stores.node_store.resolve(x);
                    let t = r.get_type();
                    // scout.goto(x, 2);
                    (r, t)
                } else {
                    // scout.goto(x, 0);
                    (r, t)
                }
            };
            if t == Type::TypeIdentifier {
                if let Some(l) = r.try_get_label() {
                    if l == i {
                        println!("success 4");
                        self.successful_match(&mut scout);
                    }
                } else {
                    todo!()
                }
            } else if is_individually_matched(t) {
                // println!("not matched"); // should be handled after
            } else if is_never_reference(t) {

                // println!("not matched");
            } else if t == Type::ArrayType {
                // println!("not matched"); // TODO not sure
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::SuperInterfaces
            || t == Type::Superclass
            || t == Type::ExtendsInterfaces
            || t == Type::TypeBound
        {
            let mut j = 0;
            let mut r;
            let mut t;
            let mut ok = false;
            let l = b.child_count();
            while j < l {
                let x = b.get_child(&j);
                r = self.stores.node_store.resolve(x);
                t = r.get_type();
                if t == Type::TS60 {
                    // ext
                    ok = true;
                } else if t == Type::TS66 {
                    // impl
                    ok = true;
                } else if t == Type::TS14 { // comma
                } else if t == Type::Spaces {
                } else if ok {
                    if t == Type::TypeIdentifier {
                        if let Some(l) = r.try_get_label() {
                            if l != i {
                                // println!("not matched"); // TODO
                            } else {
                                println!("success 4");
                                scout.up(self.sp_store);
                                self.successful_match(&mut scout);
                                break;
                            }
                        } else {
                            todo!()
                        }
                    } else if t == Type::AnnotatedType {
                        let x = r.get_child_rev(&0);
                        // let l = r.child_count();
                        let b = self.stores.node_store.resolve(x);
                        let t = b.get_type();
                        if t == Type::TypeIdentifier {
                            // scout.goto(x, l as usize - 1);
                            if let Some(l) = b.try_get_label() {
                                if l != i {
                                    // println!("not matched"); // TODO
                                } else {
                                    println!("success 6");
                                    self.successful_match(&mut scout); // TODO
                                }
                            } else {
                                todo!()
                            }
                        } else if t == Type::ScopedTypeIdentifier {
                            // println!("not matched"); // TODO should check the fully qual name
                        } else if t == Type::GenericType {
                            // println!("not matched"); // TODO should check the fully qual name
                        } else if is_individually_matched(t) || is_never_reference(t) {
                        } else {
                            todo!("{:?}", t)
                        }
                    } else if t == Type::TS4 {
                    } else if is_individually_matched(t) || is_never_reference(t) {
                    } else {
                        todo!("{:?}", t)
                    }
                }
                j += 1;
            }
        } else if &t == &Type::EnhancedForStatement {
            let mut j = 0;
            let mut r;
            let mut t;
            let mut ok = false;
            let mut ok2 = false;
            let l = b.child_count();
            while j < l {
                let x = b.get_child(&j);
                r = self.stores.node_store.resolve(x);
                t = r.get_type();
                if t == Type::TS7 {
                    // (
                    ok = true;
                } else if t == Type::TS8 {
                    // )
                    break;
                } else if t == Type::Spaces {
                } else if t == Type::TS23 {
                    // :
                    ok = false;
                    ok2 = true;
                } else if ok {
                    if t == Type::TypeIdentifier {
                        if let Some(l) = r.try_get_label() {
                            if l != i {
                                // println!("not matched"); // TODO
                            } else {
                                println!("success 10");
                                scout.goto(x, j as usize);
                                self.successful_match(&mut scout);
                                scout.up(self.sp_store);
                            }
                        } else {
                            todo!()
                        }
                    } else if t == Type::Identifier {
                    } else if is_individually_matched(t) || is_never_reference(t) {
                    } else {
                        todo!("{:?}", t)
                    }
                } else if ok2 {
                    if t == Type::Identifier {
                        if let Some(l) = r.try_get_label() {
                            if l != i {
                                // println!("not matched"); // TODO
                            } else {
                                println!("success 11");
                                scout.goto(x, j as usize);
                                self.successful_match(&mut scout);
                                scout.up(self.sp_store);
                            }
                        } else {
                            todo!()
                        }
                    } else if t == Type::This {
                    } else if is_individually_matched(t) || is_never_reference(t) {
                    } else {
                        todo!("{:?}", t)
                    }
                }
                j += 1;
            }
        } else if t == Type::GenericType {
            let x = b.get_child(&0);
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t == Type::TypeIdentifier {
                // scout.goto(x, 0);
                if let Some(l) = b.try_get_label() {
                    if l == i {
                        println!("success 5");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::This { // TODO
            } else if t == Type::ScopedTypeIdentifier {
                // println!("not matched"); // should be handled after
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::ArrayType {
            let x = b.get_child(&0);
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t == Type::TypeIdentifier {
                scout.goto(x, 0);
                if let Some(l) = b.try_get_label() {
                    if l == i {
                        println!("success 5.1");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::ScopedTypeIdentifier {
                // println!("not matched"); // should be handled after
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::InstanceofExpression {
            let x = b.get_child(&0);
            let bb = self.stores.node_store.resolve(x);
            let t = bb.get_type();
            if t == Type::Identifier {
                if let Some(l) = bb.try_get_label() {
                    // TODO should also check if is fully qual then equal to target
                    if l == i {
                        // if o == MaybeMissing
                        println!("success 6");
                        scout.goto(x, 0);
                        self.successful_match(&mut scout); // TODO
                        scout.up(self.sp_store);
                    }
                } else {
                    todo!()
                }
            } else if t == Type::This { // TODO
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else {
                todo!("{:?}", t)
            }
            let x = b.get_child_rev(&0);
            let len = b.child_count();
            let bb = self.stores.node_store.resolve(x);
            let t = bb.get_type();
            if t == Type::TypeIdentifier || t == Type::Identifier {
                let l = bb.try_get_label().unwrap();
                // TODO should also check if is fully qual then equal to target
                if l != i {
                    // println!("not matched"); // TODO
                } else {
                    // if o == MaybeMissing
                    println!("success 6.1");
                    scout.goto(x, len as usize - 1);
                    self.successful_match(&mut scout); // TODO
                    scout.up(self.sp_store);
                }
            } else if t == Type::This {
                panic!(); // sure ?
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::CastExpression {
            let mut j = 0;
            let mut r;
            let mut t;
            let mut ok = false;
            let mut ok2 = false;
            let l = b.child_count();
            while j < l {
                let x = b.get_child(&j);
                r = self.stores.node_store.resolve(x);
                t = r.get_type();
                if t == Type::TS7 {
                    // (
                    ok = true;
                } else if t == Type::TS8 {
                    // )
                    ok = false;
                    ok2 = true;
                } else if t == Type::Spaces {
                } else if ok {
                    if t == Type::TypeIdentifier {
                        if let Some(l) = r.try_get_label() {
                            if l != i {
                                // println!("not matched"); // TODO
                            } else {
                                println!("success 15");
                                // let mut scout = scout.clone();
                                // scout.up(self.sp_store);
                                self.successful_match(&mut scout);
                            }
                        } else {
                            todo!()
                        }
                    } else if t == Type::Identifier {
                    } else if t == Type::TS4 {
                    } else if is_individually_matched(t) || is_never_reference(t) {
                    } else {
                        todo!("{:?}", t)
                    }
                } else if ok2 {
                    if t == Type::Identifier {
                        if let Some(l) = r.try_get_label() {
                            if l != i {
                                // println!("not matched"); // TODO
                            } else {
                                println!("success 15.1");
                                // let mut scout = scout.clone();
                                // scout.up(self.sp_store);
                                self.successful_match(&mut scout);
                            }
                        } else {
                            todo!()
                        }
                    } else if t == Type::This {
                    } else if t == Type::LambdaExpression {
                    } else if is_individually_matched(t) || is_never_reference(t) {
                    } else {
                        todo!("{:?}", t)
                    }
                }
                j += 1;
            }
        } else if t == Type::MethodReference {
            let x = b.get_child(&0);
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t == Type::TypeIdentifier {
                if let Some(l) = b.try_get_label() {
                    // TODO should also check if is fully qual then equal to target
                    if l != i {
                        // println!("not matched"); // TODO
                    } else if &RefsEnum::MaybeMissing == self.ana.solver.nodes.with(o).as_ref() {
                        scout.goto(x, 0);
                        println!("success 6.1");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::Identifier {
                if let Some(l) = b.try_get_label() {
                    // TODO should also check if is fully qual then equal to target
                    if l != i {
                        // println!("not matched"); // TODO
                    } else if &RefsEnum::MaybeMissing == self.ana.solver.nodes.with(o).as_ref() {
                        scout.goto(x, 0);
                        println!("success 6.1");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::TS74 { // TODO contructor
            } else if t == Type::This {
            } else if t == Type::ScopedTypeIdentifier {
                // println!("not matched"); // TODO should check the fully qual name
            } else if t == Type::GenericType {
                // println!("not matched"); // TODO should check the fully qual name
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::FieldAccess {
            if let Some(mut scout) = self.is_field_access_exact_match(&b, o, i, scout) {
                self.successful_match(&mut scout);
            }
        } else if t == Type::ScopedTypeIdentifier {
            if let Some(mut scout) = self.is_scoped_type_identifier_exact_match(&b, o, i, scout) {
                self.successful_match(&mut scout);
            }
        } else if t == Type::ClassLiteral {
            let x = b.get_child(&0);
            let bb = self.stores.node_store.resolve(x);
            let t = bb.get_type();
            if t == Type::TypeIdentifier || t == Type::Identifier {
                if let Some(l) = b.try_get_label() {
                    if l == i {
                        scout.goto(x, 0);
                        scout.check(self.stores).unwrap();
                        println!("success 8");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::AnnotatedType {
            let x = b.get_child_rev(&0);
            let len = b.child_count() as usize;
            let bb = self.stores.node_store.resolve(x);
            let t = bb.get_type();
            if t == Type::TypeIdentifier {
                if let Some(l) = bb.try_get_label() {
                    if l == i {
                        scout.goto(x, len - 1);
                        scout.check(self.stores).unwrap();
                        println!("success 9");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::ScopedTypeIdentifier {
                // println!("not matched"); // TODO should check the fully qual name
            } else if t == Type::GenericType {
                // println!("not matched"); // TODO should check the fully qual name
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::FieldDeclaration {
            let (r, t) = {
                let mut i = 0;
                let r;
                let t;
                loop {
                    let x = b.get_child(&i);
                    let b = self.stores.node_store.resolve(x);
                    let tt = b.get_type();
                    if tt == Type::Modifiers {
                        i += 1;
                    } else if tt == Type::Annotation {
                        i += 1;
                    } else if tt == Type::Spaces {
                        i += 1;
                    } else {
                        r = b;
                        t = tt;
                        break;
                    }
                }
                (r, t)
            };
            if t == Type::TypeIdentifier {
                if let Some(l) = r.try_get_label() {
                    if l == i {
                        self.successful_match(&mut scout);
                    }
                } else {
                    todo!()
                }
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else if t == Type::ArrayType {
                // println!("not matched"); // TODO not sure
            } else {
                todo!("{:?}", t)
            }
        // } else if t == Type::ClassDeclaration {
        //     // todo!()
        //     // find extends and implements then easy
        // } else if t == Type::InterfaceDeclaration {
        //     // todo!()
        // find extends then easy
        } else if t == Type::ConstructorDeclaration {
            // todo!()
            // get ?.identifier from contructor then compare to ref
        } else if t == Type::CatchType {
            // TODO check for type union eg. A|B|C
            let x = b.get_child(&0);
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            // scout.goto(x, 0);
            if t == Type::TypeIdentifier {
                if let Some(l) = b.try_get_label() {
                    if l == i {
                        println!("success 7");
                        scout.up(self.sp_store);
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if is_individually_matched(t) || is_never_reference(t) {
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::Resource {
            // let l = b.child_count();
            let mut j = 0;
            loop {
                let x = b.get_child(&j);
                let r = self.stores.node_store.resolve(x);
                let t = r.get_type();
                if t == Type::TypeIdentifier {
                    scout.goto(x, j as usize);
                    return if let Some(l) = r.try_get_label() {
                        if l != i {
                            // println!("not matched"); // TODO
                        } else {
                            println!("success 8");
                            self.successful_match(&mut scout); // TODO
                        }
                    } else {
                        todo!()
                    };
                } else if t == Type::Modifiers {
                    // } else if t == Type::ScopedTypeIdentifier {
                    //     break;
                    // } else if  j == l{
                    //     break;
                } else {
                    break;
                }
                j += 1;
            }
            // println!("not matched");
        } else {
            todo!("{:?}", t)
        }
    }

    fn is_field_access_exact_match(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &string_interner::symbol::SymbolU32,
        mut scout: Scout,
    ) -> Option<Scout> {
        // TODO should handle and do a test case for explicit access to the member of a parent instance eg. A.super.b
        let (o_id, sup, i_id) = {
            let cs = b.get_children();

            (cs[0], cs.len() > 3, cs[cs.len() - 1])
        };
        let o_b = self.stores.node_store.resolve(o_id);
        let o_t = o_b.get_type();
        let i_b = self.stores.node_store.resolve(i_id);
        let i_t = i_b.get_type();
        if i_t == Type::This {
            if o_t == Type::FieldAccess {
                scout.goto(o_id, 0);
                return self.is_field_access_exact_match(&o_b, o, &i, scout);
            } else if let Some(l) = o_b.try_get_label() {
                match self.ana.solver.nodes.with(o).as_ref() {
                    RefsEnum::MaybeMissing => {
                        if l == i {
                            // match ?.a on a.b
                            scout.goto(o_id, 0);
                            scout.check(self.stores).unwrap();
                            println!("success is_field_access_exact_match 1");
                            return Some(scout);
                        }
                    }
                    _ => return None,
                }
            } else {
                return None;
            }
        }
        let mut matching_o = false;
        if o_t == Type::Identifier {
            if let Some(l) = o_b.try_get_label() {
                match self.ana.solver.nodes.with(o).as_ref() {
                    RefsEnum::MaybeMissing => {
                        if l == i {
                            // match ?.a on a.b
                            scout.goto(o_id, 0);
                            scout.check(self.stores).unwrap();
                            println!("success is_field_access_exact_match 1.1");
                            return Some(scout);
                        }
                    }
                    RefsEnum::ScopedIdentifier(oo, ii) => {
                        match self.ana.solver.nodes.with(*oo).as_ref() {
                            // match (?.a).b on (a).b
                            RefsEnum::MaybeMissing if l == ii.as_ref() => matching_o = true,
                            _ => return None,
                        }
                    }
                    _ => return None,
                }
            } else {
                todo!()
            }
        } else if o_t == Type::This {
            // [ ] There should be a ref finder (the recusive part that go though decls) specialized for this,
            // as it needs to handle going through type declarations (because it changes how `this` should be match).
            // NOTE but just the exact match should be ok matching this if it is what ze want to match,
            // only the recursive search needs to be specialized,
            // [ ] actually for this reason the exact match should not traverse type declarations.
            // NOTE I think there should also be more logic used when searching through hierarchy.
        } else if o_t == Type::FieldAccess {
            let (o, i) = match self.ana.solver.nodes.with(o).as_ref() {
                RefsEnum::ScopedIdentifier(oo, ii) => (*oo, *ii.as_ref()),
                RefsEnum::TypeIdentifier(_, _) => {
                    todo!()
                }
                RefsEnum::MaybeMissing | RefsEnum::Root => {
                    return None;
                }
                x => {
                    todo!("{:?}", x); // return None; // TODO handle other cases
                }
            };
            let mut scout = scout.clone();
            scout.goto(o_id, 0);

            log::trace!(
                "recursive call to is_field_access_exact_match {:?}",
                self.ana.solver.nodes.with(o)
            );
            if let Some(_) = self.is_field_access_exact_match(&o_b, o, &i, scout) {
                matching_o = true;
            }
        } else if o_t == Type::ScopedTypeIdentifier {
        } else if o_t == Type::GenericType {
        } else if is_individually_matched(o_t) || is_never_reference(o_t) {
        } else {
            todo!("{:?}", o_t)
        }
        if matching_o {
            if i_t == Type::Identifier {
                if let Some(l) = i_b.try_get_label() {
                    if l == i {
                        // scout.goto(x, 2);
                        scout.check(self.stores).unwrap();
                        println!("success is_field_access_exact_match 3");
                        return Some(scout);
                    }
                } else {
                    todo!()
                }
            // } else if i_t == Type::ScopedTypeIdentifier {
            // } else if i_t == Type::GenericType {
            // } else if is_individually_matched(i_t) || is_never_reference(i_t) {
            } else {
                todo!("{:?}", i_t)
            }
        }
        None
    }
    fn is_scoped_type_identifier_exact_match(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &string_interner::symbol::SymbolU32,
        mut scout: Scout,
    ) -> Option<Scout> {
        // TODO should handle and do a test case for explicit access to the member of a parent instance eg. A.super.b
        let x = b.get_child(&0);
        let bb = self.stores.node_store.resolve(x);
        let t = bb.get_type();
        let mut matching_o = false;
        if t == Type::TypeIdentifier {
            if let Some(l) = bb.try_get_label() {
                match self.ana.solver.nodes.with(o).as_ref() {
                    RefsEnum::MaybeMissing => {
                        if l == i {
                            scout.goto(x, 0);
                            scout.check(self.stores).unwrap();
                            println!("success is_scoped_type_identifier_exact_match 1");
                            return Some(scout);
                        }
                    }
                    RefsEnum::ScopedIdentifier(oo, ii) => {
                        match self.ana.solver.nodes.with(*oo).as_ref() {
                            RefsEnum::MaybeMissing => {
                                if l == ii.as_ref() {
                                    matching_o = true;
                                }
                            }
                            _ => {
                                return None;
                            }
                        }
                    }
                    _ => {
                        return None;
                    }
                }
            } else {
                todo!()
            }
        } else if t == Type::ScopedTypeIdentifier {
            let (o, i) = match self.ana.solver.nodes.with(o).as_ref() {
                RefsEnum::ScopedIdentifier(oo, ii) => (*oo, *ii.as_ref()),
                RefsEnum::TypeIdentifier(_, _) => {
                    todo!()
                }
                RefsEnum::MaybeMissing | RefsEnum::Root => {
                    return None;
                }
                x => {
                    todo!("{:?}", x); // return None; // TODO handle other cases
                }
            };
            let mut scout = scout.clone();
            scout.goto(x, 0);

            log::trace!(
                "recursive call to is_scoped_type_identifier_exact_match {:?}",
                self.ana.solver.nodes.with(o)
            );
            if let Some(_) = self.is_scoped_type_identifier_exact_match(&bb, o, &i, scout) {
                matching_o = true;
            }
        } else if t == Type::GenericType {
            // println!("not matched"); // TODO should check the fully qual name
        } else if is_individually_matched(t) || is_never_reference(t) {
        } else {
            todo!("{:?}", t)
        }
        if matching_o {
            let x = b.get_child_rev(&0);
            let bb = self.stores.node_store.resolve(x);
            let t = bb.get_type();

            if t == Type::TypeIdentifier {
                if let Some(l) = bb.try_get_label() {
                    if l == i {
                        // scout.goto(x, 2);
                        scout.check(self.stores).unwrap();
                        println!("success is_scoped_type_identifier_exact_match 3");
                        return Some(scout);
                    }
                } else {
                    todo!()
                }
            } else {
                panic!("{:?}", t)
            }
        }
        None
    }
    pub fn successful_match(&mut self, scout: &mut Scout) {
        self.sp_store.check(&self.stores).unwrap();
        scout.check(&self.stores).unwrap();
        let r = self.sp_store.push(scout);
        self.sp_store.check(&self.stores).unwrap();
        if ADAPT_SPOON {
            let mut scout = scout.clone();
            let r = if let Some(p) = scout.up(&self.sp_store) {
                let b = self.stores.node_store.resolve(p);
                let t = b.get_type();
                if t == Type::SuperInterfaces
                    || t == Type::Superclass
                    || t == Type::ExtendsInterfaces
                {
                    scout.up(self.sp_store);
                    self.sp_store.push(&mut scout)
                } else if t == Type::FieldDeclaration
                    || t == Type::MethodDeclaration
                    || t == Type::LocalVariableDeclaration
                    || t == Type::FormalParameter
                    || t == Type::TypeParameter
                    || t == Type::EnhancedForVariable
                    || t == Type::ExpressionStatement
                    || t == Type::CastExpression
                    || t == Type::ObjectCreationExpression
                {
                    self.sp_store.push(&mut scout)
                } else if t == Type::GenericType {
                    let mut scout2 = scout.clone();
                    if let Some(p) = scout2.up(&self.sp_store) {
                        let b = self.stores.node_store.resolve(p);
                        let t = b.get_type();
                        if t == Type::FieldDeclaration
                            || t == Type::MethodDeclaration
                            || t == Type::LocalVariableDeclaration
                            || t == Type::FormalParameter
                            || t == Type::TypeParameter
                            || t == Type::EnhancedForVariable
                            || t == Type::ExpressionStatement
                            || t == Type::CastExpression
                            || t == Type::ObjectCreationExpression
                        {
                            self.sp_store.push(&mut scout2)
                        } else if t == Type::SuperInterfaces
                            || t == Type::Superclass
                            || t == Type::ExtendsInterfaces
                        {
                            scout2.up(self.sp_store);
                            self.sp_store.push(&mut scout2)
                        } else if t == Type::TypeBound {
                            scout2.up(self.sp_store);
                            self.sp_store.push(&mut scout2)
                        } else {
                            self.sp_store.push(&mut scout)
                        }
                    } else {
                        self.sp_store.push(&mut scout)
                    }
                } else if t == Type::TypeBound {
                    scout.up(self.sp_store);
                    self.sp_store.push(&mut scout)
                } else {
                    r
                }
            } else {
                r
            };
            self.sp_store.check(&self.stores).unwrap();
            self.refs.push(r);
            let it = ExploreStructuralPositions::from((&*self.sp_store, r));
            println!("really found {:?}", it.to_position(&self.stores));
        } else {
            self.sp_store.check(&self.stores).unwrap();
            self.refs.push(r);
            let it = ExploreStructuralPositions::from((&*self.sp_store, r));
            println!("really found {:?}", it.to_position(&self.stores));
        }
    }
}

fn is_individually_matched(t: Type) -> bool {
    t == Type::ScopedTypeIdentifier
        || t == Type::FieldAccess
        || t == Type::ScopedIdentifier
        || t == Type::MethodInvocation
        || t == Type::ArrayAccess
        || t == Type::ObjectCreationExpression
        || t == Type::ParenthesizedExpression
        || t == Type::TernaryExpression
        || t == Type::GenericType
        || t == Type::AnnotatedType
        || t == Type::ArrayType
        || t == Type::CastExpression
        || t == Type::Modifiers
        || t == Type::ArrayCreationExpression
        || t == Type::BinaryExpression
        || t == Type::UnaryExpression
        || t == Type::SwitchExpression
        || t == Type::AssignmentExpression
        || t == Type::EnhancedForVariable
}
fn is_never_reference(t: Type) -> bool {
    t == Type::Comment
    || t == Type::ClassLiteral // out of scope for tool ie. reflexivity
    || t == Type::StringLiteral
    || t == Type::NullLiteral
    || t == Type::VoidType
    || t == Type::IntegralType
    || t == Type::FloatingPointType
    || t == Type::BooleanType
    // println!("not matched"); // TODO not sure
}

const ADAPT_SPOON: bool = true;

pub struct IterDeclarations<'a> {
    stores: &'a SimpleStores,
    parents: Vec<NodeIdentifier>,
    offsets: Vec<usize>,
    /// to tell that we need to pop a parent, we could also use a bitvec instead of Option::None
    remaining: Vec<Option<NodeIdentifier>>,
}

impl<'a> Debug for IterDeclarations<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterDeclarations")
            .field("parents", &self.parents())
            .field("offsets", &self.offsets())
            .field("remaining", &self.remaining)
            .finish()
    }
}

impl<'a> Iterator for IterDeclarations<'a> {
    type Item = NodeIdentifier;

    fn next(&mut self) -> Option<Self::Item> {
        let x;
        loop {
            if let Some(c) = self.remaining.pop()? {
                self.offsets.last_mut().unwrap().add_assign(1);
                x = c;
                break;
            } else {
                self.offsets.pop();
                self.parents.pop();
            }
        }

        let b = self.stores.node_store.resolve(x);
        let t = b.get_type();

        if &t == &Type::Spaces {
            return self.next();
        } else if &t == &Type::Comment {
            return self.next();
        } else if &t == &Type::PackageDeclaration {
            return self.next();
        } else if &t == &Type::ImportDeclaration {
            return self.next();
        }

        self.parents.push(x);
        self.offsets.push(0);
        self.remaining.push(None);
        if b.has_children() {
            self.remaining
                .extend(b.get_children().iter().rev().map(|x| Some(*x)));
        }

        if t.is_type_declaration() {
            Some(x)
        } else if &t == &Type::LocalVariableDeclaration {
            Some(x)
        } else if &t == &Type::EnhancedForStatement {
            Some(x)
        } else if &t == &Type::Resource {
            // TODO also need to find an "=" and find the name just before
            Some(x)
        } else if t.is_value_member() {
            Some(x)
        } else if t.is_parameter() {
            Some(x)
        } else if t.is_executable_member() {
            Some(x)
        } else {
            while !self.remaining.is_empty() {
                if let Some(x) = self.next() {
                    return Some(x);
                }
            }
            None
        }
    }
}

impl<'a> IterDeclarations<'a> {
    pub fn new(stores: &'a SimpleStores, root: NodeIdentifier) -> Self {
        Self {
            stores,
            parents: vec![],
            offsets: vec![0],
            remaining: vec![Some(root)],
        }
    }
    pub fn parents(&self) -> &[NodeIdentifier] {
        &self.parents[..self.parents.len() - 1]
    }
    pub fn offsets(&self) -> &[usize] {
        &self.offsets[..self.offsets.len() - 1]
    }
    pub fn position(&self, x: NodeIdentifier) -> StructuralPosition {
        (self.parents().to_vec(), self.offsets().to_vec(), x).into()
    }
}
pub struct IterDeclarations2<'a> {
    stores: &'a SimpleStores,
    scout: Scout,
    stack: Vec<(NodeIdentifier, usize, Option<Vec<NodeIdentifier>>)>,
}

impl<'a> Debug for IterDeclarations2<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IterDeclarations2")
            // .field("parents", &self.parents())
            .finish()
    }
}

impl<'a> Iterator for IterDeclarations2<'a> {
    type Item = Scout;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (node, offset, children) = self.stack.pop()?;
            if let Some(children) = children {
                if offset < children.len() {
                    let child = children[offset];
                    self.scout.check(&self.stores).unwrap();
                    {
                        let b = self.stores.node_store.resolve(node);
                        if b.has_children() {
                            let cs = b.get_children();
                            // println!("children: {:?} {} {:?}", node,cs.len(),cs);
                            assert!(offset < cs.len());
                            assert_eq!(child, cs[offset]);
                        } else {
                            panic!()
                        }
                    }
                    if offset == 0 {
                        match self.scout.try_node() {
                            Ok(x) => assert_eq!(x, node),
                            Err(_) => {}
                        }
                        self.scout.goto(child, offset);
                        self.scout.check(&self.stores).unwrap();
                    } else {
                        match self.scout.try_node() {
                            Ok(x) => assert_eq!(x, children[offset - 1]),
                            Err(_) => {}
                        }
                        let i = self.scout.inc(child);
                        assert_eq!(i, offset);
                        self.scout.check_size(&self.stores).expect(&format!(
                            "{:?} {} {:?} {:?} {:?}",
                            node,
                            offset,
                            child,
                            children.len(),
                            self.scout
                        ));
                        self.scout.check(&self.stores).expect(&format!(
                            "{:?} {} {:?} {:?} {:?}",
                            node, offset, child, children, self.scout
                        ));
                    }
                    self.stack.push((node, offset + 1, Some(children)));
                    self.stack.push((child, 0, None));
                    continue;
                } else {
                    self.scout.check(&self.stores).unwrap();
                    self.scout.try_up().expect("should not go higher than root");
                    self.scout.check(&self.stores).unwrap();
                    continue;
                }
            } else {
                let b = self.stores.node_store.resolve(node);
                let t = b.get_type();

                if &t == &Type::Spaces {
                    continue;
                } else if &t == &Type::Comment {
                    continue;
                } else if &t == &Type::PackageDeclaration {
                    continue;
                } else if &t == &Type::ImportDeclaration {
                    continue;
                }

                if b.has_children() {
                    let children = b.get_children();
                    self.stack.push((node, 0, Some(children.to_vec())));
                }

                if t.is_type_declaration()
                    || &t == &Type::LocalVariableDeclaration
                    || &t == &Type::EnhancedForStatement
                    || t.is_value_member()
                    || t.is_parameter()
                    || t.is_executable_member()
                {
                    assert!(b.has_children(), "{:?}", t);
                    self.scout.check(&self.stores).unwrap();
                    return Some(self.scout.clone());
                } else if &t == &Type::Resource {
                    assert!(b.has_children(), "{:?}", t);
                    self.scout.check(&self.stores).unwrap();
                    // TODO also need to find an "=" and find the name just before
                    return Some(self.scout.clone());
                } else {
                    continue;
                }
            }
        }
    }
}

impl<'a> IterDeclarations2<'a> {
    pub fn new(stores: &'a SimpleStores, scout: Scout, root: NodeIdentifier) -> Self {
        let stack = vec![(root, 0, None)];
        Self {
            stores,
            scout,
            stack,
        }
    }
}

pub fn find_all_decls(
    stores: &SimpleStores,
    ana: &mut PartialAnalysis,
    x: NodeIdentifier,
) -> Vec<usize> {
    let b = stores.node_store.resolve(x);
    let t = b.get_type();
    if &t == &Type::Spaces {
        return vec![];
    } else if &t == &Type::Comment {
        return vec![];
    } else if &t == &Type::PackageDeclaration {
        return vec![];
    } else if &t == &Type::Directory {
        // TODO if package, get top level declarations then localize if ref.
        // in the end we do not need due to the way we do the impact ana.
        // we should only come from parent of package with canonical id.
    } else if &t == &Type::MavenDirectory {
        // idem
    } else if &t == &Type::ImportDeclaration {
        return vec![];
    }
    if !b.has_children() {
        return vec![];
    }

    struct IoOut<W: std::io::Write> {
        stream: W,
    }

    impl<W: std::io::Write> std::fmt::Write for IoOut<W> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.stream
                .write_all(s.as_bytes())
                .map_err(|_| std::fmt::Error)
        }
    }

    if t.is_type_declaration() {
        println!("!found decl: {:?}", &t);
        java_tree_gen_full_compress_legion_ref::print_tree_syntax(
            &stores.node_store,
            &stores.label_store,
            &x,
        );
        println!();
    } else if &t == &Type::LocalVariableDeclaration {
        println!("!found decl: {:?}", &t);
        java_tree_gen_full_compress_legion_ref::print_tree_syntax(
            &stores.node_store,
            &stores.label_store,
            &x,
        );
        println!();
    } else if &t == &Type::EnhancedForStatement {
        println!("!found decl: {:?}", &t);
        java_tree_gen_full_compress_legion_ref::print_tree_syntax(
            &stores.node_store,
            &stores.label_store,
            &x,
        );
        println!();
    } else if &t == &Type::Resource {
        // TODO also need to find an "=" and find the name just before
        println!("!found decl: {:?}", &t);
        java_tree_gen_full_compress_legion_ref::print_tree_syntax(
            &stores.node_store,
            &stores.label_store,
            &x,
        );
        println!();
    } else if t.is_value_member() {
        println!("!found decl: {:?}", &t);
        java_tree_gen_full_compress_legion_ref::print_tree_syntax(
            &stores.node_store,
            &stores.label_store,
            &x,
        );
        println!();
    } else if t.is_parameter() {
        println!("!found decl: {:?}", &t);
        java_tree_gen_full_compress_legion_ref::print_tree_syntax(
            &stores.node_store,
            &stores.label_store,
            &x,
        );
        println!();
    } else if t.is_executable_member() {
        println!("!found decl: {:?}", &t);
        java_tree_gen_full_compress_legion_ref::print_tree_syntax(
            &stores.node_store,
            &stores.label_store,
            &x,
        );
        println!();
    }

    let mut v: Vec<usize> = vec![];
    for x in b.get_children().clone() {
        let z = find_all_decls(stores, ana, *x);
        v.extend(z);
    }
    vec![]
}
