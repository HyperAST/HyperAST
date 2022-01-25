///! fully compress all subtrees
use std::{
    cell::RefCell, collections::{HashMap, hash_map::RandomState}, fmt::Debug, hash::Hash, marker::PhantomData,
    num::NonZeroU64, ops::Deref, vec, borrow::Borrow,
};

use fasthash::t1ha0::Hasher64;
use legion::{
    storage::{Archetype, Component, IntoComponentSource},
    world::{ComponentError, EntityAccessError, EntityLocation, EntryRef},
    EntityStore,
};
use rusted_gumtree_core::tree::tree::{
    LabelStore as LabelStoreTrait, Labeled, NodeStore as NodeStoreTrait, Tree, Type, Typed,
    VersionedNodeStore, WithChildren, Stored, VersionedNodeStoreMut,
};
use string_interner::{DefaultHashBuilder, DefaultSymbol, StringInterner};
use tree_sitter::{Language, Parser, TreeCursor};
use tuples::CombinConcat;
use rusted_gumtree_core::tree::tree::NodeStoreMut as NodeStoreMutTrait;

use crate::{
    full::FullNode,
    hashed::{self, HashedCompressedNode, NodeHashs, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    nodes::{CompressedNode, HashSize, SimpleNode1, Space, self},
    store::{
        vec_map_store::{AsNodeEntityRef, Backend, Symbol, SymbolU32, VecMapStore, VecSymbol},
        TypeStore,
    },
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, hash_for_node, label_for_cursor,
        AccIndentation, Accumulator, Spaces, TreeGen,
    },
    utils::{self, clamp_u64_to_u32, make_hash},
};

// #[derive(PartialEq, Eq, Hash, Debug)]
// pub struct HashedNodeOld(
//     HashedCompressedNode<SyntaxNodeHashs<HashSize>, legion::Entity, LabelIdentifier>,
// );

type NodeIdentifier = legion::Entity;

struct HashedNodeRef<'a>(EntryRef<'a>);

struct HashedNode {
    node: CompressedNode<legion::Entity, LabelIdentifier>,
    hashs: SyntaxNodeHashs<u32>,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
}

impl HashedNode {
    fn new(
        node: CompressedNode<legion::Entity, LabelIdentifier>,
        hashs: SyntaxNodeHashs<u32>,
        metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ) -> Self {
        Self {
            node,
            hashs,
            metrics,
        }
    }
}

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

impl rusted_gumtree_core::tree::tree::Node for HashedNode {}
impl rusted_gumtree_core::tree::tree::Stored for HashedNode {
    type TreeId = NodeIdentifier;
}

impl Symbol<HashedNode> for legion::Entity {}
impl<'a> Symbol<HashedNodeRef<'a>> for legion::Entity {}

// impl Deref for HashedNodeDeref {
//     type Target = HashedNodeRef<'a>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

impl<'a> PartialEq for HashedNodeRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.location().archetype() == other.0.location().archetype()
            && self.0.location().component() == other.0.location().component()
    }
}

impl<'a> Eq for HashedNodeRef<'a> {}

impl<'a> Hash for HashedNodeRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        rusted_gumtree_core::tree::tree::WithHashs::hash(self, &Default::default()).hash(state)
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
        panic!()
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

    fn into_compressed_node(
        &self,
    ) -> Result<CompressedNode<legion::Entity, LabelIdentifier>, ComponentError> {
        if let Ok(spaces) = self.0.get_component::<Box<[Space]>>() {
            return Ok(CompressedNode::Spaces(spaces.clone()));
        }
        let kind = self.0.get_component::<Type>()?;
        let a = self.0.get_component::<LabelIdentifier>();
        let label: Option<LabelIdentifier> = a.ok().map(|x| x.clone());
        let children = self.0.get_component::<CS<legion::Entity>>();
        let children = children.ok().map(|x| x.0.clone());
        Ok(CompressedNode::new(
            *kind,
            label,
            children.unwrap_or_default(),
        ))
    }
}

impl<'a> AsRef<HashedNodeRef<'a>> for HashedNodeRef<'a> {
    fn as_ref(&self) -> &HashedNodeRef<'a> {
        self
    }
}

