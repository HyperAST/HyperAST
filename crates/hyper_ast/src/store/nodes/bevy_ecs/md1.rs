//! Metadata 1:
//! Derived data that is computed only when we find a new node.

use super::primary::acc::*;
use super::*;
use crate::types::LabelStore as _;
use string_interner::DefaultHashBuilder;

pub type Data<P, Md0, Md1> = (P, Md0, Md1);

pub fn fn_zip_data_init<P, Md0, Md1, X, Y>(
    primary: impl Fn(X, Y) -> P + 'static,
    f_md0: impl Fn() -> Md0 + 'static,
    f_md1: impl Fn() -> Md1 + 'static,
) -> impl Fn(X, Y) -> Data<P, Md0, Md1> {
    move |x, y| {
        let p = primary(x, y);
        let md0 = f_md0();
        let md1 = f_md1();
        (p, md0, md1)
    }
}

pub fn fn_zip_data_acc<P, Md0, Md1, X, Y, Z>(
    primary: impl Fn(&mut P, &X) + 'static,
    f_md0: impl Fn(&mut Md0, &Y) + 'static,
    f_md1: impl Fn(&mut Md1, &Z) + 'static,
) -> impl Fn(&mut Data<P, Md0, Md1>, (X, Y, Z)) {
    move |a, x| {
        primary(&mut a.0, &x.0);
        f_md0(&mut a.1, &x.1);
        f_md1(&mut a.2, &x.2);
    }
}

pub trait ChildHolder {
    fn is_empty(&self) -> bool;
}

impl ChildHolder for ChildrenAcc {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub fn _fn_zip_data_finish<Ty: Clone, L, CS: ChildHolder, IdL, Md0, Md1, _P, _Md0, _Md1>(
    primary: impl Fn(Ty, IdL, CS) -> _P + 'static,
    f_md0: impl Fn(Md0, (Ty, &L, bool)) -> _Md0 + 'static,
    f_md1: impl Fn(Md1, (Ty, &L, bool)) -> _Md1 + 'static,
) -> impl Fn(Data<(Ty, L, CS), Md0, Md1>, IdL) -> Data<_P, _Md0, _Md1> {
    move |a, x| {
        let b = &a.0;
        let md0 = f_md0(a.1, prep_prim(b));
        let md1 = f_md1(a.2, prep_prim(b));
        (primary(a.0 .0, x, a.0 .2), md0, md1)
    }
}

pub fn fn_zip_data_finish<Ty: Clone, L, CS: ChildHolder, IdL, Md0, Md1, _P, _Md0, _Md1>(
    primary: impl Fn(Ty, IdL, CS) -> _P + 'static,
    f_md0: impl Fn(Md0, (Ty, &L, bool)) -> _Md0 + 'static,
    f_md1: impl Fn(Md1, (Ty, &L, bool)) -> _Md1 + 'static,
) -> (
    impl Fn((Ty, L, CS), Md0, IdL) -> (_P, _Md0, (Ty, L, bool)),
    impl Fn((Ty, L, bool), Md1) -> _Md1,
) {
    (
        move |a, md0, x| {
            let md0 = f_md0(md0, (a.0.clone(), &a.1, a.2.is_empty()));
            let b = (a.0.clone(), a.1, a.2.is_empty());
            (primary(a.0, x, a.2), md0, b)
        },
        move |a, x| f_md1(x, (a.0, &a.1, a.2)),
    )
}

fn prep_prim<Ty: Clone, L, CS: ChildHolder>(b: &(Ty, L, CS)) -> (Ty, &L, bool) {
    (b.0.clone(), &b.1, b.2.is_empty())
}

#[test]
fn test_construction_dedup_entry_raw_md1() {
    let mut string_interner = crate::store::labels::LabelStore::new();
    let it = BIN_DUP
        .into_iter()
        .map(|x| x.map(|x| x.to_string(), |x| string_interner.get_or_insert(*x)));

    let backend = World::new();
    let dedup =
        hashbrown::HashMap::<_, (), ()>::with_capacity_and_hasher(1 << 10, Default::default());
    let builder = DefaultHashBuilder::default();

    let mut node_store = (backend, dedup, builder);

    let init = fn_zip_data_init(
        init,
        fn_zip0(md_sys_tree_size::init, md_sys_tree_hash::init),
        md_sys_byte_len::init,
    );

    let acc = fn_zip_data_acc(
        |a, x| acc(a, *x),
        fn_zip3(md_sys_tree_size::acc, md_sys_tree_hash::acc),
        md_sys_byte_len::acc,
    );

    trait To {
        type Target;
        fn to(&self) -> Self::Target;
    }

    // impl<A,B,C,D, F: Fn(A,B,C) -> D> To for F {
    //     type Target;

    //     fn to(&self) -> Self::Target {
    //         todo!()
    //     }
    // }

    fn fn_aaa<A, B, C, R>(f: impl Fn(A, B, &C) -> R + 'static) -> impl Fn(A, (B, &C)) -> R {
        move |a, (b, c)| f(a, b, c)
    }
    fn fn_bbb<A, B, D, E>(
        f: impl Fn(A, (&B, Option<&String>)) -> E + 'static,
    ) -> impl Fn(A, (B, &LabelAcc, D)) -> E {
        move |a, (b, c, d)| f(a, (&b, c.0.as_ref()))
    }

    pub fn fn_zip12<A, B, Z, AA, BB>(
        f: impl Fn(A) -> AA + 'static,
        g: impl Fn(B, (Z, &AA)) -> BB,
    ) -> impl Fn((A, B), Z) -> (AA, BB) {
        move |(x, y), z| {
            let a = f(x);
            let b = g(y, (z, &a));
            (a, b)
        }
    }

    // See explanation (1).
    trait KeyPair<A, B> {
        /// Obtains the first element of the pair.
        fn a(&self) -> &A;
        /// Obtains the second element of the pair.
        fn b(&self) -> &B;
        fn ab(&self) -> (&A, &B) {
            (self.a(), self.b())
        }
    }

    impl<A, B, C> KeyPair<A, B> for (A, B, C) {
        fn a(&self) -> &A {
            &self.0
        }
        fn b(&self) -> &B {
            &self.1
        }
        fn ab(&self) -> (&A, &B) {
            (self.a(), self.b())
        }
    }
    impl<A, B, C> KeyPair<A, B> for (&A, &B, C) {
        fn a(&self) -> &A {
            self.0
        }
        fn b(&self) -> &B {
            self.1
        }
        fn ab(&self) -> (&A, &B) {
            (self.a(), self.b())
        }
    }

    trait Tp<T> {
        type T1;
        type T2;
        type T3;
    }

    trait Simp {
        type T1;
        type T2;
        type T3;
        fn t1(&self) -> &Self::T1;
    }

    // impl<T, U, V> Simp<(T, U)> for (T, U, V) {
    //     fn simp(&self) -> (T, U) {
    //         todo!()
    //         // (&self.0, &self.1)
    //     }
    // }

    // fn fn_ccc<A, I: Simp<J>, J, B, R>(
    //     f: impl Fn(A, J, &B) -> R + 'static,
    // ) -> impl Fn(A, (I, &B)) -> R {
    //     move |a, (i, b)| f(a, i.simp(), &b)
    // }

    let (finish_on_dedup, finish_on_absent) = fn_zip_data_finish(
        finish,
        // fn_zip12(
        //     md_sys_tree_size::finish,
        //     fn_ccc(
        //         md_sys_tree_hash::finish,
        //         // |(ty, l, _): (_, &LabelAcc, _)| (&ty, l.0.as_ref())
        //     ),
        // ),
        |acc, prim: (_, &LabelAcc, _)| {
            fn_zip12(md_sys_tree_size::finish, fn_aaa(md_sys_tree_hash::finish))(
                acc,
                (&prim.0, prim.1 .0.as_ref()),
            )
        },
        |acc, prim| md_sys_byte_len::finish(acc, (&prim.0, prim.1 .0.as_ref(), !prim.2)),
    );

    let insert = |backend: &mut World, t, md0, md1| insert(backend, t).insert((md0, md1)).id();

    let _eq = |backend: &mut World, symbol, t: &_| eq(backend.entity(symbol), t);
    let _hash = |backend: &World, symbol| {
        backend
            .entity(symbol)
            .get::<md_sys_tree_hash::TreeHash>()
            .unwrap()
            .to_u64()
    };
    let x = construction(it, init, acc, |acc: _, l: Option<IdL>| {
        let (t, md0, t_bis) = finish_on_dedup(acc.0, acc.1, l);

        let backend = &mut node_store.0;
        let dedup = &mut node_store.1;

        let hash = md0.1.to_u64();
        let entry = dedup
            .raw_entry_mut()
            .from_hash(hash, |symbol| _eq(backend, *symbol, &t));

        use hashbrown::hash_map::RawEntryMut::*;
        match entry {
            // Occupied(occupied) => (*occupied.key(), md0, todo!("md1")),
            Occupied(occupied) => (
                *occupied.key(),
                md0,
                *backend.entity(*occupied.key()).get().unwrap(),
            ),
            Vacant(vacant) => {
                let md1 = finish_on_absent(t_bis, acc.2);
                let symbol = insert(backend, t, md0.clone(), md1.clone());
                vacant.insert_with_hasher(hash, symbol, (), |symbol| _hash(backend, *symbol));
                (symbol, md0, md1)
            }
        }
    });
    dbg!(x);
}

mod exp_optics {

