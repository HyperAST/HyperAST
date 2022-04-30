///! fully compress all subtrees
use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    io::stdout,
    ops::Deref,
    vec,
};

use hyper_ast::{nodes::IoOut, store::labels::LabelStore};
use legion::{
    storage::{Archetype, Component},
    world::{ComponentError, EntityLocation, EntryRef},
};
use num::ToPrimitive;
use string_interner::{DefaultHashBuilder, DefaultSymbol};
use tree_sitter::{Language, Parser, TreeCursor};
use tuples::CombinConcat;

use hyper_ast::{
    filter::BF,
    filter::{Bloom, BloomResult, BloomSize},
    hashed::{self, NodeHashs, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    nodes::{self, CompressedNode, HashSize, RefContainer, SimpleNode1, Space},
    store::{
        nodes::legion::{compo, CS},
        nodes::DefaultNodeStore as NodeStore,
        SimpleStores, TypeStore,
    },
    tree_gen::parser::Node as _,
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, AccIndentation, Accumulator,
        BasicAccumulator, Spaces, TreeGen,
    },
    types::{
        LabelStore as LabelStoreTrait,
        Tree,
        // NodeStore as NodeStoreTrait,
        Type,
        Typed,
    },
};

use crate::{
    full::FullNode,
    impact::{
        element::{ExplorableRef, RefsEnum},
        elements::*,
        partial_analysis::PartialAnalysis,
    },
    store::vec_map_store::Symbol,
    utils::{self, clamp_u64_to_u32},
};

pub use crate::impact::element::BulkHasher;

pub fn hash32<T: ?Sized + Hash>(t: &T) -> u32 {
    utils::clamp_u64_to_u32(&utils::hash(t))
}

pub type EntryR<'a> = EntryRef<'a>;

pub type NodeIdentifier = legion::Entity;

pub struct HashedNodeRef<'a>(EntryRef<'a>);

pub type FNode = FullNode<Global, Local>;

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

impl hyper_ast::types::Node for HashedNode {}
impl hyper_ast::types::Stored for HashedNode {
    type TreeId = NodeIdentifier;
}

impl Symbol<HashedNode> for legion::Entity {}
impl<'a> Symbol<HashedNodeRef<'a>> for legion::Entity {}

// impl<'a> RefContainer for HashedNodeRef<'a> {
//     type Result = BloomResult;

//     fn check<U: Borrow<Self::Ref> + AsRef<[u8]>>(&self, rf: U) -> Self::Result {
//         macro_rules! check {
//             ( ($e:expr, $s:expr, $rf:expr); $($t:ty),* ) => {
//                 match $e {
//                     BloomSize::Much => {
//                         log::warn!("[Too Much]");
//                         BloomResult::MaybeContain
//                     },
//                     BloomSize::None => BloomResult::DoNotContain,
//                     $( <$t>::SIZE => $s.get_component::<$t>()
//                         .unwrap()
//                         .check(0, $rf)),*
//                 }
//             };
//         }
//         let e = self.0.get_component::<BloomSize>().unwrap();
//         check![
//             (*e, self.0, rf);
//             Bloom<&'static [u8], u16>,
//             Bloom<&'static [u8], u32>,
//             Bloom<&'static [u8], u64>,
//             Bloom<&'static [u8], [u64; 2]>,
//             Bloom<&'static [u8], [u64; 4]>,
//             Bloom<&'static [u8], [u64; 8]>,
//             Bloom<&'static [u8], [u64; 16]>,
//             Bloom<&'static [u8], [u64; 32]>,
//             Bloom<&'static [u8], [u64; 64]>
//         ]
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
        hyper_ast::types::WithHashs::hash(self, &Default::default()).hash(state)
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

impl<'a> hyper_ast::types::Typed for HashedNodeRef<'a> {
    type Type = Type;

    fn get_type(&self) -> Type {
        *self.0.get_component::<Type>().unwrap()
    }
}

impl<'a> hyper_ast::types::Labeled for HashedNodeRef<'a> {
    type Label = LabelIdentifier;

    fn get_label(&self) -> &LabelIdentifier {
        self.0
            .get_component::<LabelIdentifier>()
            .expect("check with self.has_label()")
    }
}

impl<'a> hyper_ast::types::Node for HashedNodeRef<'a> {}

impl<'a> hyper_ast::types::Stored for HashedNodeRef<'a> {
    type TreeId = NodeIdentifier;
}
impl<'a> hyper_ast::types::WithChildren for HashedNodeRef<'a> {
    type ChildIdx = u16;

    fn child_count(&self) -> u16 {
        self.0
            .get_component::<CS<legion::Entity>>()
            .unwrap()
            .0
            .len()
            .to_u16()
            .expect("too much children")
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        self.0.get_component::<CS<legion::Entity>>().unwrap().0
            [num::cast::<_, usize>(*idx).unwrap()]
    }

    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        let v = &self.0.get_component::<CS<legion::Entity>>().unwrap().0;
        v[v.len() - 1 - num::cast::<_, usize>(*idx).unwrap()]
    }

    fn get_children<'b>(&'b self) -> &'b [Self::TreeId] {
        self.0
            .get_component::<CS<legion::Entity>>()
            .unwrap()
            .0
            .as_slice()
    }
}

impl<'a> hyper_ast::types::WithHashs for HashedNodeRef<'a> {
    type HK = SyntaxNodeHashsKinds;
    type HP = HashSize;

    fn hash(&self, kind: &Self::HK) -> Self::HP {
        self.0
            .get_component::<SyntaxNodeHashs<Self::HP>>()
            .unwrap()
            .hash(kind)
    }
}

impl<'a> hyper_ast::types::Tree for HashedNodeRef<'a> {
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
pub type LabelIdentifier = DefaultSymbol;

pub struct JavaTreeGen<'a> {
    pub line_break: Vec<u8>,
    pub stores: &'a mut SimpleStores,
    pub md_cache: &'a mut MDCache,
}

pub type MDCache = HashMap<NodeIdentifier, MD>;

pub struct MD {
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: Option<PartialAnalysis>,
}

// type SpacesStoreD = SpacesStore<u16, 4>;

// pub struct LabelStore {
//     count: usize,
//     internal: StringInterner, //VecMapStore<OwnedLabel, LabelIdentifier>,
// }

// impl Debug for LabelStore {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("LabelStore")
//             .field("count", &self.count)
//             .field("internal_len", &self.internal.len())
//             .field("internal", &self.internal)
//             .finish()
//     }
// }

// impl Display for LabelStore {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for (i, x) in self.internal.clone().into_iter() {
//             writeln!(f, "{:?}:{:?}", i.to_usize(), x)?
//         }
//         Ok(())
//     }
// }

