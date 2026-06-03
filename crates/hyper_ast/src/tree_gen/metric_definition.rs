// region: Stuff provided usually provided by HyperAST

/// Helper to define and build subtrees while computing metrics
pub struct Builder<C>(C);

/// The AS node types
#[derive(Clone, Copy, Debug)]
pub enum Ty {
    Class,
    Method,
    IfStatement,
    WhileStatement,
}

/// interface to an AS node
pub trait Subtree {
    fn try_get<M: Clone + 'static>(&self) -> Option<M>;
    fn get<M: Clone + 'static>(&self) -> M {
        dbg!(std::any::type_name::<M>());
        self.try_get().unwrap()
    }
    fn ty(&self) -> Ty;
    fn label(&self) -> Option<&str> {
        self.try_get()
    }
    fn push_metric<M: 'static>(&mut self, m: M);
    fn builder() -> Builder<NoMetrics<Self>>
    where
        Self: Sized,
    {
        Builder(NoMetrics::default())
    }
}

// endregion

// region: Defining computation behavior directly on metric accumulator

pub trait MetricAcc {
    type S: Subtree;
    type M: 'static;
    fn init(ty: Ty, l: Option<&str>) -> Self;
    fn acc(acc: Self, current: &Self::S) -> Self;
    fn finish(acc: Self, current: &Self::S) -> Self::M;
}
impl<T> MetricComputing for T
where
    T: MetricAcc,
{
    type S = T::S;

    fn pipe<O: MetricComputing<S = Self::S>>(self, o: O) -> impl MetricComputing<S = Self::S> {
        Chained(self, o)
    }

    type Acc = Self;

    type M = T::M;

    fn init(&self, ty: Ty, l: Option<&str>) -> Self::Acc {
        MetricAcc::init(ty, l)
    }

    fn finish(&self, acc: Self::Acc, mut current: Self::S) -> Self::S {
        let m = MetricAcc::finish(acc, &current);
        current.push_metric(m);
        current
    }

    fn acc(&self, acc: Self::Acc, current: &Self::S) -> Self::Acc {
        MetricAcc::acc(acc, current)
    }
}
// endregion

// region: Defining computation behavior of a metric

/// Define how to compute a metric.
/// Easily composed using [`MetricComputing::pipe`] (see also [`Chained`]).
/// Can be made from closures with [`Functional`]
pub trait MetricComputing {
    /// Target code of the metric computation
    type S: Subtree;
    fn pipe<O: MetricComputing<S = Self::S>>(self, o: O) -> impl MetricComputing<S = Self::S>
    where
        Self: Sized,
    {
        Chained(self, o)
    }
    /// Holds the value of the metric while it is accumulated
    type Acc;
    /// The final output metric
    type M: 'static;
    fn init(&self, ty: Ty, l: Option<&str>) -> Self::Acc;
    fn acc(&self, acc: Self::Acc, current: &Self::S) -> Self::Acc;
    fn finish(&self, acc: Self::Acc, current: Self::S) -> Self::S;
}

pub struct NoMetrics<U>(std::marker::PhantomData<U>);
impl<U> Default for NoMetrics<U> {
    fn default() -> Self {
        Self(Default::default())
    }
}
impl<S: Subtree> MetricComputing for NoMetrics<S> {
    type S = S;
    fn pipe<O: MetricComputing<S = Self::S>>(self, o: O) -> impl MetricComputing<S = Self::S> {
        // optimization: no need this is a noop anyway
        o
    }
    type Acc = ();
    fn init(&self, _ty: Ty, _l: Option<&str>) -> Self::Acc {
        ()
    }
    fn acc(&self, _acc: Self::Acc, _current: &Self::S) -> Self::Acc {
        ()
    }
    type M = ();
    fn finish(&self, _acc: Self::Acc, current: Self::S) -> Self::S {
        current
    }
}

struct Chained<F0, F1>(F0, F1);
impl<F0: MetricComputing, F1: MetricComputing<S = F0::S>> MetricComputing for Chained<F0, F1> {
    type S = F0::S;
    type Acc = (F0::Acc, F1::Acc);
    fn init(&self, ty: Ty, l: Option<&str>) -> Self::Acc {
        (self.0.init(ty, l), self.1.init(ty, l))
    }
    type M = (F0::M, F1::M);
    fn acc(&self, acc: Self::Acc, current: &Self::S) -> Self::Acc {
        let a0 = self.0.acc(acc.0, current);
        let a1 = self.1.acc(acc.1, current);
        (a0, a1)
    }
    fn finish(&self, acc: Self::Acc, current: Self::S) -> Self::S {
        let current = self.0.finish(acc.0, current);
        let current = self.1.finish(acc.1, current);
        current
    }
}

