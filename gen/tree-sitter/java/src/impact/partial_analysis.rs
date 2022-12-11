use std::{collections::HashMap, fmt::Display, hash::Hash, ops::Deref};

use enumset::{enum_set, EnumSet, EnumSetType};
use hyper_ast::types::{LabelStore, Type};
use num::ToPrimitive;

use crate::impact::{element::{Arguments, ListSet}, solver::{SolvingAssocTable, SolvingResult}};

use super::{
    declaration::{DeclType, Declarator, DisplayDecl},
    element::{IdentifierFormat, LabelPtr, RawLabelPtr, RefPtr, RefsEnum},
    java_element::Primitive,
    label_value::LabelValue,
    reference::DisplayRef,
    solver::Solver,
};

pub fn leaf_state(
    t: &Type,
    label: Option<LabelPtr>,
    id_format: Option<IdentifierFormat>,
) -> State<RefPtr, LabelPtr> {
    let r = if t == &Type::Comment {
        State::None
    } else if t.is_primitive() {
        // State::SimpleTypeIdentifier(label.unwrap())
        panic!("{:?} {:?}", t, label);
    } else if t.is_literal() {
        // State::LiteralType(label.unwrap())
        panic!("{:?} {:?}", t, label);
    } else if t == &Type::ScopedIdentifier {
        panic!();
    } else if t == &Type::ScopedTypeIdentifier {
        panic!();
    } else if t == &Type::Asterisk {
        State::Asterisk
    } else if t == &Type::ArgumentList {
        State::Arguments(vec![])
    } else if t == &Type::AnnotationArgumentList {
        State::Arguments(vec![])
    } else if t == &Type::FormalParameters {
        State::FormalParameters(vec![])
    } else if t == &Type::Super {
        panic!("{:?} {:?}", t, label);
    } else if t == &Type::This {
        //t.is_instance_ref() {
        panic!("{:?} {:?}", t, label);
    } else if t == &Type::TypeIdentifier {
        // assert!(!id_format.unwrap());
        State::SimpleTypeIdentifier(label.unwrap())
    // } else if t.is_identifier() {
    } else if t == &Type::Identifier {
        State::SimpleIdentifier(id_format.unwrap(), label.unwrap())
    } else if t == &Type::Spaces {
        State::None
    } else if t == &Type::Block {
        State::None
    } else if t == &Type::ElementValueArrayInitializer {
        State::None
    } else if t == &Type::Dimensions {
        State::Dimensions
    } else if t == &Type::TS86 {
        State::Modifiers(Visibility::None, enum_set!(NonVisibility::Static))
    } else if t == &Type::TS81 {
        State::Modifiers(Visibility::Public, enum_set!())
    } else if t == &Type::Error {
        // TODO do more clever debug things here
        State::None
    } else {
        assert_eq!(t, &Type::Comment);
        State::Todo
    };
    // println!("init: {:?} {:?}", t, r);
    r
}

#[derive(Debug, Clone)]
pub struct PartialAnalysis {
    current_node: State<RefPtr, LabelPtr>,
    pub solver: Solver,
    refs_count: u32,
}

impl Default for PartialAnalysis {
    fn default() -> Self {
        Self {
            current_node: State::None,
            solver: Default::default(),
            refs_count: 0,
        }
    }
}

const FAIL_ON_BAD_CST_NODE: bool = false;



macro_rules! missing_rule {
    () => {
        log::error!("missing rule");
        State::None
    };
    ($($arg:tt)+) => {{
        log::error!($($arg)+);
        State::None
    }};
}

impl PartialAnalysis {
    // apply before commiting/saving subtree
    pub fn resolve(mut self) -> Self {
        let mut cache: SolvingAssocTable = Default::default();
        log::debug!("resolve : {:?}", self.current_node);
        if let State::File {
            asterisk_imports,
            package,
            ..
        } = self.current_node.clone()
        {
            log::trace!("resolve file");
            let root = self.solver.intern(RefsEnum::Root);
            let mm = self.solver.intern(RefsEnum::MaybeMissing);
            let mask = self.solver.intern(RefsEnum::Mask(mm, Default::default()));
            let jlang = asterisk_imports[0];

            if let Some(package) = package {
                if asterisk_imports.is_empty() {
                    // self.solver.root = package;
                    // cache.insert(mm, Some(package));
                    panic!();
                } else {
                    // TODO explain usage
                    let refs = [package, root];
                    let result = self.solver.intern(RefsEnum::Or(
                        refs.iter().copied().collect()
                    ));
                    cache.insert(mask, SolvingResult::new(result, refs.into_iter().collect()));
                    // TODO explain usage
                    if package == jlang {
                        // let a = asterisk_imports[1..].iter().map(|imp| {
                        //     self.solver.intern(RefsEnum::Mask(
                        //         *imp,
                        //         vec![package, root].into_boxed_slice(),
                        //     ))
                        // });
                        let a = asterisk_imports[1..].iter().copied();
                        let a = a.chain([package, root].into_iter());
                        let refs:Vec<_> = a.collect();
                        let result = self.solver.intern(RefsEnum::Or(
                            refs.clone().into()
                        ));
                        cache.insert(mm, SolvingResult::new(result, refs.into()));
                    } else {
                        // let a = asterisk_imports.iter().map(|imp| {
                        //     self.solver.intern(RefsEnum::Mask(
                        //         *imp,
                        //         vec![package, root].into_boxed_slice(),
                        //     ))
                        // });
                        let a = asterisk_imports.into_iter();
                        let a = a.chain([package, root].into_iter());
                        let refs: ListSet<RefPtr> = a.collect();
                        refs.iter().for_each(|x| {
                            let x = self.solver.nodes.with(*x);
                            log::trace!("#| {:?}", x);
                        });
                        let result = self.solver.intern(RefsEnum::Or(
                            refs.iter().copied().collect()
                        ));
                        cache.insert(mm, SolvingResult::new(result, refs));
                    };
                }
            } else {
                cache.insert(
                    mask,
                    SolvingResult::new(mm, [mm].into_iter().collect())
                );
            }
        } else if let State::Declarations(ds) = self.current_node.clone(){
            for (_,d,_) in ds {
                if let Some(&d)  = d.node() {
                    assert!(!self.solver.has_choice(d),"{:?}",self.solver.nodes.with(d))
                }
            }
        }
        let (mut cache, mut solver) = (cache, self.solver);
        match &mut self.current_node {
            State::File {
                package,
                global,
                local,
                ..
            } => {
                for x in global {
                    x.1 = match &x.1 {
                        DeclType::Runtime(t) => {
                            let mut r = vec![];
                            for y in t.iter() {
                                let s = solver.solve_aux(&mut cache, *y);
                                if s.is_matched() {
                                    for z in s.iter() {
                                        if !r.contains(z) {
                                            r.push(*z);
                                        }
                                    }
                                } else {
                                    let w = s.waiting.unwrap();
                                    if !r.contains(&w) {
                                        r.push(w);
                                    }
                                }
                            }
                            DeclType::Runtime(r.into())
                        }
                        DeclType::Compile(t, s, i) => {
                            if let Some(p) = package {
                                let mut r = vec![];
                                for y in s.iter() {
                                    let s = solver.solve_aux(&mut cache, *y);
                                    if s.is_matched() {
                                        for z in s.iter() {
                                            if !r.contains(z) {
                                                r.push(*z);
                                            }
                                        }
                                    } else {
                                        let w = s.waiting.unwrap();
                                        if !r.contains(&w) {
                                            r.push(w);
                                        }
                                    }
                                }
                                let s = r.into();
                                let mut r = vec![];
                                for y in i.iter() {
                                    let s = solver.solve_aux(&mut cache, *y);
                                    if s.is_matched() {
                                        for z in s.iter() {
                                            if !r.contains(z) {
                                                r.push(*z);
                                            }
                                        }
                                    } else {
                                        let w = s.waiting.unwrap();
                                        if !r.contains(&w) {
                                            r.push(w);
                                        }
                                    }
                                }
                                let i = r.into();
                                DeclType::Compile(
                                    solver.try_solve_node_with(*t, *p).unwrap_or(*t),
                                    s,
                                    i,
                                )
                            } else {
                                // TODO check
                                DeclType::Compile(*t, s.clone(), i.clone())
                                // DeclType::Compile(
                                //     *t,
                                //     s.as_ref().copied(),
                                //     i.iter().copied().collect(),
                                // )
                            }
                        }
                    };
                }
                for x in local {
                    x.1 = match &x.1 {
                        DeclType::Runtime(t) => {
                            let mut r = vec![];
                            for y in t.iter() {
                                let s = solver.solve_aux(&mut cache, *y);
                                if s.is_matched() {
                                    for z in s.iter() {
                                        if !r.contains(z) {
                                            r.push(*z);
                                        }
                                    }
                                } else {
                                    let w = s.waiting.unwrap();
                                    if !r.contains(&w) {
                                        r.push(w);
                                    }
                                }
                            }
                            DeclType::Runtime(r.into())
                        }
                        DeclType::Compile(t, s, i) => {
                            if let Some(p) = package {
                                let mut r = vec![];
                                for y in s.iter() {
                                    let s = solver.solve_aux(&mut cache, *y);
                                    if s.is_matched() {
                                        for z in s.iter() {
                                            if !r.contains(z) {
                                                r.push(*z);
                                            }
                                        }
                                    } else {
                                        let w = s.waiting.unwrap();
                                        if !r.contains(&w) {
                                            r.push(w);
                                        }
                                    }
                                }
                                let s = r.into();
                                let mut r = vec![];
                                for y in i.iter() {
                                    let s = solver.solve_aux(&mut cache, *y);
                                    if s.is_matched() {
                                        for z in s.iter() {
                                            if !r.contains(z) {
                                                r.push(*z);
                                            }
                                        }
                                    } else {
                                        let w = s.waiting.unwrap();
                                        if !r.contains(&w) {
                                            r.push(w);
                                        }
                                    }
                                }
                                let i = r.into();
                                DeclType::Compile(
                                    solver.try_solve_node_with(*t, *p).unwrap_or(*t),
                                    s,
                                    i,
                                )
                            } else {
                                log::warn!("resolution of local type decl without a package should not append");
                                DeclType::Compile(
                                    *t,
                                    s.as_ref().into(),
                                    i.as_ref().into(),
                                )
                            }
                        }
                    };
                }

                // let mut r = bitvec::vec::BitVec::<Lsb0, usize>::default();
                // r.resize(solver.refs.len(), false);
                // let mm = solver.intern(RefsEnum::MaybeMissing);
                // for i in solver.refs.iter_ones() {
                //     match solver.nodes[i] {
                //         RefsEnum::ConstructorInvocation(o, _) if o == mm => {
                //             panic!();
                //         } // not possible ?
                //         RefsEnum::Invocation(o, _, _) if o == mm => {}
                //         _ => {
                //             r.set(i, true);
                //         }
                //     }
                // }
                // // TODO also remove the ones that refs the one s removed as they cannot really be resolved anymore
                // solver.refs = r;
            }
            State::Declarations(ds) => {
                for (_,d,_) in ds {
                    if let Some(&d)  = d.node() {
                        assert!(!solver.has_choice(d),"{:?}",solver.nodes.with(d))
                    }
                }
            }
            _ => (),
        };
        let (_, prev_solver) = solver.resolve(cache);
        let mut solver = Solver::default();
        let mut counted_intern = solver.counted_extend(&prev_solver);
        let current_node = self.current_node.map(|x| counted_intern.intern_external(&mut solver,x),|x| x);
        Self {
            current_node,
            solver,
            refs_count:counted_intern.count.to_u32().unwrap(),
        }
    }

    // pub fn refs(&self) -> impl Iterator<Item = LabelValue> + '_ {
    //     self.solver.refs()
    // }

