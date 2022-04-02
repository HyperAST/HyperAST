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
        StructuralPositionStore, TreePath,
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
}

/// Main traversal of HyperAST
/// Recusive traversal, it goes through declaration without handling them particularly
/// thus is should not search for references to `this` or `super`
impl<'a> RefsFinder<'a> {
    /// Find all references to `target` that was declared in `package`
    ///
    /// returns the indexes that should be used on self.sp_store the `StructuralPositionStore`
    pub fn find_all(mut self, package: RefPtr, target: RefPtr, mut x: Scout) -> Vec<usize> {
        self.find_refs(package, target, &mut x);
        self.refs
    }

    fn find_refs(&mut self, package: RefPtr, target: RefPtr, scout: &mut Scout) -> Vec<RefPtr> {
        let current = scout.node_always(&self.sp_store);
        let b = self.stores.node_store.resolve(current);
        let t = b.get_type();
        if t == Type::Spaces {
            return vec![];
        } else if t == Type::Comment {
            return vec![];
        } else if t == Type::PackageDeclaration {
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
        } else if t == Type::Program {
            println!(
                "d=1 {} {:?} {:?}",
                DisplayRef::from((self.ana.solver.nodes.with(target), &self.stores.label_store)),
                &t,
                scout.to_position(&self.sp_store, &self.stores)
            );
        } else if t == Type::Directory {
            println!(
                "d=1 {} {:?} {:?}",
                DisplayRef::from((self.ana.solver.nodes.with(target), &self.stores.label_store)),
                &t,
                scout.to_position(&self.sp_store, &self.stores)
            );
            // TODO if package, get top level declarations then localize if ref.
            // in the end we do not need due to the way we do the impact ana.
            // we should only come from parent of package with canonical id.
        } else if t == Type::MavenDirectory {
            println!(
                "d=1 {:?} {:?}",
                &t,
                scout.to_position(&self.sp_store, &self.stores)
            );
            // idem
        } else if t == Type::ImportDeclaration {
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

            if t == Type::MethodInvocation // find object
                || t == Type::FormalParameter // find simple type
                || t == Type::GenericType // find simple type
                || t == Type::TypeBound // find simple type
                || t == Type::ObjectCreationExpression // find simple object
                || t == Type::ScopedIdentifier // find identifier
                || t == Type::ScopedAbsoluteIdentifier // find identifier
                || t == Type::ScopedTypeIdentifier
                || t == Type::CatchType // TODO to check
                || t == Type::FieldAccess // TODO to check
                || t == Type::FieldDeclaration // TODO to check
                || t == Type::Superclass // TODO to check for hierachy
                || t == Type::SuperInterfaces // TODO to check for hierachy
                || t == Type::ExtendsInterfaces // TODO to check for hierachy
                || t == Type::InstanceofExpression // TODO to handle
                || t == Type::AnnotatedType // TODO to handle
                || t == Type::ClassLiteral // to handle A.class
                || t == Type::ArrayType // TODO to handle A[]
                || t == Type::MethodReference // TODO to handle A::m
                || t == Type::CastExpression // TODO to handle (A)x
                // || t == Type::ConstructorDeclaration // TODO to handle constructors
                || t == Type::ConstantDeclaration // find simple type
                || t == Type::LocalVariableDeclaration // find simple type
                || t == Type::EnhancedForVariable
                // || t == Type::ForStatement // no need
                // || t == Type::TryWithResourcesStatement // no need
                // || t == Type::CatchClause // no need
                || t == Type::EnhancedForStatement // TODO to handle declarative staements
                || t == Type::Resource
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
            // } else if t == Type::ClassDeclaration || t == Type::InterfaceDeclaration {
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
            } else if t == Type::TypeIdentifier {
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
            } else if t == Type::MethodDeclaration {
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
}

/// prerequisite: recusive traversal limited to expressions ie. should not cross declarations
impl<'a> RefsFinder<'a> {
    pub fn exact_match(&mut self, target: RefPtr, mut scout: Scout) {
        let d = ExplorableRef {
            rf: target,
            nodes: &self.ana.solver.nodes,
        };

        match d.as_ref().clone() {
            RefsEnum::Root => panic!(),
            RefsEnum::MaybeMissing => panic!(),
            RefsEnum::ScopedIdentifier(o, i) => {
                self.exact_match_scoped_references(o, i.as_ref(), scout)
            }
            RefsEnum::TypeIdentifier(o, i) => {
                self.exact_match_scoped_references(o, i.as_ref(), scout)
            }
            RefsEnum::MethodReference(_, _) => todo!(),
            RefsEnum::ConstructorReference(_) => todo!(),
            RefsEnum::Invocation(_, _, _) => todo!(),
            RefsEnum::ConstructorInvocation(_, _) => todo!(),
            RefsEnum::Primitive(_) => panic!(),
            RefsEnum::Array(_) => todo!(),
            RefsEnum::This(_) => {
                let b = self.stores.node_store.resolve(scout.node_always(self.sp_store));
                if b.get_type() != Type::This {
                    // println!("not matched");
                } else {
                    assert!(!b.has_children()); // TODO
                    self.successful_match(&mut scout);
                }
            }
            RefsEnum::Super(_) => todo!(),
            RefsEnum::ArrayAccess(_) => todo!(),
            RefsEnum::Mask(_, _) => panic!(),
            RefsEnum::Or(_) => todo!(),
        }
    }

    pub fn exact_match_scoped_references(
        &mut self,
        o: RefPtr,
        i: &LabelIdentifier,
        mut scout: Scout,
    ) {
        let b = self.stores.node_store.resolve(scout.node_always(&self.sp_store));
        let t = b.get_type();
        if t == Type::MethodInvocation {
            self.exact_match_method_invocation(&b, o, i, &mut scout);
        } else if t == Type::ObjectCreationExpression {
            self.exact_match_object_creation_expression2(&b, o, i, &mut scout);
        } else if t == Type::MethodDeclaration {
            self.exact_match_method_declaration(&b, o, i, &mut scout);
        } else if t == Type::FormalParameter
            || t == Type::LocalVariableDeclaration
            || t == Type::EnhancedForVariable
            || t == Type::ConstantDeclaration
        {
            self.exact_match_variable_declaration(&b, o, i, &mut scout);
        } else if t == Type::SuperInterfaces
            || t == Type::Superclass
            || t == Type::ExtendsInterfaces
            || t == Type::TypeBound
        {
            self.exact_match_extend_impl_things(&b, o, i, &mut scout);
        } else if t == Type::EnhancedForStatement {
            self.exact_match_enhanced_for_statement(&b, o, i, &mut scout);
        } else if t == Type::GenericType {
            self.exact_match_generic_type(&b, o, i, &mut scout);
        } else if t == Type::ArrayType {
            self.exact_match_array_type(&b, o, i, &mut scout);
        } else if t == Type::InstanceofExpression {
            self.exact_match_instanceof_expression(&b, o, i, &mut scout);
        } else if t == Type::CastExpression {
            self.exact_match_catch_expression(&b, o, i, &mut scout);
        } else if t == Type::MethodReference {
            self.exact_match_method_reference(&b, o, i, &mut scout);
        } else if t == Type::FieldAccess {
            if let Some(mut scout) = self.is_field_access_exact_match(&b, o, i, scout) {
                self.successful_match(&mut scout);
            }
        } else if t == Type::ScopedTypeIdentifier {
            if let Some(mut scout) = self.is_scoped_type_identifier_exact_match(&b, o, i, scout) {
                self.successful_match(&mut scout);
            }
        } else if t == Type::ClassLiteral {
            self.exact_match_class_literal(&b, o, i, &mut scout);
        } else if t == Type::AnnotatedType {
            self.exact_match_annotated_type(&b, o, i, &mut scout);
        } else if t == Type::FieldDeclaration {
            self.exact_match_field_declaration(&b, o, i, &mut scout);
        } else if t == Type::ConstructorDeclaration {
            // todo!()
            // get ?.identifier from contructor then compare to ref
        } else if t == Type::CatchType {
            self.exact_match_catch_type(&b, o, i, &mut scout);
        } else if t == Type::Resource {
            self.exact_match_resource(&b, o, i, &mut scout)
        } else {
            todo!("{:?}", t)
        }
    }

    fn exact_match_resource(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &LabelIdentifier,
        mut scout: &mut Scout,
    ) {
        let mut j = 0;
        loop {
            let x = b.get_child(&j);
            let r = self.stores.node_store.resolve(x);
            let t = r.get_type();
            if t == Type::TypeIdentifier {
                if let Some(l) = r.try_get_label() {
                    if l == i {
                        scout.goto(x, j as usize);
                        println!("success 8");
                        self.successful_match(&mut scout);
                    }
                } else {
                    todo!()
                };
                return;
            } else if t == Type::Identifier {
                if let Some(l) = r.try_get_label() {
                    if l == i {
                        scout.goto(x, j as usize);
                        println!("success 8.1");
                        self.successful_match(&mut scout);
                        return;
                    }
                } else {
                    todo!()
                };
            } else if t == Type::Modifiers {
            } else {
                return;
            }
            j += 1;
        }
    }

    fn exact_match_catch_type(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                    self.successful_match(scout); // TODO
                }
            } else {
                todo!()
            }
        } else if is_individually_matched(t) || is_never_reference(t) {
        } else {
            todo!("{:?}", t)
        }
    }

    fn exact_match_field_declaration(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                    self.successful_match(scout);
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
    }

    fn exact_match_annotated_type(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                    self.successful_match(scout); // TODO
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
    }

    fn exact_match_class_literal(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
        let x = b.get_child(&0);
        let bb = self.stores.node_store.resolve(x);
        let t = bb.get_type();
        if t == Type::TypeIdentifier || t == Type::Identifier {
            if let Some(l) = b.try_get_label() {
                if l == i {
                    scout.goto(x, 0);
                    scout.check(self.stores).unwrap();
                    println!("success 8");
                    self.successful_match(scout); // TODO
                }
            } else {
                todo!()
            }
        } else if is_individually_matched(t) || is_never_reference(t) {
        } else {
            todo!("{:?}", t)
        }
    }

    fn exact_match_method_reference(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                    self.successful_match(scout); // TODO
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
                    self.successful_match(scout); // TODO
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
    }

    fn exact_match_catch_expression(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                            self.successful_match(scout);
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
                            self.successful_match(scout);
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
    }

    fn exact_match_instanceof_expression(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                    self.successful_match(scout); // TODO
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
                self.successful_match(scout); // TODO
                scout.up(self.sp_store);
            }
        } else if t == Type::This {
            panic!(); // sure ?
        } else if is_individually_matched(t) || is_never_reference(t) {
        } else {
            todo!("{:?}", t)
        }
    }

    fn exact_match_array_type(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
        let x = b.get_child(&0);
        let b = self.stores.node_store.resolve(x);
        let t = b.get_type();
        if t == Type::TypeIdentifier {
            scout.goto(x, 0);
            if let Some(l) = b.try_get_label() {
                if l == i {
                    println!("success 5.1");
                    self.successful_match(scout); // TODO
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
    }

    fn exact_match_generic_type(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
        let x = b.get_child(&0);
        let b = self.stores.node_store.resolve(x);
        let t = b.get_type();
        if t == Type::TypeIdentifier {
            // scout.goto(x, 0);
            if let Some(l) = b.try_get_label() {
                if l == i {
                    println!("success 5");
                    self.successful_match(scout); // TODO
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
    }

    fn exact_match_extend_impl_things(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                            self.successful_match(scout);
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
                                self.successful_match(scout); // TODO
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
    }

    fn exact_match_variable_declaration(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                    self.successful_match(scout);
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
    }

    fn exact_match_method_declaration(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                    self.successful_match(scout);
                }
            } else {
                todo!()
            }
        } else if is_individually_matched(t) || is_never_reference(t) {
        } else {
            todo!("{:?}", t)
        }
    }

    fn exact_match_method_invocation(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
        let x = b.get_child(&0);
        scout.goto(x, 0);
        let b = self.stores.node_store.resolve(x);
        let t = b.get_type();
        if t == Type::TypeIdentifier {
            if let Some(l) = b.try_get_label() {
                if l == i {
                    println!("success 1");
                    self.successful_match(scout); // TODO
                }
            } else {
                todo!()
            }
        } else if t == Type::Identifier {
            if let Some(l) = b.try_get_label() {
                if l == i {
                    println!("success 2");
                    self.successful_match(scout); // TODO
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
    }

    fn exact_match_object_creation_expression2(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
        let cs = b.get_children();
        let mut j = cs.len() - 1;
        let has_body = self.stores.node_store.resolve(cs[j]).get_type() == Type::ClassBody;
        if has_body {
            j -= 1;
        }

        let mut matched = false;
        let mut o = o;

        loop {
            let x = cs[j];
            let r = self.stores.node_store.resolve(x);
            let t = r.get_type();

            if t == Type::TypeIdentifier {
                if let Some(l) = r.try_get_label() {
                    if i == l {
                        matched = true;
                    }
                //     match self.ana.solver.nodes.with(o).as_ref() {
                //         RefsEnum::MaybeMissing => {
                //             if l == i {
                //                 scout.goto(x, 0);
                //                 scout.check(self.stores).unwrap();
                //                 println!("success is_scoped_type_identifier_exact_match 1");
                //                 return Some(scout);
                //             }
                //         }
                //         RefsEnum::ScopedIdentifier(oo, ii) => {
                //             match self.ana.solver.nodes.with(*oo).as_ref() {
                //                 RefsEnum::MaybeMissing => {
                //                     if l == ii.as_ref() {
                //                         matching_o = true;
                //                     }
                //                 }
                //                 _ => {
                //                     return None;
                //                 }
                //             }
                //         }
                //         _ => {
                //             return None;
                //         }
                //     }
                } else {
                    todo!()
                }
            } else if t == Type::TS74 {
                // new token
                if j == 0 {
                    if matched {
                        match self.ana.solver.nodes.with(o).as_ref() {
                            RefsEnum::MaybeMissing => {
                                self.successful_match(scout);
                                if has_body {
                                    let x = cs[cs.len() - 1];
                                    scout.goto(x, cs.len() - 1);
                                }
                            }
                            _ => {}
                        }
                    }
                    return;
                }
                j -= 1;
                break;
            } else if t == Type::GenericType {
                // TODO need full check if creating anonymous class
                println!("need to handle new C<T>() "); // wierd if needed, should be before name
            } else if t == Type::ScopedTypeIdentifier {
                // TODO need full check if creating anonymous class
                println!("need to handle new a.b.C() ");
                // NOTE need to uptdate o with what remains

                let mut scout2 = scout.clone();
                scout2.goto(x, j);
                let bb = self.stores.node_store.resolve(x);
                if let Some(oo) =
                    self.exact_match_object_creation_expression2_aux(&bb, o, i, scout2)
                {
                    o = oo;
                    matched = true;
                }
            } else if t == Type::TypeArguments {
            } else if t == Type::ArgumentList {
            } else if t == Type::Spaces {
            } else if t == Type::Annotation {
            } else if t == Type::MarkerAnnotation {
            } else {
                todo!("{:?}", t)
            }
            if j == 0 {
                return;
            }
            j -= 1;
        }
        loop {
            let r = self.stores.node_store.resolve(cs[j]);
            let t = r.get_type();
            if t == Type::TypeIdentifier || t == Type::Identifier {
                if let Some(l) = r.try_get_label() {
                    if matched {
                        match self.ana.solver.nodes.with(o).as_ref() {
                            RefsEnum::ScopedIdentifier(oo, ii) => {
                                match self.ana.solver.nodes.with(*oo).as_ref() {
                                    RefsEnum::MaybeMissing => {
                                        if l == ii.as_ref() {
                                            println!("success 3");
                                            self.successful_match(scout);
                                            if has_body {
                                                let x = cs[cs.len() - 1];
                                                scout.goto(x, cs.len() - 1);
                                            }
                                        }
                                    }
                                    _ => {
                                        return;
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match self.ana.solver.nodes.with(o).as_ref() {
                            RefsEnum::ScopedIdentifier(oo, ii) => {
                                match self.ana.solver.nodes.with(*oo).as_ref() {
                                    RefsEnum::MaybeMissing => {
                                        if l == ii.as_ref() {
                                            println!("success 3");
                                            let xx = cs[j];
                                            scout.goto(xx, j);
                                            self.successful_match(scout)
                                        }
                                    }
                                    _ => {
                                        return;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    todo!()
                }
                return;
            } else if t == Type::Annotation {
            } else if t == Type::MarkerAnnotation {
            } else if t == Type::Spaces {
            } else if t == Type::TS19 {
            } else if t == Type::GenericType {
                // TODO need full check if creating anonymous class
                println!("need to handle a<T>.new b.C() ");
            } else if t == Type::ScopedTypeIdentifier {
                // TODO need a unit test
                // TODO need full check if creating anonymous class
                println!("need to handle a.new b.C() ");
                if matched {
                    let (oo, ii) = match self.ana.solver.nodes.with(o).as_ref() {
                        RefsEnum::ScopedIdentifier(o, i) => (*o, *i.as_ref()),
                        RefsEnum::TypeIdentifier(o, i) => (*o, *i.as_ref()),
                        _ => return,
                    };

                    let mut scout2 = scout.clone();
                    let xx = cs[j];
                    scout2.goto(xx, j);
                    let bb = self.stores.node_store.resolve(xx);
                    if let Some(_) =
                        self.is_scoped_type_identifier_exact_match(&bb, oo, &ii, scout2)
                    {
                        self.successful_match(scout);
                        if has_body {
                            let x = cs[cs.len() - 1];
                            scout.goto(x, cs.len() - 1);
                        }
                    }
                }
                return;
            } else {
                todo!("{:?}", t)
            }
            if j == 0 {
                if matched {
                    match self.ana.solver.nodes.with(o).as_ref() {
                        RefsEnum::MaybeMissing => {
                            self.successful_match(scout);
                            if has_body {
                                let x = cs[cs.len() - 1];
                                scout.goto(x, cs.len() - 1);
                            }
                        }
                        _ => {}
                    }
                }
                return;
            }
            j -= 1;
        }
    }

    fn exact_match_object_creation_expression2_aux(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &LabelIdentifier,
        mut scout: Scout,
    ) -> Option<usize> {
        let x = b.get_child_rev(&0);
        let bb = self.stores.node_store.resolve(x);
        let t = bb.get_type();

        if t == Type::TypeIdentifier {
            if let Some(l) = bb.try_get_label() {
                if l != i {
                    return None;
                }
            } else {
                todo!()
            }
        } else {
            panic!("{:?}", t)
        }

        let x = b.get_child(&0);
        let bb = self.stores.node_store.resolve(x);
        let t = bb.get_type();
        if t == Type::TypeIdentifier {
            if let Some(l) = bb.try_get_label() {
                match self.ana.solver.nodes.with(o).as_ref() {
                    RefsEnum::MaybeMissing => {
                        if l == i {
                            return Some(o);
                        }
                    }
                    RefsEnum::ScopedIdentifier(oo, ii) => {
                        if l == ii.as_ref() {
                            return Some(*oo);
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
                "recursive call to exact_match_object_creation_expression2_aux {:?}",
                self.ana.solver.nodes.with(o)
            );
            return self.exact_match_object_creation_expression2_aux(&bb, o, &i, scout);
        } else if t == Type::GenericType {
            // println!("not matched"); // TODO should check the fully qual name
        } else if is_individually_matched(t) || is_never_reference(t) {
        } else {
            todo!("{:?}", t)
        }
        None
    }
    fn exact_match_object_creation_expression2_aux_old(
        &mut self,
        b: &HashedNodeRef,
        o: usize,
        i: &string_interner::symbol::SymbolU32,
        mut scout: Scout,
    ) -> Option<usize> {
        let x = b.get_child(&0);
        let bb = self.stores.node_store.resolve(x);
        let t = bb.get_type();
        let mut matching_o = None;
        if t == Type::TypeIdentifier {
            if let Some(l) = bb.try_get_label() {
                match self.ana.solver.nodes.with(o).as_ref() {
                    RefsEnum::MaybeMissing => {
                        if l == i {
                            return Some(o);
                        }
                    }
                    RefsEnum::ScopedIdentifier(oo, ii) => {
                        match self.ana.solver.nodes.with(*oo).as_ref() {
                            RefsEnum::MaybeMissing => {
                                if l == ii.as_ref() {
                                    matching_o = Some(o);
                                }
                            }
                            RefsEnum::ScopedIdentifier(_, ii) => {
                                if l == ii.as_ref() {
                                    matching_o = Some(o);
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
                "recursive call to exact_match_object_creation_expression2_aux {:?}",
                self.ana.solver.nodes.with(o)
            );
            matching_o = self.exact_match_object_creation_expression2_aux(&bb, o, &i, scout);
        } else if t == Type::GenericType {
            // println!("not matched"); // TODO should check the fully qual name
        } else if is_individually_matched(t) || is_never_reference(t) {
        } else {
            todo!("{:?}", t)
        }
        if let Some(o) = matching_o {
            let x = b.get_child_rev(&0);
            let bb = self.stores.node_store.resolve(x);
            let t = bb.get_type();

            if t == Type::TypeIdentifier {
                if let Some(l) = bb.try_get_label() {
                    if l == i {
                        println!("success exact_match_object_creation_expression2_aux 3");
                        return Some(o);
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

    fn exact_match_object_creation_expression(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
        let mut j = 0;
        let l = b.child_count();
        let mut matched = false;
        let mut met_annot = 0;
        let (b, t) = {
            let mut r;
            let mut t;
            loop {
                let x = b.get_child(&j);
                r = self.stores.node_store.resolve(x);
                t = r.get_type();
                if t == Type::TS74 {
                    // find new
                    // TODO but should alse construct the fully qualified name in the mean time
                    j += 1;
                    break;
                }
                j += 1;
            }
            loop {
                let x = b.get_child(&j);
                r = self.stores.node_store.resolve(x);
                t = r.get_type();
                if t != Type::Spaces && t != Type::Comment && t != Type::MarkerAnnotation {
                } else if t != Type::Annotation {
                    met_annot = 1;
                    break;
                }
                j += 1;
            }
            (r, t)
        };
        if t == Type::TypeIdentifier {
            if let Some(l) = b.try_get_label() {
                if l == i {
                    matched = true;
                    println!("success 3");
                    self.successful_match(scout); // TODO
                }
            } else {
                todo!()
            }
        } else if t == Type::GenericType {
            if l > (4 + met_annot) {
                // TODO need full check if creating anonymous class
            }
        } else if t == Type::ScopedTypeIdentifier {
            if l > (4 + met_annot) {
                // TODO need full check if creating anonymous class
            }
        } else if t == Type::TypeArguments {
            if l > (4 + met_annot) {
                // TODO need full check if creating anonymous class
            }
        } else {
            todo!("{:?}", t)
        }
        if matched {
            let mut r;
            let mut t;
            while j < l {
                let x = b.get_child(&j);
                r = self.stores.node_store.resolve(x);
                t = r.get_type();
                if t == Type::ClassBody {
                    assert!(l > (4 + met_annot));
                    // found annonymous class
                    scout.goto(x, j as usize);
                    self.sp_store.check(&self.stores).unwrap();
                    scout.check(&self.stores).unwrap();
                    let r = self.sp_store.push(scout);
                    self.sp_store.check(&self.stores).unwrap();
                    self.refs.push(r);
                    let it = ExploreStructuralPositions::from((&*self.sp_store, r));
                    println!("really found {:?}", it.to_position(&self.stores));
                }
                j += 1;
            }
        }
    }

    fn exact_match_enhanced_for_statement(
        &mut self,
        b: &HashedNodeRef,
        o: RefPtr,
        i: &LabelIdentifier,
        scout: &mut Scout,
    ) {
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
                            self.successful_match(scout);
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
                            self.successful_match(scout);
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
                    || t == Type::Resource
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
                            || t == Type::Resource
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
                } else if t == Type::ObjectCreationExpression {
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

/// WARN not exaustive set
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
/// WARN not exaustive set
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
