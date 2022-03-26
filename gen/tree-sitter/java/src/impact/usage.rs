use core::fmt;
use std::{fmt::Debug, io::stdout, ops::AddAssign};

use hyper_ast::{
    filter::{BloomResult, BloomSize},
    nodes::RefContainer,
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
    impact::element::{IdentifierFormat, LabelPtr},
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

/// impl smthing more efficient by dispatching earlier
/// make a comparator that only take scopedIdentifier, one that only take ...
/// fo the node identifier it is less obvious
pub fn eq_node_ref(d: ExplorableRef, stores: &SimpleStores, x: NodeIdentifier) -> bool {
    match d.as_ref() {
        RefsEnum::Root => todo!(),
        RefsEnum::MaybeMissing => todo!(),
        RefsEnum::ScopedIdentifier(o, i) => eq_node_scoped_id(d.with(*o), i.as_ref(), stores, x),
        RefsEnum::TypeIdentifier(o, i) => eq_node_scoped_id(d.with(*o), i.as_ref(), stores, x),
        RefsEnum::MethodReference(_, _) => todo!(),
        RefsEnum::ConstructorReference(_) => todo!(),
        RefsEnum::Invocation(_, _, _) => todo!(),
        RefsEnum::ConstructorInvocation(_, _) => todo!(),
        RefsEnum::Primitive(_) => todo!(),
        RefsEnum::Array(_) => todo!(),
        RefsEnum::This(_) => {
            let b = stores.node_store.resolve(x);
            if b.get_type() != Type::This {
                false
            } else {
                assert!(!b.has_children()); // TODO
                true
            }
        }
        RefsEnum::Super(_) => todo!(),
        RefsEnum::ArrayAccess(_) => todo!(),
        RefsEnum::Mask(_, _) => todo!(),
        RefsEnum::Or(_) => todo!(),
    }
}

pub fn eq_node_scoped_id(
    o: ExplorableRef,
    i: &LabelIdentifier,
    stores: &SimpleStores,
    x: NodeIdentifier,
) -> bool {
    let b = stores.node_store.resolve(x);
    let t = b.get_type();
    if t == Type::MethodInvocation {
        let x = b.get_child(&0);
        let b = stores.node_store.resolve(x);
        let t = b.get_type();
        if t == Type::TypeIdentifier {
            if b.has_label() {
                let l = b.get_label();
                if l != i {
                    false // TODO
                } else {
                    true // TODO
                }
            } else {
                todo!()
            }
        } else if t == Type::Identifier {
            if b.has_label() {
                let l = b.get_label();
                if l != i {
                    false // TODO
                } else {
                    true // TODO
                }
            } else {
                todo!()
            }
        } else if t == Type::FieldAccess {
            false // should be handled after
        } else if t == Type::ScopedTypeIdentifier {
            false // should be handled after
        } else if t == Type::ScopedIdentifier {
            false // should be handled after
        } else if t == Type::MethodInvocation {
            false // should be handled after
        } else if t == Type::ArrayAccess {
            false // should be handled after
        } else if t == Type::ObjectCreationExpression {
            false // should be handled after
        } else if t == Type::ParenthesizedExpression {
            false // should be handled after
        } else if t == Type::TernaryExpression {
            false // should be handled after
        } else if t == Type::StringLiteral {
            false // should be handled after
        } else if t == Type::This {
            false // TODO not exactly sure but if scoped should be handled after
        } else if t == Type::Super {
            false // TODO not exactly sure but if scoped should be handled after
        } else if t == Type::ClassLiteral {
            false // out of scope for tool ie. reflexivity
        } else if t == Type::Comment {
            false
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
                r = stores.node_store.resolve(x);
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
                r = stores.node_store.resolve(x);
                t = r.get_type();
                if t != Type::Spaces
                    && t != Type::Comment
                    && t != Type::MarkerAnnotation
                    && t != Type::Annotation
                {
                    break;
                }
                i += 1;
            }
            (r, t)
        };
        if t == Type::TypeIdentifier {
            if b.has_label() {
                let l = b.get_label();
                if l != i {
                    false // TODO
                } else {
                    true // TODO
                }
            } else {
                todo!()
            }
        } else if t == Type::GenericType {
            false // should be handled after
        } else if t == Type::ScopedTypeIdentifier {
            false // should be handled after
        } else if t == Type::TypeArguments {
            false // should be handled after
        } else {
            todo!("{:?}", t)
        }
    } else if t == Type::FormalParameter || t == Type::LocalVariableDeclaration {
        // let (r,t) = {
        //     let mut i = 0;
        //     let mut r;
        //     let mut t;
        //     loop {
        //         let x = b.get_child(&i);
        //         r = java_tree_gen.stores.node_store.resolve(x);
        //         t = r.get_type();
        //         if t != Type::Modifiers {
        //             break;
        //         }
        //         i+=1;
        //     }
        //     (r,t)
        // };
        let (r, t) = {
            let x = b.get_child(&0);
            let r = stores.node_store.resolve(x);
            let t = r.get_type();
            if t == Type::Modifiers {
                let x = b.get_child(&2);
                let r = stores.node_store.resolve(x);
                let t = r.get_type();
                (r, t)
            } else {
                (r, t)
            }
        };
        if t == Type::TypeIdentifier {
            if r.has_label() {
                let l = r.get_label();
                if l != i {
                    false // TODO
                } else {
                    true // TODO
                }
            } else {
                todo!()
            }
        } else if t == Type::ScopedTypeIdentifier {
            false // should be handled after
        } else if t == Type::GenericType {
            false // should be handled after
        } else if t == Type::ArrayType {
            false // TODO not sure
        } else if t == Type::IntegralType {
            false // TODO not sure
        } else if t == Type::FloatingPointType {
            false // TODO not sure
        } else if t == Type::BooleanType {
            false // TODO not sure
        } else if t == Type::Comment {
            false
        } else {
            todo!("{:?}", t)
        }
    } else if t == Type::GenericType {
        let x = b.get_child(&0);
        let b = stores.node_store.resolve(x);
        let t = b.get_type();
        if t == Type::TypeIdentifier {
            if b.has_label() {
                let l = b.get_label();
                if l != i {
                    false // TODO
                } else {
                    true // TODO
                }
            } else {
                todo!()
            }
        } else if t == Type::ScopedTypeIdentifier {
            false // should be handled after
        } else {
            todo!("{:?}", t)
        }
    } else if t == Type::ScopedTypeIdentifier {
        let x = b.get_child_rev(&0);
        let b = stores.node_store.resolve(x);
        let t = b.get_type();
        if t == Type::TypeIdentifier {
            if b.has_label() {
                let l = b.get_label();
                if l != i {
                    false // TODO
                } else {
                    true // TODO
                }
            } else {
                todo!()
            }
        } else if t == Type::ScopedTypeIdentifier {
            false // TODO should check the fully qual name
        } else if t == Type::GenericType {
            false // TODO should check the fully qual name
        } else if t == Type::Comment {
            false
        } else {
            todo!("{:?}", t)
        }
    } else if t == Type::CatchType {
        // TODO check for type union eg. A|B|C
        let x = b.get_child(&0);
        let b = stores.node_store.resolve(x);
        let t = b.get_type();
        if t == Type::TypeIdentifier {
            if b.has_label() {
                let l = b.get_label();
                if l != i {
                    false // TODO
                } else {
                    true // TODO
                }
            } else {
                todo!()
            }
        } else if t == Type::ScopedTypeIdentifier {
            false // should be handled after
        } else if t == Type::Comment {
            false
        } else {
            todo!("{:?}", t)
        }
    } else if t == Type::Resource {
        let l = b.child_count();
        let mut j = 0;
        loop {
            let x = b.get_child(&j);
            let r = stores.node_store.resolve(x);
            let t = r.get_type();

            j += 1;
            if t == Type::TypeIdentifier {
                return if r.has_label() {
                    let l = r.get_label();
                    if l != i {
                        false // TODO
                    } else {
                        true // TODO
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
        }
        false
    } else {
        todo!("{:?}", t)
    }
}

pub fn remake_pkg_ref(
    stores: &SimpleStores,
    ana: &mut PartialAnalysis,
    x: NodeIdentifier,
) -> RefPtr {
    let b = stores.node_store.resolve(x);
    let t = b.get_type();
    if t == Type::ScopedAbsoluteIdentifier {
        let x = b.get_child(&0);
        let o = remake_pkg_ref(stores, ana, x);

        let x = b.get_child(&1);
        let b = stores.node_store.resolve(x);
        let i = b.get_label();
        let f = IdentifierFormat::from(stores.label_store.resolve(i));
        let l = LabelPtr::new(*i, f);
        let i = ana.solver.intern(RefsEnum::ScopedIdentifier(o, l));
        i
    } else if t == Type::Identifier {
        let i = b.get_label();
        let o = ana.solver.intern(RefsEnum::Root);
        let f = IdentifierFormat::from(stores.label_store.resolve(i));
        let l = LabelPtr::new(*i, f);
        let i = ana.solver.intern(RefsEnum::ScopedIdentifier(o, l));
        i
    } else if t == Type::PackageDeclaration {
        let x = b.get_child(&1);
        remake_pkg_ref(stores, ana, x)
    } else {
        todo!()
    }
}
pub fn eq_root_scoped(d: ExplorableRef, stores: &SimpleStores, b: HashedNodeRef) -> bool {
    match d.as_ref() {
        RefsEnum::Root => todo!(),
        RefsEnum::MaybeMissing => false,
        RefsEnum::ScopedIdentifier(o, i) => {
            let t = b.get_type();
            if t == Type::ScopedAbsoluteIdentifier {
                let mut bo = false;
                for x in b.get_children().iter().rev() {
                    // print_tree_syntax(
                    //     &java_tree_gen.stores.node_store,
                    //     &java_tree_gen.stores.label_store,
                    //     &x,
                    // );
                    // println!();
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
                        if b.has_label() {
                            let l = b.get_label();
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
                if b.has_label() {
                    let l = b.get_label();
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
        x => panic!("{:?}", x),
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
    // pub fn new(
    //     stores: &'a SimpleStores,
    //     ana: &'a mut PartialAnalysis,
    // ) -> Self {
    //     Self {
    //         stores,
    //         ana,
    //         sp_store:Default::default(),
    //     }
    // }
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

    pub fn find_all(mut self, i: RefPtr, mut x: Scout) -> Vec<usize> {
        // let position = (0,Some(root));
        // self.refs = StructuralPositions::from(root);
        // let mut x = Scout::from(root);
        self.find_refs(i, &mut x);
        self.refs
    }

    fn find_refs(&mut self, target: RefPtr, scout: &mut Scout) -> Vec<RefPtr> {
        let current = scout.node(&self.sp_store);
        let b = self.stores.node_store.resolve(current);
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
            println!("d=1 {:?}", &t);
            let c = {
                let d = self.ana.solver.nodes.with(target);
                b.check(Into::<Box<[u8]>>::into(d.clone()).as_ref())
            };
            if let BloomResult::MaybeContain = c {
                println!("+++import+++++Maybe contains");
                java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                    &self.stores.node_store,
                    &self.stores.label_store,
                    &current,
                );
                println!();
                let (stic, scop, asterisk) = {
                    let b = self.stores.node_store.resolve(current);
                    let mut scop = None;
                    let mut sstatic = false;
                    let mut asterisk = false;
                    for c in b.get_children() {
                        let b = self.stores.node_store.resolve(*c);
                        match b.get_type() {
                            Type::TS86 => sstatic = true,
                            Type::Asterisk => asterisk = true,
                            Type::Identifier => scop = Some((*c, b)),
                            Type::ScopedAbsoluteIdentifier => scop = Some((*c, b)),
                            _ => (),
                        }
                    }
                    (sstatic, scop.unwrap(), asterisk)
                };
                let d = ExplorableRef {
                    rf: target,
                    nodes: &self.ana.solver.nodes,
                };
                if stic {
                    return vec![]; // TODO
                } else if asterisk {
                    return vec![]; // TODO
                } else if eq_root_scoped(d, self.stores, scop.1) {
                    let d = self.ana.solver.nodes.with(target);
                    let i = if let RefsEnum::ScopedIdentifier(_, i) = d.as_ref() {
                        *i
                    } else {
                        panic!()
                    };
                    let o = self.ana.solver.intern(RefsEnum::MaybeMissing);
                    let i = self.ana.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                    // let i = handle_import(
                    //     java_tree_gen,
                    //     self.ana,
                    //     self.stores.node_store.resolve(scop.0),
                    // );
                    println!("import matched ref");
                    return vec![i];
                } else {
                    return vec![];
                }
            } else {
                println!("Do not contains");
                return vec![];
            }
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
                || &t == &Type::LocalVariableDeclaration // find simple type
                || &t == &Type::ObjectCreationExpression // find simple object
                || &t == &Type::ScopedIdentifier // find identifier
                || &t == &Type::ScopedAbsoluteIdentifier // find identifier
                || &t == &Type::ScopedTypeIdentifier
                || &t == &Type::CatchType // TODO to check
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
            let z = self.find_refs(target, scout);
            v.extend(z);
            for w in v.clone() {
                let z = self.find_refs(w, scout);
                v.extend(z)
            }
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
        fn is_individually_matched(t: Type) -> bool {
            t == Type::FieldAccess
                || t == Type::ScopedTypeIdentifier
                || t == Type::ScopedIdentifier
                || t == Type::MethodInvocation
                || t == Type::ArrayAccess
                || t == Type::ObjectCreationExpression
                || t == Type::ParenthesizedExpression
                || t == Type::TernaryExpression
                || t == Type::GenericType
        }
        fn is_never_reference(t: Type) -> bool {
            t == Type::Comment
            || t == Type::ClassLiteral // out of scope for tool ie. reflexivity
            || t == Type::StringLiteral
        }
        let b = self.stores.node_store.resolve(scout.node(&self.sp_store));
        let t = b.get_type();
        if t == Type::MethodInvocation {
            let x = b.get_child(&0);
            scout.goto(x, 0);
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t == Type::TypeIdentifier {
                if b.has_label() {
                    let l = b.get_label();
                    if l != i {
                        // println!("not matched"); // TODO
                    } else {println!("success 1");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::Identifier {
                if b.has_label() {
                    let l = b.get_label();
                    if l != i {
                        // println!("not matched"); // TODO
                    } else {println!("success 2");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if is_individually_matched(t) {
                // println!("not matched"); // should be handled after
            } else if is_never_reference(t) {
                // println!("not matched");
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
                        scout.goto(x, i as usize);
                        break;
                    }
                    i += 1;
                }
                (r, t)
            };
            if t == Type::TypeIdentifier {
                if b.has_label() {
                    let l = b.get_label();
                    if l != i {
                        // println!("not matched"); // TODO
                    } else {println!("success 3");
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
        } else if t == Type::FormalParameter || t == Type::LocalVariableDeclaration {
            // let (r,t) = {
            //     let mut i = 0;
            //     let mut r;
            //     let mut t;
            //     loop {
            //         let x = b.get_child(&i);
            //         r = java_tree_gen.stores.node_store.resolve(x);
            //         t = r.get_type();
            //         if t != Type::Modifiers {
            //             break;
            //         }
            //         i+=1;
            //     }
            //     (r,t)
            // };
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
                if r.has_label() {
                    let l = r.get_label();
                    if l != i {
                        // println!("not matched"); // TODO
                    } else {println!("success 4");
                        self.successful_match(&mut scout); // TODO
                        // src/main/java/spoon/pattern/internal/DefaultGenerator.java", offset: 16
                        {
                            scout.up(self.sp_store);
                            let r = self.sp_store.push(&mut scout);
                            assert!(self.sp_store.check(&self.stores));
                            let it = ExploreStructuralPositions::from((&*self.sp_store, r));
                            println!("success 4 up 1 {:?}", it.to_position(&self.stores));
                        }
                        {
                            scout.up(self.sp_store);
                            let r = self.sp_store.push(&mut scout);
                            assert!(self.sp_store.check(&self.stores));
                            let it = ExploreStructuralPositions::from((&*self.sp_store, r));
                            println!("success 4 up 2 {:?}", it.to_position(&self.stores));
                        }
                        {
                            scout.up(self.sp_store);
                            let r = self.sp_store.push(&mut scout);
                            assert!(self.sp_store.check(&self.stores));
                            let it = ExploreStructuralPositions::from((&*self.sp_store, r));
                            println!("success 4 up 3 {:?}", it.to_position(&self.stores));
                        }
                        {
                            let b = self.stores.node_store.resolve(scout.node(self.sp_store));
                            let o = 0;//(b.child_count() -2) as usize;
                            let x = b.get_children()[o];
                            scout.goto(x,o);
                            let r = self.sp_store.push(&mut scout);
                            assert!(self.sp_store.check(&self.stores));
                            let it = ExploreStructuralPositions::from((&*self.sp_store, r));
                            println!("success 4 goto 0 {:?}", it.to_position(&self.stores));
                            scout.up(self.sp_store);
                            let o = 1;
                            let x = b.get_children()[o];
                            scout.goto(x,o);
                            let r = self.sp_store.push(&mut scout);
                            assert!(self.sp_store.check(&self.stores));
                            let it = ExploreStructuralPositions::from((&*self.sp_store, r));
                            println!("success 4 goto 1 {:?}", it.to_position(&self.stores));
                            scout.up(self.sp_store);
                            let o = 2;
                            let x = b.get_children()[o];
                            scout.goto(x,o);
                            let r = self.sp_store.push(&mut scout);
                            assert!(self.sp_store.check(&self.stores));
                            let it = ExploreStructuralPositions::from((&*self.sp_store, r));
                            println!("success 4 goto 2 {:?}", it.to_position(&self.stores));
                            scout.up(self.sp_store);
                        }
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
            } else if t == Type::IntegralType {
                // println!("not matched"); // TODO not sure
            } else if t == Type::FloatingPointType {
                // println!("not matched"); // TODO not sure
            } else if t == Type::BooleanType {
                // println!("not matched"); // TODO not sure
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::GenericType {
            let x = b.get_child(&0);
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t == Type::TypeIdentifier {
                scout.goto(x, 0);
                if b.has_label() {
                    let l = b.get_label();
                    if l != i {
                        // println!("not matched"); // TODO
                    } else {println!("success 5");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::ScopedTypeIdentifier {
                // println!("not matched"); // should be handled after
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::ScopedTypeIdentifier {
            let x = b.get_child_rev(&0);
            let l = b.child_count();
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            if t == Type::TypeIdentifier {
                scout.goto(x, l as usize-1);
                if b.has_label() {
                    let l = b.get_label();
                    if l != i {
                        // println!("not matched"); // TODO
                    } else {println!("success 6");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if t == Type::ScopedTypeIdentifier {
                // println!("not matched"); // TODO should check the fully qual name
            } else if t == Type::GenericType {
                // println!("not matched"); // TODO should check the fully qual name
            } else if is_individually_matched(t) {
                // println!("not matched"); // should be handled after
            } else if is_never_reference(t) {
                // println!("not matched");
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::CatchType {
            // TODO check for type union eg. A|B|C
            let x = b.get_child(&0);
            let b = self.stores.node_store.resolve(x);
            let t = b.get_type();
            scout.goto(x, 0);
            if t == Type::TypeIdentifier {
                if b.has_label() {
                    let l = b.get_label();
                    if l != i {
                        // println!("not matched"); // TODO
                    } else {println!("success 7");
                        self.successful_match(&mut scout); // TODO
                    }
                } else {
                    todo!()
                }
            } else if is_individually_matched(t) {
                // println!("not matched"); // should be handled after
            } else if is_never_reference(t) {
                // println!("not matched");
            } else {
                todo!("{:?}", t)
            }
        } else if t == Type::Resource {
            let l = b.child_count();
            let mut j = 0;
            loop {
                let x = b.get_child(&j);
                let r = self.stores.node_store.resolve(x);
                let t = r.get_type();
                if t == Type::TypeIdentifier {
                    scout.goto(x, j as usize);
                    return if r.has_label() {
                        let l = r.get_label();
                        if l != i {
                            // println!("not matched"); // TODO
                        } else {println!("success 8");
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
    pub fn successful_match(&mut self, scout: &mut Scout) {
        scout.check(&self.stores);
        let r = self.sp_store.push(scout);
        assert!(self.sp_store.check(&self.stores));
        self.refs.push(r);
        let it = ExploreStructuralPositions::from((&*self.sp_store, r));
        println!("really found {:?}", it.to_position(&self.stores));
    }
}

pub fn find_refs(
    stores: &SimpleStores,
    ana: &mut PartialAnalysis,
    path: &mut StructuralPosition,
    target: RefPtr,
    current: NodeIdentifier,
) -> Vec<RefPtr> {
    // let d: LabelValue = ;
    let b = stores.node_store.resolve(current);
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
        println!("d=1 {:?}", &t);
        let c = {
            let d = ana.solver.nodes.with(target);
            b.check(Into::<Box<[u8]>>::into(d.clone()).as_ref())
        };
        if let BloomResult::MaybeContain = c {
            println!("+++import+++++Maybe contains");
            java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                &stores.node_store,
                &stores.label_store,
                &current,
            );
            println!();
            let (stic, scop, asterisk) = {
                let b = stores.node_store.resolve(current);
                let mut scop = None;
                let mut sstatic = false;
                let mut asterisk = false;
                for c in b.get_children() {
                    let b = stores.node_store.resolve(*c);
                    match b.get_type() {
                        Type::TS86 => sstatic = true,
                        Type::Asterisk => asterisk = true,
                        Type::Identifier => scop = Some((*c, b)),
                        Type::ScopedAbsoluteIdentifier => scop = Some((*c, b)),
                        _ => (),
                    }
                }
                (sstatic, scop.unwrap(), asterisk)
            };
            let d = ExplorableRef {
                rf: target,
                nodes: &ana.solver.nodes,
            };
            if stic {
                return vec![]; // TODO
            } else if asterisk {
                return vec![]; // TODO
            } else if eq_root_scoped(d, stores, scop.1) {
                let d = ana.solver.nodes.with(target);
                let i = if let RefsEnum::ScopedIdentifier(_, i) = d.as_ref() {
                    *i
                } else {
                    panic!()
                };
                let o = ana.solver.intern(RefsEnum::MaybeMissing);
                let i = ana.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                // let i = handle_import(
                //     java_tree_gen,
                //     ana,
                //     stores.node_store.resolve(scop.0),
                // );
                println!("import matched ref");
                return vec![i];
            } else {
                return vec![];
            }
        } else {
            println!("Do not contains");
            return vec![];
        }
    }
    if !b.has_children() {
        return vec![];
    }
    println!("d=1 {:?}", &t);
    let c = if b.get_component::<BloomSize>().is_ok() {
        let d = ana.solver.nodes.with(target);
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
            || &t == &Type::LocalVariableDeclaration // find simple type
            || &t == &Type::ObjectCreationExpression // find simple object
            || &t == &Type::ScopedIdentifier // find identifier
            || &t == &Type::ScopedAbsoluteIdentifier // find identifier
            || &t == &Type::ScopedTypeIdentifier
            || &t == &Type::CatchType // TODO to check
            || &t == &Type::Resource
        // TODO to check
        // find identifier
        {
            // Here, for now, we try to find Identifiers (not invocations)
            // thus we either search directly for scoped identifiers
            // or we search for simple identifiers because they do not present refs in themself
            println!("!found {:?}", &t);
            java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                &stores.node_store,
                &stores.label_store,
                &current,
            );
            println!();

            let d = ExplorableRef {
                rf: target,
                nodes: &ana.solver.nodes,
            };

            if eq_node_ref(d, stores, current) {
                // let mut position = path.to_position(&stores);
                // position.set_len(b.get_bytes_len() as usize);
                // println!("really found {:?}", position);
                println!("really found");
            }
        } else if &t == &Type::TypeIdentifier {
            println!("!found TypeIdentifier");
            let mut out = IoOut { stream: stdout() };
            java_tree_gen_full_compress_legion_ref::serialize(
                &stores.node_store,
                &stores.label_store,
                &current,
                &mut out,
                "\n",
            );
        } else if &t == &Type::MethodDeclaration {
            // java_tree_gen::print_tree_syntax(
            //     &stores.node_store,
            //     &stores.label_store,
            //     &x,
            // );
            let mut out = IoOut { stream: stdout() };
            java_tree_gen_full_compress_legion_ref::serialize(
                &stores.node_store,
                &stores.label_store,
                &current,
                &mut out,
                "\n",
            );
        }
    } else {
        println!("Do not contains");
        return vec![];
    }

    let mut v: Vec<usize> = vec![];
    println!("c_count {}", b.child_count());
    path.push();
    for x in b.get_children().clone() {
        path.inc(*x);
        let z = find_refs(stores, ana, path, target, *x);
        v.extend(z);
        for w in v.clone() {
            let z = find_refs(stores, ana, path, w, *x);
            v.extend(z)
        }
    }
    path.pop();
    vec![]
}

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
        let (node, offset, children) = self.stack.pop()?;
        if let Some(children) = children {
            if offset < children.len() {
                let child = children[offset];
                assert!(self.scout.check(&self.stores));
                {
                    let b = self.stores.node_store.resolve(node);
                    if b.has_children() {
                        let cs = b.get_children();
                        // println!("children: {:?} {} {:?}", node,cs.len(),cs);
                        assert!(offset<cs.len());
                        assert_eq!(child,cs[offset]);
                    } else { panic!()}

                }
                if offset == 0 {
                    match self.scout.try_node() {
                        Ok(x) => assert_eq!(x,node),
                        Err(_) => {},
                    }
                    self.scout.goto(child, offset);
                    assert!(self.scout.check(&self.stores));
                } else {
                    match self.scout.try_node() {
                        Ok(x) => assert_eq!(x,children[offset-1]),
                        Err(_) => {},
                    }
                    let i = self.scout.inc(child);
                    assert_eq!(i,offset);
                    assert!(self.scout.check_size(&self.stores),"{:?} {} {:?} {:?} {:?}", node, offset, child, children.len(),self.scout);
                    assert!(self.scout.check(&self.stores),"{:?} {} {:?} {:?} {:?}", node, offset, child, children,self.scout);
                }
                self.stack.push((node, offset + 1, Some(children)));
                self.stack.push((child, 0, None));
                self.next()
            } else {
                assert!(self.scout.check(&self.stores));
                self.scout.try_up().expect("should not go higher than root");
                assert!(self.scout.check(&self.stores));
                self.next()
            }
        } else {
            let b = self.stores.node_store.resolve(node);
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
                assert!(b.has_children(),"{:?}",t);
                assert!(self.scout.check(&self.stores));
                Some(self.scout.clone())
            } else if &t == &Type::Resource {
                assert!(b.has_children(),"{:?}",t);
                assert!(self.scout.check(&self.stores));
                // TODO also need to find an "=" and find the name just before
                Some(self.scout.clone())
            } else {
                self.next()
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