    pub fn display_refs<'a, LS: LabelStore<str, I = RawLabelPtr>>(
        &'a self,
        leafs: &'a LS,
    ) -> impl Iterator<Item = impl Display + 'a> + 'a {
        self.solver.iter_refs().map(move |x| {
            let r: DisplayRef<LS> = (x, leafs).into();
            r
        })
    }

    pub fn print_refs<LS: LabelStore<str, I = RawLabelPtr>>(&self, leafs: &LS) {
        for x in self.display_refs(leafs) {
            println!("    {}", x);
        }
    }

    pub fn lower_estimate_refs_count(&self) -> u32 {
        self.solver.lower_estimate_refs_count()
    }
    
    pub fn estimated_refs_count(&self) -> u32 {
        self.refs_count.max(self.solver.lower_estimate_refs_count()*2)
    }

    pub fn print_decls<LS: LabelStore<str, I = RawLabelPtr>>(&self, leafs: &LS) {
        // let it = self.solver.iter_decls().map(move |x| {
        //     let r: DisplayDecl<LS> = (x, leafs).into();
        //     r
        // });
        for x in self.display_decls(leafs) {
            println!("    {}", x);
        }
    }
    pub fn display_decls<'a, LS: LabelStore<str, I = RawLabelPtr>>(
        &'a self,
        leafs: &'a LS,
    ) -> impl Iterator<Item = impl Display + 'a> + 'a {
        self.solver.iter_decls().map(move |x| {
            let r: DisplayDecl<LS> = (x, leafs).into();
            r
        })
    }

    pub fn decls_count(&self) -> usize {
        self.solver.decls_count()
    }

    pub fn init<F: FnMut(&str) -> RawLabelPtr>(
        kind: &Type,
        label: Option<&str>,
        mut intern_label: F,
    ) -> Self {
        let mut solver: Solver = Default::default();
        let mut intern_label = |x| LabelPtr::new(intern_label(x), IdentifierFormat::from(x));
        if kind == &Type::Directory {
            PartialAnalysis {
                current_node: State::None,
                solver,
                refs_count:0,
            }
        } else if kind == &Type::Program {
            // default_imports(&mut solver, intern_label);

            let i = solver.intern(RefsEnum::Root);
            let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("java")));
            let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("lang")));
            PartialAnalysis {
                current_node: State::File {
                    package: None,
                    asterisk_imports: vec![i],
                    global: vec![],
                    local: vec![],
                },
                solver,
                refs_count:0,
            }
        } else if kind == &Type::PackageDeclaration {
            // default_imports(&mut solver, |x| intern_label(x));

            // let i = solver.intern(RefsEnum::Root);
            // let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("java")));
            // let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("lang")));
            PartialAnalysis {
                current_node: State::None, //ScopedIdentifier(i),
                solver,
                refs_count:0,
            }
        } else if kind == &Type::This {
            let i = solver.intern(RefsEnum::MaybeMissing);
            let i = solver.intern_ref(RefsEnum::This(i));
            PartialAnalysis {
                current_node: State::This(i),
                solver,
                refs_count:0,
            }
        } else if kind == &Type::Super {
            let i = solver.intern(RefsEnum::MaybeMissing);
            let i = solver.intern(RefsEnum::Super(i));
            PartialAnalysis {
                current_node: State::Super(i),
                solver,
                refs_count:0,
            }
        } else if kind.is_literal() {
            let i = if kind == &Type::StringLiteral {
                let i = solver.intern(RefsEnum::Root);
                let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("java")));
                let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("lang")));
                solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("String")))
            } else {
                let p = Primitive::from(label.unwrap());
                solver.intern(RefsEnum::Primitive(p))
            };
            PartialAnalysis {
                current_node: State::LiteralType(i),
                solver,
                refs_count:0,
            }
        } else if kind.is_primitive() {
            // println!("{:?}", label);
            let p = Primitive::from(label.unwrap());
            let i = solver.intern(RefsEnum::Primitive(p));
            // let i = label.unwrap();
            // let t = solver.intern(RefsEnum::MaybeMissing);
            // let i = solver.intern(RefsEnum::ScopedIdentifier(t, i));
            PartialAnalysis {
                current_node: State::ScopedTypeIdentifier(i),
                solver,
                refs_count:0,
            }
            // panic!("{:?} {:?}",kind,label);
        } else if kind.is_type_declaration() {
            let r = solver.intern(RefsEnum::Root);
            let i = solver.intern(RefsEnum::ScopedIdentifier(r, intern_label("java")));
            let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("lang")));
            let s = solver.intern(RefsEnum::TypeIdentifier(i, intern_label("Object")));

            let d = solver.intern(RefsEnum::Super(r));
            let d = solver.intern(RefsEnum::ConstructorInvocation(d, Arguments::Unknown));
            let d = Declarator::Executable(d);
            solver.add_decl(d, DeclType::Runtime(vec![s].into())); // TODO check

            PartialAnalysis {
                current_node: State::TypeDeclaration {
                    visibility: Visibility::None,
                    identifier: DeclType::Compile(
                        0,
                        vec![s].into_boxed_slice(),
                        vec![].into_boxed_slice(),
                    ),
                    members: vec![],
                },
                solver,
                refs_count:0,
            }
        } else if kind == &Type::TypeParameter {
            let r = solver.intern(RefsEnum::Root);
            let i = solver.intern(RefsEnum::ScopedIdentifier(r, intern_label("java")));
            let i = solver.intern(RefsEnum::ScopedIdentifier(i, intern_label("lang")));
            let s = solver.intern(RefsEnum::TypeIdentifier(i, intern_label("Object")));

            PartialAnalysis {
                current_node: State::TypeDeclaration {
                    visibility: Visibility::None,
                    identifier: DeclType::Compile(
                        0,
                        vec![s].into_boxed_slice(),
                        vec![].into_boxed_slice(),
                    ),
                    members: vec![],
                },
                solver,
                refs_count:0,
            }
        } else if kind == &Type::ClassBody {
            // TODO constructor solve
            // {
            //     let t = solver.intern(RefsEnum::MaybeMissing);
            //     let t = solver.intern(RefsEnum::This(t));
            //     let i = solver.intern(RefsEnum::ConstructorInvocation(t,Arguments::Given(vec![].into_boxed_slice())));
            //     let t = solver.intern(RefsEnum::Root);
            //     let t = solver.intern(RefsEnum::This(t));
            //     let d = Declarator::Executable(i);
            //     solver.add_decl_simple(d, t);
            // }
            // {
            //     let t = solver.intern(RefsEnum::MaybeMissing);
            //     let i = solver.intern(RefsEnum::ConstructorInvocation(t,Arguments::Given(vec![].into_boxed_slice())));
            //     let t = solver.intern(RefsEnum::Root);
            //     let t = solver.intern(RefsEnum::This(t));
            //     let d = Declarator::Executable(i);
            //     solver.add_decl_simple(d, t);
            // }

            PartialAnalysis {
                current_node: State::None,
                solver,
                refs_count:0,
            }
        } else {
            let is_lowercase = label.map(|x| x.into());
            let label = label.map(intern_label);
            PartialAnalysis {
                current_node: leaf_state(kind, label, is_lowercase),
                solver,
                refs_count:0,
            }
        }
    }

    pub fn acc(self, kind: &Type, acc: &mut Self) {
        let current_node = self.current_node;
        log::trace!(
            "{:?} {:?} {:?}\n**{:?}",
            &kind,
            &acc.current_node,
            &current_node,
            acc.solver.iter_refs().map(|x|x).collect::<Vec<_>>()
        );

        macro_rules! mm {
            () => {
                acc.solver.intern(RefsEnum::MaybeMissing)
            };
        }
        macro_rules! scoped_ref {
            ( $o:expr, $i:expr ) => {{
                let o = $o;
                acc.solver.intern_ref(RefsEnum::ScopedIdentifier(o, $i))
            }};
        }
        // macro_rules! scoped_ref {
        //     ( $o:expr, $i:expr ) => {{
        //         let o = $o;
        //         acc.solver.intern_ref(RefsEnum::ScopedIdentifier(o, $i))
        //     }};
        // }
        macro_rules! scoped_type {
            ( $o:expr, $i:expr ) => {{
                let o = $o;
                acc.solver.intern_ref(RefsEnum::TypeIdentifier(o, $i))
            }};
        }
        macro_rules! spec {
            ( $o:expr, $i:expr ) => {{
                let i = $i;
                let o = $o;
                match acc.solver.nodes[i].clone() {
                    RefsEnum::This(i) => {
                        assert_eq!(i, mm!());
                        acc.solver.intern_ref(RefsEnum::This(o))
                    }
                    RefsEnum::Super(i) => {
                        assert_eq!(i, mm!());
                        acc.solver.intern_ref(RefsEnum::Super(o))
                    }
                    x => panic!("{:?}", x),
                }
            }};
        }

        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        struct Old<T>(T)
        where
            T: std::cmp::Eq + std::hash::Hash + Clone;

        //main organization top down, through type kind
        acc.current_node = if kind == &Type::Error {
            if FAIL_ON_BAD_CST_NODE {
                panic!("{:?} {:?} {:?}", kind, acc.current_node, current_node)
            } else {
                acc.current_node.take()
            }
        } else if kind == &Type::Program {
            // TODO should do things with RefsEnum:Mask
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            macro_rules! syncd {
                ( $i:expr ) => {{
                    let r = remapper.intern_external_decl(&mut acc.solver, $i.0);
                    assert!(!acc.solver.has_choice(r),"{:?}",acc.solver.nodes.with(r));
                    r
                }};
            }
            
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                // (
                //     State::File {
                //         package: p,
                //         asterisk_imports,
                //         mut global,
                //         mut local,
                //     },
                //     State::Declaration {
                //         visibility,
                //         kind: t,
                //         identifier: d,
                //     },
                // ) => {
                //     // no package declaration at start of java file
                //     if let Visibility::Public = visibility {
                //         &mut global
                //     } else {
                //         &mut local
                //     }
                //     .push((d.with_changed_node(|x| sync!(x)), sync!(t)));
                //     State::File {
                //         package: p,
                //         asterisk_imports,
                //         global,
                //         local,
                //     }
                // }
                (
                    State::File {
                        package: None,
                        asterisk_imports,
                        global,
                        local,
                    },
                    State::PackageDeclaration(p),
                ) => {
                    // for (d, t) in &self.solver.decls {
                    //     let d = d.with_changed_node(|x| sync!(Old(*x)));
                    //     let t = match t {
                    //         DeclType::Runtime(b) => {
                    //             DeclType::Runtime(b.iter().map(|x| sync!(Old(*x))).collect())
                    //         }
                    //         DeclType::Compile(t, s, i) => DeclType::Compile(
                    //             sync!(Old(*t)),
                    //             s.map(|x| sync!(Old(x))),
                    //             i.iter().map(|x| sync!(Old(*x))).collect(),
                    //         ),
                    //     };
                    //     acc.solver.add_decl(d, t);
                    // }
                    State::File {
                        package: Some(sync!(p)),
                        asterisk_imports,
                        global,
                        local,
                    }
                }
                (
                    State::File {
                        package: p,
                        asterisk_imports,
                        global,
                        local,
                    },
                    State::None,
                ) if kind == &Type::Program => State::File {
                    package: p,
                    asterisk_imports,
                    global,
                    local,
                },
                // (
                //     State::File {
                //         package: p,
                //         asterisk_imports,
                //         top_level,
                //         mut content,
                //     },
                //     State::TypeDeclaration {
                //         visibility,
                //         identifier: d,
                //         members: _,
                //     },
                // ) => {
                //     // TODO check for file's class? visibility ? etc
                //     // TODO maybe bubleup members
                //     let top_level = match d {
                //         DeclType::Compile(d, _, _) => {
                //             let d = sync!(d);
                //             let i = Declarator::Type(d);
                //             content.push((i.clone(), d));
                //             acc.solver.add_decl_simple(i.clone(), d);
                //             if let Visibility::Public = visibility {
                //                 Some((i, d))
                //             } else {
                //                 None
                //             }
                //         }
                //         _ => panic!(),
                //     };

                //     State::File {
                //         package: p,
                //         asterisk_imports,
                //         top_level,
                //         content,
                //     }
                // }
                (
                    State::File {
                        package: p,
                        mut asterisk_imports,
                        global,
                        local,
                    },
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic,
                        asterisk,
                    },
                ) => {
                    // assert!(p.is_some());
                    // TODO do something with sstatic and asterisk
                    let _ = sstatic;
                    if asterisk {
                        let d = sync!(i);
                        // TODO static
                        asterisk_imports.push(d);
                    } else {
                        let Old(i) = i;
                        match &self.solver.nodes[i] {
                            RefsEnum::ScopedIdentifier(o, i) => {
                                let o = sync!(Old(*o));
                                let r = mm!();
                                let shorten = acc.solver.intern(RefsEnum::TypeIdentifier(r, *i));
                                assert!(!acc.solver.has_choice(shorten),"{:?}",acc.solver.nodes.with(shorten));
                                let d = scoped_ref!(o, *i);
                                acc.solver.add_decl(
                                    Declarator::Type(shorten),
                                    DeclType::Runtime(vec![d].into()),
                                );
                            }
                            RefsEnum::TypeIdentifier(o, i) => {
                                let o = sync!(Old(*o));
                                let r = mm!();
                                let shorten = acc.solver.intern(RefsEnum::TypeIdentifier(r, *i));
                                assert!(!acc.solver.has_choice(shorten),"{:?}",acc.solver.nodes.with(shorten));
                                let d = scoped_ref!(o, *i);
                                acc.solver.add_decl(
                                    Declarator::Type(shorten),
                                    DeclType::Runtime(vec![d].into()),
                                );
                            }
                            RefsEnum::Invocation(o, i, p) => {
                                let o = sync!(Old(*o));
                                let r = mm!();
                                let p = p.map(|x| sync!(Old(*x)));
                                let shorten =
                                    acc.solver.intern(RefsEnum::Invocation(r, *i, p.clone()));
                                assert!(!acc.solver.has_choice(shorten),"{:?}",acc.solver.nodes.with(shorten));
                                let d =
                                    acc.solver
                                        .intern_ref(RefsEnum::Invocation(o, *i, p.clone())); // TODO use it
                                let _ = d;
                                acc.solver.add_decl(
                                    Declarator::Type(shorten),
                                    DeclType::Runtime(vec![].into()),
                                );
                            }
                            _ => panic!(),
                        };
                    }
                    State::File {
                        package: p,
                        asterisk_imports,
                        global,
                        local,
                    }
                }
                (
                    State::File {
                        package: p,
                        asterisk_imports,
                        mut global,
                        mut local,
                    },
                    State::TypeDeclaration {
                        visibility,
                        identifier,
                        members,
                    },
                ) => {
                    // assert!(p.is_some());
                    // check for file's class? visibility? etc
                    // TODO maybe bubleup members
                    // remove asterisk import if declared in file
                    let identifier = match (identifier, p) {
                        (DeclType::Compile(d, sup, int), Some(p)) => {
                            let d = sync!(d);
                            let sup = sup.iter().map(|x| sync!(*x)).collect();
                            let int = int.iter().map(|x| sync!(*x)).collect();
                            assert!(!acc.solver.has_choice(p),"{:?}",acc.solver.nodes.with(p));
                            let solved = acc.solver.try_solve_node_with(d, p).unwrap();
                                let i = Declarator::Type(solved);
                            let solved = DeclType::Compile(solved, sup, int);
                            if let Visibility::Public = visibility {
                                global.push((i.clone(), solved.clone()));
                            } else {
                                local.push((i.clone(), solved.clone()));
                            }
                            acc.solver.add_decl(i.clone(), solved.clone());
                            assert!(!acc.solver.has_choice(d),"{:?}",acc.solver.nodes.with(d));
                            let i = Declarator::Type(d);
                            if let Visibility::Public = visibility {
                                global.push((i.clone(), solved.clone()));
                            } else {
                                local.push((i.clone(), solved.clone()));
                            }
                            acc.solver.add_decl(i.clone(), solved);
                            d
                        }
                        (DeclType::Compile(d, sup, int), None) => {
                            let d = sync!(d);
                            assert!(!acc.solver.has_choice(d),"{:?}",acc.solver.nodes.with(d));
                            let i = Declarator::Type(d);
                            let sup = sup.iter().map(|x| sync!(*x)).collect();
                            let int = int.iter().map(|x| sync!(*x)).collect();
                            let t = DeclType::Compile(d, sup, int);
                            if let Visibility::Public = visibility {
                                global.push((i.clone(), t.clone()));
                            } else {
                                local.push((i.clone(), t.clone()));
                            }
                            acc.solver.add_decl(i.clone(), t);
                            d
                        }
                        _ => panic!(),
                    };
                    log::trace!("{}", members.len());
                    for (v, d, t) in members {
                        let d = d.with_changed_node(|i| syncd!(*i));
                        let t = t.map(|x| sync!(x)); // TODO try solving t
                                                     // println!("d:{:?} t:{:?}", &d, &t);

                        let container =
                            if Visibility::Public == visibility && Visibility::Public == v {
                                &mut global
                            } else {
                                &mut local
                            };

                        match &d {
                            Declarator::Executable(d) => {
                                // TODO constructor solve
                                if let Some(p) = p {
                                    // let t = acc.solver.try_solve_node_with(t, p).unwrap_or(t);
                                    {
                                        let d = Declarator::Executable(*d);
                                        acc.solver.add_decl(d, t.clone());
                                    }
                                    let solved = acc.solver.try_solve_node_with(*d, p).unwrap();
                                    let d = Declarator::Executable(solved);
                                    acc.solver.add_decl(d.clone(), t.clone());
                                    container.push((d, t));
                                } else {
                                    let d = Declarator::Executable(*d);
                                    acc.solver.add_decl(d, t);
                                }
                            }
                            Declarator::Field(d) => {
                                if let Some(p) = p {
                                    // let t = acc.solver.try_solve_node_with(t, p).unwrap_or(t);
                                    {
                                        assert!(!acc.solver.has_choice(*d));
                                        let d = Declarator::Field(*d);
                                        acc.solver.add_decl(d, t.clone());
                                    }
                                    assert!(!acc.solver.has_choice(p));
                                    let solved = acc.solver.try_solve_node_with(*d, p).unwrap();
                                    let d = Declarator::Field(solved);
                                    acc.solver.add_decl(d.clone(), t.clone());
                                    container.push((d, t));
                                } else {
                                    let d = Declarator::Field(*d);
                                    acc.solver.add_decl(d, t);
                                }
                            }
                            Declarator::Type(d) => {
                                if let Some(p) = p {
                                    assert!(!acc.solver.has_choice(p),"{:?}",acc.solver.nodes.with(p));
                                    let solved = acc.solver.try_solve_node_with(*d, p).unwrap();
                                    let d = Declarator::Type(*d);
                                    acc.solver.add_decl(d, t.clone()); // TODO try_solve_node_with in resolve when we have a case where we avec seen the declaration ie. DeclType::Compile
                                    let d = Declarator::Type(solved);
                                    acc.solver.add_decl(d.clone(), t.clone());
                                    container.push((d, t));
                                } else {
                                    assert!(!acc.solver.has_choice(*d),"{:?}",acc.solver.nodes.with(*d));
                                    let d = Declarator::Type(*d);
                                    acc.solver.add_decl(d, t);
                                }
                            }
                            x => panic!("{:?}", x),
                        }
                    }
                    // let global = if let Visibility::Public = visibility {
                    //     assert!(global.is_none());
                    //     let d = Declarator::Type(identifier);
                    //     Some((d, identifier))
                    // } else if let Some(_) = global {
                    //     global
                    // } else {
                    //     None
                    // };
                    State::File {
                        package: p,
                        asterisk_imports,
                        global,
                        local,
                    }
                }
                // SHOULD not be needed if no rules of Program resturn None
                // (
                //     State::None,
                //     State::None
                // ) => {
                //     State::None
                // }
                // not yet implemented: Program None Declarations([(None, Variable(Old(3)), Runtime([Old(2)]))]) 
                // not yet implemented: Program None TypeDeclaration { visibility: Public, identifier: Compile(Old(11), [Old(12)], []), members: [(None, Field(Old(263)), 
                // not yet implemented: Program None ImportDeclaration { sstatic: false, identifier: Old(4), asterisk: false }
                // TODO aaa not yet implemented: Program File { package: None, asterisk_imports: [3], global: [], local: [] } Declarations([(None, Variable(Old(3)), Runtime([Old(2)]))])
                (
                    State::File {
                        package: p,
                        asterisk_imports,
                        global,
                        local,
                    },
                    State::Declarations(_),
                ) => {
                    State::File {
                        package: p,
                        asterisk_imports,
                        global,
                        local,
                    }
                }
                (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind == &Type::PackageDeclaration {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (State::None, State::ScopedIdentifier(i)) => {
                    // TODO complete refs
                    let i = sync!(i);
                    // if jl == i {
                    //     acc.solver.decls = Default::default();
                    // }
                    State::PackageDeclaration(i)
                }
                (State::None, State::SimpleIdentifier(_, i)) => {
                    // TODO complete refs
                    let o = acc.solver.intern(RefsEnum::Root);
                    let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                    State::PackageDeclaration(i)
                }
                (State::None, State::Annotation) => State::None,
                (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind == &Type::ImportDeclaration {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (State::None, State::Modifiers(v, n)) => State::Modifiers(v, n),
                (State::Modifiers(Visibility::None, n), State::ScopedIdentifier(i)) => {
                    // println!("{:?}",n);
                    assert_eq!(n, enum_set!(NonVisibility::Static));
                    let i = sync!(i);

                    let (o, i) = match &acc.solver.nodes[i] {
                        RefsEnum::ScopedIdentifier(o, i) => (*o, *i),
                        _ => panic!(),
                    };
                    if o >= acc.solver.refs.len() {
                        acc.solver.refs.resize(o + 1, false);
                    }
                    acc.solver.refs.set(o, true);
                    let i = acc
                        .solver
                        .intern_ref(RefsEnum::Invocation(o, i, Arguments::Unknown));
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic: true,
                        asterisk: false,
                    } // TODO use static
                }
                (State::None, State::ScopedIdentifier(i)) => {
                    let i = sync!(i);
                    if i >= acc.solver.refs.len() {
                        acc.solver.refs.resize(i + 1, false);
                    }
                    acc.solver.refs.set(i, true);
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic: false,
                        asterisk: false,
                    }
                }
                (State::None, State::SimpleIdentifier(_,i)) => {
                    let r = acc.solver.intern(RefsEnum::Root);
                    let i = scoped_ref!(r,i);
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic: false,
                        asterisk: false,
                    }
                }
                (
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic,
                        asterisk: false,
                    },
                    State::Asterisk,
                ) => {
                    // TODO say we import members/classes
                    // acc.solver.refs.set(i, false); // TODO check if its ok to not remove ref
                    State::ImportDeclaration {
                        identifier: i,
                        sstatic,
                        asterisk: true,
                    }
                }
                (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind.is_type_declaration() {
            match current_node.map(|x| Old(x), |x| x) {
                State::Modifiers(v, n) => {
                    let mut remapper = acc.solver.extend(&self.solver);
                    if let State::TypeDeclaration { visibility, .. } = &mut acc.current_node {
                        *visibility = v;
                        acc.current_node.take()
                    } else if State::None == acc.current_node {
                        assert_eq!(kind, &Type::EnumConstant);
                        State::TypeDeclaration {
                            visibility: Visibility::None,
                            identifier: DeclType::Compile(
                                0,
                                vec![].into_boxed_slice(),
                                vec![].into_boxed_slice(),
                            ),
                            members: vec![],
                        }
                    } else {
                        panic!("{:?} {:?}", kind, acc.current_node)
                    }
                }
                State::SimpleIdentifier(case, i) => {
                    // assert!(!case);
                    if let State::TypeDeclaration { identifier, .. } = &mut acc.current_node {
                        if let DeclType::Compile(ii, _, _) = identifier {
                            let r = mm!();
                            let i = acc.solver.intern(RefsEnum::TypeIdentifier(r, i));
                            *ii = i;
                        } else {
                            panic!("{:?}", acc.current_node)
                        }
                        acc.current_node.take()
                    } else {
                        assert_eq!(kind, &Type::EnumConstant);
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::TypeIdentifier(r, i));
                        State::TypeDeclaration {
                            visibility: Visibility::None,
                            identifier: DeclType::Compile(
                                i,
                                vec![].into_boxed_slice(),
                                vec![].into_boxed_slice(),
                            ),
                            members: vec![],
                        }
                    }
                }
                State::Arguments(_) => {
                    assert_eq!(kind, &Type::EnumConstant);
                    let mut remapper = acc.solver.extend(&self.solver);
                    // TODO materialize the construtor call
                    acc.current_node.take()
                }
                State::TypeParameters(ps) => {
                    assert!(kind == &Type::ClassDeclaration || kind == &Type::InterfaceDeclaration);
                    let mut remapper = acc.solver.extend(&self.solver);
                    macro_rules! sync {
                        ( $i:expr ) => {
                            remapper.intern_external(&mut acc.solver, $i.0)
                        };
                    }
                    // println!("typeParams {:?}", ps);
                    for t in ps {
                        if let DeclType::Compile(d,ext,imp) = &t {
                            let d = Declarator::Type(sync!(*d));
                            let mut v:Vec<_> = ext.iter().map(|t| sync!(*t)).collect();
                            v.extend(imp.iter().map(|t| sync!(*t)));
                            acc.solver.add_decl(d.clone(), DeclType::Runtime(v.into()));
                        }
                    }
                    // println!("decls after added typeParams");
                    // acc.solver.print_decls();
                    // TODO use generics when creating ref from decl ie. searching for impacts
                    acc.current_node.take()
                }
                State::ScopedTypeIdentifier(s) => {
                    assert_eq!(kind, &Type::ClassDeclaration);
                    let mut remapper = acc.solver.extend(&self.solver);
                    macro_rules! sync {
                        ( $i:expr ) => {
                            remapper.intern_external(&mut acc.solver, $i.0)
                        };
                    }
                    if let State::TypeDeclaration { identifier: i, .. } = &mut acc.current_node {
                        let s = sync!(s);
                        match i {
                            DeclType::Compile(_, ss, _) => {
                                // ?.super#constructor(...) -> ?.S
                                let r = mm!();
                                let d = acc.solver.intern(RefsEnum::Super(r));
                                let d = acc
                                    .solver
                                    .intern(RefsEnum::ConstructorInvocation(d, Arguments::Unknown));
                                let d = Declarator::Executable(d);
                                acc.solver.add_decl(d, DeclType::Runtime(vec![s].into()));
                                // TODO this one? ?.S.super#constructor(...) -> ?.S

                                *ss = vec![s].into_boxed_slice()
                            }
                            x => panic!("{:?}", x),
                        };
                        // TODO use superclass value more
                    } else {
                        panic!()
                    }
                    acc.current_node.take()
                }
                State::Interfaces(i) => {
                    assert!(
                        kind == &Type::ClassDeclaration
                            || kind == &Type::InterfaceDeclaration
                            || kind == &Type::EnumDeclaration,
                        "{:?}",
                        kind
                    );
                    let mut remapper = acc.solver.extend(&self.solver);
                    macro_rules! sync {
                        ( $i:expr ) => {
                            remapper.intern_external(&mut acc.solver, $i.0)
                        };
                    }
                    if let State::TypeDeclaration { identifier, .. } = &mut acc.current_node {
                        let i = i.into_iter().map(|x| sync!(x)).collect();
                        match identifier {
                            DeclType::Compile(_, _, ii) => *ii = i,
                            x => panic!("{:?}", x),
                        };
                        // TODO use superclass value more
                    } else {
                        panic!()
                    }
                    acc.current_node.take()
                }
                State::None => {
                    // TODO there might be things to do but need tests
                    let mut remapper = acc.solver.extend(&self.solver);
                    // let (cache, solver) = acc.solver.hierarchy_solve_extend(&self.solver);
                    // acc.solver = solver;
                    // macro_rules! sync {
                    //     ( $i:expr ) => {{
                    //         let other = $i.0;
                    //         let other = ExplorableRef {
                    //             rf: other,
                    //             nodes: &acc.solver.nodes,
                    //         };
                    //         acc.solver.intern_external(&mut cache, other)
                    //     }};
                    // }
                    acc.current_node.take()
                }
                State::Declarations(ds) => {
                    if let State::TypeDeclaration {
                        identifier,
                        members,
                        ..
                    } = &mut acc.current_node
                    {
                        let id = match &identifier {
                            DeclType::Compile(i, _, _) => *i,
                            _ => panic!(),
                        };
                        assert!(!acc.solver.has_choice(id),"{:?}",acc.solver.nodes.with(id));
                        // prime cache
                        let mut extend_cache = HashMap::<usize, usize>::default();
                        let mut extend_cache_decls = HashMap::<usize, usize>::default();
                        if let Some(mm) = self.solver.get(RefsEnum::MaybeMissing) {
                            assert!(
                                !(self.solver.refs.len() > mm && self.solver.refs[mm]),
                                "not sure what to do there"
                            );
                            // then ? -> ?.{A.B,C} and ?.this -> ?.A
                            let r = mm!();
                            let t = {
                                let d = acc.solver.intern(RefsEnum::This(r));
                                let d = Declarator::Type(d);

                                acc.solver.add_decl(d, identifier.clone()); //TODO also put it in cache

                                match &identifier {
                                    DeclType::Compile(i, s, is) => {
                                        let j = match &acc.solver.nodes[*i] {
                                            RefsEnum::ScopedIdentifier(o, i) => {
                                                Some(RefsEnum::TypeIdentifier(*o, *i))
                                            }
                                            _ => None,
                                        };
                                        let i = if let Some(i) = j {
                                            acc.solver.intern(i)
                                        } else {
                                            *i
                                        };
                                        let mut t = vec![i];
                                        t.extend(s.iter());
                                        t.extend(is.iter());
                                        t
                                    }
                                    _ => panic!(),
                                }
                            };
                            log::trace!("class decl cache {:?}", &t);
                            // if let DeclType::Compile(_, Some(s), _) = &identifier {
                            //     let d = Declarator::Type(*s);
                            //     acc.solver.add_decl_simple(d, *s);
                            // }

                            for id in t.iter() {
                                let i = match &acc.solver.nodes[*id] {
                                    RefsEnum::ScopedIdentifier(_, i) => i.clone(),
                                    RefsEnum::TypeIdentifier(_, i) => i.clone(),
                                    _ => panic!(),
                                };
                                // ?.X -> ?.X to protect from masking
                                if let Some(x) = self.solver.get(RefsEnum::ScopedIdentifier(mm, i))
                                {
                                    extend_cache.insert(x, *id);
                                }
                            }

                            // temporary
                            if let Some(i) = self.solver.get(RefsEnum::Super(mm)) {
                                extend_cache.insert(i, id);
                                extend_cache_decls.insert(i, id);
                            }

                            // let to_cache = acc.solver.intern(RefsEnum::Mask(r, t));
                            let mut t = t;
                            t.push(mm);
                            let to_cache = acc.solver.intern(RefsEnum::Or(t.into()));
                            extend_cache.insert(mm, to_cache);
                        }
                        // // then stash refs from decl
                        // let hierarchical_decls_refs: Vec<_> = acc.solver.refs.iter_ones().collect();
                        // acc.solver.refs = Default::default(); // TODO not sure;

                        // then extend refs from body with a primed cache
                        let mut remapper = acc.solver.extend_map(&self.solver, &mut extend_cache, extend_cache_decls);
                        macro_rules! sync {
                            ( $i:expr ) => {
                                remapper.intern_external(&mut acc.solver, $i.0)
                            };
                        }
                        macro_rules! syncd {
                            ( $i:expr ) => {{
                                let r = $i.0;
                                assert!(!self.solver.has_choice(r),"{:?}",self.solver.nodes.with(r));
                                let r = remapper.intern_external_decl(&mut acc.solver, r);
                                assert!(!acc.solver.has_choice(r),"{:?}",acc.solver.nodes.with(r));
                                r
                            }};
                        }
                        // then handle members considering prev thing ie. either ?.this -> ?.A or ? -> ?.{B,C}
                        // then resolve
                        // then pop ref stash extend new solver with them
                        // {
                        //     // ?.this -> ?.A
                        // }
                        // // TODO an extend that replace ? -> ?.{B,C}
                        // // idem for the following types
                        // // then only call resolve with:
                        // { // for A extends B implements C
                        //      // ?.B -> ?.B
                        //      // ?.C -> ?.C
                        //      // ?.super -> ?.B
                        //      // ?.B.super -> ?.B
                        //      // ?.C.super -> ?.C
                        // }
                        {
                            // ?.super -> ?.super
                            let r = mm!();
                            let i = acc.solver.intern(RefsEnum::Super(r));
                            let d = Declarator::Type(i);
                            acc.solver.add_decl(d, identifier.clone());
                        }
                        {
                            // ?.A -> ?.A
                            let d = Declarator::Type(id);
                            acc.solver.add_decl(d, identifier.clone());
                        }
                        {
                            // ?.A.this -> ?.A
                            //     let d = acc.solver.intern(RefsEnum::This(id));
                            //     let d = Declarator::Type(d);
                            //     acc.solver.add_decl(d, identifier.clone());
                            //     // TODO this one? ?.S.super -> ?.S
                        }
                        {
                            // ?.A#() -> ?.A
                            // let d = acc.solver.intern(RefsEnum::ConstructorInvocation(
                            //     id,
                            //     Arguments::Given(vec![].into_boxed_slice()),
                            // ));
                            // let d = Declarator::Executable(d);
                            // acc.solver.add_decl(d, identifier.clone());
                        }
                        {
                            // ?.this#(...) -> ?.A
                            // let d = mm!();
                            // let d = acc.solver.intern(RefsEnum::This(d));
                            // let d = acc
                            //     .solver
                            //     .intern(RefsEnum::ConstructorInvocation(d, Arguments::Unknown));
                            // let d = Declarator::Executable(d);
                            // acc.solver.add_decl(d, identifier.clone());
                        }
                        {
                            // ?.this -> ?.A
                            let d = mm!();
                            let d = acc.solver.intern(RefsEnum::This(d));
                            let d = Declarator::Type(d);
                            acc.solver.add_decl(d, identifier.clone());
                        }
                        // let (mut cache, solver) = acc.solver.hierarchy_solve_extend(&self.solver);
                        // acc.solver = solver;
                        // macro_rules! sync {
                        //     ( $i:expr ) => {{
                        //         let other = $i.0;
                        //         let other = ExplorableRef {
                        //             rf: other,
                        //             nodes: &self.solver.nodes,
                        //         };
                        //         acc.solver
                        //             .hierarchy_solve_intern_external(&mut cache, other)
                        //             .unwrap()
                        //     }};
                        // }
                        // println!("adding members");
                        for (v, d, t) in ds {
                            let d = d.with_changed_node(|i| syncd!(*i));
                            let t = t.map(|x| sync!(x));
                            // println!("d:{:?} t:{:?}", &d, &t);
                            match &d {
                                Declarator::Executable(d) => {
                                    match acc.solver.nodes[*d].clone() {
                                        RefsEnum::ConstructorInvocation(o, p) => {
                                            // constructor solve
                                            {
                                                // TODO test if it does ?.A#(p) => ?.A
                                                let d = Declarator::Executable(*d);
                                                acc.solver.add_decl(d, identifier.clone());
                                            }
                                            {
                                                // TODO not sure how to change o
                                                // given class A, it might be better to solve ?.this#(p) here to ?.A.this#(p) and in general ?.A.this -> ?A.
                                                let solved = acc
                                                    .solver
                                                    .intern(RefsEnum::ConstructorInvocation(id, p));
                                                // acc.solver.solve_node_with(*d, i); // to spec ?.this#(p)
                                                let d = Declarator::Executable(solved);
                                                acc.solver.add_decl(d.clone(), identifier.clone());
                                                members.push((v, d, t));
                                                // members.push((v, d, id));
                                            }
                                        }
                                        RefsEnum::Invocation(o, i, p) => {
                                            {
                                                let d = Declarator::Executable(*d);
                                                acc.solver.add_decl(d, t.clone());
                                            }
                                            {
                                                let solved =
                                                    acc.solver.try_solve_node_with(*d, id).unwrap();
                                                let d = Declarator::Executable(solved);
                                                acc.solver.add_decl(d.clone(), t.clone());
                                                members.push((v, d, t.clone()));
                                            }
                                            {
                                                let r = mm!();
                                                let r = acc.solver.intern(RefsEnum::This(r));
                                                let solved =
                                                    acc.solver.try_solve_node_with(*d, id).unwrap();
                                                let d = Declarator::Executable(solved);
                                                acc.solver.add_decl(d.clone(), t.clone());
                                                members.push((v, d, t));
                                            }
                                        }
                                        x => {
                                            log::error!("executable in declarations of a type declaration should handle: {:?}", x)
                                        }
                                    }
                                }
                                Declarator::Field(d) => {
                                    {
                                        // ?.d => t
                                        assert!(!acc.solver.has_choice(*d),"{:?}",acc.solver.nodes.with(*d));
                                        let d = Declarator::Field(*d);
                                        acc.solver.add_decl(d, t.clone());
                                    }
                                    {
                                        // println!("{:?}",acc.solver.nodes.with(*d));
                                        assert!(!acc.solver.has_choice(id),"{:?}",acc.solver.nodes.with(id));
                                        // ?.id.d => t
                                        let solved =
                                            acc.solver.try_solve_node_with(*d, id).unwrap();
                                        let d = Declarator::Field(solved);
                                        acc.solver.add_decl(d.clone(), t.clone());
                                        members.push((v, d, t.clone()));
                                    }
                                    {
                                        // ?.this.d => t
                                        let r = mm!();
                                        let r = acc.solver.intern(RefsEnum::This(r));
                                        let solved = acc.solver.try_solve_node_with(*d, r).unwrap();
                                        let d = Declarator::Field(solved);
                                        acc.solver.add_decl(d.clone(), t.clone());
                                        members.push((v, d, t));
                                    }
                                }
                                Declarator::Type(d) => {
                                    {
                                        assert!(!acc.solver.has_choice(*d),"{:?}",acc.solver.nodes.with(*d));
                                        let d = Declarator::Type(*d);
                                        acc.solver.add_decl(d, t.clone());
                                    }
                                    {
                                        assert!(!acc.solver.has_choice(id),"{:?}",acc.solver.nodes.with(id));
                                        let solved =
                                            acc.solver.try_solve_node_with(*d, id).unwrap();
                                        let d = Declarator::Type(solved);
                                        acc.solver.add_decl(d.clone(), t.clone());
                                        members.push((v, d, t.clone()));
                                    }
                                    {
                                        let r = mm!();
                                        let r = acc.solver.intern(RefsEnum::This(r));
                                        let solved = acc.solver.try_solve_node_with(*d, r).unwrap();
                                        let d = Declarator::Type(solved);
                                        acc.solver.add_decl(d.clone(), t.clone());
                                        members.push((v, d, t));
                                    }
                                }
                                x => {
                                    log::error!("type declaration should handle the following declaration {:?}", x)
                                }
                            }
                        }
                        // println!("members added");
                        // let (mut cache, solver) = acc.solver.resolve();
                        // acc.solver = solver;
                        // println!("class declaration solved");
                    } else {
                        panic!()
                    }
                    acc.current_node.take()
                }
                y => missing_rule!("{:?} {:?} {:?}", kind, &acc.current_node, y),
            }
        } else if kind.is_type_body() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            macro_rules! syncd {
                ( $i:expr ) => {{
                    let r = remapper.intern_external(&mut acc.solver, $i.0);
                    assert!(!acc.solver.has_choice(r),"{:?}",acc.solver.nodes.with(r));
                    r
                }};
            }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (
                    rest,
                    State::TypeDeclaration {
                        visibility,
                        identifier: d,
                        members,
                    },
                ) if kind == &Type::ClassBody
                    || kind == &Type::InterfaceBody
                    || kind == &Type::EnumBody
                    || kind == &Type::AnnotationTypeBody
                    || kind == &Type::EnumBodyDeclarations =>
                {
                    // TODO also solve members ?
                    // TODO visibility
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    match d {
                        DeclType::Runtime(_) => panic!(),
                        DeclType::Compile(t, sup, int) => {
                            let t = sync!(t);
                            let sup = sup.iter().map(|x| sync!(x)).collect();
                            let int = int.iter().map(|x| sync!(x)).collect();
                            let d = Declarator::Type(t);
                            let t = DeclType::Compile(t, sup, int);
                            acc.solver.add_decl(d.clone(), t.clone());
                            v.push((visibility, d, t));
                        }
                    };
                    for (visibility, d, t) in members {
                        let t = t.map(|x| sync!(x));
                        match d {
                            Declarator::None => panic!(),
                            Declarator::Package(_) => panic!(),
                            Declarator::Type(d) => {
                                let d = sync!(d);
                                assert!(!acc.solver.has_choice(d),"{:?}",acc.solver.nodes.with(d));
                                let d = Declarator::Type(d);
                                acc.solver.add_decl(d.clone(), t.clone());
                                v.push((visibility, d, t));}
                            Declarator::Field(d) => {
                                let d = sync!(d);
                                assert!(!acc.solver.has_choice(d),"{:?}",acc.solver.nodes.with(d));
                                let d = Declarator::Field(d);
                                acc.solver.add_decl(d.clone(), t.clone());
                                v.push((visibility, d, t));
                            }
                            Declarator::Executable(d) => {
                                let d = sync!(d);
                                assert!(!acc.solver.has_choice(d),"{:?}",acc.solver.nodes.with(d));
                                let d = Declarator::Executable(d);
                                acc.solver.add_decl(d.clone(), t.clone());
                                v.push((visibility, d, t));
                            }
                            Declarator::Variable(_) => panic!(),
                        };
                    }
                    State::Declarations(v)
                }
                (rest, State::None) if kind == &Type::ClassBody || kind == &Type::EnumBodyDeclarations => {
                    match &rest {
                        State::Declarations(_) => (),
                        State::None => (),
                        _ => panic!(),
                    }
                    rest
                }
                (
                    rest,
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: d,
                    },
                ) if kind == &Type::ClassBody
                    || kind == &Type::InterfaceBody
                    || kind == &Type::AnnotationTypeBody
                    || kind == &Type::EnumBodyDeclarations =>
                {
                    let t = t.map(|x| sync!(x));
                    let d = d.with_changed_node(|i| syncd!(*i));
                    match &d {
                        Declarator::Type(_) => (),
                        Declarator::Field(_) => (),
                        Declarator::Executable(_) => (),
                        _ => panic!(),
                    };
                    acc.solver.add_decl(d.clone(), t.clone());
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    v.push((visibility, d, t));
                    // TODO make a member declaration and make use of visibilty
                    State::Declarations(v)
                }
                (rest, State::Declarations(u))
                    if kind == &Type::ClassBody
                        || kind == &Type::InterfaceBody
                        || kind == &Type::AnnotationTypeBody
                        || kind == &Type::EnumBodyDeclarations =>
                {
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    for (visibility, d, t) in u {
                        let t = t.map(|x| sync!(x));
                        let d = d.with_changed_node(|i| syncd!(*i));
                        match &d {
                            Declarator::Type(_) => (),
                            Declarator::Field(_) => (),
                            Declarator::Executable(_) => (),
                            _ => panic!(),
                        };
                        acc.solver.add_decl(d.clone(), t.clone());
                        v.push((visibility, d, t));
                    }
                    State::Declarations(v)
                }
                (rest, State::Declarations(u)) if kind == &Type::EnumBody => {
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    for (visibility, d, t) in u {
                        let t = t.map(|x| sync!(x));
                        let d = d.with_changed_node(|i| syncd!(*i));

                        match &d {
                            Declarator::Type(_) => (),
                            Declarator::Field(_) => (),
                            Declarator::Executable(_) => (),
                            _ => panic!(),
                        };
                        acc.solver.add_decl(d.clone(), t.clone());
                        v.push((visibility, d, t));
                    }
                    State::Declarations(v)
                }
                (rest, State::None) if kind == &Type::EnumBody => {
                    let v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    State::Declarations(v)
                }
                (
                    rest,
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: d,
                        parameters: p,
                    },
                ) if kind == &Type::ClassBody
                    || kind == &Type::InterfaceBody
                    || kind == &Type::EnumBodyDeclarations =>
                {
                    let t = t.unwrap().map(|x| sync!(x));
                    let r = mm!();
                    let p = p
                        .into_iter()
                        .map(|(_, t)| {
                            let t = t.map(|x| sync!(x));
                            // TODO should construct RefsEnum::Or
                            match t {
                                DeclType::Runtime(v) => v[0],
                                DeclType::Compile(t, _, _) => t,
                            }
                        })
                        .collect();
                    let d =
                        acc.solver
                            .intern(RefsEnum::Invocation(r, d.unwrap(), Arguments::Given(p)));
                    let d = Declarator::Executable(d);
                    acc.solver.add_decl(d.clone(), t.clone());
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    v.push((visibility, d, t));
                    // TODO make a member declaration and make use of visibilty
                    State::Declarations(v)
                }
                (
                    rest,
                    State::ConstructorImplementation {
                        visibility,
                        identifier: i,
                        parameters: p,
                    },
                ) if kind == &Type::ClassBody || kind == &Type::EnumBodyDeclarations => {
                    let mut v = match rest {
                        State::Declarations(u) => u,
                        State::None => vec![],
                        _ => panic!(),
                    };
                    let p = p
                        .into_iter()
                        .map(|(_, t)| {
                            let t = t.map(|x| sync!(x));
                            match t {
                                DeclType::Runtime(v) => v[0],
                                DeclType::Compile(t, _, _) => t,
                            }
                        })
                        .collect();
                    let t = i.unwrap();
                    let r = mm!();
                    let t = acc.solver.intern(RefsEnum::ScopedIdentifier(r, t));
                    let t = DeclType::Runtime(vec![t].into());
                    let i = acc.solver.intern(RefsEnum::MaybeMissing);
                    let i = acc.solver.intern(RefsEnum::This(i));
                    let i = acc
                        .solver
                        .intern(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                    let d = Declarator::Executable(i);
                    // TODO constructor solve
                    acc.solver.add_decl(d.clone(), t.clone());
                    v.push((visibility, d, t));

                    // TODO make a member declaration and make use of visibilty
                    State::Declarations(v)
                }
                (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind.is_value_member() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            // if kind == &Type::FieldDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::ConstantDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::EnumConstant {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::AnnotationTypeElementDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else {
            //     panic!("{:?}",kind)
            // }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (State::SimpleTypeIdentifier(t), State::Declarator(Declarator::Variable(i)))
                    if kind == &Type::FieldDeclaration =>
                {
                    let t = scoped_type!(mm!(), t);
                    let Old(i) = i;
                    match self.solver.nodes[i] {
                        RefsEnum::Array(i) => {
                            let i = sync!(Old(i));
                            assert!(!acc.solver.has_choice(i));
                            let i = Declarator::Field(i);
                            let t = acc.solver.intern(RefsEnum::Array(t));
                            let t = DeclType::Runtime(vec![t].into());

                            let v = vec![(Visibility::None, i, t)];
                            State::Declarations(v)
                        }
                        _ => {
                            let i = sync!(Old(i));
                            assert!(!acc.solver.has_choice(i));
                            let i = Declarator::Field(i);
                            let t = DeclType::Runtime(vec![t].into());

                            let v = vec![(Visibility::None, i, t)];
                            State::Declarations(v)
                        }
                    }
                }
                (State::Modifiers(v, n), State::SimpleTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = scoped_type!(mm!(), t);
                    let t = DeclType::Runtime(vec![t].into());
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: v,
                        kind: t,
                        identifier: i,
                    }
                }
                (State::Modifiers(v, n), State::ScopedTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = sync!(t);
                    let t = DeclType::Runtime(vec![t].into());
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: v,
                        kind: t,
                        identifier: i,
                    }
                }
                (State::None, State::SimpleTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = scoped_type!(mm!(), t);
                    let t = DeclType::Runtime(vec![t].into());
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: Visibility::None,
                        kind: t,
                        identifier: i,
                    }
                }
                (State::None, State::ScopedTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = sync!(t);
                    let t = DeclType::Runtime(vec![t].into());
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: Visibility::None,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: _,
                    },
                    // State::Declarations(v),
                    State::Declarator(Declarator::Variable(i)),
                ) if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration => {
                    let Old(i) = i;
                    match self.solver.nodes[i] {
                        RefsEnum::Array(i) => {
                            let i = sync!(Old(i));
                            assert!(!acc.solver.has_choice(i),"{:?}",acc.solver.nodes.with(i));
                            let i = Declarator::Field(i);
                            let t = match t {
                                DeclType::Runtime(v) => DeclType::Runtime(
                                    v.iter()
                                        .map(|t| acc.solver.intern_ref(RefsEnum::Array(*t)))
                                        .collect(),
                                ),
                                DeclType::Compile(_, _, _) => todo!(),
                            };
                            // State::Declaration {a
                            //     visibility,
                            //     kind: t,
                            //     identifier: i,
                            // }

                            let v = vec![(visibility, i, t)];
                            State::Declarations(v)
                        }
                        _ => {
                            let i = sync!(Old(i));
                            assert!(!acc.solver.has_choice(i),"{:?}",acc.solver.nodes.with(i));
                            let i = Declarator::Field(i);
                            // State::Declaration {
                            //     visibility,
                            //     kind: t,
                            //     identifier: i,
                            // }

                            let v = vec![(visibility, i, t)];
                            // let Old(i) = i;
                            // match self.solver.nodes[i] {
                            //     RefsEnum::Array(i) => {
                            //         let t = acc.solver.intern_ref(RefsEnum::Array(t));
                            //     }
                            //     _ => {}
                            // };
                            State::Declarations(v)
                        }
                    }
                }
                // not yet implemented: FieldDeclaration Declaration { visibility: None, kind: Runtime([2]), identifier: None } None 
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: Declarator::None,
                    },
                    State::None,
                ) if kind == &Type::FieldDeclaration => {
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: Declarator::None,
                    }
                }
                (State::Declarations(mut v), State::Declarator(Declarator::Variable(i)))
                    if kind == &Type::FieldDeclaration || kind == &Type::ConstantDeclaration =>
                {
                    let (visibility, _, t) = &v[0];
                    let visibility = *visibility;
                    let t = t.clone();
                    let Old(i) = i;
                    match self.solver.nodes[i] {
                        RefsEnum::Array(i) => {
                            let i = sync!(Old(i));

                            assert!(!acc.solver.has_choice(i),"{:?}",acc.solver.nodes.with(i));
                            let i = Declarator::Field(i);
                            let t = match t {
                                DeclType::Runtime(v) => DeclType::Runtime(
                                    v.iter()
                                        .map(|t| acc.solver.intern_ref(RefsEnum::Array(*t)))
                                        .collect(),
                                ),
                                DeclType::Compile(_, _, _) => todo!(),
                            };

                            v.push((visibility, i, t));
                            State::Declarations(v)
                        }
                        _ => {
                            let i = sync!(Old(i));


                            assert!(!acc.solver.has_choice(i),"{:?}",acc.solver.nodes.with(i));
                            let i = Declarator::Field(i);

                            v.push((visibility, i, t));
                            State::Declarations(v)
                        }
                    }
                }
                (State::Declarations(v), State::None)
                    if kind == &Type::ConstantDeclaration =>
                {
                    // TODO check if right is ok to be none
                    // reproduce ConstantDeclaration Declarations([(None, Field(3), Runtime([2]))]) None'
                    // with ["target/release/hyper_ast_benchmark", "apache/dubbo", "", "e831b464837ae5d2afac9841559420aeaef6c52b", "", "results_1000_commits/dubbo"]
                    State::Declarations(v)
                }
                (State::Modifiers(v, n), State::ScopedTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = sync!(t);
                    let t = DeclType::Runtime(vec![t].into());
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: v,
                        kind: t,
                        identifier: i,
                    }
                }
                (State::None, State::Modifiers(v, n))
                    if kind == &Type::FieldDeclaration
                        || kind == &Type::ConstantDeclaration
                        || kind == &Type::AnnotationTypeElementDeclaration =>
                {
                    State::Modifiers(v, n)
                }
                (State::None, State::SimpleTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration
                        || kind == &Type::AnnotationTypeElementDeclaration =>
                {
                    let t = scoped_type!(mm!(), t);
                    let t = DeclType::Runtime(vec![t].into());
                    State::Declaration {
                        visibility: Visibility::None,
                        kind: t,
                        identifier: Declarator::None,
                    }
                }
                (State::None, State::ScopedTypeIdentifier(t))
                    if kind == &Type::FieldDeclaration
                        || kind == &Type::AnnotationTypeElementDeclaration =>
                {
                    let t = sync!(t);
                    let t = DeclType::Runtime(vec![t].into());
                    State::Declaration {
                        visibility: Visibility::None,
                        kind: t,
                        identifier: Declarator::None,
                    }
                }
                (State::Modifiers(v, n), State::ScopedTypeIdentifier(t))
                    if kind == &Type::AnnotationTypeElementDeclaration =>
                {
                    let t = sync!(t);
                    let t = DeclType::Runtime(vec![t].into());
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: v,
                        kind: t,
                        identifier: i,
                    }
                }
                (State::Modifiers(v, n), State::SimpleTypeIdentifier(t))
                    if kind == &Type::AnnotationTypeElementDeclaration =>
                {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let t = scoped_type!(mm!(), t);
                    let t = DeclType::Runtime(vec![t].into());
                    let i = Declarator::None;
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility: v,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: _,
                    },
                    State::SimpleIdentifier(_, i),
                ) if kind == &Type::AnnotationTypeElementDeclaration => {
                    // TODO simple type identifier should be a type identifier ie. already scoped
                    let r = mm!();
                    let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                    let i = Declarator::Field(i);
                    // not used directly
                    // acc.solver.add_decl_simple(i.clone(), t);
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    },
                    State::LiteralType(_),
                ) if kind == &Type::AnnotationTypeElementDeclaration && i != Declarator::None => {
                    // TODO do something with default value
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    },
                    State::ScopedIdentifier(_),
                ) if kind == &Type::AnnotationTypeElementDeclaration => {
                    // TODO do something with default value
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    },
                    State::None,
                ) if kind == &Type::AnnotationTypeElementDeclaration && i != Declarator::None => {
                    // TODO do something with default value
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    }
                }
                // not yet implemented: AnnotationTypeElementDeclaration Declaration { visibility: None, kind: Runtime([2]), identifier: Field(4) } Dimensions 
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    },
                    State::Dimensions,
                ) if kind == &Type::AnnotationTypeElementDeclaration => {
                    let t = match t {
                        DeclType::Runtime(v) => DeclType::Runtime(
                            v.iter()
                                .map(|t| acc.solver.intern_ref(RefsEnum::Array(*t)))
                                .collect(),
                        ),
                        DeclType::Compile(_, _, _) => todo!(),
                    };
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    },
                    State::Annotation,
                ) if kind == &Type::AnnotationTypeElementDeclaration && i != Declarator::None => {
                    // TODO do something with default value
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: i,
                    }
                }
                (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind.is_executable_member() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            // if kind == &Type::MethodDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::ConstructorDeclaration {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else {
            //     panic!("{:?}",kind)
            // }
            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (State::None, State::SimpleTypeIdentifier(t))
                    if kind == &Type::MethodDeclaration =>
                {
                    let t = scoped_type!(mm!(), t);
                    State::ScopedTypeIdentifier(t)
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: _,
                        identifier: _,
                        parameters: _,
                    },
                    State::SimpleTypeIdentifier(t),
                ) if kind == &Type::MethodDeclaration => {
                    let t = scoped_type!(mm!(), t);
                    let t = DeclType::Runtime(vec![t].into());
                    State::MethodImplementation {
                        visibility,
                        kind: Some(t),
                        identifier: None,
                        parameters: vec![].into_boxed_slice(),
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: _,
                        identifier: _,
                        parameters: _,
                    },
                    State::ScopedTypeIdentifier(t),
                ) if kind == &Type::MethodDeclaration => {
                    let t = sync!(t);
                    let t = DeclType::Runtime(vec![t].into());
                    State::MethodImplementation {
                        visibility,
                        kind: Some(t),
                        identifier: None,
                        parameters: vec![].into_boxed_slice(),
                    }
                }
                (State::None, State::ScopedTypeIdentifier(t))
                    if kind == &Type::MethodDeclaration =>
                {
                    let t = sync!(t);
                    State::ScopedTypeIdentifier(t)
                }
                (State::ScopedTypeIdentifier(t), State::SimpleIdentifier(_, i))
                    if kind == &Type::MethodDeclaration =>
                {
                    let t = DeclType::Runtime(vec![t].into());
                    State::MethodImplementation {
                        visibility: Visibility::None,
                        kind: Some(t),
                        identifier: Some(i),
                        parameters: vec![].into_boxed_slice(),
                    }
                }
                (State::None, State::SimpleIdentifier(_, i))
                    if kind == &Type::ConstructorDeclaration =>
                {
                    State::ConstructorImplementation {
                        visibility: Visibility::None,
                        identifier: Some(i),
                        parameters: vec![].into_boxed_slice(),
                    }
                }
                (State::None, State::Modifiers(v, n))
                    if kind == &Type::MethodDeclaration
                        || kind == &Type::ConstructorDeclaration =>
                {
                    State::Modifiers(v, n)
                }
                (State::Modifiers(v, n), State::SimpleTypeIdentifier(t))
                    if kind == &Type::MethodDeclaration =>
                {
                    let t = scoped_type!(mm!(), t);
                    let t = DeclType::Runtime(vec![t].into());
                    State::MethodImplementation {
                        visibility: v,
                        kind: Some(t),
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (State::Modifiers(v, n), State::ScopedTypeIdentifier(t))
                    if kind == &Type::MethodDeclaration =>
                {
                    let t = sync!(t);
                    let t = DeclType::Runtime(vec![t].into());
                    State::MethodImplementation {
                        visibility: v,
                        kind: Some(t),
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (
                    State::MethodImplementation {
                        visibility: v,
                        kind: _,
                        identifier: _,
                        parameters: _,
                    },
                    State::SimpleTypeIdentifier(t),
                ) if kind == &Type::MethodDeclaration => {
                    let t = scoped_type!(mm!(), t);
                    let t = DeclType::Runtime(vec![t].into());
                    State::MethodImplementation {
                        visibility: v,
                        kind: Some(t),
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (State::None, State::TypeParameters(t)) if kind == &Type::MethodDeclaration => {
                    for t in t {
                        if let DeclType::Compile(d,ext,imp) = &t {
                            let d = Declarator::Type(sync!(*d));
                            let mut v:Vec<_> = ext.iter().map(|t| sync!(*t)).collect();
                            v.extend(imp.iter().map(|t| sync!(*t)));
                            acc.solver.add_decl(d.clone(), DeclType::Runtime(v.into()));
                        }
                    }

                    State::MethodImplementation {
                        visibility: Visibility::None,
                        kind: None,
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (State::None, State::TypeParameters(t))
                    if kind == &Type::ConstructorDeclaration =>
                {
                    for t in t {
                        if let DeclType::Compile(d,ext,imp) = &t {
                            let d = Declarator::Type(sync!(*d));
                            let mut v:Vec<_> = ext.iter().map(|t| sync!(*t)).collect();
                            v.extend(imp.iter().map(|t| sync!(*t)));
                            acc.solver.add_decl(d.clone(), DeclType::Runtime(v.into()));
                        }
                    }

                    State::ConstructorImplementation {
                        visibility: Visibility::None,
                        identifier: None,
                        parameters: Default::default(),
                    }
                }
                (State::Modifiers(v, n), State::TypeParameters(t))
                    if kind == &Type::MethodDeclaration
                        || kind == &Type::ConstructorDeclaration =>
                {
                    for t in t {
                        let d = if let DeclType::Compile(d,_,_) = &t {
                            let d = Declarator::Type(sync!(*d));
                            // let v = ext.iter().map(|t| t).chain(
                            //     imp.iter().map(|t| t)
                            // ).map(|t|sync!(*t)).collect();
                            // acc.solver.add_decl(d.clone(), DeclType::Runtime(v));
                            d
                        } else { panic!() };
                        let t =t.map(|t|sync!(*t));
                        acc.solver.add_decl(d.clone(), t);
                    }

                    if kind == &Type::MethodDeclaration {
                        State::MethodImplementation {
                            visibility: v,
                            kind: None,
                            identifier: None,
                            parameters: Default::default(),
                        }
                    } else {
                        State::ConstructorImplementation {
                            visibility: v,
                            identifier: None,
                            parameters: Default::default(),
                        }
                    }
                }
                (
                    State::ConstructorImplementation {
                        visibility,
                        identifier: None,
                        parameters,
                    },
                    State::SimpleIdentifier(_, i),
                ) if kind == &Type::ConstructorDeclaration => State::ConstructorImplementation {
                    visibility,
                    identifier: Some(i),
                    parameters,
                },
                (State::Modifiers(v, n), State::SimpleIdentifier(_, i))
                    if kind == &Type::ConstructorDeclaration =>
                {
                    State::ConstructorImplementation {
                        visibility: v,
                        identifier: Some(i),
                        parameters: Default::default(),
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i,
                        parameters: _,
                    },
                    State::FormalParameters(p),
                ) if kind == &Type::MethodDeclaration => {
                    let p = p
                        .into_iter()
                        .map(|(i, t)| {
                            let i = sync!(i);
                            let t = t.map(|x| sync!(x));
                            acc.solver.add_decl(Declarator::Variable(i), t.clone()); // TODO use variable or parameter ?
                            (i, t)
                        })
                        .collect();
                    // let r = mm!();
                    // let i = acc
                    //     .solver
                    //     .intern(RefsEnum::Invocation(r, i, Arguments::Given(p)));
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i,
                        parameters: p,
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i0,
                        parameters: p,
                    },
                    State::SimpleIdentifier(_, i),
                ) if kind == &Type::MethodDeclaration => {
                    assert_eq!(i0, None);
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: Some(i),
                        parameters: p,
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier,
                        parameters: p,
                    },
                    State::Dimensions,
                ) if kind == &Type::MethodDeclaration => {
                    let t = t.map(|t| {
                        let t = match t {
                            DeclType::Runtime(v) => DeclType::Runtime(
                                v.iter()
                                    .map(|t| acc.solver.intern_ref(RefsEnum::Array(*t)))
                                    .collect(),
                            ),
                            DeclType::Compile(_, _, _) => todo!(),
                        };
                        t
                    });
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier,
                        parameters: p,
                    }
                }
                (x, State::Throws)
                    if kind == &Type::MethodDeclaration
                        || kind == &Type::ConstructorDeclaration =>
                {
                    if let State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i,
                        parameters: p,
                    } = x
                    {
                        assert_eq!(&Type::MethodDeclaration, kind);
                        State::MethodImplementation {
                            visibility,
                            kind: t,
                            identifier: i,
                            parameters: p,
                        }
                    } else if let State::ConstructorImplementation {
                        visibility,
                        identifier: i,
                        parameters: p,
                    } = x
                    {
                        assert_eq!(&Type::ConstructorDeclaration, kind);
                        State::ConstructorImplementation {
                            visibility,
                            identifier: i,
                            parameters: p,
                        }
                    } else {
                        missing_rule!("{:?} {:?} Throws", kind, x)
                    }
                }
                (
                    State::ConstructorImplementation {
                        visibility,
                        identifier: i,
                        parameters: _,
                    },
                    State::FormalParameters(p),
                ) if kind == &Type::ConstructorDeclaration => {
                    let p = p
                        .into_iter()
                        .map(|(i, t)| {
                            let i = sync!(i);
                            let t = t.map(|x| sync!(x));
                            acc.solver.add_decl(Declarator::Variable(i), t.clone()); // TODO use variable or parameter ?
                            (i, t)
                        })
                        .collect();
                    State::ConstructorImplementation {
                        visibility,
                        identifier: i,
                        parameters: p,
                    }
                }
                (
                    State::MethodImplementation {
                        visibility,
                        kind: t,
                        identifier: i,
                        parameters: p,
                    },
                    State::None,
                ) if kind == &Type::MethodDeclaration => {
                    if let (Some(t),Some(i))= (t.clone(),i) {
                        let r = mm!();
                        let p: Box<[_]> = p
                            .into_iter()
                            .map(|(i, t)| {
                                // TODO should transform to RefEnum::Or
                                match t {
                                    DeclType::Runtime(v) => v[0],
                                    DeclType::Compile(t, _, _) => *t,
                                }
                            })
                            .collect();
                        let i =
                            acc.solver
                                .intern(RefsEnum::Invocation(r, i, Arguments::Given(p)));
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: Declarator::Executable(i),
                        }
                    } else {
                        State::MethodImplementation {
                            visibility,
                            kind: t,
                            identifier: i,
                            parameters: p,
                        }
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: Declarator::Executable(i),
                    },
                    State::FormalParameters(p),
                ) if kind == &Type::MethodDeclaration => {
                    let p: Box<[_]> = p
                        .into_iter()
                        .map(|(i, t)| {
                            // TODO should transform to RefEnum::Or
                            match t {
                                DeclType::Runtime(v) => sync!(v[0]),
                                DeclType::Compile(t, _, _) => sync!(t),
                            }
                        })
                        .collect();
                    let (r,i) = if let RefsEnum::Invocation(r,i,_) = acc.solver.nodes[i] {
                        (r,i)
                    } else {panic!()};
                    
                    let i = acc.solver
                            .intern(RefsEnum::Invocation(r, i, Arguments::Given(p)));
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier: Declarator::Executable(i),
                    }
                }
                (
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier,
                    },
                    State::None,
                ) if kind == &Type::MethodDeclaration => {
                    State::Declaration {
                        visibility,
                        kind: t,
                        identifier,
                    }
                }
                (
                    State::ConstructorImplementation {
                        visibility,
                        identifier,
                        parameters,
                    },
                    State::None,
                ) if kind == &Type::ConstructorDeclaration => {
                    let t = identifier.unwrap();
                    let t = scoped_type!(mm!(), t);
                    let p: Box<[RefPtr]> = parameters
                        .into_iter()
                        .map(|(_, t)| {
                            // TODO should transform to RefEnum::Or
                            match t {
                                DeclType::Runtime(v) => v[0],
                                DeclType::Compile(t, _, _) => *t,
                            }
                        })
                        .collect();
                    {
                        let p = p.clone();
                        let i = acc.solver.intern(RefsEnum::MaybeMissing);
                        let i = acc.solver.intern(RefsEnum::This(i));
                        let i = acc
                            .solver
                            .intern(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                        let d = Declarator::Executable(i);
                        acc.solver.add_decl(d, DeclType::Runtime(vec![t].into()));
                    }
                    {
                        let i = acc
                            .solver
                            .intern(RefsEnum::ConstructorInvocation(t, Arguments::Given(p)));
                        let d = Declarator::Executable(i);
                        acc.solver.add_decl(d, DeclType::Runtime(vec![t].into()));
                    }
                    State::ConstructorImplementation {
                        visibility,
                        identifier,
                        parameters,
                    }
                }
                (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            }
        } else if kind.is_statement() {
            if kind.is_declarative_statement() {
                let mut remapper = acc.solver.local_solve_extend(&self.solver);
                macro_rules! sync {
                    ( $i:expr ) => {
                        remapper.intern_external(&mut acc.solver, $i.0)
                    };
                }
                macro_rules! syncd {
                    ( $i:expr ) => {{
                        let r = remapper.intern_external(&mut acc.solver, $i.0);
                        assert!(!acc.solver.has_choice(r),"{:?}",acc.solver.nodes.with(r));
                        r
                    }};
                }
                if kind == &Type::LocalVariableDeclaration {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (State::None, State::Modifiers(v, n)) => State::Modifiers(v, n),
                        (State::None, State::ScopedTypeIdentifier(t)) => {
                            let t = sync!(t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (
                            State::ScopedTypeIdentifier(t),
                            State::Declarator(Declarator::Variable(i)),
                        ) => {
                            let t = DeclType::Runtime(vec![t].into());
                            let v =
                                vec![(Visibility::None, Declarator::Variable(sync!(i)), t.clone())];
                            let Old(i) = i;
                            match self.solver.nodes[i] {
                                RefsEnum::Array(_) => {
                                    let i = sync!(Old(i));
                                    match t {
                                        DeclType::Runtime(v) => {
                                            for t in v.iter() {
                                                acc.solver.intern_ref(RefsEnum::Array(*t));
                                            }
                                        }
                                        DeclType::Compile(_, _, _) => todo!(),
                                    }
                                }
                                _ => {}
                            };
                            State::Declarations(v)
                        }
                        (
                            State::Declarations(mut v),
                            State::Declarator(Declarator::Variable(i)),
                        ) => {
                            let x = {
                                let (visibility, _, t) = &v[0];
                                {
                                    let Old(i) = i;
                                    match self.solver.nodes[i] {
                                        RefsEnum::Array(_) => {
                                            let i = sync!(Old(i));
                                            match t {
                                                DeclType::Runtime(v) => {
                                                    for t in v.iter() {
                                                        acc.solver.intern_ref(RefsEnum::Array(*t));
                                                    }
                                                }
                                                DeclType::Compile(_, _, _) => todo!(),
                                            }
                                        }
                                        _ => {}
                                    };
                                }
                                (*visibility, Declarator::Variable(sync!(i)), t.clone())
                            };
                            v.push(x);
                            State::Declarations(v)
                        }
                        (State::None, State::SimpleTypeIdentifier(t)) => {
                            let t = scoped_type!(mm!(), t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (State::Modifiers(v, n), State::SimpleTypeIdentifier(t)) => {
                            let t = scoped_type!(mm!(), t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (State::Modifiers(v, n), State::ScopedTypeIdentifier(t)) => {
                            let t = sync!(t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (State::ScopedTypeIdentifier(t), State::None) => {
                            State::ScopedTypeIdentifier(t)
                        }
                        (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                    }
                // } else if kind == &Type::TryWithResourcesStatement {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                //     }
                // } else if kind == &Type::CatchClause {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                //     }
                // } else if kind == &Type::ForStatement {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                //     }
                // } else if kind == &Type::EnhancedForStatement {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                //     }
                // } else if kind == &Type::Scope {
                //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
                //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                //     }
                } else {
                    // panic!("{:?}",kind)
                    // }
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (State::None, State::None) if kind == &Type::TryWithResourcesStatement => {
                            State::None
                        }
                        (State::None, State::Modifiers(Visibility::None, n))
                            if kind == &Type::EnhancedForStatement && n.eq(&enum_set!()) =>
                        {
                            State::None
                        }
                        (State::None, State::FormalParameters(p))
                            if kind == &Type::TryWithResourcesStatement =>
                        {
                            // TODO it implicitly calls close on resource so need to materialize it
                            p.into_iter().for_each(|(i, t)| {
                                let i = sync!(i);
                                let t = t.map(|x| sync!(x));
                                acc.solver.add_decl(Declarator::Variable(i), t);
                                // TODO use variable or parameter ?
                            });
                            State::None
                        }
                        (State::None, State::None) if kind == &Type::CatchClause => State::None,
                        (
                            State::None,
                            State::CatchParameter {
                                kinds: b,
                                identifier: i,
                            },
                        ) if kind == &Type::CatchClause => {
                            let r = mm!();
                            let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                            let d = Declarator::Variable(i);
                            // let b = b.into_iter().map(|t|
                            //     sync!(*t)
                            // ).collect();
                            let b = b.iter().map(|x| sync!(x)).collect();
                            acc.solver.add_decl(d.clone(), DeclType::Runtime(b));
                            State::None
                        }
                        (State::None, State::SimpleTypeIdentifier(t))
                            if kind == &Type::EnhancedForStatement =>
                        {
                            let t = scoped_type!(mm!(), t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (State::None, State::ScopedTypeIdentifier(t))
                            if kind == &Type::EnhancedForStatement =>
                        {
                            let t = sync!(t);
                            State::ScopedTypeIdentifier(t)
                        }
                        (
                            State::None,
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                        ) if kind == &Type::EnhancedForStatement => {
                            let t = t.map(|x| sync!(x));
                            let d = d.with_changed_node(|i| syncd!(*i));
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }
                        (State::ScopedTypeIdentifier(t), State::SimpleIdentifier(_, i))
                            if kind == &Type::EnhancedForStatement =>
                        {
                            let t = DeclType::Runtime(vec![t].into());
                            let r = mm!();
                            let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                            let i = Declarator::Variable(i);
                            acc.solver.add_decl(i.clone(), t.clone());
                            // TODO also make a special state for variable declarations
                            State::Declaration {
                                visibility: Visibility::None,
                                kind: t,
                                identifier: i,
                            }
                        }

                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::SimpleIdentifier(_, i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let r = mm!();
                            // let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                            scoped_ref!(r, i);
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::This(i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let i = sync!(i);
                            let i = acc.solver.intern_ref(RefsEnum::This(i));
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::FieldIdentifier(i),
                        ) if kind == &Type::EnhancedForStatement => State::Declaration {
                            visibility,
                            kind: t,
                            identifier: d,
                        },
                        (
                            State::Declaration {
                                visibility: _,
                                kind: _,
                                identifier: _,
                            },
                            State::None,
                        ) if kind == &Type::EnhancedForStatement => State::None,
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::ScopedIdentifier(i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let i = sync!(i); // Not necessary
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::Invocation(i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let i = sync!(i); // Not necessary
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }
                        (
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                            State::ConstructorInvocation(i),
                        ) if kind == &Type::EnhancedForStatement => {
                            let i = sync!(i); // Not necessary
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            }
                        }

                        (rest, State::Declarations(v)) if kind == &Type::ForStatement => {
                            for (_, d, t) in v {
                                let t = t.map(|x| sync!(x));
                                let d = d.with_changed_node(|i| syncd!(*i));
                                match rest {
                                    State::None => (),
                                    _ => panic!(),
                                }
                                match &d {
                                    Declarator::Variable(_) => acc.solver.add_decl(d, t),
                                    _ => todo!(),
                                };
                            }
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }

                        // (
                        //     State::None,
                        //     State::Declaration {
                        //         visibility,
                        //         kind: t,
                        //         identifier: d,
                        //     },
                        // ) if kind == &Type::ForStatement => {
                        //     let t = sync!(t);
                        //     let d = d.with_changed_node(|i| sync!(*i));
                        //     State::Declaration {a
                        //         visibility,
                        //         kind: t,
                        //         identifier: d,
                        //     }
                        // }
                        (State::None, State::SimpleIdentifier(_, i))
                            if kind == &Type::ForStatement || kind == &Type::DoStatement =>
                        {
                            scoped_ref!(mm!(), i);
                            State::None
                        }
                        (State::None, _)
                            if kind == &Type::ForStatement || kind == &Type::DoStatement =>
                        {
                            State::None
                        }
                        (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                    }
                }
            } else if kind.is_block_related() {
                let mut remapper = acc.solver.local_solve_extend(&self.solver);
                macro_rules! sync {
                    ( $i:expr ) => {
                        remapper.intern_external(&mut acc.solver, $i.0)
                    };
                }
                macro_rules! syncd {
                    ( $i:expr ) => {{
                        let r = remapper.intern_external(&mut acc.solver, $i.0);
                        assert!(!acc.solver.has_choice(r),"{:?}",acc.solver.nodes.with(r));
                        r
                    }};
                }
                // maybe fusion with structural statement
                if kind == &Type::StaticInitializer {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (State::None, State::Modifiers(_, _)) => State::None,
                        (State::None, State::None) => State::None,
                        (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else if kind == &Type::ConstructorBody {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (State::None, State::None) if kind == &Type::ConstructorBody => State::None,
                        (rest, State::ConstructorInvocation(i))
                            if kind == &Type::ConstructorBody =>
                        {
                            let i = sync!(i);
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            State::None
                        }
                        (
                            rest,
                            State::Declaration {
                                visibility,
                                kind: t,
                                identifier: d,
                            },
                        ) if kind == &Type::ConstructorBody => {
                            let t = t.map(|x| sync!(x));
                            let d = d.with_changed_node(|i| syncd!(*i));
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            match &d {
                                Declarator::Variable(_) => acc.solver.add_decl(d, t),
                                _ => todo!(),
                            };
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            // TODO declarations needed in ConstructorDeclaration
                            State::None
                        }
                        (rest, State::Declarations(v)) => {
                            for (_, d, t) in v {
                                let t = t.map(|x| sync!(x));
                                let d = d.with_changed_node(|i| syncd!(*i));
                                match rest {
                                    State::None => (),
                                    _ => panic!(),
                                }
                                match &d {
                                    Declarator::Variable(_) => acc.solver.add_decl(d, t),
                                    _ => todo!(),
                                };
                            }
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }
                        (
                            rest,
                            State::TypeDeclaration {
                                visibility,
                                identifier: d,
                                members: _,
                            },
                        ) => {
                            match d {
                                DeclType::Runtime(_) => panic!(),
                                DeclType::Compile(t, sup, int) => {
                                    let t = sync!(t);
                                    let d = Declarator::Type(t);
                                    let sup = sup.iter().map(|x| sync!(x)).collect();
                                    let int = int.iter().map(|x| sync!(x)).collect();
                                    let t = DeclType::Compile(t, sup, int);
                                    acc.solver.add_decl(d, t);
                                }
                            };
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }
                        (State::None, State::None) => State::None,
                        (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else if kind == &Type::Block {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (rest, State::Declarations(v)) => {
                            for (_, d, t) in v {
                                let t = t.map(|x| sync!(x));
                                let d = d.with_changed_node(|i| syncd!(*i));
                                match rest {
                                    State::None => (),
                                    _ => panic!(),
                                }
                                match &d {
                                    Declarator::Variable(i) => match acc.solver.nodes[*i] {
                                        RefsEnum::Array(i) => {
                                            let i = Declarator::Variable(i);
                                            let t = match t {
                                                DeclType::Runtime(v) => DeclType::Runtime(
                                                    v.iter()
                                                        .map(|t| {
                                                            acc.solver
                                                                .intern_ref(RefsEnum::Array(*t))
                                                        })
                                                        .collect(),
                                                ),
                                                DeclType::Compile(_, _, _) => todo!(),
                                            };
                                            acc.solver.add_decl(i, t)
                                        }
                                        _ => {
                                            let i = Declarator::Variable(*i);
                                            acc.solver.add_decl(i, t)
                                        }
                                    },
                                    _ => todo!(),
                                };
                            }
                            // we do not need declarations apart of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }
                        (
                            rest,
                            State::TypeDeclaration {
                                visibility,
                                identifier: d,
                                members: _,
                            },
                        ) => {
                            match d {
                                DeclType::Runtime(_) => panic!(),
                                DeclType::Compile(t, sup, int) => {
                                    let t = sync!(t);
                                    let d = Declarator::Type(t);
                                    let sup = sup.iter().map(|x| sync!(x)).collect();
                                    let int = int.iter().map(|x| sync!(x)).collect();
                                    let t = DeclType::Compile(t, sup, int);
                                    acc.solver.add_decl(d, t);
                                }
                            };
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            // we do not need declarations outside of the map to solve local variable
                            // because a local variable declaration is never visible from outside
                            State::None
                        }
                        (State::None, State::None) => State::None,
                        (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else if kind == &Type::SwitchBlockStatementGroup {
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (rest, State::Declarations(v)) => {
                            for (_, d, t) in v {
                                let t = t.map(|x| sync!(x));
                                let d = d.with_changed_node(|i| syncd!(*i));
                                match rest {
                                    State::None => (),
                                    _ => panic!(),
                                }
                                match &d {
                                    Declarator::Variable(_) => acc.solver.add_decl(d, t),
                                    _ => todo!(),
                                };
                            }
                            State::None
                        }
                        (
                            rest,
                            State::TypeDeclaration {
                                visibility,
                                identifier: d,
                                members: _,
                            },
                        ) => {
                            match d {
                                DeclType::Runtime(_) => panic!(),
                                DeclType::Compile(t, sup, int) => {
                                    let t = sync!(t);
                                    let d = Declarator::Type(t);
                                    let sup = sup.iter().map(|x| sync!(x)).collect();
                                    let int = int.iter().map(|x| sync!(x)).collect();
                                    let t = DeclType::Compile(t, sup, int);
                                    acc.solver.add_decl(d, t);
                                }
                            };
                            match rest {
                                State::None => (),
                                _ => panic!(),
                            }
                            State::None
                        }
                        (State::None, State::None) => State::None,
                        (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else if kind == &Type::SwitchBlock {
                    // TODO retrieve decls not in Block from SwitchBlockStatementGroup
                    match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                        (rest, State::None) => rest, // TODO handle fall through declarations
                        (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                    }
                } else {
                    panic!("{:?}", kind)
                }
            } else if kind.is_structural_statement() {
                let mut remapper = acc.solver.local_solve_extend(&self.solver);
                macro_rules! sync {
                    ( $i:expr ) => {
                        remapper.intern_external(&mut acc.solver, $i.0)
                    };
                }
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::None) => State::None,

                    (State::None, State::LiteralType(t)) if kind == &Type::IfStatement => {
                        let t = sync!(t);
                        State::None
                    }
                    (State::None, State::LiteralType(_))
                        if kind == &Type::IfStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::WhileStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        State::None
                    }
                    (State::None, State::FieldIdentifier(_)) if kind == &Type::IfStatement => {
                        State::None
                    }
                    (State::None, State::ConstructorInvocation(i))
                        if kind == &Type::IfStatement =>
                    {
                        let i = sync!(i);
                        State::None
                    }
                    (State::None, State::ConstructorInvocation(i))
                        if kind == &Type::WhileStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        let i = sync!(i);
                        State::None
                    }
                    (State::None, State::Invocation(i))
                        if kind == &Type::WhileStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        let i = sync!(i);
                        State::None
                    }
                    (State::None, State::This(i))
                        if kind == &Type::WhileStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        State::None
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::IfStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::WhileStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        State::None
                    }
                    (State::None, State::FieldIdentifier(i))
                        if kind == &Type::IfStatement
                            || kind == &Type::DoStatement
                            || kind == &Type::WhileStatement
                            || kind == &Type::SwitchStatement =>
                    {
                        State::None
                    }
                    (State::None, State::Invocation(i)) if kind == &Type::IfStatement => {
                        State::None
                    }
                    (State::None, State::Declarations(_)) if kind == &Type::IfStatement => {
                        State::None
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind.is_simple_statement() {
                let mut remapper = acc.solver.extend(&self.solver);
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, rest) => {
                        match rest {
                            State::None | State::FieldIdentifier(_) |
                            State::ScopedIdentifier(_) |
                            State::MethodReference(_) |
                            State::Invocation(_) |
                            State::ConstructorInvocation(_) |
                            State::LiteralType(_) |
                            State::This(_) |
                            State::LambdaExpression(_) => State::None,
                            State::SimpleIdentifier(_, i) => {
                                if kind == &Type::ExpressionStatement
                                    || kind == &Type::AssertStatement
                                    || kind == &Type::ReturnStatement
                                    || kind == &Type::SynchronizedStatement
                                    || kind == &Type::ThrowStatement
                                {
                                    scoped_ref!(mm!(), i);
                                    State::None
                                } else if kind == &Type::LabeledStatement
                                    || kind == &Type::BreakStatement
                                    || kind == &Type::ContinueStatement
                                {
                                    State::None
                                } else {
                                    missing_rule!("{:?} None State::SimpleIdentifier", kind)
                                }
                            }
                            x => missing_rule!("{:?} None {:?}", kind, x),
                        }
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else {
                missing_rule!("{:?} should be block related", kind)
            }
        } else if kind.is_parameter() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            if kind == &Type::Resource {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleIdentifier(_, t)) if kind == &Type::Resource => {
                        let t = scoped_ref!(mm!(), t);
                        State::ScopedIdentifier(t)
                    }
                    (State::None, State::ScopedIdentifier(t)) if kind == &Type::Resource => {
                        let t = sync!(t);
                        State::ScopedIdentifier(t)
                    }
                    (State::None, State::ScopedTypeIdentifier(t)) if kind == &Type::Resource => {
                        let t = sync!(t);
                        let t = DeclType::Runtime(vec![t].into());
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: t,
                            identifier: Declarator::None,
                        }
                    }
                    (State::None, State::SimpleTypeIdentifier(t)) if kind == &Type::Resource => {
                        let t = scoped_type!(mm!(), t);
                        let t = DeclType::Runtime(vec![t].into());
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: t,
                            identifier: Declarator::None,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: _,
                        },
                        State::SimpleIdentifier(_, i),
                    ) if kind == &Type::Resource => {
                        let i = scoped_ref!(mm!(), i);
                        let i = Declarator::Variable(i);
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: i,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: Declarator::Variable(i),
                        },
                        rest,
                    ) if kind == &Type::Resource => {
                        match rest {
                            State::ConstructorInvocation(_) => (),
                            State::Invocation(_) => (),
                            State::MethodReference(_) => (),
                            State::ScopedIdentifier(_) => (), // not sure
                            State::LambdaExpression(_) => (),
                            x => log::error!("{:?} Declaration {:?}", kind, x),
                        };
                        let d = Declarator::Variable(i);
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: d,
                        }
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::FormalParameter {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::Modifiers(v, _))
                        if kind == &Type::FormalParameter || kind == &Type::SpreadParameter =>
                    {
                        State::Declaration {
                            visibility: v,
                            kind: DeclType::Runtime(vec![].into()),
                            identifier: Declarator::None,
                        }
                    }
                    (State::None, State::SimpleTypeIdentifier(t))
                        if kind == &Type::FormalParameter || kind == &Type::SpreadParameter =>
                    {
                        // TODO spread parameter is hard for invocation matching on check ? cannot use param ?
                        // TODO spread parameter is hard for decl matching on solve
                        // NOTE method ref resolution (overloading)
                        // 1)strict invocation: fixed arity method resolution, no boxing/unboxing )
                        // 2)loose invocation: fixed arity method resolution, boxing/unboxing
                        // 3)variable arity invocation: variable arity method resolution, boxing/unboxing
                        let t = scoped_type!(mm!(), t);
                        let t = DeclType::Runtime(vec![t].into());
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: t,
                            identifier: Declarator::None,
                        }
                    }
                    (State::None, State::ScopedTypeIdentifier(t))
                        if kind == &Type::FormalParameter || kind == &Type::SpreadParameter =>
                    {
                        let t = sync!(t);
                        let t = DeclType::Runtime(vec![t].into());
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: t,
                            identifier: Declarator::None,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: _,
                            identifier: Declarator::None,
                        },
                        State::SimpleTypeIdentifier(t),
                    ) if kind == &Type::FormalParameter || kind == &Type::SpreadParameter => {
                        // TODO spread parameter is hard for invocation matching on check ? cannot use param ?
                        // TODO spread parameter is hard for decl matching on solve
                        // NOTE method ref resolution (overloading)
                        // 1)strict invocation: fixed arity method resolution, no boxing/unboxing )
                        // 2)loose invocation: fixed arity method resolution, boxing/unboxing
                        // 3)variable arity invocation: variable arity method resolution, boxing/unboxing
                        let t = scoped_type!(mm!(), t);
                        let t = DeclType::Runtime(vec![t].into());
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: Declarator::None,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: _,
                            identifier: Declarator::None,
                        },
                        State::ScopedTypeIdentifier(t),
                    ) if kind == &Type::FormalParameter || kind == &Type::SpreadParameter => {
                        let t = sync!(t);
                        let t = DeclType::Runtime(vec![t].into());
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: Declarator::None,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: Declarator::None,
                        },
                        State::SimpleIdentifier(_, i),
                    ) if kind == &Type::FormalParameter => {
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        let i = Declarator::Variable(i);
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: i,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: Declarator::None,
                        },
                        State::SimpleIdentifier(_, i),
                    ) if kind == &Type::FormalParameter => {
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        let i = Declarator::Variable(i);
                        // no need because wont be used directly
                        // acc.solver.add_decl_simple(i.clone(), t);
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: i,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: i,
                        },
                        State::Dimensions,
                    ) if kind == &Type::FormalParameter => {
                        let t = match t {
                            DeclType::Runtime(v) => DeclType::Runtime(
                                v.iter()
                                    .map(|t| acc.solver.intern_ref(RefsEnum::Array(*t)))
                                    .collect(),
                            ),
                            DeclType::Compile(_, _, _) => todo!(),
                        };
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: i,
                        }
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::TypeParameter {
                match current_node.map(|x| Old(x), |x| x) {
                    State::None => acc.current_node.take(),
                    State::Annotation => acc.current_node.take(),
                    State::SimpleIdentifier(_, i) =>
                    {
                        if let State::TypeDeclaration { identifier: DeclType::Compile{0:a, ..}, .. } = &mut acc.current_node {
                            let mm = mm!();
                            *a = acc.solver.intern(RefsEnum::TypeIdentifier(mm, i));
                            acc.current_node.take()
                        } else {
                            panic!("{:?} {:?}", kind, acc.current_node)
                        }
                    },
                    // State::ScopedTypeIdentifier(t) =>
                    // {
                    //     if let State::TypeDeclaration { identifier: DeclType::Compile{0:a, ..}, .. } = &mut acc.current_node {
                    //         // *a = sync!(t);
                    //         acc.current_node.take()
                    //     } else {
                    //         panic!("{:?} {:?}", kind, acc.current_node)
                    //     }
                    // },
                    State::TypeBound(ext,imp) =>
                    {
                        if let State::TypeDeclaration { identifier: DeclType::Compile{1:a, 2:b,..}, .. } = &mut acc.current_node {
                            // *a = vec![sync!(ext)].into();
                            // *b = imp.iter().map(|x|sync!(x)).collect();
                            acc.current_node.take()
                        } else {
                            panic!("{:?} {:?}", kind, acc.current_node)
                        }
                    }
                    x => missing_rule!("{:?} TypeDeclaration {:?}", kind, x),
                }
            } else if kind == &Type::SpreadParameter {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: _,
                        },
                        State::Declarator(d),
                    ) if kind == &Type::SpreadParameter => {
                        // let t = DeclType::Runtime(vec![t].into());
                        let i = match d {
                            Declarator::Variable(t) => sync!(t),
                            _ => panic!(),
                        };
                        let i = Declarator::Variable(i);
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: i,
                        }
                        // State::ScopedTypeIdentifier(t)
                    }
                    (State::None, State::ScopedTypeIdentifier(t))
                        if kind == &Type::SpreadParameter =>
                    {
                        let t = sync!(t);
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: DeclType::Runtime(vec![t].into()),
                            identifier: Declarator::None,
                        }
                    }
                    (State::None, State::SimpleTypeIdentifier(t))
                        if kind == &Type::SpreadParameter =>
                    {
                        let t = scoped_type!(mm!(), t);
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: DeclType::Runtime(vec![t].into()),
                            identifier: Declarator::None,
                        }
                    }
                    (State::None, State::Modifiers(v, n)) if kind == &Type::SpreadParameter => {
                        assert_eq!(v, Visibility::None);
                        State::None // do the rest if need non visibility modifiers
                                    // State::Declaration {
                                    //     visibility: v,
                                    //     kind: DeclType::Runtime(Default::default()),
                                    //     identifier: Declarator::None,
                                    // }
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::CatchFormalParameter {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::Modifiers(Visibility::None, n))
                        if kind == &Type::CatchFormalParameter =>
                    {
                        assert_eq!(n, enum_set!());
                        State::None
                    }
                    (State::None, State::CatchTypes(v)) if kind == &Type::CatchFormalParameter => {
                        State::CatchTypes(v.iter().map(|x| sync!(x)).collect())
                    }
                    (State::CatchTypes(v), State::SimpleIdentifier(_, i))
                        if kind == &Type::CatchFormalParameter =>
                    {
                        State::CatchParameter {
                            kinds: v.into_boxed_slice(),
                            identifier: i,
                        }
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else {
                panic!("{:?}", kind)
            }
        } else if kind.is_expression() {
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            if kind == &Type::LambdaExpression {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleIdentifier(_, i))
                        if kind == &Type::LambdaExpression =>
                    {
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        let i = Declarator::Variable(i);
                        acc.solver
                            .add_decl(i.clone(), DeclType::Runtime(vec![].into()));
                        State::Declarations(vec![(
                            Visibility::None,
                            i,
                            DeclType::Runtime(vec![].into()),
                        )]) // r
                    }
                    (State::None, State::Declarations(v)) if kind == &Type::LambdaExpression => {
                        let v = v
                            .into_iter()
                            .map(|(v, i, t)| {
                                let i = i.with_changed_node(|i| {
                                    let i = sync!(*i);
                                    assert!(!acc.solver.has_choice(i),"{:?}",acc.solver.nodes.with(i));
                                    i
                                });
                                let t = t.map(|x| sync!(x));
                                acc.solver.add_decl(i.clone(), t.clone());
                                (v, i, t)
                            })
                            .collect();
                        State::Declarations(v)
                    }
                    (State::None, State::FormalParameters(v))
                        if kind == &Type::LambdaExpression =>
                    {
                        let v = v
                            .into_iter()
                            .map(|(i, t)| {
                                let i = sync!(i);
                                assert!(!acc.solver.has_choice(i),"{:?}",acc.solver.nodes.with(i));
                                let t = t.map(|x| sync!(x));
                                let i = Declarator::Variable(i);
                                acc.solver.add_decl(i.clone(), t.clone());
                                (Visibility::None, i, t)
                            })
                            .collect();
                        State::Declarations(v)
                    }
                    (State::Declarations(p), State::None) if kind == &Type::LambdaExpression => {
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::Invocation(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let i = sync!(i);
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::FieldIdentifier(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let i = sync!(i);
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::ConstructorInvocation(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let i = sync!(i);
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::ScopedIdentifier(i))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let i = sync!(i);
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::SimpleIdentifier(_, i))
                        if kind == &Type::LambdaExpression =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        // TODO solve references to parameters
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::This(i)) if kind == &Type::LambdaExpression => {
                        let i = sync!(i);
                        // TODO solve references to parameters
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (State::Declarations(p), State::LiteralType(t))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::Declarations(p), State::LambdaExpression(e))
                        if kind == &Type::LambdaExpression =>
                    {
                        // TODO solve references to parameters
                        let i = mm!();
                        State::LambdaExpression(i)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ArrayCreationExpression {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(i))
                        if kind == &Type::ArrayCreationExpression =>
                    {
                        let i = scoped_type!(mm!(),i);
                        let i = acc
                            .solver
                            .intern(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                        State::ConstructorInvocation(i)
                    }
                    (State::None, State::ScopedTypeIdentifier(i))
                        if kind == &Type::ArrayCreationExpression =>
                    {
                        let i = sync!(i);
                        let i = acc
                            .solver
                            .intern(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                        State::ConstructorInvocation(i)
                    }
                    (State::None, State::Annotation) if kind == &Type::ArrayCreationExpression => {
                        State::None
                    }
                    (State::ConstructorInvocation(i), State::None)
                        if kind == &Type::ArrayCreationExpression =>
                    {
                        State::ConstructorInvocation(i)
                    }
                    (State::ConstructorInvocation(i), rest)
                        if kind == &Type::ArrayCreationExpression =>
                    {
                        match rest {
                            State::Dimensions => (),
                            State::ScopedIdentifier(_) => (),
                            State::LiteralType(_) => (),
                            State::SimpleIdentifier(_, i) => {
                                scoped_ref!(mm!(), i);
                            }
                            x => todo!("{:?}", x),
                        };
                        let (o, p) = match &acc.solver.nodes[i] {
                            RefsEnum::ConstructorInvocation(o, p) => (*o, p.clone()),
                            x => todo!("{:?}", x),
                        };
                        let i = acc.solver.intern(RefsEnum::Array(o));
                        let i = acc.solver.intern(RefsEnum::ConstructorInvocation(i, p));
                        State::ConstructorInvocation(i)
                    }
                    // // (State::ConstructorInvocation(i), State::LiteralType(_))
                    // //     if kind == &Type::ArrayCreationExpression =>
                    // // {
                    // //     // TODO use the dimension expr
                    // //     State::ConstructorInvocation(i)
                    // // }
                    // (State::ScopedIdentifier(i), State::LiteralType(_))
                    //     if kind == &Type::ArrayCreationExpression =>
                    // {
                    //     let i = acc
                    //         .solver
                    //         .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                    //     // TODO use dimension
                    //     State::ConstructorInvocation(i)
                    // }
                    // (
                    //     State::ScopedIdentifier(i),
                    //     State::FieldIdentifier(_),
                    // ) if kind == &Type::ArrayCreationExpression => {
                    //     let i = acc
                    //         .solver
                    //         .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Unknown));
                    //     // TODO use dimension
                    //     State::ConstructorInvocation(i)
                    // }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ObjectCreationExpression {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::Annotation) if kind == &Type::ObjectCreationExpression => {
                        State::None
                    }
                    (State::None, State::SimpleTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let r = mm!();
                        State::InvocationId(r, i)
                    }
                    (State::None, State::SimpleIdentifier(case, i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::SimpleIdentifier(case, i)
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::SimpleIdentifier(case, o), State::SimpleTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        // TODO use case maybe
                        let o = scoped_ref!(mm!(), o);
                        State::InvocationId(o, i)
                    }
                    (State::ScopedIdentifier(o), State::SimpleTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::InvocationId(o, i)
                    }
                    (State::SimpleTypeIdentifier(o), State::SimpleTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let o = scoped_type!(mm!(), o);
                        State::InvocationId(o, i)
                    }
                    (State::ScopedTypeIdentifier(o), State::ScopedTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let i = sync!(i);
                        let i = acc.solver.try_solve_node_with(i, o).unwrap();
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::SimpleIdentifier(_, ii), State::ScopedTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let o = scoped_ref!(mm!(), ii);
                        let i = sync!(i);
                        let i = acc.solver.try_solve_node_with(i, o).unwrap();
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::None, State::ScopedTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::None, State::TypeArguments(_))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::None
                    }
                    (State::ScopedTypeIdentifier(i), State::Arguments(p))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let r = acc
                            .solver
                            .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                        State::ConstructorInvocation(r)
                    }
                    (State::InvocationId(r, i), State::Arguments(p))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        // TODO invocationId may not be the best way
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let i = scoped_ref!(r, i);
                        let r = acc
                            .solver
                            .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                        State::ConstructorInvocation(r)
                    }
                    (State::ConstructorInvocation(r), State::Declarations(v))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::ConstructorInvocation(r)
                    }
                    (State::ConstructorInvocation(r), State::None)
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::ConstructorInvocation(r)
                    }
                    (State::None, State::ConstructorInvocation(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::None // TODO check this
                    }
                    (State::None, State::This(i)) if kind == &Type::ObjectCreationExpression => {
                        let i = sync!(i);
                        let o = match acc.solver.nodes[i] {
                            RefsEnum::This(o) => o,
                            _ => panic!(),
                        };
                        State::ScopedTypeIdentifier(o)
                    }
                    (State::ScopedTypeIdentifier(o), State::SimpleTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let i = scoped_type!(o, i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::ScopedTypeIdentifier(o), State::ScopedTypeIdentifier(i))
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        let i = sync!(i);
                        let i = acc.solver.try_solve_node_with(i, o).unwrap();
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::None, State::None)
                        if kind == &Type::ObjectCreationExpression =>
                    {
                        State::None
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::MethodInvocation {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    // MethodInvocation f()
                    (State::None, State::SimpleIdentifier(case, t))
                        if kind == &Type::MethodInvocation =>
                    {
                        State::SimpleIdentifier(case, t)
                    }
                    (State::SimpleIdentifier(_, i), State::Arguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let r = mm!();
                        let r =
                            acc.solver
                                .intern_ref(RefsEnum::Invocation(r, i, Arguments::Given(p)));
                        State::ScopedIdentifier(r) // or should it be an invocation
                    }
                    (State::SimpleIdentifier(case, i), State::TypeArguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        // TODO use type arguments
                        State::SimpleIdentifier(case, i)
                    }
                    (State::ScopedIdentifier(i), State::TypeArguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        // TODO handle type argmuments
                        // todo!(
                        //     "{:?}",
                        //     ExplorableRef {
                        //         rf: i,
                        //         nodes: &acc.solver.nodes
                        //     }
                        // );
                        // let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        // let r = mm!();
                        // let r =
                        //     acc.solver
                        //         .intern_ref(RefsEnum::Invocation(r, i, Arguments::Given(p)));
                        State::ScopedIdentifier(i) // or should it be an invocation
                    }
                    // MethodInvocation x.f()
                    (State::None, expr) if kind == &Type::MethodInvocation => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(_, t) => {
                                panic!("should be handled specifically")
                            }
                            State::This(i) => sync!(i),
                            State::Super(i) => sync!(i),
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            State::None => panic!(""),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::Arguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        // todo!("{:?}",ExplorableRef{rf:i,nodes:&acc.solver.nodes});
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        match &acc.solver.nodes[i].clone() {
                            RefsEnum::ScopedIdentifier(o, i) => {
                                let r = acc.solver.intern_ref(RefsEnum::Invocation(
                                    *o,
                                    *i,
                                    Arguments::Given(p),
                                ));
                                State::ScopedIdentifier(r)
                            }
                            x => panic!("{:?} {:?}", acc.solver.nodes.with(i), x),
                        }
                    }
                    (State::ScopedIdentifier(o), expr) if kind == &Type::MethodInvocation => {
                        match expr {
                            State::SimpleIdentifier(_, i) => State::InvocationId(o, i),
                            State::This(i) => State::ScopedIdentifier(spec!(o, sync!(i))),
                            State::Super(i) => State::ScopedIdentifier(spec!(o, sync!(i))),
                            x => panic!("{:?} {:?}", acc.solver.nodes.with(o), x),
                        }
                    }
                    (State::SimpleIdentifier(case, o), expr) if kind == &Type::MethodInvocation => {
                        // TODO use case
                        match expr {
                            State::SimpleIdentifier(_, i) => {
                                State::InvocationId(scoped_ref!(mm!(), o), i)
                            }
                            State::This(i) => {
                                State::ScopedIdentifier(spec!(scoped_ref!(mm!(), o), sync!(i)))
                            }
                            State::Super(i) => {
                                State::ScopedIdentifier(spec!(scoped_ref!(mm!(), o), sync!(i)))
                            }
                            State::None => { // TODO can do better finding cause of None
                                State::ScopedIdentifier(scoped_ref!(mm!(), o))
                            }
                            x => panic!("{:?}", x),
                        }
                    }
                    (State::InvocationId(o, i), State::Arguments(p))
                        if kind == &Type::MethodInvocation =>
                    {
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let r =
                            acc.solver
                                .intern_ref(RefsEnum::Invocation(o, i, Arguments::Given(p)));
                        State::ScopedIdentifier(r) // or should it be an invocation
                    }
                    (State::InvocationId(o, i), State::None)
                        if kind == &Type::MethodInvocation =>
                    {
                        // TODO check, I suppose it is caused by module identifiers
                        // to reproduce on ["target/release/hyper_ast_benchmark", "alibaba/fastjson", "", "f56b5d895f97f4cc3bd787c600a3ee67ba56d4db", "", "results_1000_commits2/fastjson"]
                        // State::InvocationId(o, i)
                        let r =
                            acc.solver
                                .intern_ref(RefsEnum::Invocation(o, i, Arguments::Given(vec![].into_boxed_slice())));
                        State::ScopedIdentifier(r)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::MethodReference {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, expr) if kind == &Type::MethodReference => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(case, t) => scoped_ref!(mm!(), t), // TODO use case
                            State::SimpleTypeIdentifier(t) => scoped_type!(mm!(), t),
                            State::This(t) => sync!(t),
                            State::Super(t) => sync!(t),
                            State::ScopedTypeIdentifier(i) => sync!(i), // TODO fix related to getting type alias from tree-sitter API
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i), // TODO check panic!("not possible"),
                            State::Invocation(i) => {log::warn!("not possible");sync!(i)},
                            State::ConstructorInvocation(i) => {log::warn!("not possible");sync!(i)},
                            State::None => panic!("should handle before"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::SimpleIdentifier(_, i))
                        if kind == &Type::MethodReference =>
                    {
                        let r = acc.solver.intern_ref(RefsEnum::MethodReference(o, i));
                        State::MethodReference(r)
                    }
                    (State::ScopedIdentifier(o), State::None) if kind == &Type::MethodReference => {
                        let r = acc
                            .solver
                            .intern_ref(RefsEnum::ConstructorInvocation(o, Arguments::Unknown));
                        State::MethodReference(r)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ExplicitConstructorInvocation {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    // this() or super()
                    // TODO ExplicitConstructorInvocation try not to pollute ref resolution
                    (State::None, expr) if kind == &Type::ExplicitConstructorInvocation => {
                        match &expr {
                            State::SimpleIdentifier(case, i) => State::SimpleIdentifier(*case, *i),
                            State::ScopedIdentifier(i) => State::ScopedIdentifier(sync!(*i)),
                            State::This(i) => State::This(sync!(*i)),
                            State::Super(i) => State::Super(sync!(*i)),
                            State::TypeArguments(_) => State::None,
                            x => panic!("{:?}", x),
                        }
                    }
                    (State::ScopedIdentifier(o), State::SimpleIdentifier(_, i))
                        if kind == &Type::ExplicitConstructorInvocation =>
                    {
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::Super(i))
                        if kind == &Type::ExplicitConstructorInvocation =>
                    {
                        let i = spec!(o, sync!(i));
                        State::ScopedIdentifier(i)
                    }
                    (State::SimpleIdentifier(case, o), State::Super(i))
                        if kind == &Type::ExplicitConstructorInvocation =>
                    {
                        // TODO use case if super has an object
                        let i = spec!(scoped_ref!(mm!(), o), sync!(i));
                        State::ScopedIdentifier(i)
                    }
                    (expr, State::Arguments(p)) if kind == &Type::ExplicitConstructorInvocation => {
                        let i = match expr {
                            State::ScopedIdentifier(i) => i,
                            State::Super(i) => i,
                            State::This(i) => i,
                            _ => panic!(),
                        };
                        let p = p.deref().iter().map(|i| sync!(*i)).collect();
                        let i = acc
                            .solver
                            .intern_ref(RefsEnum::ConstructorInvocation(i, Arguments::Given(p)));
                        State::ConstructorInvocation(i)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }

            // } else if kind == &Type::ClassLiteral {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::FieldAccess {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            //     }
            // } else if kind == &Type::ArrayAccess {
            //     match (acc.current_node.take(), current_node.map(|x| Old(x),|x| x)) {
            //         (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            //     }
            } else if kind == &Type::TernaryExpression {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    // TernaryExpression
                    // TernaryExpression (None,c)
                    (State::None, c) => {
                        match c {
                            State::SimpleIdentifier(_, i) => {
                                scoped_ref!(mm!(), i);
                            }
                            State::LiteralType(_) => (),
                            State::ScopedIdentifier(i) => {
                                sync!(i);
                            }
                            State::Invocation(_) => (),
                            State::FieldIdentifier(_) => (),
                            x => log::warn!("TernaryExpression None LiteralType {:?}", x),
                        };
                        State::Condition
                    }
                    // TernaryExpression (Cond,x)
                    (State::Condition, x) => match x {
                        State::LiteralType(t) => State::ScopedIdentifier(sync!(t)),
                        State::SimpleIdentifier(_, i) => State::ScopedIdentifier(scoped_ref!(mm!(), i)),
                        State::This(i) => State::ScopedIdentifier(sync!(i)),
                        State::ConstructorInvocation(i) => State::ConstructorInvocation(sync!(i)),
                        State::Invocation(i) => State::Invocation(sync!(i)),
                        State::ScopedIdentifier(i) => State::ScopedIdentifier(sync!(i)),
                        State::FieldIdentifier(i) => State::FieldIdentifier(sync!(i)),
                        State::MethodReference(i) => State::MethodReference(sync!(i)),
                        State::LambdaExpression(i) => State::LambdaExpression(sync!(i)),
                        State::None => {log::warn!("TernaryExpression Condition None");State::None},
                        x => missing_rule!("{:?} Condition {:?}", kind, x),
                    },
                    // TernaryExpression (x,y)
                    // WARN The kind of type evaluation (choosing between x and y) is not finished (without obvious bugs relative to java spec)
                    // but for now it should not impact the quality of the reference analysis.
                    (State::LiteralType(t), y) => {
                        match y {
                            State::LiteralType(_) => (),
                            State::SimpleIdentifier(_, i) => {
                                scoped_ref!(mm!(), i);
                            }
                            State::FieldIdentifier(_) => (),
                            State::ConstructorInvocation(_) => (),
                            State::Invocation(_) => (),
                            State::ScopedIdentifier(_) => (),
                            State::LambdaExpression(_) => (),
                            State::MethodReference(_) => (),
                            State::None => log::warn!("TernaryExpression LiteralType None"),
                            x => log::warn!("TernaryExpression LiteralType {:?}", &x),
                        };
                        State::LiteralType(t)
                    }
                    (x, State::LiteralType(t)) => {
                        assert_ne!(x, State::Condition);
                        match x {
                            State::LiteralType(_) => (),
                            State::SimpleIdentifier(_, i) => {
                                scoped_ref!(mm!(), i);
                            }
                            State::ConstructorInvocation(_) => (),
                            State::Invocation(_) => (),
                            State::ScopedIdentifier(_) => (),
                            State::LambdaExpression(_) => (),
                            State::MethodReference(_) => (),
                            State::This(_) => (),
                            State::Super(_) => (),
                            State::None => log::warn!("TernaryExpression None LiteralType"),
                            x => log::warn!("TernaryExpression {:?} LiteralType", &x),
                        };
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::SimpleIdentifier(_, i), y) => {
                        match y {
                            State::LiteralType(_) => panic!(),
                            State::SimpleIdentifier(_, i) => {
                                scoped_ref!(mm!(), i);
                            }
                            State::FieldIdentifier(_) => (),
                            State::ConstructorInvocation(_) => (),
                            State::Invocation(_) => (),
                            State::ScopedIdentifier(_) => (),
                            State::LambdaExpression(_) => (),
                            State::MethodReference(_) => (),
                            State::This(_) => (),
                            State::Super(_) => (),
                            State::None => log::warn!("TernaryExpression SimpleIdentifier None"),
                            x => log::warn!("TernaryExpression SimpleIdentifier {:?}", x),
                        };
                        let i = scoped_ref!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (x, State::SimpleIdentifier(_, i)) =>
                    {
                        assert_ne!(x, State::Condition);
                        match x {
                            State::LiteralType(_) => panic!(),
                            State::SimpleIdentifier(_, _) => panic!(),
                            State::ConstructorInvocation(_) => (),
                            State::Invocation(_) => (),
                            State::ScopedIdentifier(_) => (),
                            State::LambdaExpression(_) => (),
                            State::MethodReference(_) => (),
                            State::This(_) => (),
                            State::Super(_) => (),
                            State::None => log::warn!("TernaryExpression None SimpleIdentifier"),
                            x => log::warn!("TernaryExpression {:?} SimpleIdentifier", &x),
                        };
                        let i = scoped_ref!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ConstructorInvocation(_), State::This(i)) =>
                    {
                        let i = sync!(i);
                        State::This(i)
                    }
                    (State::ConstructorInvocation(i), _) =>
                    {
                        State::ConstructorInvocation(i)
                    }
                    (_, State::ConstructorInvocation(i)) =>
                    {
                        let i = sync!(i);
                        State::ConstructorInvocation(i)
                    }
                    (State::ScopedIdentifier(i), y) => {
                        match y {
                            State::LiteralType(_) => panic!(),
                            State::SimpleIdentifier(_, i) => {
                                scoped_ref!(mm!(), i);
                            }
                            State::FieldIdentifier(_) => (),
                            State::ConstructorInvocation(_) => panic!(),
                            State::Invocation(_) => (),
                            State::ScopedIdentifier(_) => (),
                            State::LambdaExpression(_) => (),
                            State::MethodReference(_) => (),
                            State::This(_) => (),
                            State::Super(_) => (),
                            State::None => log::warn!("TernaryExpression ScopedIdentifier None"),
                            x => log::warn!("TernaryExpression ScopedIdentifier {:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (_, State::ScopedIdentifier(i)) =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::FieldIdentifier(i), _) =>
                    {
                        State::FieldIdentifier(i)
                    }
                    (_, State::FieldIdentifier(i)) =>
                    {
                        let i = sync!(i);
                        State::FieldIdentifier(i)
                    }
                    (x, _) => x
                }
            } else {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(t))
                        if kind == &Type::InstanceofExpression =>
                    {
                        scoped_type!(mm!(), t);
                        State::None
                    }
                    (State::Invocation(_), State::SimpleTypeIdentifier(i))
                        if kind == &Type::InstanceofExpression =>
                    {
                        let i = scoped_type!(mm!(), i);
                        // TODO intern boolean
                        State::ScopedIdentifier(mm!())
                    }
                    (State::ScopedIdentifier(_), State::SimpleTypeIdentifier(i))
                        if kind == &Type::InstanceofExpression =>
                    {
                        let i = scoped_type!(mm!(), i);
                        // TODO intern boolean
                        State::ScopedIdentifier(mm!())
                    }
                    (State::ScopedIdentifier(_), State::ScopedTypeIdentifier(i))
                        if kind == &Type::InstanceofExpression =>
                    {
                        // TODO intern boolean
                        State::ScopedIdentifier(mm!())
                    }

                    // array access
                    (State::None, expr) if kind == &Type::ArrayAccess => {
                        // TODO simp more FieldIdentifiers to ScopedIdentifier
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(case, t) => scoped_ref!(mm!(), t), // TODO use case
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), expr) if kind == &Type::ArrayAccess => {
                        // TODO create RefsEnum variant to use access expr and solve type of array
                        match expr {
                            State::LiteralType(t) => (),
                            State::SimpleIdentifier(_, t) => {
                                scoped_ref!(mm!(), t);
                            }
                            State::This(t) => (),
                            State::ScopedIdentifier(_) => (),
                            State::FieldIdentifier(_) => (),
                            State::Invocation(_) => (),
                            State::ConstructorInvocation(_) => (),
                            State::LambdaExpression(_) => (),
                            State::MethodReference(_) => (),
                            // State::None => (), // TODO check
                            x => panic!("{:?}", x),
                        };
                        let o = acc.solver.intern_ref(RefsEnum::ArrayAccess(o));
                        State::ScopedIdentifier(o)
                    }
                    // field access
                    (State::None, expr) if kind == &Type::FieldAccess =>
                    //TODO get type
                    {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(case, t) => scoped_ref!(mm!(), t), // TODO use case
                            State::This(i) => sync!(i),
                            State::Super(i) => sync!(i),
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            State::None => panic!("should handle super"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::SimpleIdentifier(_, i))
                        if kind == &Type::FieldAccess =>
                    {
                        let i = scoped_ref!(o, i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::This(i)) if kind == &Type::FieldAccess => {
                        // TODO check every State::This and State::Super because it is not correctly repr grammar 
                        let i = acc.solver.intern_ref(RefsEnum::This(o));
                        State::ScopedIdentifier(i)
                    }

                    // literal
                    (State::None, State::ScopedTypeIdentifier(i))
                        if kind == &Type::ClassLiteral =>
                    {
                        // TODO should return Class<i>
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::SimpleTypeIdentifier(i))
                        if kind == &Type::ClassLiteral =>
                    {
                        // TODO should return Class<i>
                        let i = scoped_type!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }

                    // CastExpression
                    (State::None, expr) if kind == &Type::CastExpression => {
                        let t = match expr {
                            State::SimpleTypeIdentifier(t) => scoped_type!(mm!(), t),
                            State::ScopedTypeIdentifier(i) => sync!(i),
                            State::None => panic!("should handle before"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::ScopedTypeIdentifier(t), expr) if kind == &Type::CastExpression => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::This(i) => sync!(i),
                            State::SimpleIdentifier(_, t) => scoped_ref!(mm!(), t),
                            State::SimpleTypeIdentifier(t) => scoped_type!(mm!(), t), // should not append
                            State::ScopedIdentifier(i) => sync!(i),
                            State::LambdaExpression(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            State::MethodReference(i) => sync!(i),
                            State::ScopedTypeIdentifier(i) => panic!(),
                            State::None => panic!("should handle before"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(t)
                    }
                    (State::ScopedIdentifier(t), expr) if kind == &Type::CastExpression => {
                        // should be ScopedTypeIdentifier but cannot get alias from treesitter rust API cleanly
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::This(i) => sync!(i),
                            State::SimpleIdentifier(_, t) => scoped_ref!(mm!(), t),
                            State::SimpleTypeIdentifier(t) => scoped_type!(mm!(), t), // should not append
                            State::ScopedIdentifier(i) => sync!(i),
                            State::LambdaExpression(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            State::MethodReference(i) => sync!(i),
                            State::ScopedTypeIdentifier(i) => panic!(),
                            State::None => panic!("should handle before"),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(t)
                    }
                    (State::None, State::SimpleIdentifier(_, i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::AssignmentExpression
                            || kind == &Type::InstanceofExpression
                            || kind == &Type::UpdateExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::AssignmentExpression
                            || kind == &Type::InstanceofExpression
                            || kind == &Type::UpdateExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::ConstructorInvocation(i))
                        if kind == &Type::InstanceofExpression
                            || kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::ParenthesizedExpression
                            || kind == &Type::UpdateExpression =>
                    {
                        let i = sync!(i);
                        State::ConstructorInvocation(i)
                    }
                    (State::None, State::Invocation(i))
                        if kind == &Type::InstanceofExpression
                            || kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::ParenthesizedExpression
                            || kind == &Type::UpdateExpression =>
                    {
                        // TODO TODO regroup right and match inside
                        let i = sync!(i);
                        State::Invocation(i)
                    }
                    (State::None, State::FieldIdentifier(i))
                        if kind == &Type::InstanceofExpression
                            || kind == &Type::UpdateExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::LiteralType(t))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::AssignmentExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::None, State::FieldIdentifier(i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::UnaryExpression
                            || kind == &Type::UpdateExpression
                            || kind == &Type::AssignmentExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::FieldIdentifier(i)
                    }
                    (State::None, State::This(i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::InstanceofExpression
                            || kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(_), State::LiteralType(t))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::ScopedIdentifier(i), State::This(_))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::ConstructorInvocation(_))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::LambdaExpression(_))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::FieldIdentifier(_), State::ScopedIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i0), State::SimpleIdentifier(_, i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::ScopedIdentifier(i0)
                    }
                    (State::ScopedIdentifier(i0), State::ScopedIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i0)
                    }
                    (State::ScopedIdentifier(i0), State::MethodReference(_))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i0)
                    }
                    (State::ScopedIdentifier(i0), State::FieldIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i0)
                    }
                    (State::FieldIdentifier(i0), State::FieldIdentifier(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        State::FieldIdentifier(i0)
                    }
                    
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::LambdaExpression(i))
                        if kind == &Type::ParenthesizedExpression =>
                    {
                        let i = sync!(i);
                        State::LambdaExpression(i)
                    }
                    (State::None, State::SimpleIdentifier(_, i))
                        if kind == &Type::ParenthesizedExpression =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }

                    (State::ScopedIdentifier(il), State::SimpleIdentifier(_, ir))
                        if kind == &Type::BinaryExpression =>
                    {
                        scoped_ref!(mm!(), ir);
                        State::ScopedIdentifier(il)
                    }
                    (State::ScopedIdentifier(i), State::LiteralType(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::This(t))
                        if kind == &Type::BinaryExpression =>
                    {
                        let t = sync!(t);
                        State::ScopedIdentifier(i)
                    }
                    (State::FieldIdentifier(i), State::ScopedIdentifier(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }
                    (State::LiteralType(t), State::SimpleIdentifier(_, ir))
                        if kind == &Type::BinaryExpression =>
                    {
                        scoped_ref!(mm!(), ir);
                        // TODO not that obvious in general
                        State::LiteralType(t)
                    }
                    (State::LiteralType(t), _) if kind == &Type::BinaryExpression => {
                        // TODO not that obvious in general
                        State::LiteralType(t)
                    }
                    (State::ScopedIdentifier(i), State::ScopedIdentifier(_))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::LiteralType(t), State::SimpleTypeIdentifier(i))
                        if kind == &Type::BinaryExpression =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::LiteralType(t)
                    }
                    (State::LiteralType(t), _) if kind == &Type::BinaryExpression => {
                        State::LiteralType(t)
                    }
                    (State::Invocation(i), State::Invocation(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::This(i), State::Invocation(_)) if kind == &Type::BinaryExpression => {
                        State::ScopedIdentifier(i)
                    }
                    (State::Invocation(i), State::LiteralType(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::Invocation(i), State::This(t)) if kind == &Type::BinaryExpression => {
                        let t = sync!(t);
                        State::Invocation(i)
                    }
                    (State::Invocation(i), State::ScopedIdentifier(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::Invocation(i), State::FieldIdentifier(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::Invocation(i)
                    }
                    (State::FieldIdentifier(i), State::LiteralType(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }
                    (State::FieldIdentifier(i), State::Invocation(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }
                    (State::Invocation(i0), State::SimpleIdentifier(_, i))
                        if kind == &Type::BinaryExpression =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::Invocation(i0)
                    }
                    (State::FieldIdentifier(i0), State::SimpleIdentifier(_, i))
                        if kind == &Type::BinaryExpression =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::FieldIdentifier(i0)
                    }
                    (State::FieldIdentifier(i), State::FieldIdentifier(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::FieldIdentifier(i)
                    }

                    (State::Invocation(_), State::LiteralType(t))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::FieldIdentifier(_), State::LiteralType(t))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::FieldIdentifier(i0), State::SimpleIdentifier(_, i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::FieldIdentifier(i0)
                    }
                    (State::FieldIdentifier(_), State::ConstructorInvocation(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = sync!(i);
                        State::ConstructorInvocation(i)
                    }
                    (State::FieldIdentifier(_), State::Invocation(i))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let i = sync!(i);
                        State::ConstructorInvocation(i)
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::AssignmentExpression =>
                    {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::LiteralType(t))
                        if kind == &Type::AssignmentExpression =>
                    {
                        let t = sync!(t);
                        State::LiteralType(t)
                    }
                    (State::ScopedIdentifier(i), State::Invocation(_))
                        if kind == &Type::BinaryExpression
                            || kind == &Type::AssignmentExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(i), State::ConstructorInvocation(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::ScopedIdentifier(i)
                    }
                    (State::ConstructorInvocation(t), State::LiteralType(i))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::ConstructorInvocation(t)
                    }
                    (State::ConstructorInvocation(t), State::SimpleIdentifier(_, i))
                        if kind == &Type::BinaryExpression =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::ConstructorInvocation(t)
                    }
                    (State::ConstructorInvocation(t), State::ConstructorInvocation(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::ConstructorInvocation(t)
                    }
                    (State::ConstructorInvocation(t), State::ScopedIdentifier(_))
                        if kind == &Type::BinaryExpression =>
                    {
                        State::ConstructorInvocation(t)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            }
        } else if kind == &Type::Directory {
            // acc.current_node.take()
            // let mut s: Solver = Default::default();
            // s.nodes = self.solver.nodes;
            // s.refs.resize(self.solver.refs.len(), false);
            // let mut remapper = acc.solver.extend(&s);
            let is_first = acc.solver.is_empty();
            let mut remapper = if is_first {
                acc.solver = self.solver;
                // acc.solver.extend(&self.solver)
                None
            } else {
                Some(acc.solver.extend(&self.solver))
            };
            macro_rules! sync {
                ( $i:expr ) => {
                    match &mut remapper {
                        None => $i.0,
                        Some(remapper) => remapper.intern_external(&mut acc.solver, $i.0),
                    }
                };
            }

            macro_rules! syncd {
                ( $i:expr ) => {{
                    match &mut remapper {
                        None => {
                            let r = $i.0;
                            assert!(!acc.solver.has_choice(r),"{:?}",acc.solver.nodes.with(r));
                            r
                        },
                        Some(remapper) => {
                            let r = remapper.intern_external(&mut acc.solver, $i.0);
                            assert!(!acc.solver.has_choice(r),"{:?}",acc.solver.nodes.with(r));
                            r
                        },
                    }
                }};
            }

            if is_first 
            && let State::Directory{..} = acc.current_node 
            && let State::Directory{global_decls} = current_node {
                State::Directory{
                    global_decls,
                }
            } else {

            match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                (rest, State::None) => rest,
                (State::None, State::File{
                    package,
                    local,
                    global,
                    ..//TODO
                }) =>
                {
                    // assert!(package.is_some());
                    let global_decls = global.iter().map(|(d,t)| {
                        let d = d.with_changed_node(|x|syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        (d,t)
                    }).collect();
                    let package_local = local.iter().map(|(d,t)| {
                        let d = d.with_changed_node(|x|syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        (d,t)
                    }).collect();
                    let package = package.map(|p|sync!(p));


                    State::Package{
                        package,
                        global_decls,
                        package_local,
                    }
                }
                (State::Directory{
                    mut global_decls,
                }, State::File{
                    package,
                    local,
                    global,
                    ..//TODO
                }) =>
                {
                    // assert!(package.is_some());
                    let package = package.map(|p|sync!(p));

                    global.iter().for_each(|(d,t)| {
                        let d = d.with_changed_node(|x| syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        global_decls.push((d,t));
                    });

                    let package_local = local.iter().map(|(d,t)| {
                        let d = d.with_changed_node(|x|syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        (d,t)
                    }).collect();

                    State::Package{
                        package,
                        global_decls,
                        package_local,
                    }
                }
                (State::Package{
                    package,
                    mut global_decls,
                    mut package_local,
                    ..//TODO
                }, State::File{
                    package: p,
                    local,
                    global,
                    ..//TODO
                }) =>
                {
                    // assert_eq!(package,p.map(|p|sync!(p)));

                    global.iter().for_each(|(d,t)| {
                        let d = d.with_changed_node(|x| syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        global_decls.push((d,t));
                    });
                    local.iter().for_each(|(d,t)| {
                        let d = d.with_changed_node(|x| syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        package_local.push((d,t));
                    });

                    State::Package{
                        package,
                        global_decls,
                        package_local,
                    }
                }
                (State::None, State::Directory{
                    global_decls:global,
                }) =>
                {
                    let global_decls = global.iter().map(|(d,t)| {
                        let d = d.with_changed_node(|x|syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        (d,t)
                    }).collect();

                    State::Directory{
                        global_decls,
                    }
                }
                (State::None, State::Package{
                    global_decls:global,
                    ..
                }) =>
                {
                    let global_decls = global.iter().map(|(d,t)| {
                        let d = d.with_changed_node(|x|syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        (d,t)
                    }).collect();

                    State::Directory{
                        global_decls,
                    }
                }
                (State::Directory{
                    mut global_decls,
                }, State::Package{
                    global_decls:global,
                    ..
                }) =>
                {
                    global.iter().for_each(|(d,t)| {
                        let d = d.with_changed_node(|x| syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        global_decls.push((d,t));
                    });

                    State::Directory{
                        global_decls,
                    }
                }
                (State::Package{
                    package,
                    mut global_decls,
                    package_local
                }, State::Directory{
                    global_decls:global,
                }) =>
                {
                    global.iter().for_each(|(d,t)| {
                        let d = d.with_changed_node(|x| syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        global_decls.push((d,t));
                    });

                    State::Package{
                        package,
                        global_decls,
                        package_local,
                    }
                }
                (State::Package{
                    package,
                    mut global_decls,
                    package_local
                }, State::Package{
                    global_decls:global,
                    ..
                }) =>
                {
                    global.iter().for_each(|(d,t)| {
                        let d = d.with_changed_node(|x| syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        global_decls.push((d,t));
                    });

                    State::Package{
                        package,
                        global_decls,
                        package_local,
                    }
                }
                (State::Directory{
                    mut global_decls,
                }, State::Directory{
                    global_decls:global,
                }) =>
                {
                    global.iter().for_each(|(d,t)| {
                        let d = d.with_changed_node(|x| syncd!(*x));
                        let t = t.map(|x| sync!(x));
                        acc.solver.add_decl(d.clone(), t.clone());
                        global_decls.push((d,t));
                    });
                    State::Directory{
                        global_decls,
                    }
                }
                (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
            }
        }
        } else {
            // rest that is not easily categorized ie. used at multiple places
            let mut remapper = acc.solver.extend(&self.solver);
            macro_rules! sync {
                ( $i:expr ) => {
                    remapper.intern_external(&mut acc.solver, $i.0)
                };
            }
            if kind == &Type::Annotation || kind == &Type::MarkerAnnotation {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleIdentifier(_, i))
                        if kind == &Type::MarkerAnnotation =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        // TODO handle annotations correctly
                        State::Annotation
                    }
                    (State::None, State::ScopedIdentifier(i))
                        if kind == &Type::MarkerAnnotation =>
                    {
                        let i = sync!(i);
                        // TODO handle annotations correctly
                        State::Annotation
                    }
                    (State::None, State::SimpleIdentifier(_, i)) if kind == &Type::Annotation => {
                        let i = scoped_ref!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::ScopedIdentifier(i)) if kind == &Type::Annotation => {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::SimpleIdentifier(_, i), State::Arguments(p))
                        if kind == &Type::Annotation =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::Annotation
                    }
                    (State::ScopedIdentifier(i), State::Arguments(p))
                        if kind == &Type::Annotation =>
                    {
                        State::Annotation
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::AnnotationArgumentList
                || kind == &Type::ElementValuePair
                || kind == &Type::ElementValueArrayInitializer
            {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (state, rest) if kind == &Type::AnnotationArgumentList => {
                        let mut v = match state {
                            State::Arguments(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };

                        match rest {
                            State::Annotation => (),
                            State::None => (), // TODO check source
                            State::ElementValuePair(p, i) => (),
                            State::LiteralType(_) => (),
                            State::FieldIdentifier(_) => (),
                            State::ScopedIdentifier(_) => (),
                            State::SimpleIdentifier(_, i) => {
                                scoped_ref!(mm!(), i);
                            }
                            x => panic!("{:?}", x),
                        };

                        State::Arguments(v)
                    }
                    (State::None, State::SimpleIdentifier(case, i))
                        if kind == &Type::ElementValuePair =>
                    {
                        State::SimpleIdentifier(case, i)
                    }
                    (State::SimpleIdentifier(_, i), State::None)
                        if kind == &Type::ElementValuePair =>
                    {
                        let i = scoped_ref!(mm!(), i);
                        State::ScopedIdentifier(i)
                    }
                    (State::SimpleIdentifier(p, i), State::Annotation)
                        if kind == &Type::ElementValuePair =>
                    {
                        State::SimpleIdentifier(p, i)
                    }
                    (State::SimpleIdentifier(_, p), expr) if kind == &Type::ElementValuePair => {
                        let t = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(_, t) => {
                                scoped_ref!(mm!(), t)
                            }
                            State::ScopedIdentifier(t) => sync!(t),
                            State::FieldIdentifier(t) => sync!(t),
                            State::Invocation(t) => sync!(t),
                            State::ConstructorInvocation(t) => sync!(t),
                            x => panic!("{:?}", x),
                        };
                        State::ElementValuePair(p, t)
                    }
                    (rest, State::Annotation) if kind == &Type::ElementValueArrayInitializer => {
                        rest
                    }
                    (rest, expr) if kind == &Type::ElementValueArrayInitializer => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(_, t) => {
                                scoped_ref!(mm!(), t)
                            }
                            State::ScopedIdentifier(t) => sync!(t),
                            State::FieldIdentifier(t) => sync!(t),
                            State::Invocation(t) => sync!(t),
                            State::ConstructorInvocation(t) => sync!(t),
                            x => panic!("{:?}", x),
                        };
                        match rest {
                            State::ScopedIdentifier(i) => State::ScopedIdentifier(i),
                            State::None => State::ScopedIdentifier(i),
                            _ => panic!(),
                        }
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ScopedAbsoluteIdentifier {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::ScopedIdentifier(i)) => {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::SimpleIdentifier(_, i)) => {
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::SimpleIdentifier(_, i)) => {
                        let o = acc.solver.intern(RefsEnum::Root);
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                        State::ScopedIdentifier(i)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ScopedIdentifier {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::ScopedIdentifier(i)) => {
                        let i = sync!(i);
                        State::ScopedIdentifier(i)
                    }
                    (State::ScopedIdentifier(o), State::SimpleIdentifier(_, i)) => {
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                        State::ScopedIdentifier(i)
                    }
                    (State::None, State::SimpleIdentifier(case, i)) => {
                        // TODO use case
                        let o = acc.solver.intern(RefsEnum::MaybeMissing);
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(o, i));
                        State::ScopedIdentifier(i)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ScopedTypeIdentifier {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(i))
                        if kind == &Type::ScopedTypeIdentifier =>
                    {
                        let o = mm!();
                        // let i = acc.solver.intern_ref(RefsEnum::TypeIdentifier(o, i));
                        let i = scoped_type!(o, i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::None, State::ScopedTypeIdentifier(i))
                        if kind == &Type::ScopedTypeIdentifier =>
                    {
                        let i = sync!(i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::ScopedTypeIdentifier(o), State::SimpleTypeIdentifier(i))
                        if kind == &Type::ScopedTypeIdentifier =>
                    {
                        let i = scoped_type!(o, i); // TODO check if nne typeidentifier
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::ScopedTypeIdentifier(o), State::Annotation)
                        if kind == &Type::ScopedTypeIdentifier =>
                    {
                        State::ScopedTypeIdentifier(o)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::CatchType {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::SimpleTypeIdentifier(i)) if kind == &Type::CatchType => {
                        let mut v = match rest {
                            State::CatchTypes(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let o = mm!();
                        let i = scoped_type!(o, i);
                        v.push(i);
                        State::CatchTypes(v)
                    }
                    (rest, State::ScopedTypeIdentifier(i)) if kind == &Type::CatchType => {
                        let mut v = match rest {
                            State::CatchTypes(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let i = sync!(i);
                        v.push(i);
                        State::CatchTypes(v)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ArrayType {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(i)) if kind == &Type::ArrayType => {
                        let o = mm!();
                        let i = scoped_type!(o, i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::None, State::ScopedTypeIdentifier(i)) if kind == &Type::ArrayType => {
                        let i = sync!(i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::SimpleTypeIdentifier(i), State::Dimensions)
                        if kind == &Type::ArrayType =>
                    {
                        let o = mm!();
                        let i = scoped_type!(o, i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (State::ScopedTypeIdentifier(i), State::Dimensions)
                        if kind == &Type::ArrayType =>
                    {
                        let i = acc.solver.intern(RefsEnum::Array(i));
                        State::ScopedTypeIdentifier(i)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ResourceSpecification || kind == &Type::FormalParameters {
                // TODO look like local decl
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::ScopedIdentifier(i)) if kind == &Type::ResourceSpecification => {
                        rest
                    }
                    (
                        rest,
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: d,
                        },
                    ) if kind == &Type::FormalParameters
                        || kind == &Type::ResourceSpecification =>
                    {
                        // TODO do better than simple identifier
                        // TODO own State declaration (for parameters)
                        let mut v = match rest {
                            State::FormalParameters(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let t = t.map(|x| sync!(x));
                        let i = match d {
                            Declarator::Variable(i) => sync!(i),
                            _ => panic!(),
                        };
                        v.push((i, t));
                        State::FormalParameters(v)
                    }
                    (rest, State::None)
                        if kind == &Type::FormalParameters
                            || kind == &Type::ResourceSpecification =>
                    {
                        let mut v = match rest {
                            State::FormalParameters(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        State::FormalParameters(v)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::Wildcard
                || kind == &Type::WildcardSuper
                || kind == &Type::WildcardExtends
            {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::Annotation) if kind == &Type::Wildcard => State::None,
                    (State::None, rest) if kind == &Type::Wildcard => {
                        match rest {
                            State::None => (),
                            State::WildcardExtends(_) => (),
                            State::WildcardSuper(_) => (),
                            x => panic!("{:?}", x),
                        }
                        // TODO solve correctly ie. DeclType::Runtime
                        let r = mm!();
                        State::ScopedTypeIdentifier(r)
                    }
                    (State::None, rest) if kind == &Type::WildcardExtends => {
                        let t = match rest {
                            State::SimpleTypeIdentifier(t) => scoped_type!(mm!(), t),
                            State::ScopedTypeIdentifier(t) => sync!(t),
                            x => panic!("{:?}", x),
                        };
                        State::WildcardExtends(t)
                    }
                    (State::None, rest) if kind == &Type::WildcardSuper => {
                        let t = match rest {
                            State::SimpleIdentifier(_, t) => scoped_ref!(mm!(), t),
                            State::SimpleTypeIdentifier(t) => scoped_type!(mm!(), t),
                            State::ScopedTypeIdentifier(t) => sync!(t),
                            State::Super(t) => sync!(t),
                            x => panic!("{:?}", x),
                        };
                        State::WildcardSuper(t)
                    }
                    (State::WildcardSuper(i), State::SimpleTypeIdentifier(t))
                        if kind == &Type::WildcardSuper =>
                    {
                        let o = mm!();
                        let t = scoped_type!(o, t);
                        State::WildcardSuper(i)
                    }
                    (State::WildcardSuper(i), State::ScopedTypeIdentifier(t))
                        if kind == &Type::WildcardSuper =>
                    {
                        State::WildcardSuper(i)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::SwitchLabel {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::LiteralType(_)) if kind == &Type::SwitchLabel => {
                        State::None
                    }
                    (State::None, State::FieldIdentifier(_)) if kind == &Type::SwitchLabel => {
                        State::None
                    }
                    (State::None, State::ScopedIdentifier(_)) if kind == &Type::SwitchLabel => {
                        State::None
                    }
                    (State::None, State::SimpleIdentifier(_, _)) if kind == &Type::SwitchLabel => {
                        State::None
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::Modifiers {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::Modifiers(v0, n0), State::Modifiers(v, n)) => State::Modifiers(
                        if v0 == Visibility::None {
                            v
                        } else {
                            assert_eq!(v, Visibility::None);
                            v0
                        },
                        n0.union(n),
                    ),

                    (State::None, State::Modifiers(v, n)) if kind == &Type::Modifiers => {
                        State::Modifiers(v, n)
                    }

                    (State::None, State::Annotation) if kind == &Type::Modifiers => {
                        State::Modifiers(Visibility::None, Default::default())
                    }

                    (State::Modifiers(v, n), State::Annotation) if kind == &Type::Modifiers => {
                        State::Modifiers(v, n)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ArgumentList {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::MethodReference(i)) if kind == &Type::ArgumentList => {
                        let i = sync!(i);
                        let mut v = match rest {
                            State::Arguments(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        v.push(i);
                        State::Arguments(v)
                    }
                    (rest, State::LambdaExpression(i)) if kind == &Type::ArgumentList => {
                        let i = sync!(i);
                        let mut v = match rest {
                            State::Arguments(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        v.push(i);
                        State::Arguments(v)
                    }
                    (rest, State::None) if kind == &Type::ArgumentList => rest,
                    (rest, expr) if kind == &Type::ArgumentList => {
                        // TODO do better than simple identifier
                        let mut v = match rest {
                            State::Arguments(l) => l,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(_, t) => scoped_ref!(mm!(), t),
                            State::This(t) => sync!(t),
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            x => panic!("{:?}", x),
                        };
                        v.push(i);
                        State::Arguments(v)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::TypeArguments {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::None) if kind == &Type::TypeArguments => {
                        let v = vec![];
                        State::TypeArguments(v)
                    }
                    (rest, State::SimpleTypeIdentifier(t)) if kind == &Type::TypeArguments => {
                        let mut v = match rest {
                            State::TypeArguments(v) => v,
                            State::None => vec![],
                            _ => vec![],
                        };
                        let o = mm!();
                        let t = acc.solver.intern_ref(RefsEnum::TypeIdentifier(o, t));
                        v.push(t);
                        State::TypeArguments(v)
                    }
                    (rest, State::ScopedTypeIdentifier(i)) if kind == &Type::TypeArguments => {
                        let mut v = match rest {
                            State::TypeArguments(v) => v,
                            State::None => vec![],
                            _ => vec![],
                        };
                        let t = sync!(i);
                        v.push(t);
                        State::TypeArguments(v)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::InferredParameters {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::SimpleIdentifier(_, i)) if kind == &Type::InferredParameters => {
                        let mut v = match rest {
                            State::Declarations(v) => v,
                            State::None => vec![],
                            _ => todo!(),
                        };
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        let i = Declarator::Variable(i);
                        acc.solver
                            .add_decl(i.clone(), DeclType::Runtime(vec![].into()));
                        v.push((Visibility::None, i, DeclType::Runtime(vec![].into())));
                        State::Declarations(v)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::TypeParameters {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, 
                    State::TypeDeclaration{ 
                        identifier,
                        .. 
                    }) => {
                        let mut v = match rest {
                            State::TypeParameters(v) => v,
                            State::None => vec![],
                            _ => todo!(),
                        };
                        v.push(identifier.map(|x|sync!(*x)));
                        State::TypeParameters(v)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::ArrayInitializer {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    // ArrayInit
                    (State::None, expr) if kind == &Type::ArrayInitializer => {
                        match expr {
                            State::LiteralType(t) => (),
                            State::SimpleIdentifier(_, t) => {
                                scoped_ref!(mm!(), t);
                            }
                            State::This(t) => (),
                            State::ScopedIdentifier(_) => (),
                            State::FieldIdentifier(_) => (),
                            State::Invocation(_) => (),
                            State::ConstructorInvocation(_) => (),
                            State::LambdaExpression(_) => (),
                            State::MethodReference(_) => (),
                            State::None => (), // TODO check
                            x => panic!("{:?}", x),
                        };
                        State::None
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::Throws {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::SimpleTypeIdentifier(i)) if kind == &Type::Throws => {
                        let i = scoped_type!(mm!(), i);
                        State::Throws
                    }
                    (rest, State::ScopedTypeIdentifier(i)) if kind == &Type::Throws => {
                        State::Throws
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::VariableDeclarator {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleIdentifier(_, i))
                        if kind == &Type::VariableDeclarator =>
                    {
                        let r = mm!();
                        let i = acc.solver.intern(RefsEnum::ScopedIdentifier(r, i));
                        State::Declarator(Declarator::Variable(i))
                    }
                    (State::Declarator(Declarator::Variable(v)), State::Dimensions)
                        if kind == &Type::VariableDeclarator =>
                    {
                        let v = acc.solver.intern(RefsEnum::Array(v));
                        State::Declarator(Declarator::Variable(v))
                    }
                    (State::Declarator(Declarator::Variable(v)), State::SimpleIdentifier(_, i))
                        if kind == &Type::VariableDeclarator =>
                    {
                        scoped_ref!(mm!(), i);
                        State::Declarator(Declarator::Variable(v))
                    }
                    (State::Declarator(Declarator::Variable(v)), _)
                        if kind == &Type::VariableDeclarator =>
                    {
                        State::Declarator(Declarator::Variable(v))
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::DimensionsExpr {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, expr) if kind == &Type::DimensionsExpr => {
                        let i = match expr {
                            State::LiteralType(t) => sync!(t),
                            State::SimpleIdentifier(_, t) => scoped_ref!(mm!(), t),
                            State::ScopedIdentifier(i) => sync!(i),
                            State::FieldIdentifier(i) => sync!(i),
                            State::Invocation(i) => sync!(i),
                            State::ConstructorInvocation(i) => sync!(i),
                            x => panic!("{:?}", x),
                        };
                        State::ScopedIdentifier(i)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::GenericType {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(t)) if kind == &Type::GenericType => {
                        let t = scoped_type!(mm!(), t);
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::None, State::ScopedTypeIdentifier(t)) if kind == &Type::GenericType => {
                        let t = sync!(t);
                        State::ScopedTypeIdentifier(t)
                    }
                    // (State::SimpleTypeIdentifier(t), State::None) if kind == &Type::GenericType => {
                    //     let t = scoped_type!(mm!(), t);
                    //     // TODO use arguments
                    //     State::ScopedTypeIdentifier(t)
                    // }
                    // (State::SimpleTypeIdentifier(t), State::TypeArguments(_))
                    //     if kind == &Type::GenericType =>
                    // {
                    //     let t = scoped_type!(mm!(), t);
                    //     // TODO use arguments
                    //     State::ScopedTypeIdentifier(t)
                    // }
                    (State::ScopedTypeIdentifier(t), State::TypeArguments(_))
                        if kind == &Type::GenericType =>
                    {
                        // TODO use arguments
                        State::ScopedTypeIdentifier(t)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::TypeBound {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, rest) if kind == &Type::TypeBound => {
                        match rest {
                            State::SimpleTypeIdentifier(t) => {
                                let t = scoped_type!(mm!(), t);
                                State::TypeBound(t,Default::default())
                            },
                            State::ScopedTypeIdentifier(t) => {
                                let t = sync!(t);
                                State::TypeBound(t,Default::default())
                            },
                            State::None => State::None,
                            x => todo!("{:?} None {:?}", kind, x),
                        }
                    }
                    (val, rest) if kind == &Type::TypeBound => {
                        let (ext, mut imp) = match val {
                            State::TypeBound(ext,imp) => (ext,imp.to_vec()),
                            // State::ScopedTypeIdentifier(t) => (t,vec![]),
                            x => todo!("{:?} TypeBound {:?}", kind, x),
                        };
                        match rest {
                            State::SimpleTypeIdentifier(t) => {
                                let t = scoped_type!(mm!(), t);
                                imp.push(t);
                                State::TypeBound(ext,imp.into())
                            },
                            State::ScopedTypeIdentifier(t) => {
                                imp.push(sync!(t));
                                State::TypeBound(ext,imp.into())
                            },
                            State::None => State::TypeBound(ext,imp.into()),
                            x => todo!("{:?} TypeBound {:?}", kind, x),
                        }
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::Superclass {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::SimpleTypeIdentifier(t)) if kind == &Type::Superclass => {
                        let t = scoped_type!(mm!(), t);
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::None, State::ScopedTypeIdentifier(i)) if kind == &Type::Superclass => {
                        let i = sync!(i);
                        State::ScopedTypeIdentifier(i)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else if kind == &Type::SuperInterfaces || kind == &Type::ExtendsInterfaces {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (rest, State::ScopedTypeIdentifier(t))
                        if kind == &Type::SuperInterfaces || kind == &Type::ExtendsInterfaces =>
                    {
                        let mut v = match rest {
                            State::Interfaces(v) => v,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let t = sync!(t);
                        v.push(t);
                        State::Interfaces(v)
                    }
                    (rest, State::SimpleTypeIdentifier(t))
                        if kind == &Type::SuperInterfaces || kind == &Type::ExtendsInterfaces =>
                    {
                        let mut v = match rest {
                            State::Interfaces(v) => v,
                            State::None => vec![],
                            x => panic!("{:?}", x),
                        };
                        let t = scoped_type!(mm!(), t);
                        v.push(t);
                        State::Interfaces(v)
                    }
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            } else {
                match (acc.current_node.take(), current_node.map(|x| Old(x), |x| x)) {
                    (State::None, State::ScopedIdentifier(t))
                        if kind == &Type::ModuleDeclaration =>
                    {
                        State::None
                    }
                    (State::ScopedIdentifier(t), State::None)
                        if kind == &Type::ModuleDeclaration =>
                    {
                        State::None
                    }
                    (State::None, State::SimpleIdentifier(_, _))
                        if kind == &Type::ModuleDirective =>
                    {
                        State::None
                    }
                    (State::None, State::ScopedIdentifier(t)) if kind == &Type::ModuleDirective => {
                        State::None
                    }
                    (State::ScopedIdentifier(t), State::None) if kind == &Type::ModuleDirective => {
                        State::None
                    }
                    (State::None, State::None) if kind == &Type::ModuleBody => State::None,
                    (State::None, State::None) if kind == &Type::ModuleDeclaration => State::None,
                    (State::None, State::Annotation) if kind == &Type::AnnotatedType => State::None,
                    (State::None, State::SimpleTypeIdentifier(t))
                        if kind == &Type::AnnotatedType =>
                    {
                        let t = scoped_type!(mm!(), t);
                        State::ScopedTypeIdentifier(t)
                    }
                    (State::None, State::ScopedTypeIdentifier(t))
                        if kind == &Type::AnnotatedType =>
                    {
                        let t = sync!(t);
                        State::ScopedTypeIdentifier(t)
                    }
                    // (State::None, State::SimpleIdentifier(c,i)) if kind == &Type::TypeIdentifier => {
                    //     let o = mm!();
                    //     let i = acc.solver.intern_ref(RefsEnum::TypeIdentifier(o, i));
                    //     State::ScopedTypeIdentifier(i)
                    // },
                    (State::None, State::Modifiers(Visibility::None, n))
                        if kind == &Type::EnhancedForVariable && n.eq(&enum_set!()) =>
                    {
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: DeclType::Runtime(vec![].into_boxed_slice()),
                            identifier: Declarator::None,
                        }
                    }
                    (State::None, State::ScopedTypeIdentifier(t))
                        if kind == &Type::EnhancedForVariable =>
                    {
                        let t = sync!(t);
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: DeclType::Runtime(vec![t].into_boxed_slice()),
                            identifier: Declarator::None,
                        }
                    }
                    (State::None, State::SimpleTypeIdentifier(t))
                        if kind == &Type::EnhancedForVariable =>
                    {
                        let t = scoped_type!(mm!(), t);
                        State::Declaration {
                            visibility: Visibility::None,
                            kind: DeclType::Runtime(vec![t].into_boxed_slice()),
                            identifier: Declarator::None,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: _,
                            identifier: Declarator::None,
                        },
                        State::ScopedTypeIdentifier(t),
                    ) if kind == &Type::EnhancedForVariable => {
                        let t = sync!(t);
                        State::Declaration {
                            visibility,
                            kind: DeclType::Runtime(vec![t].into_boxed_slice()),
                            identifier: Declarator::None,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: _,
                            identifier: Declarator::None,
                        },
                        State::SimpleTypeIdentifier(t),
                    ) if kind == &Type::EnhancedForVariable => {
                        let t = scoped_type!(mm!(), t);
                        State::Declaration {
                            visibility,
                            kind: DeclType::Runtime(vec![t].into_boxed_slice()),
                            identifier: Declarator::None,
                        }
                    }
                    (
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: Declarator::None,
                        },
                        State::SimpleIdentifier(_, i),
                    ) if kind == &Type::EnhancedForVariable => {
                        let i = scoped_type!(mm!(), i);
                        State::Declaration {
                            visibility,
                            kind: t,
                            identifier: Declarator::Variable(i),
                        }
                    }
                    (
                        State::None,
                        _,//State::Modifiers(v,n),
                    ) if kind == &Type::RequiresModifier => {
                        State::None // TODO maybe something to do
                    }
                    (
                        State::None,
                        _,
                    ) if kind == &Type::ModuleDirective => {
                        State::None // TODO maybe something to do
                    }
                    (
                        State::None,
                        _,
                    ) if kind == &Type::ModuleDeclaration => {
                        State::None // TODO maybe something to do
                    }
                    (
                        State::None,
                        _,
                    ) if kind == &Type::RecordDeclaration => {
                        State::None // TODO maybe something to do
                    }
                    (
                        State::None,
                        _,//State::SimpleTypeIdentifier(_),
                    ) if kind == &Type::ReceiverParameter => {
                        State::None // TODO maybe something to do
                    } 
                    (x, y) => missing_rule!("{:?} {:?} {:?}", kind, x, y),
                }
            }
        };

        log::trace!("result for {:?} is {:?}", kind, acc.current_node);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Visibility {
    Public,
    Protected,
    Private,
    None,
}

#[derive(EnumSetType, Debug)]
pub enum NonVisibility {
    Static,
    Final,
    Abstract,
    Synchronized,
    Transient,
    Strictfp,
    Native,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum State<Node = LabelValue, Leaf = LabelValue>
where
    Leaf: std::cmp::Eq + std::hash::Hash,
    Node: std::cmp::Eq + std::hash::Hash,
{
    Todo,
    None,
    Asterisk,
    Super(Node),
    This(Node),
    Condition,
    Dimensions,
    Throws,
    Root,
    Annotation,
    Modifiers(Visibility, EnumSet<NonVisibility>),
    /// a
    SimpleIdentifier(IdentifierFormat, Leaf),
    /// A or A.B
    SimpleTypeIdentifier(Leaf),
    ScopedTypeIdentifier(Node),
    WildcardExtends(Node),
    WildcardSuper(Node),
    TypeBound(Node, Box<[Node]>),
    TypeParameters(Vec<DeclType<Node>>),
    GenericType(Node),
    CatchTypes(Vec<Node>),
    CatchParameter {
        kinds: Box<[Node]>,
        identifier: Leaf,
    },
    LiteralType(Node),
    ScopedIdentifier(Node),
    PackageDeclaration(Node),
    File {
        package: Option<Node>,
        asterisk_imports: Vec<Node>,
        global: Vec<(Declarator<Node>, DeclType<Node>)>,
        local: Vec<(Declarator<Node>, DeclType<Node>)>,
    },
    Directory {
        global_decls: Vec<(Declarator<Node>, DeclType<Node>)>,
    },
    Package {
        package: Option<Node>,
        global_decls: Vec<(Declarator<Node>, DeclType<Node>)>,
        package_local: Vec<(Declarator<Node>, DeclType<Node>)>,
    },
    /// b.f() or A.f()
    Invocation(Node),
    InvocationId(Node, Leaf),
    MethodReference(Node),
    LambdaExpression(Node),
    TypeArguments(Vec<Node>),
    Arguments(Vec<Node>),
    /// A#constructor()
    ConstructorInvocation(Node),
    ImportDeclaration {
        sstatic: bool,
        identifier: Node,
        asterisk: bool,
    },
    /// a.b
    FieldIdentifier(Node),
    Interfaces(Vec<Node>),
    ElementValuePair(Leaf, Node),
    Declarator(Declarator<Node>),
    Declaration {
        visibility: Visibility,
        kind: DeclType<Node>,
        identifier: Declarator<Node>,
    },
    MethodImplementation {
        visibility: Visibility,
        kind: Option<DeclType<Node>>,
        identifier: Option<Leaf>,
        parameters: Box<[(Node, DeclType<Node>)]>,
    },
    ConstructorImplementation {
        visibility: Visibility,
        identifier: Option<Leaf>,
        parameters: Box<[(Node, DeclType<Node>)]>,
    },
    TypeDeclaration {
        visibility: Visibility,
        identifier: DeclType<Node>,
        members: Vec<(Visibility, Declarator<Node>, DeclType<Node>)>,
    },
    Declarations(Vec<(Visibility, Declarator<Node>, DeclType<Node>)>),
    FormalParameters(Vec<(Node, DeclType<Node>)>),

    ///TODO use this to make further flow type static analysis, most of the time replace None
    TypeOfValue(Box<[Leaf]>),
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Argument<Node = LabelValue>
where
    Node: Eq + Hash,
{
    Type(Node),
    Identifier(Node),
}

impl<Node, Leaf> State<Node, Leaf>
where
    Leaf: std::cmp::Eq + std::hash::Hash + Copy,
    Node: std::cmp::Eq + std::hash::Hash + Copy,
{
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, State::None)
    }
    pub fn map<N, L, FN: FnMut(Node) -> N, FL: Fn(Leaf) -> L>(&self, mut f: FN, g: FL) -> State<N, L>
    where
        L: std::cmp::Eq + std::hash::Hash,
        N: std::cmp::Eq + std::hash::Hash,
    {
        match self {
            State::Todo => State::Todo,
            State::None => State::None,
            State::Asterisk => State::Asterisk,
            State::Condition => State::Condition,
            State::Dimensions => State::Dimensions,
            State::Throws => State::Throws,
            State::Root => State::Root,
            State::Annotation => State::Annotation,
            State::TypeBound(x,y) => State::TypeBound(f(*x), y.iter().map(|x| f(*x)).collect()),
            State::SimpleIdentifier(b, l) => State::SimpleIdentifier(*b, g(*l)),
            State::SimpleTypeIdentifier(l) => State::SimpleTypeIdentifier(g(*l)),

            State::Super(i) => State::Super(f(*i)),
            State::This(i) => State::This(f(*i)),
            State::ScopedTypeIdentifier(i) => State::ScopedTypeIdentifier(f(*i)),
            State::WildcardExtends(i) => State::WildcardExtends(f(*i)),
            State::WildcardSuper(i) => State::WildcardSuper(f(*i)),
            State::GenericType(i) => State::GenericType(f(*i)),
            State::LiteralType(i) => State::LiteralType(f(*i)),
            State::ScopedIdentifier(i) => State::ScopedIdentifier(f(*i)),
            State::PackageDeclaration(i) => State::PackageDeclaration(f(*i)),
            State::Invocation(i) => State::Invocation(f(*i)),
            State::MethodReference(i) => State::MethodReference(f(*i)),
            State::LambdaExpression(i) => State::LambdaExpression(f(*i)),
            State::ConstructorInvocation(i) => State::ConstructorInvocation(f(*i)),
            State::FieldIdentifier(i) => State::FieldIdentifier(f(*i)),
            State::Declarator(d) => State::Declarator(d.with_changed_node(|x| f(*x))),
            State::Interfaces(v) => State::Interfaces(v.iter().map(|x| f(*x)).collect()),
            State::Arguments(v) => State::Arguments(v.iter().map(|x| f(*x)).collect()),
            State::TypeArguments(v) => State::TypeArguments(v.iter().map(|x| f(*x)).collect()),
            State::CatchTypes(v) => State::CatchTypes(v.iter().map(|x| f(*x)).collect()),
            State::TypeParameters(v) => State::TypeParameters(
                v.iter().map(|x| x.map(|x|f(*x))).collect()
            ),
            State::Declarations(v) => State::Declarations(
                v.iter()
                    .map(|(v, x, y)| (*v, x.with_changed_node(|x| f(*x)), y.map(|x| f(*x))))
                    .collect(),
            ),
            State::FormalParameters(v) => {
                State::FormalParameters(v.iter().map(|(x, y)| (f(*x), y.map(|x| f(*x)))).collect())
            }
            State::TypeOfValue(_) => todo!(),
            State::ElementValuePair(x, y) => State::ElementValuePair(g(*x), f(*y)),
            State::InvocationId(x, y) => State::InvocationId(f(*x), g(*y)),
            State::Modifiers(x, y) => State::Modifiers(x.clone(), y.clone()),
            State::ImportDeclaration {
                sstatic,
                identifier: i,
                asterisk,
            } => State::ImportDeclaration {
                sstatic: *sstatic,
                identifier: f(*i),
                asterisk: *asterisk,
            },
            State::CatchParameter {
                kinds: v,
                identifier: i,
            } => State::CatchParameter {
                kinds: v.iter().map(|x| f(*x)).collect(),
                identifier: g(*i),
            },

            State::File {
                package: p,
                asterisk_imports,
                global: t,
                local: v,
            } => State::File {
                package: p.map(|x| f(x)),
                asterisk_imports: asterisk_imports.iter().map(|x| f(*x)).collect(),
                global: t
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), y.map(|x| f(*x))))
                    .collect(),
                local: v
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), y.map(|x| f(*x))))
                    .collect(),
            },

            State::Package {
                package: p,
                global_decls,
                package_local,
            } => State::Package {
                package: p.map(|x| f(x)),
                global_decls: global_decls
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), y.map(|x| f(*x))))
                    .collect(),
                package_local: package_local
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), y.map(|x| f(*x))))
                    .collect(),
            },
            State::Directory { global_decls } => State::Directory {
                global_decls: global_decls
                    .iter()
                    .map(|(x, y)| (x.with_changed_node(|x| f(*x)), y.map(|x| f(*x))))
                    .collect(),
            },
            State::Declaration {
                visibility,
                kind: t,
                identifier: d,
            } => State::Declaration {
                visibility: visibility.clone(),
                kind: t.map(|x| f(*x)),
                identifier: d.with_changed_node(|x| f(*x)),
            },
            State::MethodImplementation {
                visibility,
                kind: t,
                identifier: i,
                parameters: p,
            } => State::MethodImplementation {
                visibility: visibility.clone(),
                kind: t.clone().map(|x| x.map(|x| f(*x))),
                identifier: i.map(|x| g(x)),
                parameters: p.iter().map(|(x, y)| (f(*x), y.map(|x| f(*x)))).collect(),
            },
            State::ConstructorImplementation {
                visibility,
                identifier: i,
                parameters: p,
            } => State::ConstructorImplementation {
                visibility: visibility.clone(),
                identifier: i.map(|x| g(x)),
                parameters: p.iter().map(|(x, y)| (f(*x), y.map(|x| f(*x)))).collect(),
            },
            State::TypeDeclaration {
                visibility,
                identifier: d,
                members: v,
            } => State::TypeDeclaration {
                visibility: visibility.clone(),
                identifier: d.map(|x| f(*x)),
                members: v
                    .iter()
                    .map(|(v, x, y)| (*v, x.with_changed_node(|x| f(*x)), y.map(|x| f(*x))))
                    .collect(),
            },
        }
    }
}
