use core::fmt;
use std::io::stdout;

use rusted_gumtree_core::tree::tree::{Typed, Type, WithChildren, Tree, Labeled};

use crate::{java_tree_gen_full_compress_legion_ref::{JavaTreeGen,NodeIdentifier,LabelIdentifier, HashedNodeRef, self}, impact::label_value::LabelValue, nodes::RefContainer, filter::BloomResult};

use super::{element::RefsEnum, element::ExplorableRef, partial_analysis::PartialAnalysis};

// TODO use generic node and store

/// impl smthing more efficient by dispatching earlier
/// make a comparator that only take scopedIdentifier, one that only take ...
/// fo the node identifier it is less obvious
pub fn eq_node_ref(d: ExplorableRef, java_tree_gen: &JavaTreeGen, x: NodeIdentifier) -> bool {
    match d.as_ref() {
        RefsEnum::Root => todo!(),
        RefsEnum::MaybeMissing => todo!(),
        RefsEnum::ScopedIdentifier(o, i) => {
            eq_node_scoped_id(d.with(*o),i.as_ref(),java_tree_gen, x)
        }
        RefsEnum::MethodReference(_, _) => todo!(),
        RefsEnum::ConstructorReference(_) => todo!(),
        RefsEnum::Invocation(_, _, _) => todo!(),
        RefsEnum::ConstructorInvocation(_, _) => todo!(),
        RefsEnum::Primitive(_) => todo!(),
        RefsEnum::Array(_) => todo!(),
        RefsEnum::This(_) => todo!(),
        RefsEnum::Super(_) => todo!(),
        RefsEnum::ArrayAccess(_) => todo!(),
        RefsEnum::Mask(_, _) => todo!(),
        RefsEnum::Or(_) => todo!(),
    }
}

pub fn eq_node_scoped_id(o: ExplorableRef,i:&LabelIdentifier, java_tree_gen: &JavaTreeGen, x: NodeIdentifier) -> bool {
    let b = java_tree_gen.stores.node_store.resolve(x);
    let t = b.get_type();
    if t == Type::MethodInvocation {
        let x = b.get_child(&0);
        let b = java_tree_gen.stores.node_store.resolve(x);
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
        } else {
            todo!("{:?}",t)
        }
    } else if t == Type::ObjectCreationExpression {
        let (b,t) = {
            let mut i = 0;
            let mut r;
            let mut t;
            loop {
                let x = b.get_child(&i);
                r = java_tree_gen.stores.node_store.resolve(x);
                t = r.get_type();
                if t == Type::TS74 {
                    // find new
                    // TODO but should alse construct the fully qualified name in the mean time
                    i+=1;
                    break;
                }
                i+=1;
            }
            loop {
                let x = b.get_child(&i);
                r = java_tree_gen.stores.node_store.resolve(x);
                t = r.get_type();
                if t != Type::Spaces && t != Type::Comment && t != Type::MarkerAnnotation && t != Type::Annotation {
                    break;
                }
                i+=1;
            }
            (r,t)
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
        } else {
            todo!("{:?}",t)
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
        let (r,t) = {
            let x = b.get_child(&0);
            let r = java_tree_gen.stores.node_store.resolve(x);
            let t = r.get_type();
            if t == Type::Modifiers {
                let x = b.get_child(&2);
                let r = java_tree_gen.stores.node_store.resolve(x);
                let t = r.get_type();
                (r,t)
            } else {
                (r,t)
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
        } else {
            todo!("{:?}",t)
        }
    } else if t == Type::GenericType {
        let x = b.get_child(&0);
        let b = java_tree_gen.stores.node_store.resolve(x);
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
            todo!("{:?}",t)
        }
    } else if t == Type::ScopedTypeIdentifier {
        let x = b.get_child_rev(&0);
        let b = java_tree_gen.stores.node_store.resolve(x);
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
        } else {
            todo!("{:?}",t)
        }
    } else if t == Type::CatchType {
        // TODO check for type union eg. A|B|C
        let x = b.get_child(&0);
        let b = java_tree_gen.stores.node_store.resolve(x);
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
            todo!("{:?}",t)
        }
    } else {
        todo!("{:?}",t)
    }
    

}

