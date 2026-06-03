use super::*;

#[derive(Bundle)]
pub struct Token {
    pub ty: Type,
}

#[derive(Bundle)]
pub struct Leaf {
    pub ty: Type,
    pub label: Lab,
}

#[derive(Bundle)]
pub struct Node {
    pub ty: Type,
    pub cs: Children,
}

#[derive(Bundle)]
pub struct Dir {
    pub ty: Type,
    pub names: Names,
    pub cs: Children,
}

pub(crate) mod acc {
    use std::ops::Not;

    use string_interner::DefaultHashBuilder;

    use crate::types::LabelStore;

    use super::*;
    pub struct LabelAcc(pub Option<String>);
    enum LabelAcc2<'s> {
        None,
        Some(&'s str),
    }

    pub struct ChildrenAcc(pub Vec<IdN>);
    impl ChildrenAcc {
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }
    }

    type PrimaryAcc = (Type, LabelAcc, ChildrenAcc);
    type Primary = (Type, Option<Label>, Option<Children>);

    pub fn init(ty: Ty, l: Option<String>) -> PrimaryAcc {
        (Type(ty), LabelAcc(l), ChildrenAcc(vec![]))
    }

    pub fn acc(acc: &mut PrimaryAcc, child: IdN) {
        acc.2 .0.push(child);
    }

    pub fn finish(
        ty: Type,
        l: Option<IdL>,
        cs: ChildrenAcc,
    ) -> (Type, Option<Label>, Option<Children>) {
        (
            ty,
            l.map(Label),
            // acc.1 .0.map(|x| string_interner(&x)).map(Label),
            cs.0.is_empty()
                .not()
                .then(|| cs.0.into_boxed_slice())
                .map(Children),
        )
    }

    #[test]
    fn test_construction_no_dedup() {
        let mut string_interner = crate::store::labels::LabelStore::new();
        let it = BIN
            .into_iter()
            .map(|x| x.map(|x| x.to_string(), |x| string_interner.get_or_insert(*x)));

        let mut world = World::new();

        let x = construction(it, init, acc, |acc, l| {
            let t = finish(acc.0, l, acc.2);
            dbg!(&t);
            insert(&mut world, t).id()
        });
        dbg!(x);
    }

    #[test]
    fn test_construction_dedup_inneficient() {
        let mut string_interner = crate::store::labels::LabelStore::new();
        let it = BIN_DUP
            .into_iter()
            .map(|x| x.map(|x| x.to_string(), |x| string_interner.get_or_insert(*x)));

        let mut world = World::new();
        let mut dedup = std::collections::HashMap::<Primary, IdN>::new();

        let x = construction(it, init, acc, |acc, l| {
            let t: Primary = finish(acc.0, l, acc.2);
            if let Some(x) = dedup.get(&t) {
                *x
            } else {
                dbg!(&t);
                let k = t.clone();
                let x = insert(&mut world, t).id();
                dedup.insert(k, x);
                x
            }
        });
        dbg!(x);
    }

    #[test]
    fn test_construction_dedup_entry_raw() {
        let mut string_interner = crate::store::labels::LabelStore::new();
        let it = BIN_DUP
            .into_iter()
            .map(|x| x.map(|x| x.to_string(), |x| string_interner.get_or_insert(*x)));

        let backend = World::new();
        let dedup =
            hashbrown::HashMap::<_, (), ()>::with_capacity_and_hasher(1 << 10, Default::default());
        let builder = DefaultHashBuilder::default();

        let mut node_store = (backend, dedup, builder);

        let init = init;
        let acc = acc;
        let finish = finish;
        let insert = insert;

        let x = construction(it, init, acc, |acc: PrimaryAcc, l: Option<IdL>| {
            let t: Primary = finish(acc.0, l, acc.2);
            let eq = eq;
            let insert = insert;

            let backend = &mut node_store.0;
            let dedup = &mut node_store.1;
            let builder = &node_store.2;

            let hash = crate::utils::make_hash(builder, &t);
            let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
                let entity = backend.entity(*symbol);
                eq(entity, &t)
            });

            use hashbrown::hash_map::RawEntryMut::*;
            match entry {
                Occupied(occupied) => *occupied.key(),
                Vacant(vacant) => {
                    let symbol = insert(backend, t).id();
                    vacant.insert_with_hasher(hash, symbol, (), |id| {
                        let entity = backend.entity(*id);
                        let node = (
                            entity
                                .get::<Type>()
                                .expect("all nodes in the HyperAST should have a type"),
                            entity.get::<Label>(),
                            entity.get::<Children>(),
                        );
                        crate::utils::make_hash(builder, &node)
                    });
                    symbol
                }
            }
        });
        dbg!(x);
    }

    pub trait PrimaryHolder {
        fn ty(&self) -> Option<&Type>;
        fn label(&self) -> Option<&Label>;
        fn children(&self) -> Option<&Children>;
    }

    impl PrimaryHolder for EntityRef<'_> {
        fn ty(&self) -> Option<&Type> {
            self.get::<Type>()
        }

        fn label(&self) -> Option<&Label> {
            self.get::<Label>()
        }

        fn children(&self) -> Option<&Children> {
            self.get::<Children>()
        }
    }

    pub fn eq(entity: impl PrimaryHolder, t: &Primary) -> bool {
        let kind = &t.0;
        let label_id = t.1.as_ref();
        let children = &t.2;

        let t = entity.ty();
        if t != Some(kind) {
            return false;
        }
        let l = entity.label();
        if l != label_id {
            return false;
        }
        let cs = entity.children();
        // TODO benchmark, probably no big difference
        if true {
            let cs = cs.iter().flat_map(|x| x.0.iter());
            let children = children.iter().flat_map(|x| x.0.iter());
            cs.eq(children)
        } else {
            let cs = cs.map_or(Default::default(), |x| x.0.as_ref());
            let children = children
                .as_ref()
                .map_or(Default::default(), |x| x.0.as_ref());
            cs.eq(children)
        }
    }

    pub fn insert(
        backend: &mut World,
        t: (Type, Option<Label>, Option<Children>),
    ) -> EntityWorldMut<'_> {
        let mut e = backend.spawn(t.0);
        if let Some(l) = t.1 {
            e.insert(l);
        }
        if let Some(cs) = t.2 {
            e.insert(cs);
        }
        e
    }

    pub fn fn_zip0<A, B>(
        f: impl Fn() -> A + 'static,
        g: impl Fn() -> B + 'static,
    ) -> impl Fn() -> (A, B) {
        move || {
            let a = f();
            let b = g();
            (a, b)
        }
    }
    pub fn fn_zip1<X, A, B>(
        f: impl Fn(X) -> A + 'static,
        g: impl Fn() -> B + 'static,
    ) -> impl Fn(X) -> (A, B) {
        move |x| {
            let a = f(x);
            let b = g();
            (a, b)
        }
    }
    pub fn fn_zip11<X, Y, A, B>(
        f: impl Fn(X, Y) -> A + 'static,
        g: impl Fn() -> B + 'static,
    ) -> impl Fn(X, Y) -> (A, B) {
        move |x, y| {
            let a = f(x, y);
            let b = g();
            (a, b)
        }
    }
    pub fn fn_zip12<X, Y, Z, A, B>(
        f: impl Fn(X) -> A + 'static,
        g: impl Fn(Y, (Z, &A)) -> B,
    ) -> impl Fn((X, Y), Z) -> (A, B) {
        move |(x, y), z| {
            let a = f(x);
            let b = g(y, (z, &a));
            (a, b)
        }
    }
    pub fn fn_zip2<X, Y, A, B>(
        f: impl Fn(X, Y) -> A + 'static,
        g: impl Fn(&A) -> B + 'static,
    ) -> impl Fn(X, Y) -> (A, B) {
        move |x, y| {
            let a = f(x, y);
            let b = g(&a);
            (a, b)
        }
    }
    pub fn fn_zip3<X, Y, A, B>(
        f: impl Fn(&mut A, &X) + 'static,
        g: impl Fn(&mut B, &Y) + 'static,
    ) -> impl Fn(&mut (A, B), &(X, Y)) {
        move |a, x| {
            f(&mut a.0, &x.0);
            g(&mut a.1, &x.1);
        }
    }
    pub fn fn_zip3012<X, A, B>(
        f: impl Fn(&mut A) + 'static,
        g: impl Fn(&mut B, X, &A) + 'static,
    ) -> impl Fn(&mut (A, B), X) {
        move |(a, b), x| {
            f(a);
            g(b, x, a);
        }
    }

    // | primary | md 0     md 1     md2    |
    // | data 0  | data 1 | data 2 | data 3 |
    // |  always compute  | if abs & cachd  |
    // | eg. ty  |eg. hash| byt_l  | refs
    //                               lossy
    //            for h <-|- size

    #[test]
    fn test_construction_dedup_entry_raw_md0() {
        let mut string_interner = crate::store::labels::LabelStore::new();
        let it = BIN_DUP
            .into_iter()
            .map(|x| x.map(|x| x.to_string(), |x| string_interner.get_or_insert(*x)));

        let backend = World::new();
        let dedup =
            hashbrown::HashMap::<_, (), ()>::with_capacity_and_hasher(1 << 10, Default::default());
        let builder = DefaultHashBuilder::default();

        let mut node_store = (backend, dedup, builder);

        // [`md_sys_byte_len`];
        let init = init;
        let init = fn_zip11(
            init,
            fn_zip0(md_sys_tree_size::init, md_sys_tree_hash::init),
        );

        let acc = |a: &mut (_, (_, _)), child: (IdN, (_, _))| {
            fn_zip3(
                |a, x| acc(a, *x),
                fn_zip3(md_sys_tree_size::acc, md_sys_tree_hash::acc),
            )(a, &child);

            // acc(&mut a.0, child.0);
            // fn_zip3(md_sys_tree_size::acc, md_sys_tree_hash::acc)(&mut a.1, &child.1);
            // // md_sys_tree_size::acc(&mut a.1 .0, &child.1 .0);
            // // md_sys_tree_hash::acc(&mut a.1 .1, &child.1 .1);
        };

        let finish = |acc: (PrimaryAcc, (_, _)), l| {
            let ty_lab = (&acc.0 .0, acc.0 .1 .0.as_ref());
            let md = fn_zip12(md_sys_tree_size::finish, |a, (x, s): (_, _)| {
                md_sys_tree_hash::finish(a, x , s)
            })(acc.1, ty_lab);
            (finish(acc.0.0, l, acc.0.2), md)
            // let s = md_sys_tree_size::finish(acc.1 .0);
            // let h_aux = (var_name.0, var_name.1, &s);
            // let h = md_sys_tree_hash::finish(acc.1 .1, h_aux);
            // (finish(acc.0, l), (s, h))
        };
        let insert = |backend: &mut World, t, md0| insert(backend, t).insert(md0).id();

        let x = construction(it, init, acc, |acc: _, l: Option<IdL>| {
            let (t, md0) = finish(acc, l);
            let eq = eq;

            let backend = &mut node_store.0;
            let dedup = &mut node_store.1;
            let builder = &node_store.2;

            let hash = md0.1.to_u64(); //crate::utils::make_hash(builder, &t);
            let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
                let entity = backend.entity(*symbol);
                eq(entity, &t)
            });

            use hashbrown::hash_map::RawEntryMut::*;
            match entry {
                Occupied(occupied) => (*occupied.key(), md0),
                Vacant(vacant) => {
                    let symbol = insert(backend, t, md0.clone());
                    // let symbol = insert(backend, t).insert(md0.clone()).id();
                    vacant.insert_with_hasher(hash, symbol, (), |id| {
                        let entity = backend.entity(*id);
                        entity.get::<md_sys_tree_hash::TreeHash>().unwrap().to_u64()
                    });
                    (symbol, md0)
                }
            }
        });
        dbg!(x);
    }

    fn construction_save<IdN, T, Acc, L, IdL>(
        mut it: impl Iterator<Item = Traversal<IdL, L>>,
        mut interner: impl FnMut(T) -> IdN,
        init: impl Fn(Ty, Option<L>) -> Acc,
        acc: impl Fn(&mut Acc, IdN),
        finish: impl Fn(Acc, Option<IdL>) -> T,
    ) -> IdN {
        let mut stack: Vec<(Acc,)> = vec![];

        match it.next().unwrap() {
            Traversal::Down(ty, l) => {
                stack.push((init(ty, l),));
            }
            _ => unreachable!(),
        }

        match it.next().unwrap() {
            Traversal::Down(ty, l) => {
                stack.push((init(ty, l),));
            }
            _ => unreachable!(),
        }

        match it.next().unwrap() {
            Traversal::Right(ty, l, idl) => {
                let c = stack.pop().unwrap();
                let node = finish(c.0, idl);
                let id = interner(node);
                let c = stack.last_mut().unwrap();
                acc(&mut c.0, id);
                stack.push((init(ty, l),));
            }
            _ => unreachable!(),
        }

        match it.next().unwrap() {
            Traversal::Right(ty, l, idl) => {
                let c = stack.pop().unwrap();
                let node = finish(c.0, idl);
                let id = interner(node);
                let c = stack.last_mut().unwrap();
                acc(&mut c.0, id);
                stack.push((init(ty, l),));
            }
            _ => unreachable!(),
        }

        match it.next().unwrap() {
            Traversal::Up(idl) => {
                let c = stack.pop().unwrap();
                let node = finish(c.0, idl);
                let id = interner(node);
                let c = stack.last_mut().unwrap();
                acc(&mut c.0, id);
            }
            _ => unreachable!(),
        }

        match it.next().unwrap() {
            Traversal::Up(idl) => {
                let c = stack.pop().unwrap();
                let node = finish(c.0, idl);
                let id = interner(node);
                if let Some(_) = stack.last_mut() {
                    unreachable!()
                } else {
                    return id;
                }
            }
            _ => unreachable!(),
        }

        // match it.next().unwrap() {
        //     Traversal::Down(ty, l) => todo!(),
        //     Traversal::Right(ty, l, idl) => todo!(),
        //     Traversal::Up(idl) => todo!(),
        // }
    }
}