impl<'a> rusted_gumtree_core::tree::tree::Typed for HashedNodeRef<'a> {
    type Type = Type;

    fn get_type(&self) -> Type {
        *self.0.get_component::<Type>().unwrap()
    }
}

impl<'a> rusted_gumtree_core::tree::tree::Labeled for HashedNodeRef<'a> {
    type Label = LabelIdentifier;

    fn get_label(&self) -> &LabelIdentifier {
        self.0.get_component::<LabelIdentifier>().unwrap()
    }
}

impl<'a> rusted_gumtree_core::tree::tree::Node for HashedNodeRef<'a> {}

impl<'a> rusted_gumtree_core::tree::tree::Stored for HashedNodeRef<'a> {
    type TreeId = NodeIdentifier;
}
impl<'a> rusted_gumtree_core::tree::tree::WithChildren for HashedNodeRef<'a> {
    type ChildIdx = u8;

    fn child_count(&self) -> u8 {
        self.0
            .get_component::<CS<legion::Entity>>()
            .unwrap()
            .0
            .len() as u8
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.0.get_component::<CS<legion::Entity>>().unwrap().0
            [num::cast::<_, usize>(*idx).unwrap()]
    }
    
    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        let v = &self.0.get_component::<CS<legion::Entity>>().unwrap().0;
        v[v.len()-1-num::cast::<_, usize>(*idx).unwrap()]
    }

    fn get_children<'b>(&'b self) -> &'b [Self::TreeId] {
        self.0
            .get_component::<CS<legion::Entity>>()
            .unwrap()
            .0
            .as_slice()
    }
}

impl<'a> rusted_gumtree_core::tree::tree::WithHashs for HashedNodeRef<'a> {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        self.0
            .get_component::<SyntaxNodeHashs<Self::HP>>()
            .unwrap()
            .hash(kind)
    }
}

impl<'a> rusted_gumtree_core::tree::tree::Tree for HashedNodeRef<'a> {
    fn has_children(&self) -> bool {
        self.0
            .get_component::<CS<legion::Entity>>()
            .map(|x| !x.0.is_empty())
            .unwrap_or(false)
    }

    fn has_label(&self) -> bool {
        self.0.get_component::<LabelIdentifier>().is_ok()
    }
}

impl<'a> HashedNodeRef<'a> {
    // pub(crate) fn new(
    //     hashs: SyntaxNodeHashs<HashSize>,
    //     node: CompressedNode<NodeIdentifier, LabelIdentifier>,
    // ) -> Self {
    //     Self(HashedCompressedNode::new(hashs, node))
    // }
}

// pub type HashedNode<'a> = HashedCompressedNode<SyntaxNodeHashs<HashSize>,SymbolU32<&'a HashedNode>,LabelIdentifier>;

extern "C" {
    fn tree_sitter_java() -> Language;
}

type MyLabel = str;
type LabelIdentifier = DefaultSymbol;

pub struct JavaTreeGen {
    pub line_break: Vec<u8>,
    pub stores: SimpleStores,
}

// type SpacesStoreD = SpacesStore<u16, 4>;

pub struct LabelStore {
    count: usize,
    internal: StringInterner, //VecMapStore<OwnedLabel, LabelIdentifier>,
}

impl Debug for LabelStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LabelStore")
            .field("count", &self.count)
            .field("internal_len", &self.internal.len())
            .field("internal", &self.internal)
            .finish()
    }
}

impl LabelStoreTrait<MyLabel> for LabelStore {
    type I = LabelIdentifier;
    fn get_or_insert<T: Borrow<MyLabel>>(&mut self, node: T) -> Self::I {
        self.count += 1;
        self.internal.get_or_intern(node.borrow())
    }

    fn resolve(&self, id: &Self::I) -> &MyLabel {
        self.internal.resolve(*id).unwrap()
    }
}

impl AsNodeEntityRef for HashedNode {
    type Ref<'a> = HashedNodeRef<'a>;

    fn eq(&self, other: &Self::Ref<'_>) -> bool {
        if self.node.get_type() != other.get_type() {
            return false;
        };
        todo!()
    }
}

