use std::{borrow::Borrow, fmt::Debug, hash::Hash, ops::Deref};

use hashbrown::hash_map::DefaultHashBuilder;
use legion::{
    storage::{Archetype, Component, IntoComponentSource},
    world::{ComponentError, EntityLocation},
    EntityStore,
};
use num::ToPrimitive;

use crate::{
    filter::{Bloom, BloomResult, BloomSize, BF},
    hashed::{NodeHashs, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    nodes::{CompressedNode, HashSize, RefContainer, Space},
    store::labels::DefaultLabelIdentifier,
    types::{Type, Typed, WithChildren},
    utils::make_hash, impact::serialize::{MySerialize, CachedHasher, Keyed},
};

pub type NodeIdentifier = legion::Entity;
pub type EntryRef<'a> = legion::world::EntryRef<'a>;
pub struct HashedNodeRef<'a>(EntryRef<'a>);

pub struct HashedNode {
    node: CompressedNode<legion::Entity, DefaultLabelIdentifier>,
    hashs: SyntaxNodeHashs<u32>,
}

pub struct NodeStore {
    count: usize,
    errors: usize,
    // roots: HashMap<(u8, u8, u8), NodeIdentifier>,
    dedup: hashbrown::HashMap<NodeIdentifier, (), ()>,
    internal: legion::World,
    hasher: DefaultHashBuilder, //fasthash::city::Hash64,//fasthash::RandomState<fasthash::>,
                                // internal: VecMapStore<HashedNode, NodeIdentifier, legion::World>,
}

pub mod compo {
    pub struct More<T>(pub T);
    pub struct Size(pub u32);
    pub struct Height(pub u32);
    pub struct BytesLen(pub u32);

    pub struct HStruct(pub u32);
    pub struct HLabel(pub u32);
}

#[derive(PartialEq, Eq)]
struct CS0<T: Eq, const N: usize>([T; N]);
struct CSE<const N: usize>([legion::Entity; N]);
#[derive(PartialEq, Eq, Debug)]
pub struct CS<T: Eq>(pub Vec<T>);

// * hashed node impl

impl<'a> PartialEq for HashedNode {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<'a> Eq for HashedNode {}

impl<'a> Hash for HashedNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hashs.hash(&Default::default()).hash(state)
    }
}

impl crate::types::Node for HashedNode {}
impl crate::types::Stored for HashedNode {
    type TreeId = NodeIdentifier;
}

// impl Symbol<HashedNode> for legion::Entity {}

// * hashed node reference impl

impl<'a> PartialEq for HashedNodeRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.location().archetype() == other.0.location().archetype()
            && self.0.location().component() == other.0.location().component()
    }
}

impl<'a> Eq for HashedNodeRef<'a> {}

impl<'a> Hash for HashedNodeRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        crate::types::WithHashs::hash(self, &Default::default()).hash(state)
    }
}

impl<'a> Debug for HashedNodeRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HashedNodeRef")
            .field(&self.0.location())
            .finish()
    }
}

impl<'a> Deref for HashedNodeRef<'a> {
    type Target = EntryRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> HashedNodeRef<'a> {
    pub(crate) fn new(entry: EntryRef<'a>) -> Self {
        Self(entry)
    }

    /// Returns the entity's archetype.
    pub fn archetype(&self) -> &Archetype {
        self.0.archetype()
    }

    /// Returns the entity's location.
    pub fn location(&self) -> EntityLocation {
        self.0.location()
    }