// endregion
// region: functional

/// Defining computation behavior of a metric with functions
struct Functional<T, U>(T, std::marker::PhantomData<U>);
impl<
    A,
    M: 'static,
    I: Fn(Ty, Option<&str>) -> A,
    Acc: Fn(A, &S) -> A,
    F: Fn(A, &S) -> M,
    S: Subtree,
> MetricComputing for Functional<(I, Acc, F), (A, M, S)>
{
    type S = S;
    type Acc = A;
    fn init(&self, ty: Ty, l: Option<&str>) -> Self::Acc {
        (self.0.0)(ty, l)
    }
    type M = M;
    fn finish(&self, acc: Self::Acc, mut current: Self::S) -> Self::S {
        let m = (self.0.2)(acc, &current);
        current.push_metric(m);
        current
    }
    fn acc(&self, acc: Self::Acc, current: &Self::S) -> Self::Acc {
        (self.0.1)(acc, current)
    }
}

// endregion

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    impl Ty {
        fn is_branch(&self) -> bool {
            matches!(self, Ty::IfStatement | Ty::WhileStatement)
        }
    }

    #[allow(dead_code, unreachable_code, unused)]
    #[test]
    fn test_metrics_computing_function_api() {
        struct A(u32);
        dbg!(std::any::TypeId::of::<A>());
        dbg!(std::any::TypeId::of::<M>());
        #[derive(Clone, Copy)]
        struct M(u32);
        fn mcc(s: &impl Subtree) -> M {
            s.get()
        }
        let builder = STree::builder();
        let builder = builder.with_function_metric(
            |_, _| A(0),
            |a, c| A(a.0 + mcc(c).0),
            |a, s| {
                if s.ty().is_branch() {
                    M(a.0 + 1)
                } else {
                    M(a.0)
                }
            },
        );
        let root = build_mcc_example_class(&builder);
        dbg!(mcc(&root).0);
    }
    #[test]
    fn test_metrics_computing_function_api2() {
        struct A(u32);
        impl std::ops::Add<M> for A {
            type Output = A;

            fn add(self, rhs: M) -> Self::Output {
                A(self.0 + rhs.0)
            }
        }
        impl std::ops::Add<Ty> for A {
            type Output = M;

            fn add(self, rhs: Ty) -> Self::Output {
                M(self.0 + rhs.is_branch() as u32)
            }
        }
        #[derive(Clone, Copy)]
        struct M(u32);
        fn mcc(s: &impl Subtree) -> M {
            s.get()
        }
        let builder = STree::builder();
        let builder = builder.with_function_metric(
            |_, _| A(0),       //
            |a, c| a + mcc(c), //
            |a, s| a + s.ty(), //
        );
        let root = build_mcc_example_class(&builder);
        dbg!(mcc(&root).0);
    }

    #[test]
    fn test_metrics_computing_function_api3() {
        #[derive(Default)]
        struct A(u32);
        impl std::ops::Add<M> for A {
            type Output = A;

            fn add(self, rhs: M) -> Self::Output {
                A(self.0 + rhs.0)
            }
        }
        impl std::ops::Add<Ty> for A {
            type Output = M;

            fn add(self, rhs: Ty) -> Self::Output {
                M(self.0 + rhs.is_branch() as u32)
            }
        }
        #[derive(Clone, Copy)]
        struct M(u32);
        fn mcc(s: &impl Subtree) -> M {
            s.get()
        }
        let builder = STree::builder();
        let builder = builder.with_simple_metric::<A, M>();
        let root = build_mcc_example_class(&builder);
        dbg!(mcc(&root).0);
    }

    #[test]
    fn test_metrics_computing_function_api4() {
        #[derive(Default)]
        struct A(u32);
        impl<S: Subtree> std::ops::AddAssign<&S> for A {
            fn add_assign(&mut self, rhs: &S) {
                self.0 += mcc(rhs).0;
            }
        }
        impl<S: Subtree> std::ops::Add<&S> for A {
            type Output = M;

            fn add(self, rhs: &S) -> Self::Output {
                M(self.0 + rhs.ty().is_branch() as u32)
            }
        }
        #[derive(Clone, Copy)]
        struct M(u32);
        fn mcc(s: &impl Subtree) -> M {
            s.get()
        }
        let builder = STree::builder();
        let builder = builder.with_simpler_metric::<A, M>();
        let root = build_mcc_example_class(&builder);
        dbg!(mcc(&root).0);
    }

    #[test]
    fn test_metrics_computing_trait_api() {
        struct A(u32);
        #[derive(Clone, Copy, Debug)]
        struct M(u32);
        let builder = STree::builder();
        impl MetricAcc for A {
            type S = STree;

            type M = M;

            fn init(_ty: Ty, _l: Option<&str>) -> Self {
                A(0)
            }

            fn acc(a: Self, c: &Self::S) -> Self {
                A(a.0 + c.get::<M>().0)
            }

            fn finish(a: Self, s: &Self::S) -> Self::M {
                if s.ty().is_branch() {
                    M(a.0 + 1)
                } else {
                    M(a.0)
                }
            }
        }
        let builder = builder.with_accumulator::<A>();
        let root = build_mcc_example_class(&builder);
        let m: M = root.get();
        dbg!(m);
    }

    fn build_mcc_example_class(builder: &Builder<impl MetricComputing<S = STree>>) -> STree {
        let root = Ty::Class;
        let acc_root = builder.0.init(root, None);
        let meth = build_mcc_example_meth(builder);
        let acc_root = builder.0.acc(acc_root, &meth);
        let class_members = Children(vec![meth]);
        let root = STree(root, vec![Box::new(class_members)]);
        let root = builder.0.finish(acc_root, root);
        root
    }

    fn build_mcc_example_meth(builder: &Builder<impl MetricComputing<S = STree>>) -> STree {
        let meth = Ty::Method;
        let acc_meth = builder.0.init(meth, None);
        let if_statement = build_mcc_example_if_statement(builder);
        let acc_meth = builder.0.acc(acc_meth, &if_statement);
        let meth_statements = Children(vec![if_statement]);
        let meth = STree(meth, vec![Box::new(meth_statements)]);
        let meth = builder.0.finish(acc_meth, meth);
        meth
    }

    fn build_mcc_example_if_statement(builder: &Builder<impl MetricComputing<S = STree>>) -> STree {
        let if_statement = Ty::IfStatement;
        let acc_if_statement = builder.0.init(if_statement, None);
        let if_statement = STree(if_statement, vec![]);
        let if_statement = builder.0.finish(acc_if_statement, if_statement);
        if_statement
    }

    // region: Stuff provided usually provided by HyperAST

    struct STree(Ty, Vec<Box<dyn std::any::Any>>);

    struct Children(#[allow(unused)] pub Vec<STree>);

    impl Subtree for STree {
        fn try_get<M: Clone + 'static>(&self) -> Option<M> {
            for x in &self.1 {
                let Some(m) = x.downcast_ref::<M>() else {
                    continue;
                };
                return Some(m.clone());
            }
            None
        }
        fn ty(&self) -> Ty {
            self.0
        }
        fn push_metric<M: 'static>(&mut self, m: M) {
            self.1.push(Box::new(m));
        }
    }

    // endregion

    impl Builder<()> {
        /// creates a builder without metrics
        fn new<U: Subtree>() -> Builder<NoMetrics<U>> {
            Builder(Default::default())
        }
    }

    impl<C: MetricComputing> Builder<C>
    where
        C::S: Subtree,
    {
        fn with_accumulator<A: 'static + MetricAcc<S = C::S>>(
            self,
        ) -> Builder<impl MetricComputing<S = C::S>> {
            struct Comp<A>(std::marker::PhantomData<A>);
            impl<A: 'static + MetricAcc> MetricComputing for Comp<A>
            where
                A::M: 'static,
            {
                type S = A::S;

                fn pipe<O: MetricComputing<S = Self::S>>(
                    self,
                    o: O,
                ) -> impl MetricComputing<S = Self::S> {
                    Chained(self, o)
                }

                type Acc = A;

                type M = A::M;

                fn init(&self, ty: Ty, l: Option<&str>) -> Self::Acc {
                    A::init(ty, l)
                }

                fn acc(&self, a: Self::Acc, c: &Self::S) -> Self::Acc {
                    A::acc(a, c)
                }

                fn finish(&self, a: Self::Acc, mut s: Self::S) -> Self::S {
                    let m = A::finish(a, &s);
                    s.push_metric(m);
                    s
                }
            }
            Builder(self.0.pipe(Comp::<A>(Default::default())))
        }

        fn with_simple_metric<
            A: 'static + Default + std::ops::Add<M, Output = A> + std::ops::Add<Ty, Output = M>,
            M: 'static + Copy,
        >(
            self,
        ) -> Builder<impl MetricComputing<S = C::S>> {
            struct Comp<A, M, S>(std::marker::PhantomData<(A, M, S)>);
            impl<
                A: 'static + Default + std::ops::Add<M, Output = A> + std::ops::Add<Ty, Output = M>,
                M: 'static + Copy,
                S: Subtree,
            > MetricComputing for Comp<A, M, S>
            {
                type S = S;

                fn pipe<O: MetricComputing<S = Self::S>>(
                    self,
                    o: O,
                ) -> impl MetricComputing<S = Self::S> {
                    Chained(self, o)
                }

                type Acc = A;

                type M = M;

                fn init(&self, _ty: Ty, _l: Option<&str>) -> Self::Acc {
                    A::default()
                }

                fn acc(&self, a: Self::Acc, c: &Self::S) -> Self::Acc {
                    a + c.get::<M>()
                }

                fn finish(&self, a: Self::Acc, mut s: Self::S) -> Self::S {
                    let m = a + s.ty();
                    s.push_metric(m);
                    s
                }
            }
            Builder(self.0.pipe(Comp::<A, M, C::S>(Default::default())))
        }

        fn with_simpler_metric<
            A: 'static
                + Default
                + for<'a> std::ops::AddAssign<&'a C::S>
                + for<'a> std::ops::Add<&'a C::S, Output = M>,
            M: 'static + Copy,
        >(
            self,
        ) -> Builder<impl MetricComputing<S = C::S>> {
            struct Comp<A, M, S>(std::marker::PhantomData<(A, M, S)>);
            impl<
                A: 'static
                    + Default
                    + for<'a> std::ops::AddAssign<&'a S>
                    + for<'a> std::ops::Add<&'a S, Output = M>,
                M: 'static + Copy,
                S: Subtree,
            > MetricComputing for Comp<A, M, S>
            {
                type S = S;

                fn pipe<O: MetricComputing<S = Self::S>>(
                    self,
                    o: O,
                ) -> impl MetricComputing<S = Self::S> {
                    Chained(self, o)
                }

                type Acc = A;

                type M = M;

                fn init(&self, _ty: Ty, _l: Option<&str>) -> Self::Acc {
                    A::default()
                }

                fn acc(&self, mut a: Self::Acc, c: &Self::S) -> Self::Acc {
                    a += c;
                    a
                }

                fn finish(&self, a: Self::Acc, mut s: Self::S) -> Self::S {
                    let m = a + &s;
                    s.push_metric(m);
                    s
                }
            }
            Builder(self.0.pipe(Comp::<A, M, C::S>(Default::default())))
        }
    }

    impl<C: MetricComputing> Builder<C>
    where
        C::S: Subtree,
    {
        fn with_function_metric<A, M: 'static>(
            self,
            init: impl Fn(Ty, Option<&str>) -> A,
            acc: impl Fn(A, &C::S) -> A,
            finish: impl Fn(A, &C::S) -> M,
        ) -> Builder<impl MetricComputing<S = C::S>> {
            Builder(
                self.0
                    .pipe(Functional((init, acc, finish), Default::default())),
            )
        }

        // Does not work because of limitation of closure with generics
        // fn with_simple_metric<
        //     A: Default + std::ops::Add<M, Output = A> + std::ops::Add<Ty, Output = M>,
        //     M: 'static,
        // >(
        //     self,
        // ) -> Builder<impl MetricComputing<S = C::S>> {
        //     Builder(self.0.pipe(Functional(
        //         (
        //             |_: Ty, _: &str| A::default(),
        //             |a: A, c: &C::S| a + c.get(),
        //             |a: A, s: &C::S| a + s.ty(),
        //         ),
        //         Default::default(),
        //     )))
        // }
    }
}
