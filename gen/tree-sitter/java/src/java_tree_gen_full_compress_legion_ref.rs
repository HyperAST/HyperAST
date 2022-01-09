///! fully compress all subtrees
use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
    vec,
};

use legion::{
    storage::{Archetype, Component, IntoComponentSource},
    world::{ComponentError, EntityLocation, EntryRef},
    EntityStore,
};
use rusted_gumtree_core::tree::tree::{
    LabelStore as LabelStoreTrait, NodeStore as NodeStoreTrait, Type, Typed, VersionedNodeStore,
};
use rusted_gumtree_core::tree::tree::{NodeStoreMut as NodeStoreMutTrait, Tree};
use string_interner::{DefaultHashBuilder, DefaultSymbol, StringInterner, Symbol as _};
use tree_sitter::{Language, Parser, TreeCursor};
use tuples::CombinConcat;

use crate::{
    filter::BF,
    filter::{Bloom, BloomResult, BloomSize},
    full::FullNode,
    hashed::{self, NodeHashs, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    impact::elements::*,
    nodes::{self, CompressedNode, HashSize, RefContainer, SimpleNode1, Space},
    store::{vec_map_store::Symbol, TypeStore},
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, label_for_cursor, AccIndentation,
        Accumulator, BasicAccumulator, Spaces, TreeGen,
    },
    utils::{self, clamp_u64_to_u32, make_hash},
};

type NodeIdentifier = legion::Entity;

pub struct HashedNodeRef<'a>(EntryRef<'a>);

pub struct HashedNode {
    node: CompressedNode<legion::Entity, LabelIdentifier>,
    hashs: SyntaxNodeHashs<u32>,
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

impl<'a> RefContainer for HashedNodeRef<'a> {
    type Ref = [u8];
    type Result = crate::filter::BloomResult;

    fn check<U: Borrow<Self::Ref> + AsRef<[u8]>>(&self, rf: U) -> Self::Result {
        macro_rules! check {
            ( ($e:expr, $s:expr, $rf:expr); $($t:ty),* ) => {
                match $e {
                    BloomSize::None => crate::filter::BloomResult::MaybeContain,
                    $( <$t>::Size => $s.get_component::<$t>()
                        .unwrap()
                        .check(0, $rf)),*
                }
            };
        }
        let e = self.0.get_component::<BloomSize>().unwrap();
        check![
            (*e, self.0, rf);
            Bloom<&'static [u8], u16>,
            Bloom<&'static [u8], u32>,
            Bloom<&'static [u8], u64>,
            Bloom<&'static [u8], [u64; 2]>,
            Bloom<&'static [u8], [u64; 4]>
        ]
    }
}

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

impl<'a> HashedNodeRef<'a> {}

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
impl Display for LabelStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, x) in self.internal.clone().into_iter() {
            writeln!(f, "{:?}:{:?}", i.to_usize(), x)?
        }
        Ok(())
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

pub mod compo {
    pub struct Size(pub u32);
    pub struct Height(pub u32);

    pub struct HStruct(pub u32);
    pub struct HLabel(pub u32);
}

#[derive(PartialEq, Eq)]
struct CS0<T: Eq, const N: usize>([T; N]);
struct CSE<const N: usize>([legion::Entity; N]);
#[derive(PartialEq, Eq)]
struct CS<T: Eq>(Vec<T>);

pub struct NodeStore {
    count: usize,
    errors: usize,
    roots: HashMap<(u8, u8, u8), NodeIdentifier>,
    dedup: hashbrown::HashMap<NodeIdentifier, (), ()>,
    internal: legion::World,
    hasher: DefaultHashBuilder, //fasthash::city::Hash64,//fasthash::RandomState<fasthash::>,
                                // internal: VecMapStore<HashedNode, NodeIdentifier, legion::World>,
}

pub struct PendingInsert<'a>(u64, &'a mut legion::World, &'a DefaultHashBuilder);