    /// Returns a reference to one of the entity's components.
    pub fn into_component<T: Component>(self) -> Result<&'a T, ComponentError> {
        self.0.into_component::<T>()
    }

    /// Returns a mutable reference to one of the entity's components.
    ///
    /// # Safety
    /// This function bypasses static borrow checking. The caller must ensure that the component reference
    /// will not be mutably aliased.
    pub unsafe fn into_component_unchecked<T: Component>(
        self,
    ) -> Result<&'a mut T, ComponentError> {
        self.0.into_component_unchecked::<T>()
    }

    /// Returns a reference to one of the entity's components.
    pub fn get_component<T: Component>(&self) -> Result<&T, ComponentError> {
        self.0.get_component::<T>()
    }

    /// Returns a mutable reference to one of the entity's components.
    ///
    /// # Safety
    /// This function bypasses static borrow checking. The caller must ensure that the component reference
    /// will not be mutably aliased.
    pub unsafe fn get_component_unchecked<T: Component>(&self) -> Result<&mut T, ComponentError> {
        self.0.get_component_unchecked::<T>()
    }

    pub fn into_compressed_node(
        &self,
    ) -> Result<CompressedNode<legion::Entity, DefaultLabelIdentifier>, ComponentError> {
        if let Ok(spaces) = self.0.get_component::<Box<[Space]>>() {
            return Ok(CompressedNode::Spaces(spaces.clone()));
        }
        let kind = self.0.get_component::<Type>()?;
        let a = self.0.get_component::<DefaultLabelIdentifier>();
        let label: Option<DefaultLabelIdentifier> = a.ok().map(|x| x.clone());
        let children = self.0.get_component::<CS<legion::Entity>>();
        let children = children.ok().map(|x| x.0.clone());
        Ok(CompressedNode::new(
            *kind,
            label,
            children.unwrap_or_default(),
        ))
    }

    // TODO when relativisation is applied, caller of this method should provide the size of the paren ident
    pub fn get_bytes_len(&self, p_indent_len: u32) -> u32 {
        // use crate::types::Typed;
        if self.get_type() == Type::Spaces {
            self.get_component::<Box<[Space]>>()
                .expect("spaces node should have spaces")
                .iter()
                .map(|x| {
                    if x == &Space::ParentIndentation {
                        p_indent_len
                    } else {
                        1
                    }
                })
                .sum()
        } else {
            self.0
                .get_component::<compo::BytesLen>()
                .expect(&format!(
                    "node with type {:?} don't have a len",
                    self.get_type()
                ))
                .0
        }
        // .map_or_else(|_| self
        //     .get_type().to_string().len() as u32,|x|x.0)
    }

    // TODO when relativisation is applied, caller of this method should provide the size of the paren ident
    pub fn try_get_bytes_len(&self, p_indent_len: u32) -> Option<u32> {
        // use crate::types::Typed;
        if self.get_type() == Type::Spaces {
            let s = self.get_component::<Box<[Space]>>().ok()?;
            let s = s
                .iter()
                .map(|x| {
                    if x == &Space::ParentIndentation {
                        p_indent_len
                    } else {
                        1
                    }
                })
                .sum();
            Some(s)
        } else {
            self.0.get_component::<compo::BytesLen>().map(|x| x.0).ok()
        }
        // .map_or_else(|_| self
        //     .get_type().to_string().len() as u32,|x|x.0)
    }

    pub fn is_directory(&self) -> bool {
        self.get_type().is_directory()
    }

    pub fn get_child_by_name(
        &self,
        name: &<HashedNodeRef<'a> as crate::types::Labeled>::Label,
    ) -> Option<<HashedNodeRef<'a> as crate::types::Stored>::TreeId> {
        let labels = self
            .0
            .get_component::<CS<<HashedNodeRef<'a> as crate::types::Labeled>::Label>>()
            .ok()?;
        let idx = labels.0.iter().position(|x| x == name);
        idx.map(|idx| self.get_child(&idx.to_u16().unwrap()))
    }

    pub fn get_child_idx_by_name(
        &self,
        name: &<HashedNodeRef<'a> as crate::types::Labeled>::Label,
    ) -> Option<<HashedNodeRef<'a> as crate::types::WithChildren>::ChildIdx> {
        let labels = self
            .0
            .get_component::<CS<<HashedNodeRef<'a> as crate::types::Labeled>::Label>>()
            .ok()?;
        labels
            .0
            .iter()
            .position(|x| x == name)
            .map(|x| x.to_u16().unwrap())
    }
}

impl<'a> AsRef<HashedNodeRef<'a>> for HashedNodeRef<'a> {
    fn as_ref(&self) -> &HashedNodeRef<'a> {
        self
    }
}

impl<'a> crate::types::Typed for HashedNodeRef<'a> {
    type Type = Type;

    fn get_type(&self) -> Type {
        *self.0.get_component::<Type>().unwrap()
    }
}

impl<'a> crate::types::Labeled for HashedNodeRef<'a> {
    type Label = DefaultLabelIdentifier;