pub mod compo {

    pub struct Md(pub u64);
    pub struct Size(pub u32);
    pub struct Height(pub u32);

    pub struct HStruct(pub u32);
    pub struct HLabel(pub u32);
}

trait IntoTuple {
    type Out;
    fn to_tuple(self) -> Self::Out;
}

#[derive(PartialEq, Eq)]
struct CS0<T: Eq, const N: usize>([T; N]);
struct CSE<const N: usize>([legion::Entity; N]);
#[derive(PartialEq, Eq)]
struct CS<T: Eq>(Vec<T>);

impl Backend<HashedNode> for legion::World {
    type Symbol = NodeIdentifier;

    fn len(&self) -> usize {
        legion::World::len(self)
    }

    fn with_capacity(cap: usize) -> Self {
        legion::World::new(legion::WorldOptions::default())
    }

    fn intern(&mut self, thing: HashedNode) -> Self::Symbol {
        // type MT = (Type, u32);
        // struct Mandatory {
        //     kind: Type,
        //     hash: u32,
        // }

        // impl Mandatory {
        //     fn to_tuple(self) -> MT {
        //         (self.kind, self.hash)
        //     }

        //     fn to_compare<T: Eq>(self, x: T) -> Compare<T> {
        //         Compare {
        //             mandatory: self,
        //             compare: x,
        //         }
        //     }
        // }

        // struct Compare<T: Eq> {
        //     mandatory: Mandatory,
        //     compare: T,
        // }

        // impl IntoTuple for Compare<()> {
        //     type Out = (Type, u32);
        //     fn to_tuple(self) -> Self::Out {
        //         self.mandatory.to_tuple()
        //     }
        // }

        // impl<T: Eq> IntoTuple for Compare<(T,)> {
        //     type Out = (Type, u32, T);
        //     fn to_tuple(self) -> (Type, u32, T) {
        //         self.mandatory.to_tuple().concat(self.compare)
        //     }
        // }

        // impl<T0: Eq, T1: Eq> IntoTuple for Compare<(T0, T1)> {
        //     type Out = (Type, u32, T0, T1);
        //     fn to_tuple(self) -> (Type, u32, T0, T1) {
        //         self.mandatory.to_tuple().concat(self.compare)
        //     }
        // }

        // // impl IntoComponentSource for A {
        // // }

        // let mandatory = Mandatory {
        //     kind: thing.node.get_type(),
        //     hash: thing.hashs.syntax,
        // };

        // let optional = (
        //     compo::Size(thing.metrics.size),
        //     compo::Height(thing.metrics.height),
        //     compo::HStruct(thing.hashs.structt),
        //     compo::HLabel(thing.hashs.label),
        //     compo::Md(0),
        // );
        // if thing.node.has_label() {
        //     let label: LabelIdentifier = thing.node.get_label().clone();

        //     let compare = Compare {
        //         mandatory,
        //         compare: (label,),
        //     };

        //     let tuple = compare.to_tuple().concat(optional);

        //     legion::World::push(self, tuple)
        // } else {
        //     let optional = (
        //         // thing.metrics.size,
        //         // thing.metrics.height,
        //         // thing.hashs.structt,
        //         // thing.hashs.label,
        //         compo::Md(0),
        //     );
        //     if !thing.node.has_children() {
        //         legion::World::push(self, mandatory.to_compare(()).to_tuple().concat(optional))
        //     } else {
        //         let children = thing.node.get_children();
        //         match children.len() {
        //             0 => panic!(),
        //             1 => legion::World::push(
        //                 self,
        //                 mandatory
        //                     .to_compare((CS0([children[0]]),))
        //                     .to_tuple()
        //                     .concat(optional),
        //             ),
        //             2 => legion::World::push(
        //                 self,
        //                 mandatory
        //                     .to_compare((CS0([children[0], children[1]]),))
        //                     .to_tuple()
        //                     .concat(optional),
        //             ),
        //             3 => legion::World::push(
        //                 self,
        //                 mandatory
        //                     .to_compare((CS0([children[0], children[1], children[2]]),))
        //                     .to_tuple()
        //                     .concat(optional),
        //             ),
        //             _ => {
        //                 // let (t0, children) = children.split_first().unwrap();
        //                 let x = mandatory
        //                     .to_compare((
        //                         CS0([children[0], children[1], children[2]]),
        //                         CS(children[3..].to_owned()),
        //                     ))
        //                     .to_tuple()
        //                     .concat(optional);
        //                 legion::World::push(self, x)
        //             }
        //         }
        //     }
        // };
        todo!();
        legion::World::push(self, (thing,))
    }