// impl LabelStoreTrait<MyLabel> for LabelStore {
//     type I = LabelIdentifier;
//     fn get_or_insert<T: Borrow<MyLabel>>(&mut self, node: T) -> Self::I {
//         self.count += 1;
//         self.internal.get_or_intern(node.borrow())
//     }

//     fn resolve(&self, id: &Self::I) -> &MyLabel {
//         self.internal.resolve(*id).unwrap()
//     }
// }

// #[derive(PartialEq, Eq)]
// struct CS0<T: Eq, const N: usize>([T; N]);
// struct CSE<const N: usize>([legion::Entity; N]);
// #[derive(PartialEq, Eq,Debug)]
// pub struct CS<T: Eq>(pub Vec<T>);

// pub struct NodeStore {
//     count: usize,
//     errors: usize,
//     // roots: HashMap<(u8, u8, u8), NodeIdentifier>,
//     dedup: hashbrown::HashMap<NodeIdentifier, (), ()>,
//     internal: legion::World,
//     hasher: DefaultHashBuilder, //fasthash::city::Hash64,//fasthash::RandomState<fasthash::>,
//                                 // internal: VecMapStore<HashedNode, NodeIdentifier, legion::World>,
// }

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

// impl NodeStore {
//     pub fn prepare_insertion<'a, Eq: Fn(EntryRef) -> bool, V: Hash>(
//         &'a mut self,
//         hashable: &'a V,
//         eq: Eq,
//     ) -> PendingInsert {
//         let Self {
//             dedup,
//             internal: backend,
//             ..
//         } = self;
//         let hash = make_hash(&self.hasher, hashable);
//         let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
//             let r = eq(backend.entry_ref(*symbol).unwrap());
//             r
//         });
//         PendingInsert(entry, (hash, &mut self.internal, &self.hasher))
//     }

//     pub fn insert_after_prepare<T>(
//         (vacant, (hash, internal, hasher)): (
//             crate::compat::hash_map::RawVacantEntryMut<legion::Entity, (), ()>,
//             (u64, &mut legion::World, &DefaultHashBuilder),
//         ),
//         components: T,
//     ) -> legion::Entity
//     where
//         Option<T>: IntoComponentSource,
//     {
//         let (&mut symbol, &mut ()) = {
//             let symbol = internal.push(components);
//             vacant.insert_with_hasher(hash, symbol, (), |id| {
//                 let node = internal.entry_ref(*id).map(|x| HashedNodeRef(x)).unwrap();
//                 make_hash(hasher, &node)
//             })
//         };
//         symbol
//     }

//     pub fn resolve(&self, id: NodeIdentifier) -> HashedNodeRef {
//         self.internal
//             .entry_ref(id)
//             .map(|x| HashedNodeRef(x))
//             .unwrap()
//     }
// }

// impl Debug for NodeStore {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("NodeStore")
//             .field("count", &self.count)
//             .field("errors", &self.errors)
//             .field("internal_len", &self.internal.len())
//             // .field("internal", &self.internal)
//             .finish()
//     }
// }

// impl<'a> NodeStoreTrait<'a, NodeIdentifier, HashedNodeRef<'a>> for NodeStore {
//     fn resolve(&'a self, id: &NodeIdentifier) -> HashedNodeRef<'a> {
//         self.internal
//             .entry_ref(id.clone())
//             .map(|x| HashedNodeRef(x))
//             .unwrap()
//     }
// }

// impl<'a> NodeStoreMutTrait<'a, HashedNode, HashedNodeRef<'a>> for NodeStore {}

// impl NodeStore {
//     pub fn len(&self) -> usize {
//         self.internal.len()
//     }
// }

// impl<'a> VersionedNodeStore<'a, NodeIdentifier, HashedNodeRef<'a>> for NodeStore {
//     fn resolve_root(&self, version: (u8, u8, u8), node: NodeIdentifier) {
//         todo!()
//     }
// }

#[derive(Debug)]
pub struct Global {
    pub(crate) depth: usize,
    pub(crate) position: usize,
}

#[derive(Debug, Clone)]
pub struct Local {
    pub compressed_node: NodeIdentifier,
    // * metadata: computation results from concrete code of node and its children
    // they can be qualitative metadata .eg a hash or they can be quantitative .eg lines of code
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub ana: Option<PartialAnalysis>,
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
    pub hashs: U,
    pub size: u32,
    pub height: u32,
}

impl<U: NodeHashs> SubTreeMetrics<U> {
    pub fn acc(&mut self, other: Self) {
        self.height = self.height.max(other.height);
        self.size += other.size;
        self.hashs.acc(&other.hashs);
    }
}

pub struct Acc {
    simple: BasicAccumulator<Type, NodeIdentifier>,
    label: Option<String>,
    start_byte: usize,
    end_byte: usize,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: Option<PartialAnalysis>,
    padding_start: usize,
    indentation: Spaces,
}

// impl Acc {
//     pub(crate) fn new(kind: Type) -> Self {
//         Self {
//             simple: BasicAccumulator::new(kind),
//             label:None,
//             start_byte: 0,
//             metrics: Default::default(),
//             ana: Default::default(),
//             padding_start: 0,
//             indentation: Space::format_indentation(&"\n".as_bytes().to_vec()),
//         }
//     }
// }

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

// pub struct SimpleStores {
//     pub label_store: LabelStore,
//     pub type_store: TypeStore,
//     pub node_store: NodeStore,
// }

// impl Default for SimpleStores {
//     fn default() -> Self {
//         Self {
//             label_store: LabelStore::new(),
//             type_store: TypeStore {},
//             node_store: NodeStore::new(),
//         }
//     }
// }

#[repr(transparent)]
pub struct TNode<'a>(tree_sitter::Node<'a>);

impl<'a> hyper_ast::tree_gen::parser::Node<'a> for TNode<'a> {
    fn kind(&self) -> &str {
        self.0.kind()
    }

    fn start_byte(&self) -> usize {
        self.0.start_byte()
    }

    fn end_byte(&self) -> usize {
        self.0.end_byte()
    }

    fn child_count(&self) -> usize {
        self.0.child_count()
    }

    fn child(&self, i: usize) -> Option<Self> {
        self.0.child(i).map(|x| TNode(x))
    }

    fn is_named(&self) -> bool {
        self.0.is_named()
    }
}
#[repr(transparent)]
pub struct TTreeCursor<'a>(tree_sitter::TreeCursor<'a>);