    fn get_label(&self) -> &DefaultLabelIdentifier {
        self.0
            .get_component::<DefaultLabelIdentifier>()
            .expect("check with self.has_label()")
    }
}

impl<'a> crate::types::Node for HashedNodeRef<'a> {}

impl<'a> crate::types::Stored for HashedNodeRef<'a> {
    type TreeId = NodeIdentifier;
}
impl<'a> crate::types::WithChildren for HashedNodeRef<'a> {
    type ChildIdx = u16;

    fn child_count(&self) -> u16 {
        self.0
            .get_component::<CS<legion::Entity>>()
            .unwrap_or_else(|x| {
                panic!(
                    "too much children: {}\n{}",
                    x,
                    std::backtrace::Backtrace::force_capture()
                );
            })
            .0
            .len()
            .to_u16()
            .expect("too much children")
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.0
            .get_component::<CS<legion::Entity>>()
            .unwrap_or_else(|x| {
                log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                panic!("{}",x)
            })
            .0.get(num::cast::<_, usize>(*idx).unwrap()).map(|x|*x).unwrap_or_else(|| {
                log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                panic!()
            })
    }

    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        let cs = self.0
            .get_component::<CS<legion::Entity>>()
            .unwrap_or_else(|x| {
                log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                panic!("{}",x)
            });
        let v = &cs.0;
        v[v.len() - 1 - num::cast::<_, usize>(*idx).unwrap()]
    }

    fn get_children<'b>(&'b self) -> &'b [Self::TreeId] {
        let cs = self.0
            .get_component::<CS<legion::Entity>>()
            .unwrap_or_else(|x| {
                log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                panic!("{}",x)
            });
        cs.0.as_slice()
    }
}

impl<'a> crate::types::WithHashs for HashedNodeRef<'a> {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        self.0
            .get_component::<SyntaxNodeHashs<Self::HP>>()
            .unwrap()
            .hash(kind)
    }
}

impl<'a> crate::types::Tree for HashedNodeRef<'a> {
    fn has_children(&self) -> bool {
        self.get_component::<CS<crate::store::nodes::legion::NodeIdentifier>>()
            .map(|x| !x.0.is_empty())
            .unwrap_or(false)
    }

    fn has_label(&self) -> bool {
        self.0.get_component::<DefaultLabelIdentifier>().is_ok()
    }

    fn try_get_label(&self) -> Option<&Self::Label> {
        self.0.get_component::<DefaultLabelIdentifier>().ok()
    }
}

impl<'a> HashedNodeRef<'a> {}

impl<'a> RefContainer for HashedNodeRef<'a> {
    type Result = BloomResult;

    fn check<U: MySerialize+Keyed<usize>>(&self, rf: U) -> Self::Result {
        use crate::filter::BF as _;

        let e = self.0.get_component::<BloomSize>().unwrap();

        macro_rules! check {
            ( $($t:ty),* ) => {
                match *e {
                    BloomSize::Much => {
                        log::trace!("[Too Much]");
                        BloomResult::MaybeContain
                    },
                    BloomSize::None => BloomResult::DoNotContain,
                    $( <$t>::SIZE => {
                        let x = CachedHasher::<usize,<$t as BF<[u8]>>::S, <$t as BF<[u8]>>::H>::once(rf);
                        let x = x.into_iter().map(|x|<$t>::check_raw(self.0.get_component::<$t>().unwrap(), x));

                        for x in x {
                            if let BloomResult::MaybeContain = x {
                                return BloomResult::MaybeContain
                            }
                        }
                        BloomResult::DoNotContain
                    }),*
                }
            };
        }
        check![
            Bloom<&'static [u8], u16>,
            Bloom<&'static [u8], u32>,
            Bloom<&'static [u8], u64>,
            Bloom<&'static [u8], [u64; 2]>,
            Bloom<&'static [u8], [u64; 4]>,
            Bloom<&'static [u8], [u64; 8]>,
            Bloom<&'static [u8], [u64; 16]>,
            Bloom<&'static [u8], [u64; 32]>,
            Bloom<&'static [u8], [u64; 64]>
        ]
    }
}

// impl<'a> Symbol<HashedNodeRef<'a>> for legion::Entity {}

// * Node store impl

pub struct PendingInsert<'a>(
    crate::compat::hash_map::RawEntryMut<'a, legion::Entity, (), ()>,
    (u64, &'a mut legion::World, &'a DefaultHashBuilder),
);