    /// base optic
    pub trait Optic<Optics> {
        type Image: ?Sized;
    }

    /// Review
    pub trait Review<Optics>: Optic<Optics> {
        fn review(optics: Optics, from: Self::Image) -> Self
        where
            Self::Image: Sized;
    }

    /// Traversal
    pub trait TraversalRef<Optics>: Optic<Optics> {
        fn traverse_ref(&self, optics: Optics) -> Vec<&Self::Image>;
    }

    pub trait TraversalMut<Optics>: TraversalRef<Optics> {
        fn traverse_mut(&mut self, optics: Optics) -> Vec<&mut Self::Image>;
    }

    pub trait Traversal<Optics>: TraversalMut<Optics> {
        fn traverse(self, optics: Optics) -> Vec<Self::Image>
        where
            Self::Image: Sized;
    }

    /// Prism
    pub trait PrismRef<Optics>: TraversalRef<Optics> {
        fn preview_ref(&self, optics: Optics) -> Option<&Self::Image>;
    }

    pub trait PrismMut<Optics>: PrismRef<Optics> + TraversalMut<Optics> {
        fn preview_mut(&mut self, optics: Optics) -> Option<&mut Self::Image>;
    }

    pub trait Prism<Optics>: PrismMut<Optics> + Traversal<Optics> {
        fn preview(self, optics: Optics) -> Option<Self::Image>
        where
            Self::Image: Sized;
    }

    /// Lens
    pub trait LensRef<Optics>: PrismRef<Optics> {
        fn view_ref(&self, optics: Optics) -> &Self::Image;
    }

    pub trait LensMut<Optics>: LensRef<Optics> + PrismMut<Optics> {
        fn view_mut(&mut self, optics: Optics) -> &mut Self::Image;
    }

    pub trait Lens<Optics>: LensMut<Optics> + Prism<Optics> {
        fn view(self, optics: Optics) -> Self::Image
        where
            Self::Image: Sized;
    }

    mod optics {
        pub struct Current;

        pub mod option {
            pub struct Some<Optics = super::Current>(pub Optics);
        }
    }

    impl Optic<optics::Current> for i32 {
        type Image = i32;
    }

    impl TraversalRef<optics::Current> for i32 {
        fn traverse_ref(&self, _: optics::Current) -> Vec<&Self::Image> {
            unimplemented!("see prism")
        }
    }

    impl PrismRef<optics::Current> for i32 {
        fn preview_ref(&self, _: optics::Current) -> Option<&Self::Image> {
            Some(self)
        }
    }

    impl<T> Optic<optics::Current> for Option<T> {
        type Image = Self;
    }

    impl<T> TraversalRef<optics::Current> for Option<T> {
        fn traverse_ref(&self, _: optics::Current) -> Vec<&Self::Image> {
            unimplemented!("see prism")
        }
    }

    impl<T> PrismRef<optics::Current> for Option<T> {
        fn preview_ref(&self, _: optics::Current) -> Option<&Self::Image> {
            Some(self)
        }
    }

    impl<Optics, T: PrismRef<Optics>> Optic<optics::option::Some<Optics>> for Option<T> {
        type Image = T::Image;
    }

    impl<Optics, T: PrismRef<Optics>> TraversalRef<optics::option::Some<Optics>> for Option<T> {
        fn traverse_ref(&self, optics: optics::option::Some<Optics>) -> Vec<&Self::Image> {
            unimplemented!("see prism")
        }
    }

    impl<Optics, T: PrismRef<Optics>> PrismRef<optics::option::Some<Optics>> for Option<T> {
        fn preview_ref(&self, optics: optics::option::Some<Optics>) -> Option<&Self::Image> {
            if let (Some(x), optics::option::Some(o)) = (self, optics) {
                x.preview_ref(o)
            } else {
                None
            }
        }
    }

    fn print_i32<Pm, T: PrismRef<Pm, Image = i32>>(t: &T, pm: Pm) {
        //                              ^ `T` may have a value of `i32`
        let x = t.preview_ref(pm);
        dbg!(x);
    }
    fn print_option_i32<Pm, T: PrismRef<Pm, Image = Option<i32>>>(t: &T, pm: Pm) {
        //                              ^ `T` may have a value of `Option<i32>`
        let x = t.preview_ref(pm);
        dbg!(x);
    }

    #[test]
    fn test_option() {
        let x = Some(42);
        print_i32(&x, optics::option::Some(optics::Current));
        print_option_i32(&x, optics::Current);

        // fff(&x, optics::Current);
        // type mismatch resolving `<Option<i32> as Optic<Current>>::Image == i32`
        // expected type `i32`
        // found enum `std::option::Option<i32>`
    }

    // fn may_have_i32<Pm, T: PrismMut<Pm, Image = i32>>(t: &mut T, pm: Pm) {
    //     //                       ^ `T` may have a value of `i32`
    //     t.preview_mut(pm).map(|x| *x += 1);
    // }
}

mod exp_optics2 {

    pub trait Optic<Optics> {
        type Image: ?Sized;
    }

    /// Review
    pub trait Review<Optics>: Optic<Optics> {
        fn review(optics: Optics, from: Self::Image) -> Self
        where
            Self::Image: Sized;
    }

    /// Prism
    pub trait PrismRef<Optics>: Optic<Optics> {
        fn preview_ref(&self, optics: Optics) -> Option<&Self::Image>;
    }

    pub trait PrismMut<Optics>: PrismRef<Optics> + Optic<Optics> {
        fn preview_mut(&mut self, optics: Optics) -> Option<&mut Self::Image>;
    }

    pub trait Prism<Optics>: PrismMut<Optics> + Optic<Optics> {
        fn preview(self, optics: Optics) -> Option<Self::Image>
        where
            Self::Image: Sized;
    }

    /// Lens
    pub trait LensRef<Optics>: PrismRef<Optics> {
        fn view_ref(&self, optics: Optics) -> &Self::Image;
    }

    pub trait LensMut<Optics>: LensRef<Optics> + PrismMut<Optics> {
        fn view_mut(&mut self, optics: Optics) -> &mut Self::Image;
    }

    pub trait Lens<Optics>: LensMut<Optics> + Prism<Optics> {
        fn view(self, optics: Optics) -> Self::Image
        where
            Self::Image: Sized;
    }

    mod optics {
        pub struct Current;

        pub mod option {
            pub struct Some<Optics = super::Current>(pub Optics);
            impl<O> Some<O> {
                fn c(o: O) -> Self {
                    Self(o)
                }
                // fn s() -> impl Fn(O) -> Self {
                //     |o| { Self(o) }
                // }
            }
            trait Pipe {
                fn p<O>(self, o: impl Fn(Self) -> O) -> O
                where
                    Self: Sized;
            }
            impl Pipe for super::Current {
                fn p<O>(self, o: impl Fn(Self) -> O) -> O
                where
                    Self: Sized,
                {
                    o(self)
                }
            }
            impl<OO> Pipe for Some<OO> {
                fn p<O>(self, o: impl Fn(Self) -> O) -> O
                where
                    Self: Sized,
                {
                    o(self)
                }
            }

            // impl<O> Some<O> {
            //     fn s<OO, F: Fn(OO) -> Some<OO>>(f: F) -> impl Fn(O) -> Some<O> {
            //         |o| { Some(f(o)) }
            //     }
            // }
            #[test]
            fn test_chain() {
                Some::c(Some::c(Some(super::Current)));
                // Some::c(Some)::c(Some);
                let optic = super::Current.p(Some).p(Some);
                // Some::s()(Some::s()(super::Current));
                // Some::s(Some::s)(Some::s)(super::Current);
            }
        }