impl<'a> hyper_ast::tree_gen::parser::TreeCursor<'a, TNode<'a>> for TTreeCursor<'a> {
    fn node(&self) -> TNode<'a> {
        TNode(self.0.node())
    }

    fn goto_first_child(&mut self) -> bool {
        self.0.goto_first_child()
    }

    fn goto_parent(&mut self) -> bool {
        self.0.goto_parent()
    }

    fn goto_next_sibling(&mut self) -> bool {
        self.0.goto_next_sibling()
    }
}

impl<'a> TreeGen for JavaTreeGen<'a> {
    type Node1 = SimpleNode1<NodeIdentifier, String>;
    type Acc = Acc;
    type Stores = SimpleStores;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b>;

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn pre(
        &mut self,
        text: &[u8],
        node: &Self::Node<'_>,
        stack: &Vec<Self::Acc>,
        sum_byte_length: usize,
    ) -> <Self as TreeGen>::Acc {
        let type_store = &mut self.stores().type_store;
        let parent_indentation = &stack.last().unwrap().indentation();
        let kind = node.kind();
        let kind = type_store.get(kind);

        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            sum_byte_length,
            &parent_indentation,
        );

        let label = node
            .extract_label(text)
            .and_then(|x| Some(std::str::from_utf8(&x).unwrap().to_owned()));

        let ana = self.build_ana(&kind);
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            label,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
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
        // node: &Self::Node<'_>,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let node_store = &mut self.stores.node_store;
        // let label_store = &mut self.stores.label_store;

        Self::handle_spacing(
            acc.padding_start,
            acc.start_byte,
            text,
            node_store,
            &(depth + 1),
            position,
            parent,
        );
        self.make(depth, position, acc)
        // let metrics = acc.metrics;
        // let hashed_kind = &clamp_u64_to_u32(&utils::hash(&acc.simple.kind));
        // let hashed_label = &clamp_u64_to_u32(&utils::hash(&label));
        // let hsyntax = hashed::inner_node_hash(
        //     hashed_kind,
        //     hashed_label,
        //     &acc.metrics.size,
        //     &acc.metrics.hashs.syntax,
        // );
        // let hashable = &hsyntax; //(hlabel as u64) << 32 & hsyntax as u64;

        // let label_id = if let Some(label) = label.as_ref() {
        //     if &acc.simple.kind == &Type::Comment {
        //         // None // TODO check
        //         Some(label_store.get_or_insert(label.as_str()))
        //     } else if acc.simple.kind.is_literal() {
        //         let tl = acc.simple.kind.literal_type();
        //         // let tl = label_store.get_or_insert(tl);

        //         Some(label_store.get_or_insert(label.as_str()))
        //     } else {
        //         let rf = label_store.get_or_insert(label.as_str());
        //         Some(rf)
        //     }
        // } else if acc.simple.kind.is_primitive() {
        //     None
        // } else if let Some(_) = acc.ana {
        //     None
        // } else if acc.simple.kind == Type::TS86
        //     || acc.simple.kind == Type::TS81
        //     || acc.simple.kind == Type::Asterisk
        //     || acc.simple.kind == Type::Dimensions
        //     || acc.simple.kind == Type::Block
        //     || acc.simple.kind == Type::ElementValueArrayInitializer
        // {
        //     None
        // } else if acc.simple.kind == Type::ArgumentList
        //     || acc.simple.kind == Type::FormalParameters
        //     || acc.simple.kind == Type::AnnotationArgumentList
        // {
        //     None
        // } else if acc.simple.kind == Type::SwitchLabel || acc.simple.kind == Type::Modifiers {
        //     None
        // } else if acc.simple.kind == Type::BreakStatement
        //     || acc.simple.kind == Type::ContinueStatement
        //     || acc.simple.kind == Type::Wildcard
        //     || acc.simple.kind == Type::ConstructorBody
        //     || acc.simple.kind == Type::InterfaceBody
        //     || acc.simple.kind == Type::SwitchBlock
        //     || acc.simple.kind == Type::ClassBody
        //     || acc.simple.kind == Type::EnumBody
        //     || acc.simple.kind == Type::AnnotationTypeBody
        //     || acc.simple.kind == Type::TypeArguments
        //     || acc.simple.kind == Type::ArrayInitializer
        //     || acc.simple.kind == Type::ReturnStatement
        //     || acc.simple.kind == Type::Error
        // {
        //     None
        // } else {
        //     None
        // };

        // let eq = |x: EntryRef| {
        //     let t = x.get_component::<Type>().ok();
        //     if &t != &Some(&acc.simple.kind) {
        //         // println!("typed: {:?} {:?}", acc.simple.kind, t);
        //         return false;
        //     }
        //     let l = x.get_component::<LabelIdentifier>().ok();
        //     if l != label_id.as_ref() {
        //         // println!("labeled: {:?} {:?}", acc.simple.kind, label);
        //         return false;
        //     } else {
        //         let cs = x.get_component::<CS<legion::Entity>>().ok();
        //         let r = match cs {
        //             Some(CS(cs)) => cs == &acc.simple.children,
        //             None => acc.simple.children.is_empty(),
        //         };
        //         if !r {
        //             // println!("cs: {:?} {:?}", acc.simple.kind, acc.simple.children);
        //             return false;
        //         }
        //     }
        //     true
        // };
        // let insertion = node_store.prepare_insertion(&hashable, eq);

        // if let Some(id) = insertion.occupied_id() {
        //     let md = self.md_cache.get(&id).unwrap();
        //     let ana = md.ana.clone();
        //     let metrics = md.metrics.clone();
        //     let full_node = FullNode {
        //         global: Global { depth, position },
        //         local: Local {
        //             compressed_node: id,
        //             metrics,
        //             ana,
        //         },
        //     };
        //     return full_node;
        // }
        // let ana = if let Some(label) = label.as_ref() {
        //     assert!(acc.ana.is_none());
        //     if &acc.simple.kind == &Type::Comment {
        //         None
        //     } else if acc.simple.kind.is_literal() {
        //         let tl = acc.simple.kind.literal_type();
        //         // let tl = label_store.get_or_insert(tl);

        //         Some(PartialAnalysis::init(&acc.simple.kind, Some(tl), |x| {
        //             label_store.get_or_insert(x)
        //         }))
        //     } else {
        //         let rf = label_store.get_or_insert(label.as_str());

        //         Some(PartialAnalysis::init(
        //             &acc.simple.kind,
        //             Some(label.as_str()),
        //             |x| label_store.get_or_insert(x),
        //         ))
        //     }
        // } else if acc.simple.kind.is_primitive() {
        //     let node = insertion.resolve(acc.simple.children[0]);
        //     let label = node.get_type().to_string();
        //     if let Some(ana) = acc.ana {
        //         todo!("{:?} {:?}", acc.simple.kind, ana)
        //     }
        //     // let rf = label_store.get_or_insert(label.as_str());