    fn shrink_to_fit(&mut self) {}

    fn resolve(&self, symbol: Self::Symbol) -> Option<<HashedNode as AsNodeEntityRef>::Ref<'_>> {
        let entity = symbol;
        let er = legion::EntityStore::entry_ref(self, entity).ok();
        er.map(|x| HashedNodeRef(x))
    }

    unsafe fn resolve_unchecked(
        &self,
        symbol: Self::Symbol,
    ) -> <HashedNode as AsNodeEntityRef>::Ref<'_> {
        let entity = symbol;
        let er = legion::EntityStore::entry_ref(self, entity).unwrap();
        HashedNodeRef(er)
    }
}

pub struct NodeStore {
    count: usize,
    errors: usize,
    roots: HashMap<(u8, u8, u8), NodeIdentifier>,
    dedup: hashbrown::HashMap<NodeIdentifier, (), ()>,
    internal: legion::World,
    hasher: DefaultHashBuilder, //fasthash::city::Hash64,//fasthash::RandomState<fasthash::>,
                                // internal: VecMapStore<HashedNode, NodeIdentifier, legion::World>,
}

pub struct PendingInsert<'a, V>(u64, &'a V, &'a mut legion::World, &'a DefaultHashBuilder);

impl NodeStore {
    pub fn prepare_insertion<'a, Eq: Fn(EntryRef) -> bool, V: Hash>(
        &'a mut self,
        hashable: &'a V,
        eq: Eq,
    ) -> (
        crate::compat::hash_map::RawEntryMut<legion::Entity, (), ()>,
        PendingInsert<V>,
    ) {
        let Self {
            dedup,
            internal: backend,
            ..
        } = self;
        let hash = make_hash(&self.hasher, hashable);
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            // SAFETY: This is safe because we only operate on symbols that
            //         we receive from our backend making them valid.
            // node.eq(unsafe { backend.resolve_unchecked(*symbol) })
            let r = eq(backend.entry_ref(*symbol).unwrap());
            // if !r {
            //     println!("{}", hash);
            // }
            r
        });
        (
            entry,
            PendingInsert(hash, hashable, &mut self.internal, &self.hasher),
        )
    }

    pub fn insert_after_prepare<T, V: Hash>(
        vacant: crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
        PendingInsert(hash, hashable, internal, hasher): PendingInsert<V>,
        components: T,
    ) -> legion::Entity
    where
        Option<T>: IntoComponentSource,
    {
        let (&mut symbol, &mut ()) = {
            let symbol = internal.push(components);
            vacant.insert_with_hasher(hash, symbol, (), |id| {
                // SAFETY: This is safe because we only operate on symbols that
                //         we receive from our backend making them valid.
                let node = internal.entry_ref(*id).map(|x| HashedNodeRef(x)).unwrap();
                make_hash(hasher, &node)
                // make_hash(hasher, &hashable)
            })
        };
        symbol
    }

    pub fn insert<Eq: Fn(EntryRef) -> bool, V: Hash, T, U: FnOnce() -> T>(
        &mut self,
        hashable: &V,
        eq: Eq,
        make_components: U,
    ) -> legion::Entity
    where
        Option<T>: IntoComponentSource,
    {
        let Self {
            dedup,
            internal: backend,
            ..
        } = self;
        let hash = make_hash(&self.hasher, hashable);
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            // SAFETY: This is safe because we only operate on symbols that
            //         we receive from our backend making them valid.
            // node.eq(unsafe { backend.resolve_unchecked(*symbol) })
            eq(backend.entry_ref(*symbol).unwrap())
        });
        use crate::compat::hash_map::RawEntryMut;
        let (&mut symbol, &mut ()) = match entry {
            RawEntryMut::Occupied(occupied) => occupied.into_key_value(),
            RawEntryMut::Vacant(vacant) => {
                let symbol = self.internal.push(make_components());
                vacant.insert_with_hasher(hash, symbol, (), |id| {
                    // SAFETY: This is safe because we only operate on symbols that
                    //         we receive from our backend making them valid.
                    let node = self
                        .internal
                        .entry_ref(*id)
                        .map(|x| HashedNodeRef(x))
                        .unwrap();
                    make_hash(&self.hasher, &node)
                })
            }
        };
        symbol
    }

    fn resolve(&self, id: NodeIdentifier) -> HashedNodeRef {
        self.internal
            .entry_ref(id)
            .map(|x| HashedNodeRef(x))
            .unwrap()
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

impl<'a> NodeStoreTrait<'a, NodeIdentifier, HashedNodeRef<'a>> for NodeStore {

    fn resolve(&'a self, id: &NodeIdentifier) -> HashedNodeRef<'a> {
        self.internal
            .entry_ref(id.clone())
            .map(|x| HashedNodeRef(x))
            .unwrap()
    }
}

impl<'a> NodeStoreMutTrait<'a, HashedNode, HashedNodeRef<'a>> for NodeStore {

}