        pub mod positional {
            pub struct _0<Optics = super::Current>(pub Optics);
            pub struct _1<Optics = super::Current>(pub Optics);
            pub struct _2<Optics = super::Current>(pub Optics);
            pub struct _3<Optics = super::Current>(pub Optics);
            pub struct _4<Optics = super::Current>(pub Optics);
            pub struct _5<Optics = super::Current>(pub Optics);
            pub struct _6<Optics = super::Current>(pub Optics);
            pub struct _7<Optics = super::Current>(pub Optics);
        }

        pub mod result {
            pub struct Ok<Optics = super::Current>(pub Optics);
            pub struct Err<Optics = super::Current>(pub Optics);
        }
    }

    fn print_i32<Pm, T: PrismRef<Pm, Image = i32>>(t: &T, pm: Pm) {
        //                              ^ `T` may have a value of `i32`
        let x = t.preview_ref(pm);
        dbg!(x);
    }
    fn print_option_i32<Pm, T: PrismRef<Pm, Image = Option<i32>>>(t: &T, pm: Pm) {
        //                              ^ `T` may have a value of `Option<i32>`
        let x = t.preview_ref(pm);
        dbg!(x);
    }

    #[test]
    fn test_option() {
        let x = Some(42);
        print_i32(&x, optics::option::Some(optics::Current));
        print_option_i32(&x, optics::Current);

        // fff(&x, optics::Current);
        // type mismatch resolving `<Option<i32> as Optic<Current>>::Image == i32`
        // expected type `i32`
        // found enum `std::option::Option<i32>`
    }

    // fn may_have_i32<Pm, T: PrismMut<Pm, Image = i32>>(t: &mut T, pm: Pm) {
    //     //                       ^ `T` may have a value of `i32`
    //     t.preview_mut(pm).map(|x| *x += 1);
    // }

    #[derive(Debug)]
    struct MyMd1(i32);

    fn fff<Pm, T: PrismRef<Pm, Image = MyMd1>>(t: &T, pm: Pm) {
        //                              ^ `T` may have a value of `i32`
        let x = t.preview_ref(pm);
        dbg!(x);
    }

    // # current

    impl Optic<optics::Current> for () {
        type Image = Self;
    }

    impl PrismRef<optics::Current> for () {
        fn preview_ref(&self, _: optics::Current) -> Option<&Self::Image> {
            Some(self)
        }
    }

    impl Review<optics::Current> for () {
        fn review(_: optics::Current, from: Self::Image) -> Self
        where
            Self::Image: Sized,
        {
            from
        }
    }

    impl Optic<optics::Current> for i32 {
        type Image = Self;
    }

    impl PrismRef<optics::Current> for i32 {
        fn preview_ref(&self, _: optics::Current) -> Option<&Self::Image> {
            Some(self)
        }
    }

    impl Review<optics::Current> for i32 {
        fn review(_: optics::Current, from: Self::Image) -> Self
        where
            Self::Image: Sized,
        {
            from
        }
    }

    impl<T> Optic<optics::Current> for Option<T> {
        type Image = Self;
    }

    impl<T> PrismRef<optics::Current> for Option<T> {
        fn preview_ref(&self, _: optics::Current) -> Option<&Self::Image> {
            Some(self)
        }
    }

    impl<T> Review<optics::Current> for Option<T> {
        fn review(_: optics::Current, from: Self::Image) -> Self
        where
            Self::Image: Sized,
        {
            from
        }
    }

    impl<T, E> Optic<optics::Current> for Result<T, E> {
        type Image = Self;
    }

    impl<T, E> PrismRef<optics::Current> for Result<T, E> {
        fn preview_ref(&self, _: optics::Current) -> Option<&Self::Image> {
            Some(self)
        }
    }

    impl<T, E> Review<optics::Current> for Result<T, E> {
        fn review(_: optics::Current, from: Self::Image) -> Self
        where
            Self::Image: Sized,
        {
            from
        }
    }

    impl<T> Optic<optics::Current> for (T,) {
        type Image = Self;
    }

    impl<T> PrismRef<optics::Current> for (T,) {
        fn preview_ref(&self, _: optics::Current) -> Option<&Self::Image> {
            Some(self)
        }
    }

    impl<T> Review<optics::Current> for (T,) {
        fn review(_: optics::Current, from: Self::Image) -> Self
        where
            Self::Image: Sized,
        {
            from
        }
    }

    impl<T, E> Optic<optics::Current> for (T, E) {
        type Image = Self;
    }

    impl<T, E> PrismRef<optics::Current> for (T, E) {
        fn preview_ref(&self, _: optics::Current) -> Option<&Self::Image> {
            Some(self)
        }
    }

    impl<T, E> Review<optics::Current> for (T, E) {
        fn review(_: optics::Current, from: Self::Image) -> Self
        where
            Self::Image: Sized,
        {
            from
        }
    }

    // # Some

    impl<Optics, T: Optic<Optics>> Optic<optics::option::Some<Optics>> for Option<T> {
        type Image = T::Image;
    }

    impl<Optics, T: PrismRef<Optics>> PrismRef<optics::option::Some<Optics>> for Option<T> {
        fn preview_ref(&self, optics: optics::option::Some<Optics>) -> Option<&Self::Image> {
            if let (Some(x), optics::option::Some(o)) = (self, optics) {
                x.preview_ref(o)
            } else {
                None
            }
        }
    }

    impl<Optics, T: Review<Optics>> Review<optics::option::Some<Optics>> for Option<T> {
        fn review(
            optics::option::Some(optics): optics::option::Some<Optics>,
            from: Self::Image,
        ) -> Self
        where
            Self::Image: Sized,
        {
            Some(Review::review(optics, from))
        }
    }

    // # Ok

    impl<Optics, T: Optic<Optics>, E> Optic<optics::result::Ok<Optics>> for Result<T, E> {
        type Image = T::Image;
    }

    impl<Optics, T: PrismRef<Optics>, E> PrismRef<optics::result::Ok<Optics>> for Result<T, E> {
        fn preview_ref(&self, optics: optics::result::Ok<Optics>) -> Option<&Self::Image> {
            if let (Ok(x), optics::result::Ok(o)) = (self, optics) {
                x.preview_ref(o)
            } else {
                None
            }
        }
    }

    impl<Optics, T: Review<Optics>, E> Review<optics::result::Ok<Optics>> for Result<T, E> {
        fn review(optics::result::Ok(optics): optics::result::Ok<Optics>, from: Self::Image) -> Self
        where
            Self::Image: Sized,
        {
            Ok(Review::review(optics, from))
        }
    }

    // # Err

    impl<Optics, T, E: Optic<Optics>> Optic<optics::result::Err<Optics>> for Result<T, E> {
        type Image = E::Image;
    }

    impl<Optics, T, E: PrismRef<Optics>> PrismRef<optics::result::Err<Optics>> for Result<T, E> {
        fn preview_ref(&self, optics: optics::result::Err<Optics>) -> Option<&Self::Image> {
            if let (Err(x), optics::result::Err(o)) = (self, optics) {
                x.preview_ref(o)
            } else {
                None
            }
        }
    }

    impl<Optics, T, E: Review<Optics>> Review<optics::result::Err<Optics>> for Result<T, E> {
        fn review(
            optics::result::Err(optics): optics::result::Err<Optics>,
            from: Self::Image,
        ) -> Self
        where
            Self::Image: Sized,
        {
            Err(Review::review(optics, from))
        }
    }

    // # _0

    impl<Optics, T: Optic<Optics>> Optic<optics::positional::_0<Optics>> for (T,) {
        type Image = T::Image;
    }

    impl<Optics, T: PrismRef<Optics>> PrismRef<optics::positional::_0<Optics>> for (T,) {
        fn preview_ref(&self, optics: optics::positional::_0<Optics>) -> Option<&Self::Image> {
            let ((x,), optics::positional::_0(o)) = (self, optics);
            x.preview_ref(o)
        }
    }

    impl<Optics, T: Review<Optics>> Review<optics::positional::_0<Optics>> for (T,) {
        fn review(
            optics::positional::_0(optics): optics::positional::_0<Optics>,
            from: Self::Image,
        ) -> Self
        where
            Self::Image: Sized,
        {
            (Review::review(optics, from),)
        }
    }

    impl<Optics, T: Optic<Optics>, U> Optic<optics::positional::_0<Optics>> for (T, U) {
        type Image = T::Image;
    }

    impl<Optics, T: PrismRef<Optics>, U> PrismRef<optics::positional::_0<Optics>> for (T, U) {
        fn preview_ref(&self, optics: optics::positional::_0<Optics>) -> Option<&Self::Image> {
            let ((x, _), optics::positional::_0(o)) = (self, optics);
            x.preview_ref(o)
        }
    }