        //     Some(PartialAnalysis::init(
        //         &acc.simple.kind,
        //         Some(label.as_str()),
        //         |x| label_store.get_or_insert(x),
        //     ))
        // } else if let Some(ana) = acc.ana {
        //     // nothing to do, resolutions at the end of post ?
        //     Some(ana)
        // } else if acc.simple.kind == Type::TS86
        //     || acc.simple.kind == Type::TS81
        //     || acc.simple.kind == Type::Asterisk
        //     || acc.simple.kind == Type::Dimensions
        //     || acc.simple.kind == Type::Block
        //     || acc.simple.kind == Type::ElementValueArrayInitializer
        // {
        //     Some(PartialAnalysis::init(&acc.simple.kind, None, |x| {
        //         label_store.get_or_insert(x)
        //     }))
        // } else if acc.simple.kind == Type::ArgumentList
        //     || acc.simple.kind == Type::FormalParameters
        //     || acc.simple.kind == Type::AnnotationArgumentList
        // {
        //     assert!(acc
        //         .simple
        //         .children
        //         .iter()
        //         .all(|x| { !insertion.resolve(*x).has_children() }));
        //     // TODO decls

        //     Some(PartialAnalysis::init(&acc.simple.kind, None, |x| {
        //         label_store.get_or_insert(x)
        //     }))
        // } else if acc.simple.kind == Type::SwitchLabel || acc.simple.kind == Type::Modifiers {
        //     // TODO decls
        //     None
        // } else if acc.simple.kind == Type::BreakStatement
        //     || acc.simple.kind == Type::ContinueStatement
        //     || acc.simple.kind == Type::Wildcard
        //     || acc.simple.kind == Type::ConstructorBody
        //     || acc.simple.kind == Type::InterfaceBody
        //     || acc.simple.kind == Type::SwitchBlock
        //     || acc.simple.kind == Type::ClassBody
        //     || acc.simple.kind == Type::EnumBody
        //     || acc.simple.kind == Type::AnnotationTypeBody
        //     || acc.simple.kind == Type::TypeArguments
        //     || acc.simple.kind == Type::ArrayInitializer
        //     || acc.simple.kind == Type::ReturnStatement
        //     || acc.simple.kind == Type::Error
        // {
        //     // TODO maybe do something later?
        //     None
        // } else {
        //     assert!(
        //         acc.simple.children.is_empty()
        //             || !acc
        //                 .simple
        //                 .children
        //                 .iter()
        //                 .all(|x| { !insertion.resolve(*x).has_children() }),
        //         "{:?}",
        //         &acc.simple.kind
        //     );
        //     None
        // };
        // // TODO resolution now?
        // let ana = match ana {
        //     Some(ana) if &acc.simple.kind == &Type::ClassBody => {
        //         log::trace!("refs in class body");
        //         for x in ana.display_refs(&self.stores.label_store) {
        //             log::trace!("    {}", x);
        //         }
        //         log::trace!("decls in class body");
        //         for x in ana.display_decls(&self.stores.label_store) {
        //             log::trace!("    {}", x);
        //         }
        //         let ana = ana.resolve();
        //         log::trace!("refs in class body after resolution");

        //         for x in ana.display_refs(&self.stores.label_store) {
        //             log::trace!("    {}", x);
        //         }
        //         Some(ana)
        //     }
        //     Some(ana) if acc.simple.kind.is_type_declaration() => {
        //         log::trace!("refs in class decl");
        //         for x in ana.display_refs(&self.stores.label_store) {
        //             log::trace!("    {}", x);
        //         }
        //         log::trace!("decls in class decl");
        //         for x in ana.display_decls(&self.stores.label_store) {
        //             log::trace!("    {}", x);
        //         }
        //         let ana = ana.resolve();
        //         log::trace!("refs in class decl after resolution");

        //         for x in ana.display_refs(&self.stores.label_store) {
        //             log::trace!("    {}", x);
        //         }
        //         // TODO assert that ana.solver.refs does not contains mentions to ?.this
        //         Some(ana)
        //     }
        //     Some(ana) if &acc.simple.kind == &Type::MethodDeclaration => {
        //         // debug!("refs in method decl:");
        //         // for x in ana.display_refs(&self.stores.label_store) {
        //         //     debug!("    {}", x);
        //         // }
        //         let ana = ana.resolve();
        //         // debug!("refs in method decl after resolution:");
        //         // for x in ana.display_refs(&self.stores.label_store) {
        //         //     debug!("    {}", x);
        //         // }
        //         Some(ana)
        //     }
        //     Some(ana) if &acc.simple.kind == &Type::ConstructorDeclaration => {
        //         // debug!("refs in construtor decl:");
        //         // for x in ana.display_refs(&self.stores.label_store) {
        //         //     debug!("    {}", x);
        //         // }
        //         // debug!("decls in construtor decl");
        //         // for x in ana.display_decls(&self.stores.label_store) {
        //         //     debug!("    {}", x);
        //         // }
        //         let ana = ana.resolve();
        //         // debug!("refs in construtor decl after resolution:");
        //         // for x in ana.display_refs(&self.stores.label_store) {
        //         //     debug!("    {}", x);
        //         // }
        //         Some(ana)
        //     }
        //     Some(ana) if &acc.simple.kind == &Type::Program => {
        //         debug!("refs in program");
        //         for x in ana.display_refs(&self.stores.label_store) {
        //             debug!("    {}", x);
        //         }
        //         debug!("decls in program");
        //         for x in ana.display_decls(&self.stores.label_store) {
        //             debug!("    {}", x);
        //         }
        //         let ana = ana.resolve();
        //         // TODO assert that ana.solver.refs does not contains mentions to ?.this
        //         Some(ana)
        //     }
        //     // Some(ana) if &acc.simple.kind == &Type::Directory => {
        //     //     debug!("refs in directory");
        //     //
        //     // for x in ana.display_refs(&self.stores.label_store) {
        //     //     debug!("    {}", x);
        //     // }
        //     //     debug!("decls in directory");
        //     //     for x in ana.display_decls(&self.stores.label_store) {
        //         //     debug!("    {}", x);
        //         // }
        //     //     let ana = ana.resolve();
        //     //     Some(ana)
        //     // }
        //     Some(ana) => Some(ana), // TODO
        //     None => None,
        // };