impl NodeStore {
    pub fn len(&self) -> usize {
        self.internal.len()
    }
}

impl<'a> VersionedNodeStore<'a, NodeIdentifier, HashedNodeRef<'a>> for NodeStore {
    fn resolve_root(&self, version: (u8, u8, u8), node: <HashedNode as Stored>::TreeId) {
        todo!()
    }
}

// impl<'a> VersionedNodeStoreMut<'a, HashedNode, HashedNodeRef<'a>> for NodeStore {
//     fn as_root(&mut self, version: (u8, u8, u8), id: NodeIdentifier) {
//         assert!(self.roots.insert(version, id).is_none());
//     }

//     fn insert_as_root(&mut self, version: (u8, u8, u8), node: HashedNode) -> <HashedNode as Stored>::TreeId {
//         let r = todo!();
//         self.as_root(version, r);
//         r
//     }
// }

#[derive(Debug)]
pub struct Global {
    pub(crate) depth: usize,
    pub(crate) position: usize,
}

#[derive(Debug)]
pub struct Local {
    pub(crate) compressed_node: NodeIdentifier,
    pub(crate) metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct SubTreeMetrics<U: NodeHashs> {
    pub(crate) hashs: U,
    pub(crate) size: u32,
    pub(crate) height: u32,
}

impl<U: NodeHashs> SubTreeMetrics<U> {
    fn acc(&mut self, other: Self) {
        self.height = self.height.max(other.height);
        self.size += other.size;
        self.hashs.acc(&other.hashs);
    }
}

pub struct BasicAccumulator {
    kind: Type,
    children: Vec<NodeIdentifier>,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
}

impl BasicAccumulator {
    pub(crate) fn new(kind: Type) -> Self {
        Self {
            kind,
            children: vec![],
            metrics: Default::default(),
        }
    }
}

impl Accumulator for BasicAccumulator {
    type Node = FullNode<Global, Local>;
    fn push(&mut self, full_node: Self::Node) {
        self.children.push(full_node.local.compressed_node);
        self.metrics.acc(full_node.local.metrics)
    }
}

pub struct AccumulatorWithIndentation {
    simple: BasicAccumulator,
    padding_start: usize,
    indentation: Spaces,
}

impl AccumulatorWithIndentation {
    pub(crate) fn new(kind: Type) -> Self {
        Self {
            simple: BasicAccumulator::new(kind),
            padding_start: 0,
            indentation: Space::format_indentation(&"\n".as_bytes().to_vec()),
        }
    }
}

impl Accumulator for AccumulatorWithIndentation {
    type Node = FullNode<Global, Local>;
    fn push(&mut self, full_node: Self::Node) {
        self.simple.push(full_node);
    }
}

impl AccIndentation for AccumulatorWithIndentation {
    fn indentation<'a>(&'a self) -> &'a Spaces {
        &self.indentation
    }
}