impl NodeStore {
    pub fn prepare_insertion<'a, Eq: Fn(EntryRef) -> bool, V: Hash>(
        &'a mut self,
        hashable: &'a V,
        eq: Eq,
    ) -> (
        crate::compat::hash_map::RawEntryMut<'a, legion::Entity, (), ()>,
        PendingInsert,
    ) {
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
        (entry, PendingInsert(hash, &mut self.internal, &self.hasher))
    }

    pub fn insert_after_prepare<T>(
        (vacant, PendingInsert(hash, internal, hasher)): (
            crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
            PendingInsert,
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

impl<'a> NodeStoreMutTrait<'a, HashedNode, HashedNodeRef<'a>> for NodeStore {}

impl NodeStore {
    pub fn len(&self) -> usize {
        self.internal.len()
    }
}

impl<'a> VersionedNodeStore<'a, NodeIdentifier, HashedNodeRef<'a>> for NodeStore {
    fn resolve_root(&self, version: (u8, u8, u8), node: NodeIdentifier) {
        todo!()
    }
}

#[derive(Debug)]
pub struct Global {
    pub(crate) depth: usize,
    pub(crate) position: usize,
}

#[derive(Debug)]
pub struct Local {
    pub(crate) compressed_node: NodeIdentifier,
    pub(crate) metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub(crate) ana: Option<PartialAnalysis>,
}

impl Local {
    fn acc(self, acc: &mut Acc) {
        acc.simple.push(self.compressed_node);
        acc.metrics.acc(self.metrics);

        if let Some(s) = self.ana {
            // TODO use to simplify when stabilized
            // s.acc(&acc.simple.kind,acc.ana.get_or_insert_default());
            if let Some(aaa) = &mut acc.ana {
                s.acc(&acc.simple.kind, aaa);
            } else {
                let mut aaa = Default::default();
                s.acc(&acc.simple.kind, &mut aaa);
                acc.ana = Some(aaa);
            }
        }
    }
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

pub struct Acc {
    simple: BasicAccumulator<Type, NodeIdentifier>,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: Option<PartialAnalysis>,
    padding_start: usize,
    indentation: Spaces,
}

impl Acc {
    pub(crate) fn new(kind: Type) -> Self {
        Self {
            simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: Default::default(),
            padding_start: 0,
            indentation: Space::format_indentation(&"\n".as_bytes().to_vec()),
        }
    }
}

impl Accumulator for Acc {
    type Node = FullNode<Global, Local>;
    fn push(&mut self, full_node: Self::Node) {
        full_node.local.acc(self);
    }
}

impl AccIndentation for Acc {
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
    type Acc = Acc;
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
        let kind = node.kind();
        let kind = type_store.get(kind);
        // let kind = handle_wildcard_kind(kind, node);

        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            sum_byte_length,
            &parent_indentation,
        );
        let ana = self.build_ana(&kind);
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            metrics: Default::default(),
            ana,
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
        let label = label_for_cursor(text, &node)
            .and_then(|x| Some(std::str::from_utf8(&x).unwrap().to_owned()));
        let metrics = acc.metrics;
        let hashed_kind = &clamp_u64_to_u32(&utils::hash(&acc.simple.kind));
        let hashed_label = &clamp_u64_to_u32(&utils::hash(&label));
        let hsyntax = hashed::inner_node_hash(
            hashed_kind,
            hashed_label,
            &acc.metrics.size,
            &acc.metrics.hashs.syntax,
        );
        let hashable = &hsyntax; //(hlabel as u64) << 32 & hsyntax as u64;

        let (ana, label) = if let Some(label) = label.as_ref() {
            assert!(acc.ana.is_none());
            if &acc.simple.kind == &Type::Comment {
                (None, Some(label_store.get_or_insert(label.as_str())))
            } else if acc.simple.kind.is_literal() {
                let tl = acc.simple.kind.literal_type();
                let tl = label_store.get_or_insert(tl);
                (
                    Some(PartialAnalysis::init(&acc.simple.kind, Some(tl), |x| {
                        label_store.get_or_insert(x)
                    })),
                    Some(label_store.get_or_insert(label.as_str())),
                )
            } else {
                let rf = label_store.get_or_insert(label.as_str());
                (
                    Some(PartialAnalysis::init(&acc.simple.kind, Some(rf), |x| {
                        label_store.get_or_insert(x)
                    })),
                    Some(rf),
                )
            }
        } else if acc.simple.kind.is_primitive() {
            let node = node_store.resolve(acc.simple.children[0]);
            let label = node.get_type().to_string();
            if let Some(ana) = acc.ana {
                todo!("{:?} {:?}", acc.simple.kind, ana)
            }
            let rf = label_store.get_or_insert(label.as_str());
            (
                Some(PartialAnalysis::init(&acc.simple.kind, Some(rf), |x| {
                    label_store.get_or_insert(x)
                })),
                None,
            )
        } else if let Some(ana) = acc.ana {
            // nothing to do, resolutions at the end of post ?
            (Some(ana), None)
        } else if acc.simple.kind == Type::TS86
            || acc.simple.kind == Type::TS81
            || acc.simple.kind == Type::Dimensions
        {
            (
                Some(PartialAnalysis::init(&acc.simple.kind, None, |x| {
                    label_store.get_or_insert(x)
                })),
                None,
            )
        } else if acc.simple.kind == Type::ArgumentList || acc.simple.kind == Type::FormalParameters
        {
            assert!(acc
                .simple
                .children
                .iter()
                .all(|x| { !node_store.resolve(*x).has_children() }));
            // TODO decls
            (
                Some(PartialAnalysis::init(&acc.simple.kind, None, |x| {
                    label_store.get_or_insert(x)
                })),
                None,
            )
        } else if acc.simple.kind == Type::SwitchLabel || acc.simple.kind == Type::Modifiers {
            // TODO decls
            (None, None)
        } else if acc.simple.kind == Type::BreakStatement
            || acc.simple.kind == Type::ContinueStatement
            || acc.simple.kind == Type::Wildcard
            || acc.simple.kind == Type::Block
            || acc.simple.kind == Type::ConstructorBody
            || acc.simple.kind == Type::InterfaceBody
            || acc.simple.kind == Type::SwitchBlock
            || acc.simple.kind == Type::ClassBody
            || acc.simple.kind == Type::AnnotationTypeBody
            || acc.simple.kind == Type::TypeArguments
            || acc.simple.kind == Type::ArrayInitializer
            || acc.simple.kind == Type::Asterisk
            || acc.simple.kind == Type::ReturnStatement
        // case of "return;"
        {
            // TODO maybe do something later?
            (None, None)
        } else {
            println!("{:?}", &acc.simple.kind);
            assert!(
                acc.simple.children.is_empty()
                    || !acc
                        .simple
                        .children
                        .iter()
                        .all(|x| { !node_store.resolve(*x).has_children() })
            );
            (None, None)
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
                &acc.metrics.size,
                &acc.metrics.hashs.structt,
            ),
            label: hashed::inner_node_hash(
                hashed_kind,
                hashed_label,
                &acc.metrics.size,
                &acc.metrics.hashs.label,
            ),
            syntax: hsyntax,
        };

        // TODO resolution now?
        let ana = match ana {
            Some(ana) if &acc.simple.kind == &Type::ClassBody => {
                println!("refs in class body");
                ana.print_refs(&self.stores.label_store);
                Some(ana.resolve())
            }
            Some(ana) if &acc.simple.kind == &Type::ClassDeclaration => {
                println!("refs in class decl");
                ana.print_refs(&self.stores.label_store);
                Some(ana.resolve())
            }
            Some(ana) if &acc.simple.kind == &Type::MethodDeclaration => {
                println!("refs in method decl:");
                ana.print_refs(&self.stores.label_store);
                let ana = ana.resolve();
                println!("refs in method decl after resolution:");
                ana.print_refs(&self.stores.label_store);
                Some(ana)
            }
            Some(ana) if &acc.simple.kind == &Type::Program => {
                println!("refs in program");
                ana.print_refs(&self.stores.label_store);
                Some(ana.resolve())
            }
            Some(ana) => Some(ana), // TODO
            None => None,
        };

        let compressed_node = match tmp {
            hashbrown::hash_map::RawEntryMut::Occupied(occupied) => {
                occupied.into_key_value().0.clone()
            }
            hashbrown::hash_map::RawEntryMut::Vacant(vacant) => {
                let vacant = (vacant, rest);
                match label {
                    None => {
                        macro_rules! insert {
                            ( $c:expr, $t:ty ) => {
                                NodeStore::insert_after_prepare(
                                    vacant,
                                    $c.concat((
                                        <$t>::Size,
                                        <$t>::from(ana.as_ref().unwrap().refs()),
                                    )),
                                )
                            };
                        }
                        match acc.simple.children.len() {
                            0 => {
                                assert_eq!(0, metrics.size);
                                assert_eq!(0, metrics.height);
                                NodeStore::insert_after_prepare(
                                    vacant,
                                    (acc.simple.kind.clone(), hashs, BloomSize::None),
                                )
                            }
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
                                let c = (
                                    acc.simple.kind.clone(),
                                    compo::Size(metrics.size + 1),
                                    compo::Height(metrics.height + 1),
                                    hashs,
                                    CS(a),
                                );
                                match ana.as_ref().map(|x| x.refs_count()).unwrap_or(0) {
                                    // x if x > 256 => (BloomSize::BX,Bloom::from(&refs)),
                                    x if x > 150 => {
                                        insert!(c, Bloom::<&'static [u8], [u64; 4]>)
                                    }
                                    x if x > 50 => {
                                        insert!(c, Bloom::<&'static [u8], [u64; 2]>)
                                    }
                                    x if x > 25 => {
                                        insert!(c, Bloom::<&'static [u8], u64>)
                                    }
                                    x if x > 12 => {
                                        insert!(c, Bloom::<&'static [u8], u32>)
                                    }
                                    x if x > 4 => {
                                        insert!(c, Bloom::<&'static [u8], u16>)
                                    }
                                    _ => NodeStore::insert_after_prepare(
                                        vacant,
                                        c.concat((BloomSize::None,)),
                                    ),
                                }
                            }
                        }
                    }
                    Some(label) => {
                        assert!(acc.simple.children.is_empty());
                        NodeStore::insert_after_prepare(
                            vacant,
                            (acc.simple.kind.clone(), hashs, label),
                        )
                    }
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
                ana,
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
        let ana = self.build_ana(&kind);
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            metrics: Default::default(),
            ana,
            padding_start: 0,
            indentation: indent,
        }
    }
}

// /// make new types to handle wildcard precisely
// fn handle_wildcard_kind(kind: Type, node: &tree_sitter::Node) -> Type {
//     if kind == Type::Wildcard {
//         if node.child_by_field_name(b"extends").is_some() {
//             Type::WildcardExtends
//         } else if node.child_by_field_name(b"super").is_some() {
//             Type::WildcardSuper
//         } else {
//             kind
//         }
//     } else {
//         kind
//     }
// }

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
                    NodeStore::insert_after_prepare((vacant, rest), (spaces, hashs))
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
                    ana: Default::default(),
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
        let mut stack = vec![];
        stack.push(self.init_val(text, &cursor.node()));
        let sum_byte_length = self.gen(text, &mut stack, &mut cursor);

        let mut acc = stack.pop().unwrap();

        Self::handle_final_space(
            &0,
            sum_byte_length,
            text,
            &mut self.stores.node_store,
            acc.metrics.size as usize + 1,
            &mut acc,
        );
        let mut r = Acc::new(self.stores().type_store.get("file"));

        let full_node = self.post(
            &mut r,
            0,
            acc.metrics.size as usize,
            text,
            &cursor.node(),
            acc,
        );

        {
            let a = &full_node.local.compressed_node;

            let b = self.stores.node_store.resolve(*a);
            // println!(
            //     "rset: {:#?}",
            //     full_node
            //         .local
            //         .ana
            //         .as_ref()
            //         .map(|x| x
            //             .refs()
            //             .map(|x| std::str::from_utf8(&x).unwrap().to_owned())
            //             .collect::<Vec<_>>())
            //         .unwrap_or_default()
            // );

            match full_node.local.ana.as_ref() {
                Some(x) => {
                    println!("refs:",);
                    x.print_refs(&self.stores.label_store);
                }
                None => println!("None"),
            };

            let dd = full_node.local.ana.as_ref();
            if let Some(d) = dd.and_then(|dd| dd.refs().next()) {
                let c = b.check(d.deref().deref());

                let s = std::str::from_utf8(&d).unwrap();

                match c {
                    BloomResult::MaybeContain => println!("Maybe contains {}", s),
                    BloomResult::DoNotContain => println!("Do not contains {}", s),
                }
                let f = |x| {
                    let c = b.check(s.as_bytes());
                    match c {
                        BloomResult::MaybeContain => println!("Maybe contains {}", s),
                        BloomResult::DoNotContain => println!("Do not contains {}", s),
                    }
                };
                f("X");
                f("XX");
                f("Z");
                f("f()");
            }
        }

        full_node
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

    fn build_ana(&mut self, kind: &Type) -> Option<PartialAnalysis> {
        let label_store = &mut self.stores.label_store;
        if kind == &Type::Program || kind == &Type::ClassBody {
            Some(PartialAnalysis::init(kind, None, |x| {
                label_store.get_or_insert(x)
            }))
        } else {
            None
        }
    }
}

pub fn print_tree_structure(node_store: &NodeStore, id: &NodeIdentifier) {
    nodes::print_tree_structure(
        |id| -> _ {
            node_store
                .resolve(id.clone())
                .into_compressed_node()
                .unwrap()
        },
        id,
    )
}

pub fn print_tree_labels(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> _ {
            node_store
                .resolve(id.clone())
                .into_compressed_node()
                .unwrap()
        },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
    )
}

pub fn print_tree_syntax(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
    nodes::print_tree_labels(
        |id| -> _ {
            node_store
                .resolve(id.clone())
                .into_compressed_node()
                .unwrap()
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
            node_store
                .resolve(id.clone())
                .into_compressed_node()
                .unwrap()
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
            dedup: hashbrown::HashMap::<_, (), ()>::with_capacity_and_hasher(
                1 << 10,
                Default::default(),
            ),
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