        // let hashs = SyntaxNodeHashs {
        //     structt: hashed::inner_node_hash(
        //         hashed_kind,
        //         &0,
        //         &acc.metrics.size,
        //         &acc.metrics.hashs.structt,
        //     ),
        //     label: hashed::inner_node_hash(
        //         hashed_kind,
        //         hashed_label,
        //         &acc.metrics.size,
        //         &acc.metrics.hashs.label,
        //     ),
        //     syntax: hsyntax,
        // };
        // let vacant = insertion.vacant();
        // let compressed_node = match label_id {
        //     None => {
        //         macro_rules! insert {
        //             ( $c:expr, $t:ty ) => {
        //                 NodeStore::insert_after_prepare(
        //                     vacant,
        //                     $c.concat((<$t>::SIZE, <$t>::from(ana.as_ref().unwrap().refs()))),
        //                 )
        //             };
        //         }
        //         match acc.simple.children.len() {
        //             0 => {
        //                 assert_eq!(0, metrics.size);
        //                 assert_eq!(0, metrics.height);
        //                 NodeStore::insert_after_prepare(
        //                     vacant,
        //                     (acc.simple.kind.clone(), hashs, BloomSize::None),
        //                 )
        //             }
        //             // 1 => NodeStore::insert_after_prepare(
        //             //     vacant,
        //             //     rest,
        //             //     (acc.simple.kind.clone(), CS0([acc.simple.children[0]])),
        //             // ),
        //             // 2 => NodeStore::insert_after_prepare(
        //             //     vacant,
        //             //     rest,
        //             //     (
        //             //         acc.simple.kind.clone(),
        //             //         CS0([
        //             //             acc.simple.children[0],
        //             //             acc.simple.children[1],
        //             //         ]),
        //             //     ),
        //             // ),
        //             // 3 => NodeStore::insert_after_prepare(
        //             //     vacant,
        //             //     rest,
        //             //     (
        //             //         acc.simple.kind.clone(),
        //             //         CS0([
        //             //             acc.simple.children[0],
        //             //             acc.simple.children[1],
        //             //             acc.simple.children[2],
        //             //         ]),
        //             //     ),
        //             // ),
        //             _ => {
        //                 let a = acc.simple.children;
        //                 let c = (
        //                     acc.simple.kind.clone(),
        //                     compo::Size(metrics.size + 1),
        //                     compo::Height(metrics.height + 1),
        //                     hashs,
        //                     CS(a),
        //                 );
        //                 match ana.as_ref().map(|x| x.refs_count()).unwrap_or(0) {
        //                     x if x > 2048 => NodeStore::insert_after_prepare(
        //                         vacant,
        //                         c.concat((BloomSize::Much,)),
        //                     ),
        //                     x if x > 1024 => {
        //                         insert!(c, Bloom::<&'static [u8], [u64; 64]>)
        //                     }
        //                     x if x > 512 => {//2048
        //                         insert!(c, Bloom::<&'static [u8], [u64; 32]>)
        //                     }
        //                     x if x > 256 => {
        //                         insert!(c, Bloom::<&'static [u8], [u64; 16]>)
        //                     }
        //                     x if x > 150 => {
        //                         insert!(c, Bloom::<&'static [u8], [u64; 8]>)
        //                     }
        //                     x if x > 100 => {
        //                         insert!(c, Bloom::<&'static [u8], [u64; 4]>)
        //                     }
        //                     x if x > 30 => {
        //                         insert!(c, Bloom::<&'static [u8], [u64; 2]>)
        //                     }
        //                     x if x > 15 => {
        //                         insert!(c, Bloom::<&'static [u8], u64>)
        //                     }
        //                     x if x > 8 => {
        //                         insert!(c, Bloom::<&'static [u8], u32>)
        //                     }
        //                     x if x > 0 => {
        //                         insert!(c, Bloom::<&'static [u8], u16>)
        //                     }
        //                     _ => NodeStore::insert_after_prepare(
        //                         vacant,
        //                         c.concat((BloomSize::None,)),
        //                     ),
        //                 }
        //             }
        //         }
        //     }
        //     Some(label) => {
        //         assert!(acc.simple.children.is_empty());
        //         NodeStore::insert_after_prepare(
        //             vacant,
        //             (acc.simple.kind.clone(), hashs, label, BloomSize::None), // None not sure
        //         )
        //     }
        // };

        // let metrics = SubTreeMetrics {
        //     size: metrics.size + 1,
        //     height: metrics.height + 1,
        //     hashs,
        // };
        // // TODO see if possible to only keep md in md_cache, but would need a generational cache I think
        // self.md_cache.insert(
        //     compressed_node,
        //     MD {
        //         metrics: metrics.clone(),
        //         ana: ana.clone(),
        //     },
        // );

        // let full_node = FullNode {
        //     global: Global { depth, position },
        //     local: Local {
        //         compressed_node,
        //         metrics,
        //         ana,
        //     },
        // };
        // full_node
    }

    fn init_val(&mut self, text: &[u8], node: &Self::Node<'_>) -> Self::Acc {
        let type_store = &mut self.stores().type_store;
        let kind = type_store.get(node.kind());
        let parent_indentation = Space::try_format_indentation(&self.line_break)
            .unwrap_or_else(|| vec![Space::Space; self.line_break.len()]);
        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            0,
            &parent_indentation,
        );
        let ana = self.build_ana(&kind);
        let label = node
            .extract_label(text)
            // let label = label_for_cursor(text, &node)
            .and_then(|x| Some(std::str::from_utf8(&x).unwrap().to_owned()));
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            label,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
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

impl<'a> JavaTreeGen<'a> {
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

            let insertion = node_store.prepare_insertion(&hashable, eq);

            let hashs = SyntaxNodeHashs {
                structt: 0,
                label: 0,
                syntax: hsyntax,
            };

            let compressed_node = if let Some(id) = insertion.occupied_id() {
                id
            } else {
                let vacant = insertion.vacant();
                NodeStore::insert_after_prepare(
                    vacant,
                    (
                        Type::Spaces,
                        spaces,
                        hashs,
                        // compo::BytesLen((pos - padding_start).to_u32().expect("too much spaces")),
                        BloomSize::None,
                    ),
                )
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

    pub fn new<'b>(stores: &'b mut SimpleStores, md_cache: &'b mut MDCache) -> JavaTreeGen<'b> {
        JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
        }
    }