pub struct SimpleStores {
    pub label_store: LabelStore,
    pub type_store: TypeStore,
    pub node_store: NodeStore,
}

impl Default for SimpleStores {
    fn default() -> Self {
        Self {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        }
    }
}

impl<'a> TreeGen for JavaTreeGen {
    type Node1 = SimpleNode1<NodeIdentifier, String>;
    type Acc = AccumulatorWithIndentation;
    type Stores = SimpleStores;
    type Text = [u8];

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn pre(
        &mut self,
        text: &[u8],
        node: &tree_sitter::Node,
        stack: &Vec<Self::Acc>,
        sum_byte_length: usize,
    ) -> <Self as TreeGen>::Acc {
        let type_store = &mut self.stores().type_store;
        let parent_indentation = &stack.last().unwrap().indentation();
        let kind = type_store.get(node.kind());

        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            sum_byte_length,
            &parent_indentation,
        );
        AccumulatorWithIndentation {
            simple: BasicAccumulator {
                kind,
                children: vec![],
                metrics: Default::default(),
            },
            padding_start: sum_byte_length,
            indentation: indent,
        }
    }
    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        depth: usize,
        position: usize,
        text: &[u8],
        node: &tree_sitter::Node,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let node_store = &mut self.stores.node_store;
        let label_store = &mut self.stores.label_store;

        Self::handle_spacing(
            acc.padding_start,
            node.start_byte(),
            text,
            node_store,
            &(depth + 1),
            position,
            parent,
        );

        // let (node, metrics) = {
        let label = label_for_cursor(text, &node)
            .and_then(|x| Some(std::str::from_utf8(&x).unwrap().to_owned()));
        //     let acc = acc.simple;
        //     let node = SimpleNode1 {
        //         kind: acc.kind,
        //         label,
        //         children: acc.children,
        //     };
        let metrics = acc.simple.metrics;
        //     (node, metrics)
        // };
        // let (compressible_node, metrics) = {

        let hashed_kind = &clamp_u64_to_u32(&utils::hash(&acc.simple.kind));
        let hashed_label = &clamp_u64_to_u32(&utils::hash(&label));
        let hsyntax = hashed::inner_node_hash(
            hashed_kind,
            hashed_label,
            &acc.simple.metrics.size,
            &acc.simple.metrics.hashs.syntax,
        );
        let hashable = &hsyntax; //(hlabel as u64) << 32 & hsyntax as u64;

        let label = match label {
            Some(l) => Some(label_store.get_or_insert(l)),
            None => None,
        };

        let eq = |x: EntryRef| {
            let t = x.get_component::<Type>().ok();
            if &t != &Some(&acc.simple.kind) {
                // println!("typed: {:?} {:?}", acc.simple.kind, t);
                return false;
            }
            let l = x.get_component::<LabelIdentifier>().ok();
            if l != label.as_ref() {
                // println!("labeled: {:?} {:?}", acc.simple.kind, label);
                return false;
            } else {
                let cs = x.get_component::<CS<legion::Entity>>().ok();
                let r = match cs {
                    Some(CS(cs)) => cs == &acc.simple.children,
                    None => acc.simple.children.is_empty(),
                };
                if !r {
                    // println!("cs: {:?} {:?}", acc.simple.kind, acc.simple.children);
                    return false;
                }
            }
            true
        };
        let (tmp, rest) = node_store.prepare_insertion(&hashable, eq);

        let hashs = SyntaxNodeHashs {
            structt: hashed::inner_node_hash(
                hashed_kind,
                &0,
                &acc.simple.metrics.size,
                &acc.simple.metrics.hashs.structt,
            ),
            label: hashed::inner_node_hash(
                hashed_kind,
                hashed_label,
                &acc.simple.metrics.size,
                &acc.simple.metrics.hashs.label,
            ),
            syntax: hsyntax,
        };

        let compressed_node = match tmp {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
                occupied.into_key_value().0.clone()
            }
            hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {
                match label {
                    None => match acc.simple.children.len() {
                        0 => NodeStore::insert_after_prepare(
                            vacant,
                            rest,
                            (acc.simple.kind.clone(), hashs),
                        ),
                        // 1 => NodeStore::insert_after_prepare(
                        //     vacant,
                        //     rest,
                        //     (acc.simple.kind.clone(), CS0([acc.simple.children[0]])),
                        // ),
                        // 2 => NodeStore::insert_after_prepare(
                        //     vacant,
                        //     rest,
                        //     (
                        //         acc.simple.kind.clone(),
                        //         CS0([
                        //             acc.simple.children[0],
                        //             acc.simple.children[1],
                        //         ]),
                        //     ),
                        // ),
                        // 3 => NodeStore::insert_after_prepare(
                        //     vacant,
                        //     rest,
                        //     (
                        //         acc.simple.kind.clone(),
                        //         CS0([
                        //             acc.simple.children[0],
                        //             acc.simple.children[1],
                        //             acc.simple.children[2],
                        //         ]),
                        //     ),
                        // ),
                        _ => {
                            let a = acc.simple.children;
                            NodeStore::insert_after_prepare(
                                vacant,
                                rest,
                                (acc.simple.kind.clone(), hashs, CS(a)),
                            )
                        }
                    },
                    Some(label) => {
                        assert!(acc.simple.children.is_empty());
                        NodeStore::insert_after_prepare(
                        vacant,
                        rest,
                        (acc.simple.kind.clone(), hashs, label),
                    )},
                }
            }
        };

        let metrics = SubTreeMetrics {
            size: metrics.size + 1,
            height: metrics.height + 1,
            hashs,
        };

        let full_node = FullNode {
            global: Global { depth, position },
            local: Local {
                compressed_node,
                metrics,
            },
        };
        full_node
    }

    fn init_val(&mut self, text: &[u8], node: &tree_sitter::Node) -> Self::Acc {
        let type_store = &mut self.stores().type_store;
        let kind = type_store.get(node.kind());

        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            0,
            &Space::format_indentation(&self.line_break),
        );
        AccumulatorWithIndentation {
            simple: BasicAccumulator {
                kind,
                children: vec![],
                metrics: Default::default(),
            },
            padding_start: 0,
            indentation: indent,
        }
    }
}

