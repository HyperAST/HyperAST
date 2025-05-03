use bevy_ecs::prelude::*;

mod md_simple;

mod primary;
pub(self) use primary::*;

// | primary  |  md 0       md 1       md2     |
// | data 0   |  data 1  |  data 2  |  data 3  |
// |    always compute   |  if abs  &  cachd   |
// | eg. ty   | eg. hash | eg. bt_l | eg. refs |
//                                     lossy
//            for h <-|- size

mod md1;

// mod md2;

mod inner;

pub use inner::{HashedNodeRef, NodeIdentifier, NodeStore, NodeStoreInner};

pub use bevy_ecs::component::Component;

pub use super::legion::dyn_builder;
pub use inner::eq_node;
pub use inner::eq_space;

mod md_sys_tree_size {
    use super::*;

    #[derive(Component, Debug, Clone)]
    pub struct TreeSize(usize);
    pub struct TreeSizeAcc(usize);

    // self -> acc
    pub fn init() -> TreeSizeAcc {
        TreeSizeAcc(0)
    }
    // self -> md
    fn init_leaf((): ()) -> TreeSize {
        TreeSize(1)
    }
    // &mut acc , child, md
    pub fn acc(acc: &mut TreeSizeAcc, child: &TreeSize) {
        acc.0 += child.0;
    }
    // acc, self -> md
    pub fn finish(acc: TreeSizeAcc) -> TreeSize {
        TreeSize(acc.0 + 1)
    }

    impl TreeSize {
        pub fn to_usize(&self) -> usize {
            self.0
        }
    }
}

mod md_sys_byte_len {
    use super::*;

    pub fn aaa_byte_len((ty, label, children): (Type, Option<&Lab>, Option<&Children>)) -> ByteLen {
        let r = match (label, children) {
            (None, None) => ty.0.as_bytes().len(),
            (Some(Lab(l)), None) => l.as_bytes().len(),
            (_, Some(Children(cs))) => todo!(),
        };
        ByteLen(r)
    }

    pub struct ByteLenAcc(usize);

    // self -> acc
    pub fn init() -> ByteLenAcc {
        ByteLenAcc(0)
    }

    // &mut acc , child, md
    pub fn acc(acc: &mut ByteLenAcc, child: &ByteLen) {
        acc.0 += child.0;
    }

    // init and acc consititute a monoid, here neutral is init as 0 and acc as a plus

    type HasChildren = bool;

    // acc, self -> md
    pub fn finish(
        acc: ByteLenAcc,
        (ty, label, cs): (&Type, Option<impl ToString>, HasChildren),
    ) -> ByteLen {
        if cs {
            ByteLen(acc.0)
        } else {
            ByteLen(match label {
                None => ty.0.as_bytes().len(),
                Some(l) => l.to_string().as_bytes().len(),
            })
        }
    }
}

mod md_sys_tree_hash {
    use crate::hashed::inner_node_hash;
    use crate::utils::{self, clamp_u64_to_u32};

    use super::*;

    use super::md_sys_tree_size::TreeSize;

    #[derive(Component, Debug, Clone)]
    pub struct TreeHash(u32);
    impl TreeHash {
        // TODO concat syntax and structural hash
        pub(crate) fn to_u64(&self) -> u64 {
            self.0 as u64
        }
    }
    pub struct TreeHashAcc(u32);

    // self -> acc
    pub fn init() -> TreeHashAcc {
        TreeHashAcc(0)
    }
    // self -> md
    pub fn init_leaf(x: (&Type, Option<&impl std::hash::Hash>), s: &TreeSize) -> TreeHash {
        finish(init(), x, s)
    }
    // &mut acc , child, md
    pub fn acc(acc: &mut TreeHashAcc, child: &TreeHash) {
        acc.0 = acc.0.wrapping_add(child.0);
    }
    // acc, self -> md
    pub fn finish(
        acc: TreeHashAcc,
        (ty, label): (&Type, Option<&impl std::hash::Hash>),
        size: &TreeSize,
    ) -> TreeHash {
        let kind = clamp_u64_to_u32(&utils::hash(&ty.0));
        let label = clamp_u64_to_u32(&utils::hash(&label.map(|x| x)));
        let size = size.to_usize() as u32;
        TreeHash(inner_node_hash(kind, label, size, acc.0))
    }

    #[derive(Component, Debug, Clone)]
    pub struct TreeSizeNoSpace(usize);
    impl TreeSizeNoSpace {
        fn to_usize(&self) -> u32 {
            self.0 as u32
        }
    }
    fn finish_label(
        acc: TreeHashAcc,
        (ty, label, size): (Type, Option<&Lab>, TreeSizeNoSpace),
    ) -> TreeHash {
        if ty.is_space() {
            TreeHash(0)
        } else {
            let kind = clamp_u64_to_u32(&utils::hash(&ty.0));
            let label = clamp_u64_to_u32(&utils::hash(&label.map(|x| x.0)));
            let size = size.to_usize() as u32;
            TreeHash(inner_node_hash(kind, label, size, acc.0))
        }
    }
    fn finish_structure(acc: TreeHashAcc, (ty, size): (Type, TreeSizeNoSpace)) -> TreeHash {
        let kind = clamp_u64_to_u32(&utils::hash(&ty.0));
        let label = 0;
        let size = size.to_usize() as u32;
        TreeHash(inner_node_hash(kind, label, size, acc.0))
    }
}

mod compressed_component;

mod exp;

fn precompute_md<T: Bundle>(e: &mut EntityWorldMut<'_>, compute: impl Fn(&World, EntityRef) -> T) {
    e.insert(compute(e.world(), EntityRef::from(&*e)));
}