    pub fn generate_file(
        &mut self,
        name: &[u8],
        text: &[u8],
        cursor: TreeCursor,
    ) -> FullNode<Global, Local> {
        let mut init = self.init_val(text, &TNode(cursor.node()));
        let mut xx = TTreeCursor(cursor);
        let node_store = &mut self.stores.node_store;
        Self::handle_spacing(
            init.padding_start,
            init.start_byte,
            text,
            node_store,
            &(0 + 1),
            0,
            &mut init,
        );
        init.label = Some(std::str::from_utf8(name).unwrap().to_owned());
        let mut stack = vec![init];

        let sum_byte_length = self.gen(text, &mut stack, &mut xx);

        let mut acc = stack.pop().unwrap();

        Self::handle_final_space(
            &0,
            sum_byte_length,
            text,
            &mut self.stores.node_store,
            acc.metrics.size as usize + 1,
            &mut acc,
        );
        let full_node = self.make(0, acc.metrics.size as usize, acc);

        match full_node.local.ana.as_ref() {
            Some(x) => {
                log::debug!("refs in file:",);
                for x in x.display_refs(&self.stores.label_store) {
                    log::debug!("    {}", x);
                }
                log::debug!("decls in file:",);
                for x in x.display_decls(&self.stores.label_store) {
                    log::debug!("    {}", x);
                }
            }
            None => log::debug!("None"),
        };

        full_node
    }