impl JavaTreeGen {
    fn handle_spacing(
        padding_start: usize,
        pos: usize,
        text: &[u8],
        node_store: &mut NodeStore,
        depth: &usize,
        position: usize,
        parent: &mut <Self as TreeGen>::Acc,
    ) {
        let tmp = get_spacing(padding_start, pos, text, parent.indentation());
        if let Some(relativized) = tmp {
            let hsyntax = utils::clamp_u64_to_u32(&utils::hash(&relativized));
            let hashable = &hsyntax;

            let spaces = relativized.into_boxed_slice();

            let eq = |x: EntryRef| {
                let t = x.get_component::<Box<[Space]>>().ok();
                if t != Some(&spaces) {
                    return false;
                }
                true
            };

            let (tmp, rest) = node_store.prepare_insertion(&hashable, eq);

            let hashs = SyntaxNodeHashs {
                structt: 0,
                label: 0,
                syntax: hsyntax,
            };

            let compressed_node = match tmp {
                hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
                    occupied.into_key_value().0.clone()
                }
                hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {
                    NodeStore::insert_after_prepare(vacant, rest, (spaces, hashs))
                }
            };

            let full_spaces_node = FullNode {
                global: Global {
                    depth: *depth,
                    position,
                },
                local: Local {
                    compressed_node,
                    metrics: SubTreeMetrics {
                        size: 1,
                        height: 1,
                        hashs,
                    },
                },
            };
            parent.push(full_spaces_node);
        };
    }

    /// end of tree but not end of file,
    /// thus to be a bijection, we need to get the last spaces
    fn handle_final_space(
        depth: &usize,
        sum_byte_length: usize,
        text: &[u8],
        node_store: &mut NodeStore,
        position: usize,
        parent: &mut <Self as TreeGen>::Acc,
    ) {
        if has_final_space(depth, sum_byte_length, text) {
            Self::handle_spacing(
                sum_byte_length,
                text.len(),
                text,
                node_store,
                depth,
                position,
                parent,
            )
        }
    }

    pub fn new() -> Self {
        Self {
            line_break: "\n".as_bytes().to_vec(),
            stores: SimpleStores {
                label_store: LabelStore::new(),
                type_store: TypeStore {},
                node_store: NodeStore::new(),
            },
        }
    }

    pub fn generate_default(
        &mut self,
        text: &[u8],
        mut cursor: TreeCursor,
    ) -> FullNode<Global, Local> {
        // self.generate(text, cursor)
        let mut stack = vec![];
        stack.push(self.init_val(text, &cursor.node()));
        let sum_byte_length = self.gen(text, &mut stack, &mut cursor);

        let mut acc = stack.pop().unwrap();

        Self::handle_final_space(
            &0,
            sum_byte_length,
            text,
            &mut self.stores.node_store,
            acc.simple.metrics.size as usize + 1,
            &mut acc,
        );
        let mut r = AccumulatorWithIndentation::new(self.stores().type_store.get("file"));

        let full_node = self.post(
            &mut r,
            0,
            acc.simple.metrics.size as usize,
            text,
            &cursor.node(),
            acc,
        );
        full_node
    }

    fn compress_label(
        label_store: &mut LabelStore,
        n1: <Self as TreeGen>::Node1,
    ) -> CompressedNode<NodeIdentifier, DefaultSymbol> {
        let label_id = match n1.label {
            Some(l) => Some(label_store.get_or_insert(l)),
            None => None,
        };
        CompressedNode::new(n1.kind, label_id, n1.children)
    }

    pub fn main() {
        let mut parser = Parser::new();
        parser.set_language(unsafe { tree_sitter_java() }).unwrap();

        let text = {
            let source_code1 = "class A {void test() {}}";
            source_code1.as_bytes()
        };
        // let mut parser: Parser, old_tree: Option<&Tree>
        let tree = parser.parse(text, None).unwrap();
        let mut java_tree_gen = JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: SimpleStores {
                label_store: LabelStore::new(),
                type_store: TypeStore {},
                node_store: NodeStore::new(),
            },
        };
        let _full_node = java_tree_gen.generate_default(text, tree.walk());

        // print_tree_structure(
        //     &java_tree_gen.stores.node_store,
        //     &_full_node.local.compressed_node,
        // );

        let tree = parser.parse(text, Some(&tree)).unwrap();
        let _full_node = java_tree_gen.generate_default(text, tree.walk());
    }
}