impl<'a> PendingInsert<'a> {
    pub fn occupied_id(&self) -> Option<NodeIdentifier> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => Some(occupied.key().clone()),
            _ => None,
        }
    }
    pub fn resolve(&self, id: NodeIdentifier) -> HashedNodeRef {
        self.1 .1.entry_ref(id).map(|x| HashedNodeRef(x)).unwrap()
    }
    pub fn occupied(
        &'a self,
    ) -> Option<(
        NodeIdentifier,
        (u64, &'a legion::World, &'a DefaultHashBuilder),
    )> {
        match &self.0 {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
                Some((occupied.key().clone(), (self.1 .0, self.1 .1, self.1 .2)))
            }
            _ => None,
        }
    }

    pub fn vacant(
        self,
    ) -> (
        crate::compat::hash_map::RawVacantEntryMut<'a, legion::Entity, (), ()>,
        (u64, &'a mut legion::World, &'a DefaultHashBuilder),
    ) {
        match self.0 {
            hashbrown::hash_map::RawEntryMut::Vacant(occupied) => (occupied, self.1),
            _ => panic!(),
        }
    }
    // pub fn occupied(&self) -> Option<(
    //     crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
    //     (u64, &mut legion::World, &DefaultHashBuilder),
    // )> {
    //     match self.0 {
    //         hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
    //             Some(occupied.into_key_value().0.clone())
    //         }
    //         _ => None
    //     }
    // }
}

impl NodeStore {
    pub fn prepare_insertion<'a, Eq: Fn(EntryRef) -> bool, V: Hash>(
        &'a mut self,
        hashable: &'a V,
        eq: Eq,
    ) -> PendingInsert {
        let Self {
            dedup,
            internal: backend,
            ..
        } = self;
        let hash = make_hash(&self.hasher, hashable);
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            let r = eq(backend.entry_ref(*symbol).unwrap());
            r
        });
        PendingInsert(entry, (hash, &mut self.internal, &self.hasher))
    }

    pub fn insert_after_prepare<T>(
        (vacant, (hash, internal, hasher)): (
            crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
            (u64, &mut legion::World, &DefaultHashBuilder),
        ),
        components: T,
    ) -> legion::Entity
    where
        Option<T>: IntoComponentSource,
    {
        let (&mut symbol, &mut ()) = {
            let symbol = internal.push(components);
            vacant.insert_with_hasher(hash, symbol, (), |id| {
                let node = internal.entry_ref(*id).map(|x| HashedNodeRef(x)).unwrap();
                make_hash(hasher, &node)
            })
        };
        symbol
    }

    pub fn resolve(&self, id: NodeIdentifier) -> HashedNodeRef {
        self.internal
            .entry_ref(id)
            .map(|x| HashedNodeRef(x))
            .unwrap()
    }

    pub fn try_resolve(&self, id: NodeIdentifier) -> Option<HashedNodeRef> {
        self.internal.entry_ref(id).map(|x| HashedNodeRef(x)).ok()
    }
}

impl Debug for NodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeStore")
            .field("count", &self.count)
            .field("errors", &self.errors)
            .field("internal_len", &self.internal.len())
            // .field("internal", &self.internal)
            .finish()
    }
}

impl<'b, 'a: 'b> crate::types::NodeStore<'a, NodeIdentifier, HashedNodeRef<'b>> for NodeStore {
    fn resolve(&'a self, id: &NodeIdentifier) -> HashedNodeRef<'b> {
        self.internal
            .entry_ref(id.clone())
            .map(|x| HashedNodeRef(x))
            .unwrap()
    }
}

impl<'a> crate::types::NodeStoreMut<'a, HashedNode, HashedNodeRef<'a>> for NodeStore {}

impl NodeStore {
    pub fn len(&self) -> usize {
        self.internal.len()
    }
}

impl NodeStore {
    pub fn new() -> Self {
        Self {
            count: 0,
            errors: 0,
            // roots: Default::default(),
            internal: Default::default(),
            dedup: hashbrown::HashMap::<_, (), ()>::with_capacity_and_hasher(
                1 << 10,
                Default::default(),
            ),
            hasher: Default::default(),
        }
    }
}
