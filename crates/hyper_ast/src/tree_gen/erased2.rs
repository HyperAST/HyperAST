use std::{any::Any, convert::Infallible, ops::Deref};

use util::DynEq;

/// id -> Processor {cache, query, rec}
struct Map;

trait Entry: DynEq {
    // fn as_ref(&self) -> &Self;
}

struct Registry(Vec<Box<dyn Entry>>);

trait AAA {
    type In;
    type Obj;
    type Acc;
    type Out;
    fn pre(id: Self::In, o: Self::Obj) -> Self::Acc;
    fn post(o: Self::Obj, acc: Self::Acc);
}

trait Data: 'static {}
impl<T: 'static + Sized> Data for T {}

trait PrimaryData: Data {}

/// metadata
trait DerivedData: Data {}

trait Subtree {
    fn _md(&self, id: std::any::TypeId) -> &dyn Any;
}

type Idx = usize;

trait SubtreeExt: Subtree {
    type Ty: PrimaryData;
    fn ty(&self) -> Self::Ty;
    type Label: PrimaryData;
    fn label(&self) -> Self::Label;
    type I;
    fn child(&self, idx: Idx) -> Self::I;
    fn md<Md: Data>(&self) -> &Md {
        self._md(std::any::TypeId::of::<Md>())
            .downcast_ref()
            .unwrap()
    }
}

struct PrimaryAcc<Ty, L, Cs> {
    ty: Ty,
    l: L,
    cs: Cs,
}

type InFileAcc<Ty, L, I> = PrimaryAcc<Ty, Option<L>, Vec<I>>;
type InDirAcc<Ty, L, I> = PrimaryAcc<Ty, (), (Vec<L>, Vec<I>)>;

trait Acc {}

trait MdAcc {}

trait SelfRecMd {
    fn acc(&mut self, from_child: &Self);
}

trait CompountMd<Compounds>: Sized {
    fn compound(c: Compounds) -> Self;
}

impl<T, Compounds: Data> CompountMd<Compounds> for T
where
    T: From<Compounds>,
{
    fn compound(c: Compounds) -> Self {
        c.into()
    }
}

struct TreeSizeAcc(usize);

enum TreeSize {
    Leaf,
    Node(usize),
}

impl TreeSize {
    /// 1 + sum(child.size)
    fn value(&self) -> usize {
        match self {
            TreeSize::Leaf => 1,
            TreeSize::Node(x) => x + 1,
        }
    }
}

impl SelfRecMd for TreeSizeAcc {
    fn acc(&mut self, from_child: &Self) {
        self.0 += from_child.0;
    }
}

impl From<TreeSizeAcc> for TreeSize {
    fn from(value: TreeSizeAcc) -> Self {
        if value.0 == 0 {
            TreeSize::Leaf
        } else {
            TreeSize::Node(value.0)
        }
    }
}

// Rec eg. size, H_label adding *hashes* of children
// Compound eg. H_label using *size* and *label*

struct HLabelAcc(u64);
struct HLabel(u64);

impl From<(HLabelAcc, TreeSize)> for HLabel {
    fn from(value: (HLabelAcc, TreeSize)) -> Self {
        todo!()
    }
}

// what about using systems like ECSs for the compounding

fn rec_h_label(acc: &mut HLabelAcc, value: &HLabel) {
    acc.0 += value.0;
}

fn compound_h_label((acc, size): (&HLabelAcc, Option<&TreeSize>)) -> HLabel {
    HLabel(acc.0 + size.map_or(1, |x| x.value() as u64 + 1))
}

// systems.add_rec(rec_h_label)
// systems.add_compound(compound_h_label)

fn add_rec<A: hecs::Component, C: Clone + hecs::Component>(
    e: &mut hecs::EntityBuilder,
    child: &mut hecs::EntityRef,
    f: impl Fn(&mut A, &C),
) {
    let c = child.get::<&C>().unwrap();
    use std::ops::Deref;
    let c = c.deref();
    let a: &mut A = e.get_mut().unwrap();
    f(a, c)
}
struct CCC();

fn f(e: &mut hecs::EntityBuilder) -> &CCC {
    let a = e.get::<&CCC>().unwrap();
    a
}

fn g<'a, 'b>(e: &'b mut hecs::EntityRef<'a>) -> hecs::Ref<'a, CCC> {
    // e.query::<&CCC>().get().unwrap()
    let a = e.get::<&CCC>().unwrap();
    a
}

mod util {
    use core::any::Any;
    use core::fmt::Debug;
    use std::cmp::Ordering;

    pub trait DynEq: Any {
        fn as_any(&self) -> &dyn Any;
        fn do_eq(&self, rhs: &dyn DynEq) -> bool;
    }

    impl<T> DynEq for T
    where
        T: Any + Eq,
    {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn do_eq(&self, rhs: &dyn DynEq) -> bool {
            if let Some(rhs_concrete) = rhs.as_any().downcast_ref::<Self>() {
                self == rhs_concrete
            } else {
                false
            }
        }
    }

    impl PartialEq for dyn DynEq {
        fn eq(&self, rhs: &Self) -> bool {
            self.do_eq(rhs)
        }
    }

    pub trait DynOrd {
        fn as_any(&self) -> &dyn Any;
        fn do_cmp(&self, rhs: &dyn DynOrd) -> Option<Ordering>;
    }

    impl<T> DynOrd for T
    where
        T: Any + Ord,
    {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn do_cmp(&self, other: &dyn DynOrd) -> Option<Ordering> {
            if let Some(other) = other.as_any().downcast_ref::<T>() {
                return Some(self.cmp(other));
            }
            None
        }
    }
}