    #[test]
    fn test_review() {
        let x: () = Review::review(optics::Current, ());
        dbg!(x);
        let x: (i32,) = Review::review(optics::Current, (42,));
        dbg!(x);
        let x: Option<()> = Review::review(optics::Current, Some(()));
        dbg!(x);
        let x: Option<()> = Review::review(optics::option::Some(optics::Current), ());
        dbg!(x);
        let x: Option<(i32,)> = Review::review(optics::option::Some(optics::Current), (42,));
        dbg!(x);
        let x: Option<(i32, i32)> = Review::review(optics::option::Some(optics::Current), (42, 42));
        dbg!(x);
        let x: Result<(), ()> = Review::review(optics::result::Ok(optics::Current), ());
        dbg!(x.unwrap());
        let x: Result<(), ()> = Review::review(optics::result::Err(optics::Current), ());
        dbg!(x.unwrap_err());
        let x: Option<Option<()>> = Review::review(
            optics::option::Some(optics::option::Some(optics::Current)),
            (),
        );
        dbg!(x);
        let x: Option<Result<(), ()>> = Review::review(
            optics::option::Some(optics::result::Ok(optics::Current)),
            (),
        );
        dbg!(x);
        let x: Option<Result<(i32,), ()>> = Review::review(
            optics::option::Some(optics::result::Ok(optics::Current)),
            (42,),
        );
        dbg!(x);
        let x: Result<Option<(i32,)>, ()> = Review::review(
            optics::result::Ok(optics::option::Some(optics::Current)),
            (42,),
        );
        dbg!(x.unwrap());
        let x: Result<Option<Result<(), ()>>, ()> = Review::review(
            optics::result::Ok(optics::option::Some(optics::result::Ok(optics::Current))),
            (),
        );
        dbg!(x.unwrap());
        let x: Result<Option<Result<(), ()>>, ()> = Review::review(
            optics::result::Ok(optics::option::Some(optics::result::Err(optics::Current))),
            (),
        );
        dbg!(x.unwrap());
    }
}

mod exp_optics_md {

    pub trait Optic<Optics> {
        type Image: ?Sized;
    }

    /// Review
    pub trait Review<Optics>: Optic<Optics> {
        fn review(optics: Optics, from: Self::Image) -> Self
        where
            Self::Image: Sized;
    }

    /// Prism
    pub trait PrismRef<Optics>: Optic<Optics> {
        fn preview_ref(&self, optics: Optics) -> Option<&Self::Image>;
    }

    pub trait PrismMut<Optics>: PrismRef<Optics> + Optic<Optics> {
        fn preview_mut(&mut self, optics: Optics) -> Option<&mut Self::Image>;
    }

    pub trait Prism<Optics>: PrismMut<Optics> + Optic<Optics> {
        fn preview(self, optics: Optics) -> Option<Self::Image>
        where
            Self::Image: Sized;
    }

    /// Lens
    pub trait LensRef<Optics>: PrismRef<Optics> {
        fn view_ref(&self, optics: Optics) -> &Self::Image;
    }

    pub trait LensMut<Optics>: LensRef<Optics> + PrismMut<Optics> {
        fn view_mut(&mut self, optics: Optics) -> &mut Self::Image;
    }

    pub trait Lens<Optics>: LensMut<Optics> + Prism<Optics> {
        fn view(self, optics: Optics) -> Self::Image
        where
            Self::Image: Sized;
    }

    #[derive(Debug)]
    struct MyMd1(i32);

    fn fff<Pm, T: PrismRef<Pm, Image = MyMd1>>(t: &T, pm: Pm) {
        //                              ^ `T` may have a value of `i32`
        let x = t.preview_ref(pm);
        dbg!(x);
    }

    #[derive(Debug)]
    struct TreeSizeAcc;

    #[derive(Debug)]
    struct TreeSize;

