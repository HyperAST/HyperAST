

#[test]
fn test_metrics_computing_api() {
    struct A(u32);
    #[derive(Clone, Copy)]
    struct M(u32);
    let builder = Builder(Functional::<_, STree>((), PhantomData));
    let builder = builder.with_metric(
        |_, _| A(0),
        |a, c| A(a.0 + c.get::<M>().0),
        |a, s| {
            if s.ty().is_branch() {
                M(a.0 + 1)
            } else {
                M(a.0)
            }
        },
    );
    let root = Ty::Class;
    let acc_root = builder.0.init(root, None);
    let meth = Ty::Method;
    let acc_meth = builder.0.init(meth, None);
    let if_statement = Ty::IfStatement;
    let acc_if_statement = builder.0.init(if_statement, None);
    let if_statement = STree(Ty::IfStatement, vec![]);
    let if_statement = builder.0.finish(acc_if_statement, if_statement);
    dbg!(if_statement.get::<M>().0);
    let acc_meth = builder.0.acc(acc_meth, &if_statement);
    let meth_statements = Children(vec![if_statement]);
    let meth = STree(Ty::Method, vec![Box::new(meth_statements)]);
    let meth = builder.0.finish(acc_meth, meth);
    let acc_root = builder.0.acc(acc_root, &meth);
    let class_members = Children(vec![meth]);
    let root = STree(root, vec![Box::new(class_members)]);
    let root = builder.0.finish(acc_root, root);
    dbg!(root.get::<M>().0);
}

use std::marker::PhantomData;

#[derive(Clone, Copy)]
enum Ty {
    Class,
    Method,
    IfStatement,
    WhileStatement,
}

impl Ty {
    fn is_branch(&self) -> bool {
        matches!(self, Ty::IfStatement | Ty::WhileStatement)
    }
}

trait Subtree {
    fn try_get<M: Copy + 'static>(&self) -> Option<M>;
    fn get<M: Copy + 'static>(&self) -> M {
        self.try_get().unwrap()
    }
    fn ty(&self) -> Ty;
    fn label(&self) -> Option<&str> { self.try_get() }
    fn push_metric<M: 'static>(&mut self, m: M);
}
struct STree(Ty, Vec<Box<dyn std::any::Any>>);
impl Subtree for STree {
    fn try_get<M: Copy + 'static>(&self) -> Option<M> {
        for x in &self.1 {
            let Some(m) = x.downcast_ref() else {
                continue;
            };
            return Some(*m);
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
trait MetricComputing {
    type S: Subtree;
    // fn pipe<O: MetricComputing<S = Self::S>>(self, o: O) -> impl MetricComputing<S = Self::S>;
    fn pipe<O: MetricComputing<S = Self::S>>(self, o: O) -> impl MetricComputing<S = Self::S>;
    type Acc;
    type M: 'static;
    fn init(&self, ty: Ty, l: Option<&str>) -> Self::Acc;
    fn finish(&self, acc: Self::Acc, current: Self::S) -> Self::S;
    fn acc(&self, acc: Self::Acc, current: &Self::S) -> Self::Acc;
}
struct Functional<T, U>(T, PhantomData<U>);
impl<S: Subtree> MetricComputing for Functional<(), S> {
    type S = S;
    fn pipe<O: MetricComputing<S = Self::S>>(self, o: O) -> impl MetricComputing<S = Self::S> {
        o
    }
    type Acc = ();
    fn init(&self, ty: Ty, l: Option<&str>) -> Self::Acc {
        ()
    }
    type M = ();
    fn finish(&self, acc: Self::Acc, current: Self::S) -> Self::S {
        current
    }
    fn acc(&self, acc: Self::Acc, current: &Self::S) -> Self::Acc {
        ()
    }
}
impl<A, M: 'static, I: Fn(Ty, Option<&str>) -> A, Acc: Fn(A, &S) -> A, F: Fn(A, &S) -> M, S: Subtree> MetricComputing
    for Functional<(I, Acc, F), (A, M, S)>
{
    type S = S;
    fn pipe<O: MetricComputing<S = Self::S>>(self, o: O) -> impl MetricComputing<S = Self::S> {
        Functional((self, o), PhantomData)
    }
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
impl<F0: MetricComputing, F1: MetricComputing<S = F0::S>> MetricComputing
    for Functional<(F0, F1), F0::S>
{
    type S = F0::S;
    fn pipe<O: MetricComputing<S = Self::S>>(self, o: O) -> impl MetricComputing<S = Self::S> {
        Functional((self, o), PhantomData)
    }
    type Acc = (F0::Acc, F1::Acc);
    fn init(&self, ty: Ty, l: Option<&str>) -> Self::Acc {
        (self.0 .0.init(ty, l), self.0 .1.init(ty, l))
    }
    type M = (F0::M, F1::M);
    fn finish(&self, acc: Self::Acc, current: Self::S) -> Self::S {
        let current = self.0 .0.finish(acc.0, current);
        let current = self.0 .1.finish(acc.1, current);
        current
    }
    fn acc(&self, acc: Self::Acc, current: &Self::S) -> Self::Acc {
        let a0 = self.0 .0.acc(acc.0, current);
        let a1 = self.0 .1.acc(acc.1, current);
        (a0, a1)
    }
}
struct Builder<C>(C);
impl<C: MetricComputing> Builder<C>
where
    C::S: Subtree,
{
    fn with_metric<A, M: 'static>(
        self,
        init: impl Fn(Ty, Option<&str>) -> A,
        acc: impl Fn(A, &C::S) -> A,
        finish: impl Fn(A, &C::S) -> M,
    ) -> Builder<impl MetricComputing<S = C::S>> {
        Builder(self.0.pipe(Functional((init, acc, finish), PhantomData)))
    }
}
struct Children(Vec<STree>);
struct Label(String);