pub fn print_tree_structure(node_store: &NodeStore, id: &NodeIdentifier) {
    nodes::print_tree_structure(
        |id| -> _ {
            node_store.resolve(id.clone()).into_compressed_node().unwrap()
        },
        id,
    )
}

pub fn print_tree_labels<>(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> _ {
            node_store.resolve(id.clone()).into_compressed_node().unwrap()
        },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
    )
}

pub fn print_tree_syntax(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> _ {
            node_store.resolve(id.clone()).into_compressed_node().unwrap()
        },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
    )
}

pub fn serialize<W: std::fmt::Write>(
    node_store: &NodeStore,
    label_store: &LabelStore,
    id: &NodeIdentifier,
    out: &mut W,
    parent_indent: &str,
) -> Option<String> {
    nodes::serialize(
        |id| -> _ {
            node_store.resolve(id.clone()).into_compressed_node().unwrap()
        },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
        out,
        parent_indent,
    )
}

impl NodeStore {
    pub(crate) fn new() -> Self {
        Self {
            count: 0,
            errors: 0,
            roots: Default::default(),
            internal: Default::default(),
            dedup: hashbrown::HashMap::<_,(),()>::with_capacity_and_hasher(1<<10,Default::default()),
            hasher: Default::default(),
        }
    }
}

impl LabelStore {
    pub(crate) fn new() -> Self {
        Self {
            count: 0,
            internal: Default::default(),
        }
    }
}