pub fn eq_root_scoped(d: ExplorableRef, java_tree_gen: &JavaTreeGen, b: HashedNodeRef) -> bool {
    match d.as_ref() {
        RefsEnum::Root => todo!(),
        RefsEnum::MaybeMissing => {false
        },
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
                    let b = java_tree_gen.stores.node_store.resolve(*x);
                    let t = b.get_type();
                    if t == Type::ScopedAbsoluteIdentifier {
                        if !eq_root_scoped(d.with(*o),java_tree_gen,b) {
                            return false
                        }
                    } else if t == Type::Identifier {
                        if bo {
                            return eq_root_scoped(d.with(*o),java_tree_gen,b);
                        }
                        if b.has_label() {
                            let l = b.get_label();
                            if l != i.as_ref() {
                                return false
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
                todo!("{:?}",t)
            }
        }
        x => panic!("{:?}",x),
    }
}


pub fn find_refs(
    java_tree_gen: &JavaTreeGen,
    ana: &mut PartialAnalysis,
    i: usize,
    x: java_tree_gen_full_compress_legion_ref::NodeIdentifier,
) -> Vec<usize> {
    // let d: LabelValue = ;
    let b = java_tree_gen.stores.node_store.resolve(x);
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
    } else if &t == &Type::ImportDeclaration {
        println!("d=1 {:?}", &t);
        let c = {
            let d = ExplorableRef {
                rf: i,
                nodes: &ana.solver.nodes,
            };
            b.check(Into::<LabelValue>::into(d.clone()).as_ref())
        };
        if let BloomResult::MaybeContain = c {
            println!("+++import+++++Maybe contains");
            java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                &java_tree_gen.stores.node_store,
                &java_tree_gen.stores.label_store,
                &x,
            );
            println!();
            // TODO check if same canonical name
            let (stic, scop, asterisk) = {
                let b = java_tree_gen.stores.node_store.resolve(x);
                let mut scop = None;
                let mut sstatic = false;
                let mut asterisk = false;
                for c in b.get_children() {
                    let b = java_tree_gen.stores.node_store.resolve(*c);
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
                rf: i,
                nodes: &ana.solver.nodes,
            };
            if stic {
                return vec![]; // TODO
            } else if asterisk {
                return vec![]; // TODO
            } else if eq_root_scoped(d, java_tree_gen, scop.1) {
                
                let d = ana.solver.nodes.with(i);
                let i = if let RefsEnum::ScopedIdentifier(_,i)=d.as_ref() {
                    *i
                } else {
                    panic!()
                };
                let o = ana.solver.intern(RefsEnum::MaybeMissing);
                let i = ana.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                // let i = handle_import(
                //     java_tree_gen,
                //     ana,
                //     java_tree_gen.stores.node_store.resolve(scop.0),
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
    let c = {
        let d = ana.solver.nodes.with(i);
        b.check(Into::<LabelValue>::into(d.clone()).as_ref())
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
            || &t == &Type::Resource // TODO to check
        // find identifier
        {
            // Here, for now, we try to find Identifiers (not invocations)
            // thus we either search directly for scoped identifiers
            // or we search for simple identifiers because they do not present refs in themself
            println!("!found {:?}", &t);
            java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                &java_tree_gen.stores.node_store,
                &java_tree_gen.stores.label_store,
                &x,
            );
            println!();

            let d = ExplorableRef {
                rf: i,
                nodes: &ana.solver.nodes,
            };

            if eq_node_ref(d, java_tree_gen, x) {
                println!("really found");
            }
        } else if &t == &Type::TypeIdentifier {
            println!("!found TypeIdentifier");
            let mut out = IoOut { stream: stdout() };
            java_tree_gen_full_compress_legion_ref::serialize(
                &java_tree_gen.stores.node_store,
                &java_tree_gen.stores.label_store,
                &x,
                &mut out,
                &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
            );
        } else if &t == &Type::MethodDeclaration {
            // java_tree_gen::print_tree_syntax(
            //     &java_tree_gen.stores.node_store,
            //     &java_tree_gen.stores.label_store,
            //     &x,
            // );
            let mut out = IoOut { stream: stdout() };
            java_tree_gen_full_compress_legion_ref::serialize(
                &java_tree_gen.stores.node_store,
                &java_tree_gen.stores.label_store,
                &x,
                &mut out,
                &std::str::from_utf8(&java_tree_gen.line_break).unwrap(),
            );
        }
    } else {
        println!("Do not contains");
        return vec![];
    }

    let mut v: Vec<usize> = vec![];
    for x in b.get_children().clone() {
        let z = find_refs(java_tree_gen, ana, i, *x);
        v.extend(z);
        for w in v.clone() {
            let z = find_refs(java_tree_gen, ana, w, *x);
            v.extend(z)
        }
    }
    vec![]
}