    fn build_ana(&mut self, kind: &Type) -> Option<PartialAnalysis> {
        let label_store = &mut self.stores.label_store;
        if kind == &Type::ClassBody
            || kind == &Type::PackageDeclaration
            || kind == &Type::ClassDeclaration
            || kind == &Type::EnumDeclaration
            || kind == &Type::InterfaceDeclaration
            || kind == &Type::AnnotationTypeDeclaration
            || kind == &Type::Program
        {
            Some(PartialAnalysis::init(kind, None, |x| {
                label_store.get_or_insert(x)
            }))
        } else if kind == &Type::TypeParameter {
            Some(PartialAnalysis::init(kind, None, |x| {
                label_store.get_or_insert(x)
            }))
        } else {
            None
        }
    }
    fn make(
        &mut self,
        depth: usize,
        position: usize,
        // text: &[u8],
        // node: &Self::Node<'_>,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let node_store = &mut self.stores.node_store;
        let label_store = &mut self.stores.label_store;

        let label = acc.label;
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

        let label_id = if let Some(label) = label.as_ref() {
            if &acc.simple.kind == &Type::Comment {
                // None // TODO check
                Some(label_store.get_or_insert(label.as_str()))
            } else if acc.simple.kind.is_literal() {
                let tl = acc.simple.kind.literal_type();
                // let tl = label_store.get_or_insert(tl);

                Some(label_store.get_or_insert(label.as_str()))
            } else {
                let rf = label_store.get_or_insert(label.as_str());
                Some(rf)
            }
        } else if acc.simple.kind.is_primitive() {
            None
        } else if let Some(_) = acc.ana {
            None
        } else if acc.simple.kind == Type::TS86
            || acc.simple.kind == Type::TS81
            || acc.simple.kind == Type::Asterisk
            || acc.simple.kind == Type::Dimensions
            || acc.simple.kind == Type::Block
            || acc.simple.kind == Type::ElementValueArrayInitializer
        {
            None
        } else if acc.simple.kind == Type::ArgumentList
            || acc.simple.kind == Type::FormalParameters
            || acc.simple.kind == Type::AnnotationArgumentList
        {
            None
        } else if acc.simple.kind == Type::SwitchLabel || acc.simple.kind == Type::Modifiers {
            None
        } else if acc.simple.kind == Type::BreakStatement
            || acc.simple.kind == Type::ContinueStatement
            || acc.simple.kind == Type::Wildcard
            || acc.simple.kind == Type::ConstructorBody
            || acc.simple.kind == Type::InterfaceBody
            || acc.simple.kind == Type::SwitchBlock
            || acc.simple.kind == Type::ClassBody
            || acc.simple.kind == Type::EnumBody
            || acc.simple.kind == Type::AnnotationTypeBody
            || acc.simple.kind == Type::TypeArguments
            || acc.simple.kind == Type::ArrayInitializer
            || acc.simple.kind == Type::ReturnStatement
            || acc.simple.kind == Type::Error
        {
            None
        } else {
            None
        };

        let eq = |x: EntryRef| {
            let t = x.get_component::<Type>().ok();
            if &t != &Some(&acc.simple.kind) {
                // println!("typed: {:?} {:?}", acc.simple.kind, t);
                return false;
            }
            let l = x.get_component::<LabelIdentifier>().ok();
            if l != label_id.as_ref() {
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
        let insertion = node_store.prepare_insertion(&hashable, eq);

        if let Some(id) = insertion.occupied_id() {
            let md = self.md_cache.get(&id).unwrap();
            let ana = md.ana.clone();
            let metrics = md.metrics.clone();
            let full_node = FullNode {
                global: Global { depth, position },
                local: Local {
                    compressed_node: id,
                    metrics,
                    ana,
                },
            };
            return full_node;
        }
        let ana = if acc.simple.kind == Type::Program {
            acc.ana
        } else if let Some(label) = label.as_ref() {
            assert!(acc.ana.is_none());
            if &acc.simple.kind == &Type::Comment {
                None
            } else if acc.simple.kind.is_literal() {
                let tl = acc.simple.kind.literal_type();
                // let tl = label_store.get_or_insert(tl);

                Some(PartialAnalysis::init(&acc.simple.kind, Some(tl), |x| {
                    label_store.get_or_insert(x)
                }))
            } else {
                let rf = label_store.get_or_insert(label.as_str());

                Some(PartialAnalysis::init(
                    &acc.simple.kind,
                    Some(label.as_str()),
                    |x| label_store.get_or_insert(x),
                ))
            }
        } else if acc.simple.kind.is_primitive() {
            let node = insertion.resolve(acc.simple.children[0]);
            let label = node.get_type().to_string();
            if let Some(ana) = acc.ana {
                todo!("{:?} {:?}", acc.simple.kind, ana)
            }
            // let rf = label_store.get_or_insert(label.as_str());

            Some(PartialAnalysis::init(
                &acc.simple.kind,
                Some(label.as_str()),
                |x| label_store.get_or_insert(x),
            ))
        } else if let Some(ana) = acc.ana {
            // nothing to do, resolutions at the end of post ?
            Some(ana)
        } else if acc.simple.kind == Type::TS86
            || acc.simple.kind == Type::TS81
            || acc.simple.kind == Type::Asterisk
            || acc.simple.kind == Type::Dimensions
            || acc.simple.kind == Type::Block
            || acc.simple.kind == Type::ElementValueArrayInitializer
        {
            Some(PartialAnalysis::init(&acc.simple.kind, None, |x| {
                label_store.get_or_insert(x)
            }))
        } else if acc.simple.kind == Type::ArgumentList
            || acc.simple.kind == Type::FormalParameters
            || acc.simple.kind == Type::AnnotationArgumentList
        {
            if !acc
                .simple
                .children
                .iter()
                .all(|x| !insertion.resolve(*x).has_children())
            {
                // eg. an empty body/block/paramlist/...
                log::error!("{:?} should only contains leafs", &acc.simple.kind);
            }

            Some(PartialAnalysis::init(&acc.simple.kind, None, |x| {
                label_store.get_or_insert(x)
            }))
        } else if acc.simple.kind == Type::SwitchLabel || acc.simple.kind == Type::Modifiers {
            // TODO decls
            None
        } else if acc.simple.kind == Type::BreakStatement
            || acc.simple.kind == Type::ContinueStatement
            || acc.simple.kind == Type::Wildcard
            || acc.simple.kind == Type::ConstructorBody
            || acc.simple.kind == Type::InterfaceBody
            || acc.simple.kind == Type::SwitchBlock
            || acc.simple.kind == Type::ClassBody
            || acc.simple.kind == Type::EnumBody
            || acc.simple.kind == Type::ModuleBody
            || acc.simple.kind == Type::AnnotationTypeBody
            || acc.simple.kind == Type::TypeArguments
            || acc.simple.kind == Type::ArrayInitializer
            || acc.simple.kind == Type::ReturnStatement
            || acc.simple.kind == Type::ForStatement
            || acc.simple.kind == Type::RequiresModifier
            || acc.simple.kind == Type::Error
        {
            // TODO maybe do something later?
            None
        } else {
            if !acc.simple.children.is_empty()
                && acc
                    .simple
                    .children
                    .iter()
                    .all(|x| !insertion.resolve(*x).has_children())
            {
                // eg. an empty body/block/paramlist/...
                log::error!("{:?} should only contains leafs", &acc.simple.kind);
            }
            None
        };
        // TODO resolution now?
        let ana = match ana {
            Some(ana) if &acc.simple.kind == &Type::ClassBody => {
                log::trace!("refs in class body");
                for x in ana.display_refs(&self.stores.label_store) {
                    log::trace!("    {}", x);
                }
                log::trace!("decls in class body");
                for x in ana.display_decls(&self.stores.label_store) {
                    log::trace!("    {}", x);
                }
                let ana = ana.resolve();
                log::trace!("refs in class body after resolution");

                for x in ana.display_refs(&self.stores.label_store) {
                    log::trace!("    {}", x);
                }
                Some(ana)
            }
            Some(ana) if acc.simple.kind.is_type_declaration() => {
                log::trace!("refs in class decl");
                for x in ana.display_refs(&self.stores.label_store) {
                    log::trace!("    {}", x);
                }
                log::trace!("decls in class decl");
                for x in ana.display_decls(&self.stores.label_store) {
                    log::trace!("    {}", x);
                }
                let ana = ana.resolve();
                log::trace!("refs in class decl after resolution");

                for x in ana.display_refs(&self.stores.label_store) {
                    log::trace!("    {}", x);
                }
                // TODO assert that ana.solver.refs does not contains mentions to ?.this
                Some(ana)
            }
            Some(ana) if &acc.simple.kind == &Type::MethodDeclaration => {
                log::trace!("refs in method decl:");
                for x in ana.display_refs(&self.stores.label_store) {
                    log::trace!("    {}", x);
                }
                log::trace!("decls in method decl");
                for x in ana.display_decls(&self.stores.label_store) {
                    log::trace!("    {}", x);
                }
                let ana = ana.resolve();
                log::trace!("refs in method decl after resolution:");
                for x in ana.display_refs(&self.stores.label_store) {
                    log::trace!("    {}", x);
                }
                Some(ana)
            }
            Some(ana) if &acc.simple.kind == &Type::ConstructorDeclaration => {
                // debug!("refs in construtor decl:");
                // for x in ana.display_refs(&self.stores.label_store) {
                //     log::debug!("    {}", x);
                // }
                // debug!("decls in construtor decl");
                // for x in ana.display_decls(&self.stores.label_store) {
                //     log::debug!("    {}", x);
                // }
                let ana = ana.resolve();
                // debug!("refs in construtor decl after resolution:");
                // for x in ana.display_refs(&self.stores.label_store) {
                //     log::debug!("    {}", x);
                // }
                Some(ana)
            }
            Some(ana) if &acc.simple.kind == &Type::Program => {
                log::debug!("refs in program");
                for x in ana.display_refs(&self.stores.label_store) {
                    log::debug!("    {}", x);
                }
                log::debug!("decls in program");
                for x in ana.display_decls(&self.stores.label_store) {
                    log::debug!("    {}", x);
                }
                let ana = ana.resolve();
                log::debug!("refs in program after resolve");
                for x in ana.display_refs(&self.stores.label_store) {
                    log::debug!("    {}", x);
                }
                // TODO assert that ana.solver.refs does not contains mentions to ?.this
                Some(ana)
            }
            // Some(ana) if &acc.simple.kind == &Type::Directory => {
            //     log::debug!("refs in directory");
            //
            // for x in ana.display_refs(&self.stores.label_store) {
            //     log::debug!("    {}", x);
            // }
            //     log::debug!("decls in directory");
            //     for x in ana.display_decls(&self.stores.label_store) {
            //     log::debug!("    {}", x);
            // }
            //     let ana = ana.resolve();
            //     Some(ana)
            // }
            Some(ana) => {
                Some(ana) // TODO
            }
            None => None,
        };

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
        let vacant = insertion.vacant();
        let compressed_node = match label_id {
            None => {
                log::trace!(
                    "insertion with {} refs",
                    ana.as_ref().map(|x| x.estimated_refs_count()).unwrap_or(0)
                );
                macro_rules! insert {
                    ( $c:expr, $t:ty ) => {{
                        let it = ana.as_ref().unwrap().solver.iter_refs();
                        let it =
                            BulkHasher::<_, <$t as BF<[u8]>>::S, <$t as BF<[u8]>>::H>::from(it);
                        let bloom = <$t>::from(it);
                        log::trace!("{:?}", bloom);
                        NodeStore::insert_after_prepare(vacant, $c.concat((<$t>::SIZE, bloom)))
                    }};
                }
                // type A = Bloom<&'static [u8], [u64; 64]>;
                // let it = ana.as_ref().unwrap().solver.iter_refs();
                // let it = BulkHasher::<_, <A as BF<[u8]>>::S, <A as BF<[u8]>>::H>::from(it);
                // let zazaz = A::from(it);
                match acc.simple.children.len() {
                    0 => {
                        assert_eq!(0, metrics.size);
                        assert_eq!(0, metrics.height);
                        NodeStore::insert_after_prepare(
                            vacant,
                            (
                                acc.simple.kind.clone(),
                                hashs,
                                compo::BytesLen((acc.end_byte - acc.start_byte) as u32),
                                BloomSize::None,
                            ),
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
                            compo::BytesLen((acc.end_byte - acc.start_byte) as u32),
                            hashs,
                            CS(a),
                        );
                        match ana.as_ref().map(|x| x.estimated_refs_count()).unwrap_or(0) {
                            x if x > 2048 => NodeStore::insert_after_prepare(
                                vacant,
                                c.concat((BloomSize::Much,)),
                            ),
                            x if x > 1024 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 64]>)
                            }
                            x if x > 512 => {
                                //2048
                                insert!(c, Bloom::<&'static [u8], [u64; 32]>)
                            }
                            x if x > 256 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 16]>)
                            }
                            x if x > 150 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 8]>)
                            }
                            x if x > 100 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 4]>)
                            }
                            x if x > 30 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 2]>)
                            }
                            x if x > 15 => {
                                insert!(c, Bloom::<&'static [u8], u64>)
                            }
                            x if x > 8 => {
                                insert!(c, Bloom::<&'static [u8], u32>)
                            }
                            x if x > 0 => {
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
                macro_rules! insert {
                    ( $c:expr, $t:ty ) => {{
                        log::trace!(
                            "it: {:?}",
                            BulkHasher::<_, <$t as BF<[u8]>>::S, <$t as BF<[u8]>>::H>::from(
                                ana.as_ref().unwrap().solver.iter_refs()
                            )
                            .collect::<Vec<_>>()
                        );
                        let it = ana.as_ref().unwrap().solver.iter_refs();
                        let it =
                            BulkHasher::<_, <$t as BF<[u8]>>::S, <$t as BF<[u8]>>::H>::from(it);
                        let bloom = <$t>::from(it);
                        log::trace!("{:?}", bloom);
                        NodeStore::insert_after_prepare(vacant, $c.concat((<$t>::SIZE, bloom)))
                    }};
                }
                match acc.simple.children.len() {
                    0 => {
                        assert_eq!(0, metrics.size);
                        assert_eq!(0, metrics.height);
                        assert!(acc.simple.children.is_empty());
                        NodeStore::insert_after_prepare(
                            vacant,
                            (
                                acc.simple.kind.clone(),
                                hashs,
                                label,
                                compo::BytesLen((acc.end_byte - acc.start_byte) as u32),
                                BloomSize::None,
                            ), // None not sure
                        )
                    }
                    _ => {
                        let a = acc.simple.children;
                        let c = (
                            acc.simple.kind.clone(),
                            compo::Size(metrics.size + 1),
                            compo::Height(metrics.height + 1),
                            compo::BytesLen((acc.end_byte - acc.start_byte) as u32),
                            hashs,
                            label,
                            CS(a),
                        );
                        match ana.as_ref().map(|x| x.estimated_refs_count()).unwrap_or(0) {
                            x if x > 2048 => NodeStore::insert_after_prepare(
                                vacant,
                                c.concat((BloomSize::Much,)),
                            ),
                            x if x > 1024 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 64]>)
                            }
                            x if x > 512 => {
                                //2048
                                insert!(c, Bloom::<&'static [u8], [u64; 32]>)
                            }
                            x if x > 256 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 16]>)
                            }
                            x if x > 150 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 8]>)
                            }
                            x if x > 100 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 4]>)
                            }
                            x if x > 30 => {
                                insert!(c, Bloom::<&'static [u8], [u64; 2]>)
                            }
                            x if x > 15 => {
                                insert!(c, Bloom::<&'static [u8], u64>)
                            }
                            x if x > 8 => {
                                insert!(c, Bloom::<&'static [u8], u32>)
                            }
                            x if x > 0 => {
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
        };

        let metrics = SubTreeMetrics {
            size: metrics.size + 1,
            height: metrics.height + 1,
            hashs,
        };
        // TODO see if possible to only keep md in md_cache, but would need a generational cache I think
        self.md_cache.insert(
            compressed_node,
            MD {
                metrics: metrics.clone(),
                ana: ana.clone(),
            },
        );

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
    nodes::print_tree_syntax(
        |id| -> _ {
            node_store
                .resolve(id.clone())
                .into_compressed_node()
                .unwrap()
        },
        |id| -> _ { label_store.resolve(id).to_owned() },
        id,
        &mut Into::<IoOut<_>>::into(stdout()),
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

pub struct TreeSerializer<'a> {
    node_store: &'a NodeStore,
    label_store: &'a LabelStore,
    id: NodeIdentifier,
}
impl<'a> TreeSerializer<'a> {
    pub fn new(node_store: &'a NodeStore, label_store: &'a LabelStore, id: NodeIdentifier) -> Self {
        Self {
            node_store,
            label_store,
            id,
        }
    }
}
impl<'a> Display for TreeSerializer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        serialize(self.node_store, self.label_store, &self.id, f, "\n");
        Ok(())
    }
}

pub struct TreeSyntax<'a> {
    node_store: &'a NodeStore,
    label_store: &'a LabelStore,
    id: NodeIdentifier,
}
impl<'a> TreeSyntax<'a> {
    pub fn new(node_store: &'a NodeStore, label_store: &'a LabelStore, id: NodeIdentifier) -> Self {
        Self {
            node_store,
            label_store,
            id,
        }
    }
}

impl<'a> Display for TreeSyntax<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        nodes::print_tree_syntax(
            |id| -> _ {
                self.node_store
                    .resolve(id.clone())
                    .into_compressed_node()
                    .unwrap()
            },
            |id| -> _ { self.label_store.resolve(id).to_owned() },
            &self.id,
            f,
        );
        Ok(())
    }
}

// impl NodeStore {
//     pub(crate) fn new() -> Self {
//         Self {
//             count: 0,
//             errors: 0,
//             // roots: Default::default(),
//             internal: Default::default(),
//             dedup: hashbrown::HashMap::<_, (), ()>::with_capacity_and_hasher(
//                 1 << 10,
//                 Default::default(),
//             ),
//             hasher: Default::default(),
//         }
//     }
// }

// impl LabelStore {
//     pub(crate) fn new() -> Self {
//         let mut r = Self {
//             count: 1,
//             internal: Default::default(),
//         };
//         r.get_or_insert("length"); // TODO verify/model statically
//         r
//     }
// }
