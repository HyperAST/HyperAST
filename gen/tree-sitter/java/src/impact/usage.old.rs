
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
        log::info!("d=1 {:?}", &t);
        let c = {
            let d = ana.solver.nodes.with(target);
            b.check(Into::<Box<[u8]>>::into(d.clone()).as_ref())
        };
        if let BloomResult::MaybeContain = c {
            log::info!("+++import+++++Maybe contains");
            java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                &stores.node_store,
                &stores.label_store,
                &current,
            );
            log::info!();
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
                log::info!("import matched ref");
                return vec![i];
            } else {
                return vec![];
            }
        } else {
            log::info!("Do not contains");
            return vec![];
        }
    }
    if !b.has_children() {
        return vec![];
    }
    log::info!("d=1 {:?}", &t);
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
        log::info!("++++++++++++++Maybe contains");

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
            log::info!("!found {:?}", &t);
            java_tree_gen_full_compress_legion_ref::print_tree_syntax(
                &stores.node_store,
                &stores.label_store,
                &current,
            );
            log::info!();

            let d = ExplorableRef {
                rf: target,
                nodes: &ana.solver.nodes,
            };

            if eq_node_ref(d, stores, current) {
                // let mut position = path.to_position(&stores);
                // position.set_len(b.get_bytes_len() as usize);
                // log::info!("really found {:?}", position);
                log::info!("really found");
            }
        } else if &t == &Type::TypeIdentifier {
            log::info!("!found TypeIdentifier");
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
        log::info!("Do not contains");
        return vec![];
    }

    let mut v: Vec<usize> = vec![];
    log::info!("c_count {}", b.child_count());
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