    fn acc_tree_size<Oacc, Acc, Ot, T>(acc: &mut Acc, oacc: Oacc, child: T, ot: Ot)
    where
        Acc: LensMut<Oacc, Image = TreeSizeAcc>,
        T: LensRef<Ot, Image = TreeSize>,
    {
        let acc = acc.view_mut(oacc);
        let child = child.view_ref(ot);
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct TreeHashAcc;

    #[derive(Debug)]
    struct TreeHash;

    fn acc_tree_hash<Oacc, Acc, Ot, T>(acc: &mut Acc, oacc: Oacc, child: T, ot: Ot)
    where
        Acc: LensMut<Oacc, Image = TreeHashAcc>,
        T: LensRef<Ot, Image = TreeHash>,
    {
        let acc = acc.view_mut(oacc);
        let child = child.view_ref(ot);
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct LabelAcc;

    #[derive(Debug)]
    struct Type;

    fn finish_tree_hash<Osize, Olab, Oty, T>(
        acc: TreeHashAcc,
        curr: T,
        (os, ol, ot): (Osize, Olab, Oty),
    ) where
        T: LensRef<Osize, Image = TreeSizeAcc>,
        T: LensRef<Olab, Image = LabelAcc>,
        T: LensRef<Oty, Image = Type>,
    {
        let size = curr.view_ref(os);
        let label = curr.preview_ref(ol);
        let ty = curr.view_ref(ot);
        dbg!(acc, size, label, ty);
    }

    struct TS;

    fn finish_tree_hash2<Olab, Oty, T>(acc: TreeHashAcc, curr: T, (ol, ot): (Olab, Oty))
    where
        T: PrismRef<TS, Image = TreeSizeAcc>,
        T: LensRef<Olab, Image = LabelAcc>,
        T: LensRef<Oty, Image = Type>,
    {
        let size = curr.preview_ref(TS).expect("requires the size");
        let label = curr.view_ref(ol);
        let ty = curr.view_ref(ot);
        dbg!(acc, size, label, ty);
    }

    #[test]
    fn test_finish_hash() {
        use optics::positional::*;
        use optics::Current as C;
        finish_tree_hash(
            TreeHashAcc,
            (TreeSizeAcc, LabelAcc, Type),
            (_0(C), _1(C), _2(C)),
        );
        finish_tree_hash2(TreeHashAcc, (TreeSizeAcc, LabelAcc, Type), (_1(C), _2(C)));
    }

    // acc(acc: LensMut<TreeSizeAcc>, child: LensRef<TreeSize>)
    // finish(acc: TreeSizeAcc, _: T) -> TreeSize

    // acc(acc: LensMut<TreeHashAcc>, child: LensRef<TreeHash>)
    // finish(acc: TreeHashAcc, t: LensRef<TreeSize> + LensRef<LabelAcc> + LensRef<Type>) -> TreeHash

    // U finish:
    //   primary                  |      md0          |      md1                                 |    md2
    // Type LabelAcc ChildrenAcc  | TreeSize TreeHash | ByteLen LabelHash mcc bsetmatches Height | URefsVDecls/URefs
    //            ChildNames                              LineCount SizeNoSpaces #of_whatever     | convolution_of_whatever
    //        NoSpaces                                      roles

    // acc:
    //                  Id        |  r       r ty l   | r       r ty l    r ty r prim      r     | r prim
    //                            |                   |   r l       r ty         r prim

    // compress:
    // Lang l<8bytes<16bytes      |  size    TreeHashS| split  on kw    !=0  !=0          Markers| bloom filter
    // Toffset       lless_compr  |          on kw    |
    //               inline < 3                       | !=0    !=size          size
    //               bitpacked
    //               incr bit pack|

    mod optics {
        pub struct Current;

        pub mod positional {
            pub struct _0<Optics = super::Current>(pub Optics);
            pub struct _1<Optics = super::Current>(pub Optics);
            pub struct _2<Optics = super::Current>(pub Optics);
            pub struct _3<Optics = super::Current>(pub Optics);
            pub struct _4<Optics = super::Current>(pub Optics);
            pub struct _5<Optics = super::Current>(pub Optics);
            pub struct _6<Optics = super::Current>(pub Optics);
            pub struct _7<Optics = super::Current>(pub Optics);
        }
    }

    macro_rules! make_optics {
        (<$($g:ident ,)*> $m:ty, $t:ty) => {
            impl<$($g)*> Optic<$m> for $t {
                type Image = $t;
            }

            impl<$($g)*> PrismRef<$m> for $t {
                fn preview_ref(&self, _: $m) -> Option<&Self::Image> {
                    Some(self)
                }
            }

            impl<$($g)*> LensRef<$m> for $t {
                fn view_ref(&self, _: $m) -> &Self::Image {
                    self
                }
            }
        };
        ([$($g:tt)+] $m:ty[$i:ty] $t:ty {$ee:ident => $e:expr}) => {
            impl<$($g)*> Optic<$m> for $t {
                type Image = $i;
            }

            impl<$($g)*> PrismRef<$m> for $t {
                fn preview_ref(&self, _: $m) -> Option<&Self::Image> {
                    match self {$ee => Some($e)}

                }
            }

            impl<$($g)*> LensRef<$m> for $t {
                fn view_ref(&self, _: $m) -> &Self::Image {
                    match self {$ee => $e}
                }
            }
        };
    }
    make_optics!(<> TS, TreeSizeAcc);
    // impl Optic<TS> for TreeSizeAcc {
    //     type Image = TreeSizeAcc;
    // }

    // impl PrismRef<TS> for TreeSizeAcc {
    //     fn preview_ref(&self, _: TS) -> Option<&Self::Image> {
    //         Some(self)
    //     }
    // }

    // impl LensRef<TS> for TreeSizeAcc {
    //     fn view_ref(&self, _: TS) -> &Self::Image {
    //         self
    //     }
    // }

    impl Optic<TS> for LabelAcc {
        type Image = TreeSizeAcc;
    }

    impl PrismRef<TS> for LabelAcc {
        fn preview_ref(&self, _: TS) -> Option<&Self::Image> {
            None
        }
    }

    impl Optic<TS> for Type {
        type Image = TreeSizeAcc;
    }

    impl PrismRef<TS> for Type {
        fn preview_ref(&self, _: TS) -> Option<&Self::Image> {
            None
        }
    }

    make_optics!([T: Optic<TS>, ] TS[T] (T, ) {s => &s.0});
    // make_optics!([T: Optic<TS>, U] TS[T] (T, U) {s => &s.0});
    // make_optics!([T: Optic<TS>, U, V] TS[T] (T, U, V) {s => &s.0});

    // NOTE cannot specialize on bound, also need negations to guaranty uniq
    // make_optics!([T, U: Optic<TS>] TS[U] (T, U) {s => &s.1});
    // make_optics!([T, U: Optic<TS>, V] TS[U] (T, U, V) {s => &s.1});

    impl<T: Optic<TS>, U: Optic<TS, Image = T::Image>, V: Optic<TS, Image = T::Image>> Optic<TS>
        for (T, U, V)
    {
        type Image = T::Image;
    }

    impl<T: PrismRef<TS>, U: PrismRef<TS, Image = T::Image>, V: PrismRef<TS, Image = T::Image>>
        PrismRef<TS> for (T, U, V)
    {
        fn preview_ref(&self, _: TS) -> Option<&Self::Image> {
            self.0
                .preview_ref(TS)
                .or_else(|| self.1.preview_ref(TS))
                .or_else(|| self.2.preview_ref(TS))
        }
    }

    make_optics!(<T,> optics::Current, T);
    // impl<T> Optic<optics::Current> for T {
    //     type Image = T;
    // }

    // impl<T> PrismRef<optics::Current> for T {
    //     fn preview_ref(&self, _: optics::Current) -> Option<&Self::Image> {
    //         Some(self)
    //     }
    // }

    // impl<T> LensRef<optics::Current> for T {
    //     fn view_ref(&self, _: optics::Current) -> &Self::Image {
    //         self
    //     }
    // }

    impl<Optics, T: Optic<Optics>, U> Optic<optics::positional::_0<Optics>> for (T, U) {
        type Image = T::Image;
    }

    impl<Optics, T: PrismRef<Optics>, U> PrismRef<optics::positional::_0<Optics>> for (T, U) {
        fn preview_ref(&self, optics: optics::positional::_0<Optics>) -> Option<&Self::Image> {
            let ((x, _), optics::positional::_0(o)) = (self, optics);
            x.preview_ref(o)
        }
    }

    impl<Optics, T: LensRef<Optics>, U> LensRef<optics::positional::_0<Optics>> for (T, U) {
        fn view_ref(&self, optics: optics::positional::_0<Optics>) -> &Self::Image {
            let ((x, _), optics::positional::_0(o)) = (self, optics);
            x.view_ref(o)
        }
    }

    impl<Optics, T: Optic<Optics>, U, V> Optic<optics::positional::_0<Optics>> for (T, U, V) {
        type Image = T::Image;
    }

    impl<Optics, T: PrismRef<Optics>, U, V> PrismRef<optics::positional::_0<Optics>> for (T, U, V) {
        fn preview_ref(&self, optics: optics::positional::_0<Optics>) -> Option<&Self::Image> {
            let ((x, _, _), optics::positional::_0(o)) = (self, optics);
            x.preview_ref(o)
        }
    }

    impl<Optics, T: LensRef<Optics>, U, V> LensRef<optics::positional::_0<Optics>> for (T, U, V) {
        fn view_ref(&self, optics: optics::positional::_0<Optics>) -> &Self::Image {
            let ((x, _, _), optics::positional::_0(o)) = (self, optics);
            x.view_ref(o)
        }
    }

    impl<Optics, T, U: Optic<Optics>, V> Optic<optics::positional::_1<Optics>> for (T, U, V) {
        type Image = U::Image;
    }

    impl<Optics, T, U: PrismRef<Optics>, V> PrismRef<optics::positional::_1<Optics>> for (T, U, V) {
        fn preview_ref(&self, optics: optics::positional::_1<Optics>) -> Option<&Self::Image> {
            let ((_, x, _), optics::positional::_1(o)) = (self, optics);
            x.preview_ref(o)
        }
    }

    impl<Optics, T, U: LensRef<Optics>, V> LensRef<optics::positional::_1<Optics>> for (T, U, V) {
        fn view_ref(&self, optics: optics::positional::_1<Optics>) -> &Self::Image {
            let ((_, x, _), optics::positional::_1(o)) = (self, optics);
            x.view_ref(o)
        }
    }

    impl<Optics, T, U, V: Optic<Optics>> Optic<optics::positional::_2<Optics>> for (T, U, V) {
        type Image = V::Image;
    }

    impl<Optics, T, U, V: PrismRef<Optics>> PrismRef<optics::positional::_2<Optics>> for (T, U, V) {
        fn preview_ref(&self, optics: optics::positional::_2<Optics>) -> Option<&Self::Image> {
            let ((_, _, x), optics::positional::_2(o)) = (self, optics);
            x.preview_ref(o)
        }
    }

    impl<Optics, T, U, V: LensRef<Optics>> LensRef<optics::positional::_2<Optics>> for (T, U, V) {
        fn view_ref(&self, optics: optics::positional::_2<Optics>) -> &Self::Image {
            let ((_, _, x), optics::positional::_2(o)) = (self, optics);
            x.view_ref(o)
        }
    }
}

mod exp_getter_md {

    trait GetMd<T> {
        fn md(&self) -> &T;
    }

    impl<T, U> GetMd<T> for (T, U) {
        fn md(&self) -> &T {
            todo!()
        }
    }

    // impl<T, U> GetMd<T> for (U, T) {
    //     fn md(&self) -> Option<&T> {
    //         todo!()
    //     }
    // }

    #[derive(Debug)]
    struct TreeSizeAcc;

    #[derive(Debug)]
    struct TreeSize;

    fn acc_tree_size<T>(acc: &mut TreeSizeAcc, child: T)
    where
        T: GetMd<TreeSize>,
    {
        let child = child.md();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct TreeHashAcc;

    #[derive(Debug)]
    struct TreeHash;

    fn acc_tree_hash<T>(acc: &mut TreeHashAcc, child: T)
    where
        T: GetMd<TreeHash>,
    {
        let child = child.md();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct LabelAcc;

    #[derive(Debug)]
    struct Type;

    fn finish_tree_hash<Osize, Olab, Oty, T>(acc: TreeHashAcc, curr: T)
    where
        T: GetMd<TreeSize>,
        T: GetMd<Option<LabelAcc>>,
        T: GetMd<Type>,
    {
        let size: &TreeSize = curr.md();
        let label: &Option<LabelAcc> = curr.md();
        let ty: &Type = curr.md();
        dbg!(acc, size, label, ty);
    }
}
mod aaa {
    // struct E<H,T>(H, T);
}
mod exp_coprod {

    pub enum Coproduct<H, T> {
        /// Coproduct is either H or T, in this case, it is H
        Inl(H),
        /// Coproduct is either H or T, in this case, it is T
        Inr(T),
    }

    /// Phantom type for signature purposes only (has no value)
    ///
    /// Used by the macro to terminate the Coproduct type signature
    #[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
    pub enum CNil {}

    // Inherent methods
    impl<Head, Tail> Coproduct<Head, Tail> {
        /// Instantiate a coproduct from an element.
        #[inline(always)]
        pub fn inject<T, Index>(to_insert: T) -> Self
        where
            Self: CoprodInjector<T, Index>,
        {
            CoprodInjector::inject(to_insert)
        }

        /// Borrow an element from a coproduct by type.
        #[inline(always)]
        pub fn get<S, Index>(&self) -> Option<&S>
        where
            Self: CoproductSelector<S, Index>,
        {
            CoproductSelector::get(self)
        }
    }

    /// Trait for instantiating a coproduct from an element
    ///
    /// This trait is part of the implementation of the inherent static method
    /// [`Coproduct::inject`]. Please see that method for more information.
    ///
    /// You only need to import this trait when working with generic
    /// Coproducts of unknown type. In most code, `Coproduct::inject` will
    /// "just work," with or without this trait.
    ///
    /// [`Coproduct::inject`]: enum.Coproduct.html#method.inject
    pub trait CoprodInjector<InjectType, Index> {
        /// Instantiate a coproduct from an element.
        ///
        /// Please see the [inherent static method] for more information.
        ///
        /// The only difference between that inherent method and this
        /// trait method is the location of the type parameters.
        /// (here, they are on the trait rather than the method)
        ///
        /// [inherent static method]: enum.Coproduct.html#method.inject
        fn inject(to_insert: InjectType) -> Self;
    }

    /// Used as an index into an `HList`.
    ///
    /// `Here` is 0, pointing to the head of the HList.
    ///
    /// Users should normally allow type inference to create this type.
    pub struct Here {
        _priv: (),
    }

    /// Used as an index into an `HList`.
    ///
    /// `There<T>` is 1 + `T`.
    ///
    /// Users should normally allow type inference to create this type.
    pub struct There<T> {
        _marker: std::marker::PhantomData<T>,
    }

    impl<I, Tail> CoprodInjector<I, Here> for Coproduct<I, Tail> {
        fn inject(to_insert: I) -> Self {
            Coproduct::Inl(to_insert)
        }
    }

    impl<Head, I, Tail, TailIndex> CoprodInjector<I, There<TailIndex>> for Coproduct<Head, Tail>
    where
        Tail: CoprodInjector<I, TailIndex>,
    {
        fn inject(to_insert: I) -> Self {
            let tail_inserted = <Tail as CoprodInjector<I, TailIndex>>::inject(to_insert);
            Coproduct::Inr(tail_inserted)
        }
    }

    // For turning something into a Coproduct -->

    /// Trait for borrowing a coproduct element by type
    ///
    /// This trait is part of the implementation of the inherent method
    /// [`Coproduct::get`]. Please see that method for more information.
    ///
    /// You only need to import this trait when working with generic
    /// Coproducts of unknown type. If you have a Coproduct of known type,
    /// then `co.get()` should "just work" even without the trait.
    ///
    /// [`Coproduct::get`]: enum.Coproduct.html#method.get
    pub trait CoproductSelector<S, I> {
        /// Borrow an element from a coproduct by type.
        ///
        /// Please see the [inherent method] for more information.
        ///
        /// The only difference between that inherent method and this
        /// trait method is the location of the type parameters.
        /// (here, they are on the trait rather than the method)
        ///
        /// [inherent method]: enum.Coproduct.html#method.get
        fn get(&self) -> Option<&S>;
    }

    impl<Head, Tail> CoproductSelector<Head, Here> for Coproduct<Head, Tail> {
        fn get(&self) -> Option<&Head> {
            use self::Coproduct::*;
            match *self {
                Inl(ref thing) => Some(thing),
                _ => None, // Impossible
            }
        }
    }

    impl<Head, FromTail, Tail, TailIndex> CoproductSelector<FromTail, There<TailIndex>>
        for Coproduct<Head, Tail>
    where
        Tail: CoproductSelector<FromTail, TailIndex>,
    {
        fn get(&self) -> Option<&FromTail> {
            use self::Coproduct::*;
            match *self {
                Inr(ref rest) => rest.get(),
                _ => None, // Impossible
            }
        }
    }

    #[test]
    fn test_typed_get() {
        // type I32F32 = Coprod!(i32, f32);
        type I32F32 = Coproduct<i32, Coproduct<f32, Coproduct<usize, CNil>>>;

        // You can let type inference find the desired type:
        let co1 = I32F32::inject(42f32);
        let co1_as_i32: Option<&i32> = co1.get();
        let co1_as_f32: Option<&f32> = co1.get();
        assert_eq!(co1_as_i32, None);
        assert_eq!(co1_as_f32, Some(&42f32));

        // You can also use turbofish syntax to specify the type.
        // The Index parameter should be left as `_`.
        let co2 = I32F32::inject(1i32);
        assert_eq!(co2.get::<i32, _>(), Some(&1));
        assert_eq!(co2.get::<f32, _>(), None);
    }

    #[test]
    fn test_typed_inject() {
        // type I32F32 = Coprod!(i32, f32);
        type F32USIZE = Coproduct<f32, Coproduct<usize, CNil>>;
        type I32F32USIZE = Coproduct<i32, F32USIZE>;

        // You can let type inference find the desired type:
        let co1 = F32USIZE::inject(42f32);
        let co2 = I32F32USIZE::inject(3i32);
    }
}
mod exp_coprod2 {

    pub struct Product<H, T>(H, T);

    #[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
    pub struct CNil;

    pub struct Here {
        _priv: (),
    }

    pub struct There<T> {
        _marker: std::marker::PhantomData<T>,
    }

    pub trait ProductSelector<S, I> {
        fn get(&self) -> Option<&S>;
    }

    impl<Head, Tail> ProductSelector<Head, Here> for Product<Head, Tail> {
        fn get(&self) -> Option<&Head> {
            Some(&self.0)
        }
    }

    impl<Head, FromTail, Tail, TailIndex> ProductSelector<FromTail, There<TailIndex>>
        for Product<Head, Tail>
    where
        Tail: ProductSelector<FromTail, TailIndex>,
    {
        fn get(&self) -> Option<&FromTail> {
            self.1.get()
        }
    }

    #[test]
    fn test_typed_get() {
        let co1 = Product(42, Product(3.14, Product("Hello", CNil)));
        let co1_as_i32: Option<&i32> = co1.get();
        dbg!(co1_as_i32);
        let co1_as_f32: Option<&f32> = co1.get();
        dbg!(co1_as_f32);
        let co1_as_str: Option<&&str> = co1.get();
        dbg!(co1_as_str);
    }

    #[derive(Debug)]
    struct TreeSizeAcc;

    #[derive(Debug)]
    struct TreeSize;

    fn acc_tree_size<T, I>(acc: &mut TreeSizeAcc, child: &T)
    where
        T: ProductSelector<TreeSize, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct TreeHashAcc;

    #[derive(Debug)]
    struct TreeHash;

    fn acc_tree_hash<T, I>(acc: &mut TreeHashAcc, child: &T)
    where
        T: ProductSelector<TreeHash, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct LabelAcc;

    #[derive(Debug)]
    struct Type;

    fn finish_tree_size<T>(acc: TreeSizeAcc, _curr: &T) -> TreeSize {
        TreeSize
    }

    fn finish_tree_hash<T, I, J, K>(acc: TreeHashAcc, curr: &T) -> TreeHash
    where
        T: ProductSelector<Type, I>,
        T: ProductSelector<Option<LabelAcc>, J>,
        T: ProductSelector<TreeSize, K>,
    {
        let size: Option<&TreeSize> = curr.get();
        let label: Option<&Option<LabelAcc>> = curr.get();
        let ty: Option<&Type> = curr.get();
        dbg!(acc, size, label, ty);
        TreeHash
    }

    #[test]
    fn test_typed_md() {
        let mut acc = (TreeSizeAcc, TreeHashAcc);
        let child = Product(TreeSize, Product(TreeHash, CNil));
        acc_tree_size(&mut acc.0, &child);
        acc_tree_hash(&mut acc.1, &child);
        let curr = Product(Type, Product(Some(LabelAcc), Product(TreeSize, CNil)));
        let size = finish_tree_size(acc.0, &curr);
        let hash = finish_tree_hash(acc.1, &curr);
    }
}

mod exp_prod1 {

    pub struct Product<H, T>(H, T);

    #[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
    pub struct CNil;

    pub struct Here {
        _priv: (),
    }

    pub struct There<T> {
        _marker: std::marker::PhantomData<T>,
    }

    pub trait ProductSelector<S, I> {
        fn get(&self) -> &S;
    }

    impl<Head, Tail> ProductSelector<Head, Here> for Product<Head, Tail> {
        fn get(&self) -> &Head {
            &self.0
        }
    }

    impl<Head, FromTail, Tail, TailIndex> ProductSelector<FromTail, There<TailIndex>>
        for Product<Head, Tail>
    where
        Tail: ProductSelector<FromTail, TailIndex>,
    {
        fn get(&self) -> &FromTail {
            self.1.get()
        }
    }

    #[test]
    fn test_typed_get() {
        let co1 = Product(42, Product(3.14, Product("Hello", CNil)));
        let co1_as_i32: &i32 = co1.get();
        dbg!(co1_as_i32);
        let co1_as_f32: &f32 = co1.get();
        dbg!(co1_as_f32);
        let co1_as_str: &&str = co1.get();
        dbg!(co1_as_str);
    }

    #[derive(Debug)]
    struct TreeSizeAcc;

    #[derive(Debug)]
    struct TreeSize;

    fn acc_tree_size<T, I>(acc: &mut TreeSizeAcc, child: &T)
    where
        T: ProductSelector<TreeSize, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct TreeHashAcc;

    #[derive(Debug)]
    struct TreeHash;

    fn acc_tree_hash<T, I>(acc: &mut TreeHashAcc, child: &T)
    where
        T: ProductSelector<TreeHash, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct LabelAcc;

    #[derive(Debug)]
    struct Type;

    fn finish_tree_size<T>(acc: TreeSizeAcc, _curr: &T) -> TreeSize {
        TreeSize
    }

    fn finish_tree_hash<T, I, J, K>(acc: TreeHashAcc, curr: &T) -> TreeHash
    where
        T: ProductSelector<Type, I>,
        T: ProductSelector<Option<LabelAcc>, J>,
        T: ProductSelector<TreeSize, K>,
    {
        let size: &TreeSize = curr.get();
        let label: &Option<LabelAcc> = curr.get();
        let ty: &Type = curr.get();
        dbg!(acc, size, label, ty);
        TreeHash
    }

    #[test]
    fn test_typed_md() {
        let mut acc = (TreeSizeAcc, TreeHashAcc);
        let child = Product(TreeSize, Product(TreeHash, CNil));
        acc_tree_size(&mut acc.0, &child);
        acc_tree_hash(&mut acc.1, &child);
        let curr = Product(Type, Product(Some(LabelAcc), Product(TreeSize, CNil)));
        let size = finish_tree_size(acc.0, &curr);
        let hash = finish_tree_hash(acc.1, &curr);
    }
}

mod exp_prod2 {

    pub struct Product<H, T>(H, T);

    #[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
    pub struct CNil;

    pub struct Here {
        _priv: (),
    }

    pub struct There<T> {
        _marker: std::marker::PhantomData<T>,
    }

    pub trait ProductSelector<S, I> {
        fn get(&self) -> &S;
    }

    impl<Head, Tail> ProductSelector<Head, Here> for Product<Head, Tail> {
        fn get(&self) -> &Head {
            &self.0
        }
    }

    impl<Head, FromTail, Tail, TailIndex> ProductSelector<FromTail, There<TailIndex>>
        for Product<Head, Tail>
    where
        Tail: ProductSelector<FromTail, TailIndex>,
    {
        fn get(&self) -> &FromTail {
            self.1.get()
        }
    }

    #[test]
    fn test_typed_get() {
        let co1 = Product(42, Product(3.14, Product("Hello", CNil)));
        let co1_as_i32: &i32 = co1.get();
        dbg!(co1_as_i32);
        let co1_as_f32: &f32 = co1.get();
        dbg!(co1_as_f32);
        let co1_as_str: &&str = co1.get();
        dbg!(co1_as_str);
    }

    #[derive(Debug)]
    struct TreeSizeAcc;

    #[derive(Debug)]
    struct TreeSize;

    fn acc_tree_size<T, I>(acc: &mut TreeSizeAcc, child: &T)
    where
        T: ProductSelector<TreeSize, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct TreeHashAcc;

    #[derive(Debug)]
    struct TreeHash;

    fn acc_tree_hash<T, I>(acc: &mut TreeHashAcc, child: &T)
    where
        T: ProductSelector<TreeHash, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct LabelAcc;

    #[derive(Debug)]
    struct Type;

    fn finish_tree_size<T>(acc: TreeSizeAcc, _curr: &T) -> TreeSize {
        TreeSize
    }

    fn finish_tree_hash<T, I, J, K>(acc: TreeHashAcc, curr: &T) -> TreeHash
    where
        T: ProductSelector<Type, I>,
        T: ProductSelector<Option<LabelAcc>, J>,
        T: ProductSelector<TreeSize, K>,
    {
        let size: &TreeSize = curr.get();
        let label: &Option<LabelAcc> = curr.get();
        let ty: &Type = curr.get();
        dbg!(acc, size, label, ty);
        TreeHash
    }

    #[test]
    fn test_typed_md() {
        let mut acc = (TreeSizeAcc, TreeHashAcc);
        let child = Product(TreeSize, Product(TreeHash, CNil));
        acc_tree_size(&mut acc.0, &child);
        acc_tree_hash(&mut acc.1, &child);
        let curr = Product(Type, Product(Some(LabelAcc), Product(TreeSize, CNil)));
        let size = finish_tree_size(acc.0, &curr);
        let hash = finish_tree_hash(acc.1, &curr);
    }
}

// Done playing with my half backed defs, now exp with frunk crate
mod exp_frunk {
    use frunk::*;
    use hlist::Selector;

    #[test]
    fn test_typed_get() {
        let co1 = hlist![42, 3.14, "Hello"];
        let co1_as_i32: &i32 = co1.get();
        dbg!(co1_as_i32);
        let co1_as_f32: &f32 = co1.get();
        dbg!(co1_as_f32);
        let co1_as_str: &&str = co1.get();
        dbg!(co1_as_str);
    }

    #[derive(Debug)]
    struct TreeSizeAcc;

    #[derive(Debug)]
    struct TreeSize;

    fn acc_tree_size<T, I>(acc: &mut TreeSizeAcc, child: &T)
    where
        T: Selector<TreeSize, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct TreeHashAcc;

    #[derive(Debug)]
    struct TreeHash;

    fn acc_tree_hash<T, I>(acc: &mut TreeHashAcc, child: &T)
    where
        T: Selector<TreeHash, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug)]
    struct LabelAcc;

    #[derive(Debug)]
    struct Type;

    fn finish_tree_size<T>(acc: TreeSizeAcc, _curr: &T) -> TreeSize {
        TreeSize
    }

    fn finish_tree_hash<T, I, J, K>(acc: TreeHashAcc, curr: &T) -> TreeHash
    where
        T: Selector<Type, I>,
        T: Selector<Option<LabelAcc>, J>,
        T: Selector<TreeSize, K>,
    {
        let size: &TreeSize = curr.get();
        let label: &Option<LabelAcc> = curr.get();
        let ty: &Type = curr.get();
        dbg!(acc, size, label, ty);
        TreeHash
    }

    #[test]
    fn test_typed_md() {
        let mut acc = (TreeSizeAcc, TreeHashAcc);
        let child = hlist![TreeSize, TreeHash];
        acc_tree_size(&mut acc.0, &child);
        acc_tree_hash(&mut acc.1, &child);
        let curr = hlist![Type, Some(LabelAcc), TreeSize];
        let size = finish_tree_size(acc.0, &curr);
        let hash = finish_tree_hash(acc.1, &curr);
    }
}

// Try unify acc with curr
mod exp_frunk2 {
    use std::fmt::Debug;

    use frunk::*;
    use hlist::{HMappable, Selector};

    #[test]
    fn test_typed_get() {
        let co1 = hlist![42, 3.14, "Hello"];
        let co1_as_i32: &i32 = co1.get();
        dbg!(co1_as_i32);
        let co1_as_f32: &f32 = co1.get();
        dbg!(co1_as_f32);
        let co1_as_str: &&str = co1.get();
        dbg!(co1_as_str);
    }

    #[derive(Debug, Default)]
    struct TreeSizeAcc;

    #[derive(Debug)]
    struct TreeSize;

    fn acc_tree_size<A, I, T, J>(acc: &mut A, child: &T)
    where
        A: Selector<TreeSizeAcc, I>,
        T: Selector<TreeSize, J>,
    {
        let acc = acc.get_mut();
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug, Default)]
    struct TreeHashAcc;

    #[derive(Debug)]
    struct TreeHash;

    fn acc_tree_hash<A, I, T, J>(acc: &mut A, child: &T)
    where
        A: Selector<TreeHashAcc, I>,
        T: Selector<TreeHash, J>,
    {
        let acc = acc.get_mut();
        let child = child.get();
        dbg!(acc, child);
    }

    fn acc_tree_size0<T, I>(acc: &mut TreeSizeAcc, child: &T)
    where
        T: Selector<TreeSize, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    fn acc_tree_hash0<T, I>(acc: &mut TreeHashAcc, child: &T)
    where
        T: Selector<TreeHash, I>,
    {
        let child = child.get();
        dbg!(acc, child);
    }

    #[derive(Debug, Default)]
    struct LabelAcc;

    #[derive(Debug)]
    struct Type;

    fn finish_tree_size<A, I>(acc: &A) -> TreeSize
    where
        A: Selector<TreeSizeAcc, I>,
    {
        TreeSize
    }

    fn finish_tree_hash<A, IA, IT, IL, IS>(acc: &A) -> TreeHash
    where
        A: Selector<TreeHashAcc, IA>,
        A: Selector<Type, IT>,
        A: Selector<Option<LabelAcc>, IL>,
        A: Selector<TreeSize, IS>,
    {
        let size: &TreeSize = acc.get();
        let label: &Option<LabelAcc> = acc.get();
        let ty: &Type = acc.get();
        let acc: &TreeHashAcc = acc.get();
        dbg!(acc, size, label, ty);
        TreeHash
    }

    macro_rules! hlist_repeat {
        [$e:expr; $n:tt] => {
            {
                hlist_repeat!([$n, $e] -> [])
            }
        };
        ([0, $_:expr] -> [$($body:tt)*]) => { hlist![$($body)*] };
        ([1, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([0, $e] -> [$($body)* $e,]) };
        ([2, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([1, $e] -> [$($body)* $e,]) };
        ([3, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([2, $e] -> [$($body)* $e,]) };
        ([4, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([3, $e] -> [$($body)* $e,]) };
        ([5, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([4, $e] -> [$($body)* $e,]) };
        ([6, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([5, $e] -> [$($body)* $e,]) };
        ([7, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([6, $e] -> [$($body)* $e,]) };
        ([8, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([7, $e] -> [$($body)* $e,]) };
        ([9, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([8, $e] -> [$($body)* $e,]) };
        ([10, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([9, $e] -> [$($body)* $e,]) };
        ([11, $e:expr] -> [$($body:tt)*]) => { hlist_repeat!([10, $e] -> [$($body)* $e,]) };

    }

    #[test]
    fn test_typed_md() {
        let mut acc = hlist![Type, Some(LabelAcc), TreeSizeAcc, TreeHashAcc];
        let child = hlist![TreeSize, TreeHash];
        acc_tree_size(&mut acc, &child);
        acc_tree_hash(&mut acc, &child);
        let size = finish_tree_size(&acc);
        let acc = acc.prepend(size);
        let hash = finish_tree_hash(&acc);
        let acc = acc.prepend(hash);

        let a = hlist_repeat![|x| dbg!(x); 6];
        acc.map(a);

        // instead of:
        // acc.map(hlist![
        //     |x| dbg!(x),
        //     |x| dbg!(x),
        //     |x| dbg!(x),
        //     |x| dbg!(x),
        //     |x| dbg!(x),
        //     |x| dbg!(x),
        // ]);
    }

    #[derive(Debug, Default)]
    struct ChildrenAcc;

    macro_rules! fn_poly_single {
        ($x:ident => $b:block: $bound:tt) => {{
            struct F;
            impl<I> Func<I> for F
            where
                I: $bound,
            {
                type Output = I;

                fn call(i: I) -> Self::Output {
                    match i {
                        $x => $b,
                    }
                }
            }
            Poly(F)
        }};
    }

    #[test]
    fn test_typed_md2() {
        let prim = hlist![Type, Some(LabelAcc), ChildrenAcc];
        let mut acc = hlist![TreeSizeAcc, TreeHashAcc];
        // let mut acc = HCons::default();
        let child = hlist![TreeSize, TreeHash];

        acc.to_mut().map(hlist![
            |acc| {
                acc_tree_size0(acc, &child);
            },
            |acc| {
                acc_tree_hash0(acc, &child);
            },
        ]);

        let mut acc = prim.extend(acc);
        // accs(&mut acc, &child);
        let acc = finishes(acc);

        let a = hlist_repeat![|x| dbg!(x); 7];
        acc.to_ref().map(a);
        acc.to_ref().map(Poly(F));
        struct F;
        impl<I> Func<I> for F
        where
            I: Debug,
        {
            type Output = I;

            fn call(i: I) -> Self::Output {
                dbg!(i)
            }
        }

        acc.to_ref().map(fn_poly_single! {x => {dbg!(x)}: Debug});
    }

    type AAA = HCons<
        Type,
        HCons<Option<LabelAcc>, HCons<ChildrenAcc, HCons<TreeSizeAcc, HCons<TreeHashAcc, HNil>>>>,
    >;

    fn finishes(acc: AAA) -> HCons<TreeHash, HCons<TreeSize, AAA>> {
        let size = finish_tree_size(&acc);
        let acc = acc.prepend(size);
        let hash = finish_tree_hash(&acc);
        let acc = acc.prepend(hash);
        acc
    }

    fn accs<A, AS, AH, T, TS, TH>(acc: &mut A, child: &T)
    where
        A: Selector<TreeSizeAcc, AS>,
        A: Selector<TreeHashAcc, AH>,
        T: Selector<TreeSize, TS>,
        T: Selector<TreeHash, TH>,
    {
        acc_tree_size(acc, child);
        acc_tree_hash(acc, child);
    }

    #[derive(Default)]
    struct TypeMap(std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>);

    impl<T: 'static> Selector<T, T> for TypeMap {
        fn get(&self) -> &T {
            self.0
                .get(&std::any::TypeId::of::<T>())
                .unwrap()
                .downcast_ref()
                .unwrap()
        }

        fn get_mut(&mut self) -> &mut T {
            self.0
                .get_mut(&std::any::TypeId::of::<T>())
                .unwrap()
                .downcast_mut()
                .unwrap()
        }
    }

    impl TypeMap {
        fn insert<T: std::any::Any>(&mut self, v: T) {
            self.0.insert(std::any::TypeId::of::<T>(), Box::new(v));
        }

        pub fn map<F>(self, mapper: F) -> <Self as HMappable<F>>::Output
        where
            Self: HMappable<F>,
        {
            HMappable::map(self, mapper)
        }

        fn extend<Other: Into<Self>>(mut self, other: Other) -> Self {
            let other = other.into();
            self.0.extend(other.0);
            self
        }
    }

    impl<'a> ToMut<'a> for TypeMap {
        type Output = &'a mut Self;

        fn to_mut(&'a mut self) -> Self::Output {
            self
        }
    }

    // impl Iterator for TypeMap {
    //     type Item = ();

    //     fn next(&mut self) -> Option<Self::Item> {
    //         todo!()
    //     }
    // }

    impl<F> HMappable<Poly<F>> for TypeMap where F: for<'a> Func<&'a dyn std::any::Any> {
        type Output = TypeMap;

        fn map(self, _: Poly<F>) -> Self::Output {
            for v in self.0.values() {
                F::call(v);
            }
            self
        }
    }

    impl<H: 'static, T: Into<TypeMap>> From<HCons<H, T>> for TypeMap {
        fn from(value: HCons<H, T>) -> Self {
            let (h, t) = value.pop();
            let mut s = t.into();
            s.insert(h);
            s
        }
    }

    impl From<HNil> for TypeMap {
        fn from(_: HNil) -> Self {
            Default::default()
        }
    }

    #[test]
    fn test_typed_md_typemap() {
        let prim = hlist![Type, Some(LabelAcc), ChildrenAcc];
        let mut acc = TypeMap::default(); //hlist![TreeSizeAcc, TreeHashAcc];
        acc.insert(TreeSizeAcc);
        acc.insert(TreeHashAcc);
        // let mut acc = HCons::default();
        let child = hlist![TreeSize, TreeHash];

        acc_tree_size0(acc.get_mut(), &child);
        acc_tree_hash0(acc.get_mut(), &child);
        // acc.to_mut().map(hlist![
        //     |acc| {
        //         acc_tree_size0(acc, &child);
        //     },
        //     |acc| {
        //         acc_tree_hash0(acc, &child);
        //     },
        // ]);

        let mut acc = acc.extend(prim);
        accs(&mut acc, &child);
        let size = finish_tree_size(&acc);
        acc.insert(size);
        let hash = finish_tree_hash(&acc);
        acc.insert(hash);
        // let acc = finishes(acc);

        acc.map(fn_poly_single! {x => {dbg!(x)}: Debug});
    }
}
