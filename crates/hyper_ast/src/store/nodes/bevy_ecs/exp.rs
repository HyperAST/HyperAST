use super::*;
use bevy_ecs::schedule::DynEq;
use std::any::Any;

// what about using systems like ECSs for the compounding

#[derive(Component, Debug)]
pub struct HLabelAcc(u64);

fn rec_h_label(acc: &mut HLabelAcc, value: &HLabel) {
    acc.0 += value.0;
}

fn compound_h_label((acc, size): (&HLabelAcc, Option<&TreeSize>)) -> HLabel {
    HLabel(acc.0 + size.map_or(1, |x| x.value() as u64 + 1))
}

// systems.add_rec(rec_h_label)
// systems.add_compound(compound_h_label)

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

struct CCC();

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

use strum_macros::EnumDiscriminants;

#[derive(Component, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(Component))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(TreeSizeDiscriminants))]
pub enum TreeSize {
    Leaf(tree_size::Leaf),
    Node(tree_size::Node),
}

mod tree_size {
    use super::*;
    #[derive(Component, Debug)]
    pub struct Leaf;
    #[derive(Component, Debug)]
    pub struct Node(pub usize);
}

pub trait EntityBuilder {
    fn add<T: Bundle>(&mut self, component: T) -> &mut Self;
}

impl EntityBuilder for EntityWorldMut<'_> {
    fn add<T: Bundle>(&mut self, component: T) -> &mut Self {
        self.insert(component)
    }
}

// WIP

trait SelfRecMd {
    fn acc(&mut self, from_child: &Self);
}

struct TreeSizeAcc(usize);

impl TreeSize {
    /// 1 + sum(child.size)
    fn value(&self) -> usize {
        match self {
            TreeSize::Leaf(_) => 1,
            TreeSize::Node(tree_size::Node(x)) => x + 1,
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
            TreeSize::Leaf(tree_size::Leaf)
        } else {
            TreeSize::Node(tree_size::Node(value.0))
        }
    }
}

// Rec eg. size, H_label adding *hashes* of children
// Compound eg. H_label using *size* and *label*

impl From<(HLabelAcc, TreeSize)> for HLabel {
    fn from(value: (HLabelAcc, TreeSize)) -> Self {
        todo!()
    }
}