fn compute_byte_len_aux<'r>(e: EntityRef<'r>) -> Result<usize, &'r [Entity]> {
    let r = match (e.get::<Lab>(), e.get::<Children>()) {
        (None, None) => e.get::<Type>().unwrap().0.as_bytes().len(),
        (Some(Lab(l)), None) => l.as_bytes().len(),
        (_, Some(Children(cs))) => return Err(cs),
    };
    Ok(r)
}

fn compute_byte_len(w: &World, e: EntityRef) -> ByteLen {
    let r = match compute_byte_len_aux(e) {
        Ok(r) => r,
        Err(cs) => w
            .get_many_entities_dynamic(&cs)
            .expect("should all be there")
            .iter()
            .map(|x| x.get::<ByteLen>().unwrap().to_usize())
            .sum(),
    };
    ByteLen(r)
}

fn compute_rec_byte_len(w: &World, e: EntityRef) -> ByteLen {
    let r = match compute_byte_len_aux(e) {
        Ok(r) => r,
        Err(cs) => cs
            .iter()
            .map(|x| compute_rec_byte_len(w, w.entity(*x)).to_usize())
            .sum(),
    };
    ByteLen(r)
}

fn compute_hybr_byte_len(w: &World, e: EntityRef) -> ByteLen {
    let r = match compute_byte_len_aux(e) {
        Ok(r) => r,
        Err(cs) => w
            .get_many_entities_dynamic(&cs)
            .expect("should all be there")
            .iter()
            .map(|x| {
                x.get::<ByteLen>()
                    .map_or_else(|| compute_rec_byte_len(w, *x).to_usize(), |x| x.to_usize())
            })
            .sum(),
    };
    ByteLen(r)
}

fn compute_hybrec_byte_len(w: &World, e: EntityRef) -> ByteLen {
    let r = match compute_byte_len_aux(e) {
        Ok(r) => r,
        Err(cs) => w
            .get_many_entities_dynamic(&cs)
            .expect("should all be there")
            .iter()
            .map(|x| compute_hybr_byte_len(w, *x).to_usize())
            .sum(),
    };
    ByteLen(r)
}

type L = &'static str;
type Ty = &'static str;
type IdN = Entity;
type IdL = crate::store::defaults::LabelIdentifier;

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone)]
struct Type(pub Ty);
impl Type {
    fn is_space(&self) -> bool {
        self.0 == "Spaces"
    }
}
#[derive(Component)]
struct Lab(pub L);
#[derive(Component, Debug, Hash, PartialEq, Eq, Clone)]
struct Label(pub IdL);

#[derive(Component, Debug, Hash, PartialEq, Eq, Clone)]
struct Children(pub(crate) Box<[IdN]>);

#[derive(Component)]
struct Names(pub L);

#[derive(Component)]
struct HLabel(pub u64);

#[derive(Component, Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
struct ByteLen(usize);

impl ByteLen {
    fn to_usize(&self) -> usize {
        self.0
    }
}

pub enum Traversal<L = IdL, LL = String> {
    Down(Ty, Option<LL>),
    Right(Ty, Option<LL>, Option<L>),
    Up(Option<L>),
}

impl<L, LL> Traversal<L, LL> {
    fn map<M, MM>(&self, g: impl FnMut(&LL) -> MM, f: impl FnMut(&L) -> M) -> Traversal<M, MM> {
        match self {
            Traversal::Down(ty, l) => Traversal::Down(ty, l.as_ref().map(g)),
            Traversal::Right(ty, l, idl) => {
                Traversal::Right(ty, l.as_ref().map(g), idl.as_ref().map(f))
            }
            Traversal::Up(idl) => Traversal::Up(idl.as_ref().map(f)),
        }
    }
}

pub fn construction<Node, Acc, L, IdL>(
    mut it: impl Iterator<Item = Traversal<IdL, L>>,
    init: impl Fn(Ty, Option<L>) -> Acc,
    acc: impl Fn(&mut Acc, Node),
    mut finish: impl FnMut(Acc, Option<IdL>) -> Node,
) -> Node {
    let mut stack: Vec<(Acc,)> = vec![];
    loop {
        match it.next().unwrap() {
            Traversal::Down(ty, l) => {
                stack.push((init(ty, l),));
            }
            Traversal::Right(ty, l, idl) => {
                let c = stack.pop().unwrap();
                let id = finish(c.0, idl);
                let c = stack.last_mut().unwrap();
                acc(&mut c.0, id);
                stack.push((init(ty, l),));
            }
            Traversal::Up(idl) => {
                let c = stack.pop().unwrap();
                let id = finish(c.0, idl);
                if let Some(c) = stack.last_mut() {
                    acc(&mut c.0, id);
                } else {
                    return id;
                }
            }
        }
    }
}

#[cfg(test)]
pub static BIN: &[Traversal<&'static str, &'static str>] = &[
    Traversal::Down("bin_expr", None),
    Traversal::Down("number_lit", Some("42")),
    Traversal::Right("+", None, Some("42")),
    Traversal::Right("identifier", Some("x"), None),
    Traversal::Up(Some("x")),
    Traversal::Up(None),
];

#[cfg(test)]
pub static BIN_DUP: &[Traversal<&'static str, &'static str>] = &[
    Traversal::Down("bin_expr", None),
    Traversal::Down("identifier", Some("x")),
    Traversal::Right("+", None, Some("x")),
    Traversal::Right("identifier", Some("x"), None),
    Traversal::Up(Some("x")),
    Traversal::Up(None),
];

#[cfg(test)]
mod exp_bevy;
