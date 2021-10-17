
enum A {
    Bot
}

impl Enum for A {
    type SizeT=u16;

    const SIZE: u16 = 1;
}


impl From<u16> for A {
    fn from(x: u16) -> Self {
        if x == 0 {
            A::Bot
        } else {panic!()}
    }
}

trait Enum: From<Self::SizeT> {
    type SizeT;
    const SIZE: Self::SizeT;
}

trait Bot<T> {
    const BotValue: T;
}

trait EmptyCommit: Bot<String> {}

enum EitherIndex<Spe, Idx> {
    Special(Spe),
    Index(Idx),
}

trait CustomIndex {
    type Idx: Restricted;
    type Special: Enum;

    fn value(&self) -> EitherIndex<Self::Special, Self::Idx>;
}

trait Restricted {
    fn value(&self) -> usize;
}

struct SpecialIndexS<Idx, Spe> {
    value: Idx,
    phantom: PhantomData<*const Spe>,
}

impl<Idx: Restricted + Copy + PrimInt, Spe: Enum<SizeT = Idx>> CustomIndex
    for SpecialIndexS<Idx, Spe>
{
    type Idx = Idx;
    type Special = Spe;

    fn value(&self) -> EitherIndex<Spe, Idx> {
        match self.value {
            x if x < Spe::SIZE => {
                EitherIndex::Special(Spe::from(x - (Idx::max_value() - Spe::SIZE)))
            }
            x => EitherIndex::Index(x),
        }
        // if self.value < Spe::Size {
        //     EitherIndex::Index(self.value)
        // } else {
        //     EitherIndex::Index(self.value)
        // }
    }
}

impl Restricted for u16 {
    fn value(&self) -> usize {
        *self as usize
    }
}

struct CustomIdxVec<Idx: Restricted, T,const D:T> {
    internal: Vec<T>,
    phantom: PhantomData<*const Idx>,
}

impl<Idx: CustomIndex, T> core::ops::Index<Idx> for CustomIdxVec<Idx::Idx, T> {
    type Output = T;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.internal[index.value()]
    }
}

struct CommmitId(String);

struct Versions<IdV: CustomIndex> {
    names: CustomIdxVec<IdV::Idx, SemVer>,
    commits: CustomIdxVec<IdV::Idx, CommmitId>,
    first_parents: CustomIdxVec<IdV::Idx, IdV>,
    other_parents: Vec<(IdV::Idx, IdV)>,
}