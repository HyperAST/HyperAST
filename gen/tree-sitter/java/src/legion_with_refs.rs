///! fully compress all subtrees from a Java CST
use crate::{
    types::{TIdN, TStore, Type},
    TNode,
};
use hyper_ast::{
    cyclomatic::Mcc,
    full::FullNode,
    hashed::{HashedNode, IndexingHashBuilder, MetaDataHashsBuilder},
    store::{
        defaults::LabelIdentifier,
        labels::LabelStore,
        nodes::{
            legion::{dyn_builder, eq_node, HashedNodeRef, PendingInsert},
            EntityBuilder,
        },
    },
    tree_gen::{
        parser::{Node, TreeCursor, Visibility},
        BasicGlobalData, GlobalData, Parents, PreResult, SpacedGlobalData, SubTreeMetrics,
        TextedGlobalData, TotalBytesGlobalData, TreeGen, WithByteRange,
    },
    types::{self, AnyType, NodeStoreExt, Role, TypeStore, TypeTrait, WithHashs, WithStats},
};
use legion::world::EntryRef;
use num::ToPrimitive;
use std::{collections::HashMap, fmt::Debug, vec};
use tuples::CombinConcat;

use hyper_ast::{
    filter::BF,
    filter::{Bloom, BloomSize},
    hashed::{self, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    nodes::Space,
    store::{nodes::legion::compo, nodes::DefaultNodeStore as NodeStore, SimpleStores},
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, AccIndentation, Accumulator,
        BasicAccumulator, Spaces, ZippedTreeGen,
    },
    types::{
        LabelStore as LabelStoreTrait,
        Tree,
        // NodeStore as NodeStoreTrait,
        Typed,
    },
};
// use hyper_ast::nodes::SimpleNode1;

use crate::types::JavaEnabledTypeStore;
// type Type = TypeU16<Java>;

#[cfg(feature = "impact")]
use crate::impact::partial_analysis::PartialAnalysis;
#[cfg(feature = "impact")]
use hyper_ast::impact::BulkHasher;

pub type EntryR<'a> = EntryRef<'a>;

pub type NodeIdentifier = legion::Entity;

// pub struct HashedNodeRef<'a>(EntryRef<'a>);

pub type FNode = FullNode<BasicGlobalData, Local>;

// TODO try to use a const generic for space less generation ?
// SPC: consider spaces ie. add them to the HyperAST,
// NOTE there is a big issue with the byteLen of subtree then.
// just provide a view abstracting spaces (see attempt in hyper_diff)
pub struct JavaTreeGen<'stores, 'cache, TS = TStore, More = ()> {
    pub line_break: Vec<u8>,
    pub stores: &'stores mut SimpleStores<TS>,
    pub md_cache: &'cache mut MDCache,
    pub more: More,
}

pub type MDCache = HashMap<NodeIdentifier, MD>;

// NOTE only keep compute intensive metadata (where space/time tradeoff is worth storing)
// eg. decls refs, maybe hashes but not size and height
// * metadata: computation results from concrete code of node and its children
// they can be qualitative metadata .eg a hash or they can be quantitative .eg lines of code
pub struct MD {
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: Option<PartialAnalysis>,
    mcc: Mcc,
    precomp_queries: PrecompQueries,
}

// Enables static reference analysis
const ANA: bool = false;

impl From<Local> for MD {
    fn from(x: Local) -> Self {
        MD {
            metrics: x.metrics,
            ana: x.ana,
            mcc: x.mcc,
            precomp_queries: x.precomp_queries,
        }
    }
}

pub type Global<'a> = SpacedGlobalData<'a>;

type PrecompQueries = u16;

#[derive(Debug, Clone)]
pub struct Local {
    pub compressed_node: NodeIdentifier,
    // * metadata: computation results from concrete code of node and its children
    // they can be qualitative metadata, e.g. a hash or they can be quantitative e.g. lines of code
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub ana: Option<PartialAnalysis>,
    pub mcc: Mcc,
    pub role: Option<Role>,
    pub precomp_queries: PrecompQueries,
}

impl Local {
    fn acc(self, acc: &mut Acc) {
        if self.metrics.size_no_spaces > 0 {
            acc.no_space.push(self.compressed_node)
        }
        if let Some(role) = self.role {
            let o = acc.simple.children.len();
            acc.role.acc(role, o);
        }
        acc.simple.push(self.compressed_node);
        acc.metrics.acc(self.metrics);
        acc.precomp_queries |= self.precomp_queries;

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
        self.mcc.acc(&mut acc.mcc)
    }
}

pub struct Acc {
    simple: BasicAccumulator<Type, NodeIdentifier>,
    no_space: Vec<NodeIdentifier>,
    labeled: bool,
    start_byte: usize,
    end_byte: usize,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: Option<PartialAnalysis>,
    mcc: Mcc,
    padding_start: usize,
    indentation: Spaces,
    role: RoleAcc<crate::types::Role>,
    precomp_queries: PrecompQueries,
}

impl Accumulator for Acc {
    type Node = FullNode<BasicGlobalData, Local>;
    fn push(&mut self, full_node: Self::Node) {
        full_node.local.acc(self);
    }
}

impl AccIndentation for Acc {
    fn indentation(&self) -> &Spaces {
        &self.indentation
    }
}

impl WithByteRange for Acc {
    fn has_children(&self) -> bool {
        !self.simple.children.is_empty()
    }

    fn begin_byte(&self) -> usize {
        self.start_byte
    }

    fn end_byte(&self) -> usize {
        self.end_byte
    }
}

impl Debug for Acc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Acc")
            .field("simple", &self.simple)
            .field("no_space", &self.no_space)
            .field("labeled", &self.labeled)
            .field("start_byte", &self.start_byte)
            .field("end_byte", &self.end_byte)
            .field("metrics", &self.metrics)
            .field("ana", &self.ana)
            .field("mcc", &self.mcc)
            .field("padding_start", &self.padding_start)
            .field("indentation", &self.indentation)
            .finish()
    }
}

struct RoleAcc<R> {
    pub current: Option<R>,
    pub roles: Vec<R>,
    pub offsets: Vec<u8>,
}

impl<R> Default for RoleAcc<R> {
    fn default() -> Self {
        Self {
            current: None,
            roles: Default::default(),
            offsets: Default::default(),
        }
    }
}

impl<R> RoleAcc<R> {
    fn acc(&mut self, role: R, o: usize) {
        self.roles.push(role);
        self.offsets.push(o.to_u8().unwrap());
    }

    fn add_md(self, dyn_builder: &mut impl EntityBuilder)
    where
        R: 'static + std::marker::Send + std::marker::Sync,
    {
        debug_assert!(self.current.is_none());
        if self.roles.len() > 0 {
            dyn_builder.add(self.roles.into_boxed_slice());
            dyn_builder.add(compo::RoleOffsets(self.offsets.into_boxed_slice()));
        }
    }
}

/// enables recovering of hidden nodes from tree-sitter
#[cfg(not(debug_assertions))]
const HIDDEN_NODES: bool = true;
/// enables recovering of hidden nodes from tree-sitter
// NOTE static mut allows me to change it in unit tests
#[cfg(debug_assertions)]
pub static mut HIDDEN_NODES: bool = true;

#[cfg(not(debug_assertions))]
const fn should_get_hidden_nodes() -> bool {
    HIDDEN_NODES
}
#[cfg(debug_assertions)]
pub(crate) fn should_get_hidden_nodes() -> bool {
    unsafe { HIDDEN_NODES }
}

#[repr(transparent)]
pub struct TTreeCursor<'a>(tree_sitter::TreeCursor<'a>);

impl<'a> Debug for TTreeCursor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TTreeCursor")
            .field(&self.0.node().kind())
            .finish()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(unused)]
enum TreeCursorStep {
    TreeCursorStepNone,
    TreeCursorStepHidden,
    TreeCursorStepVisible,
}

impl TreeCursorStep {
    fn ok(&self) -> Option<Visibility> {
        match self {
            TreeCursorStep::TreeCursorStepNone => None,
            TreeCursorStep::TreeCursorStepHidden => Some(Visibility::Hidden),
            TreeCursorStep::TreeCursorStepVisible => Some(Visibility::Visible),
        }
    }
}

extern "C" {
    fn ts_tree_cursor_goto_first_child_internal(
        self_: *mut tree_sitter::ffi::TSTreeCursor,
    ) -> TreeCursorStep;
    fn ts_tree_cursor_goto_next_sibling_internal(
        self_: *mut tree_sitter::ffi::TSTreeCursor,
    ) -> TreeCursorStep;
}

impl<'a> hyper_ast::tree_gen::parser::TreeCursor<'a, TNode<'a>> for TTreeCursor<'a> {
    fn node(&self) -> TNode<'a> {
        TNode(self.0.node())
    }

    fn role(&self) -> Option<std::num::NonZeroU16> {
        self.0.field_id()
    }

    fn goto_parent(&mut self) -> bool {
        self.0.goto_parent()
    }

    fn goto_first_child(&mut self) -> bool {
        self.goto_first_child_extended().is_some()
    }

    fn goto_next_sibling(&mut self) -> bool {
        self.goto_next_sibling_extended().is_some()
    }

    fn goto_first_child_extended(&mut self) -> Option<Visibility> {
        if should_get_hidden_nodes() {
            unsafe {
                let s = &mut self.0;
                let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(s);
                ts_tree_cursor_goto_first_child_internal(s)
            }
            .ok()
        } else {
            if self.0.goto_first_child() {
                Some(Visibility::Visible)
            } else {
                None
            }
        }
    }

    fn goto_next_sibling_extended(&mut self) -> Option<Visibility> {
        if should_get_hidden_nodes() {
            unsafe {
                let s = &mut self.0;
                let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(s);
                ts_tree_cursor_goto_next_sibling_internal(s)
            }
            .ok()
        } else {
            if self.0.goto_next_sibling() {
                Some(Visibility::Visible)
            } else {
                None
            }
        }
    }
}
pub trait More<HAST: TypeStore> {
    const ENABLED: bool;
    fn match_precomp_queries(
        &self,
        stores: &HAST,
        acc: &Acc,
        label: Option<&str>,
    ) -> PrecompQueries;
}

impl<HAST: TypeStore> More<HAST> for () {
    const ENABLED: bool = false;
    fn match_precomp_queries(
        &self,
        _stores: &HAST,
        _acc: &Acc,
        _label: Option<&str>,
    ) -> PrecompQueries {
        Default::default()
    }
}

pub type MoreStore<'a, 'b, 'c, TS> = SimpleStores<TS, &'b legion::World, &'c LabelStore>;

impl<'a, 'b, 'c, TS> More<MoreStore<'a, 'b, 'c, TS>> for std::sync::Arc<hyper_ast_tsquery::Query>
where
    TS: JavaEnabledTypeStore + hyper_ast::types::RoleStore<IdF = u16, Role = Role>,
{
    const ENABLED: bool = true;
    fn match_precomp_queries(
        &self,
        stores: &SimpleStores<TS, &'b legion::World, &'c LabelStore>,
        acc: &Acc,
        label: Option<&str>,
    ) -> PrecompQueries {
        let cursor = cursor_on_unbuild::TreeCursor::new(
            stores,
            acc,
            label,
            hyper_ast::position::StructuralPosition::empty(),
        );
        // dbg!(acc.simple.kind);
        // let cursor = aaa::TreeCursor::new(
        //     stores,
        //     hyper_ast::position::StructuralPosition::new(todo!()),
        // );
        let qcursor = self.matches_immediate(cursor); // TODO filter on height (and visibility?)
        let mut r = Default::default();
        for m in qcursor {
            assert!(m.pattern_index.to_usize() < 16);
            r |= 1 << m.pattern_index.to_usize() as u16;
            // dbg!(m.pattern_index.to_usize());
        }
        r
    }
}

impl<'a, 'b, 'c, TS> More<MoreStore<'a, 'b, 'c, TS>> for hyper_ast_tsquery::Query
where
    TS: JavaEnabledTypeStore<Ty = Type> + hyper_ast::types::RoleStore<IdF = u16, Role = Role>,
{
    const ENABLED: bool = true;
    fn match_precomp_queries(
        &self,
        stores: &SimpleStores<TS, &'b legion::World, &'c LabelStore>,
        acc: &Acc,
        label: Option<&str>,
    ) -> PrecompQueries {
        let cursor = cursor_on_unbuild::TreeCursor::new(
            stores,
            acc,
            label,
            hyper_ast::position::StructuralPosition::empty(),
        );
        // dbg!(acc.simple.kind);
        // let cursor = aaa::TreeCursor::new(
        //     stores,
        //     hyper_ast::position::StructuralPosition::new(todo!()),
        // );
        let qcursor = self.matches_immediate(cursor); // TODO filter on height (and visibility?)
        let mut r = Default::default();
        for m in qcursor {
            assert!(m.pattern_index.to_usize() < 7);
            r |= 1 << m.pattern_index.to_usize() as u8;
            // dbg!(m.pattern_index.to_usize());
        }
        r
    }
}

mod cursor_on_unbuild;

/// Implements [ZippedTreeGen] to offer a visitor for Java generation
impl<'stores, 'cache, TS, More> ZippedTreeGen for JavaTreeGen<'stores, 'cache, TS, More>
where
    TS: JavaEnabledTypeStore,
    More: for<'a, 'b> self::More<SimpleStores<TS, &'b legion::World, &'a LabelStore>>,
{
    // type Node1 = SimpleNode1<NodeIdentifier, String>;
    type Stores = SimpleStores<TS>;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b>;

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn init_val(&mut self, text: &[u8], node: &Self::Node<'_>) -> <Self as TreeGen>::Acc {
        let type_store = &mut self.stores().type_store;
        let kind = node.obtain_type(type_store);
        let parent_indentation = Space::try_format_indentation(&self.line_break)
            .unwrap_or_else(|| vec![Space::Space; self.line_break.len()]);
        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            0,
            &parent_indentation,
        );
        let labeled = node.has_label();
        let ana = self.build_ana(&kind);
        let mcc = Mcc::new(&kind);
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            no_space: vec![],
            labeled,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            metrics: Default::default(),
            ana,
            mcc,
            padding_start: 0,
            indentation: indent,
            role: Default::default(),
            precomp_queries: Default::default(),
        }
    }

    fn pre_skippable(
        &mut self,
        text: &Self::Text,
        cursor: &Self::TreeCursor<'_>,
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> PreResult<<Self as TreeGen>::Acc> {
        let type_store = &mut self.stores().type_store;
        let node = &cursor.node();
        let kind = node.obtain_type(type_store);
        if should_get_hidden_nodes() {
            if kind.is_repeat() {
                return PreResult::Ignore;
            } else if kind == Type::_UnannotatedType
                || kind == Type::_VariableDeclaratorList
                || kind == Type::_VariableDeclaratorId
            {
                return PreResult::Ignore;
            }
        }
        if node.0.is_missing() {
            return PreResult::Skip;
        }
        let mut acc = self.pre(text, node, stack, global);
        // TODO replace with wrapper
        if !stack
            .parent()
            .map_or(false, |a| a.simple.kind.is_supertype())
        {
            if let Some(r) = cursor.0.field_name() {
                acc.role.current = r.try_into().ok();
            }
        }
        if kind == Type::StringLiteral {
            acc.labeled = true;
            return PreResult::SkipChildren(acc);
        }
        PreResult::Ok(acc)
    }

    fn pre(
        &mut self,
        text: &[u8],
        node: &Self::Node<'_>,
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> <Self as TreeGen>::Acc {
        let type_store = &mut self.stores().type_store;
        let parent_indentation = &stack.parent().unwrap().indentation();
        let kind = node.obtain_type(type_store);
        assert!(
            global.sum_byte_length() <= node.start_byte(),
            "{}: {} <= {}",
            kind,
            global.sum_byte_length(),
            node.start_byte()
        );
        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            global.sum_byte_length(),
            &parent_indentation,
        );
        Acc {
            labeled: node.has_label(),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            metrics: Default::default(),
            ana: self.build_ana(&kind),
            mcc: Mcc::new(&kind),
            padding_start: global.sum_byte_length(),
            indentation: indent,
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            no_space: vec![],
            role: Default::default(),
            precomp_queries: Default::default(),
        }
    }

    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        global: &mut Self::Global,
        text: &[u8],
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let spacing = get_spacing(
            acc.padding_start,
            acc.start_byte,
            text,
            parent.indentation(),
        );
        if let Some(spacing) = spacing {
            parent.push(FullNode {
                global: global.into(),
                local: self.make_spacing(spacing),
            });
        }
        let label = if acc.labeled {
            std::str::from_utf8(&text[acc.start_byte..acc.end_byte])
                .ok()
                .map(|x| x.to_string())
        } else {
            None
        };
        self.make(global, acc, label)
    }
}

pub fn tree_sitter_parse(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_java::language();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(text, None).unwrap();
    if tree.root_node().has_error() {
        Err(tree)
    } else {
        Ok(tree)
    }
}

impl<'stores, 'cache, TS: JavaEnabledTypeStore> JavaTreeGen<'stores, 'cache, TS, ()> {
    pub fn new<'a, 'b>(
        stores: &'a mut SimpleStores<TS>,
        md_cache: &'b mut MDCache,
    ) -> JavaTreeGen<'a, 'b, TS, ()> {
        JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
            more: (),
        }
    }
}

impl<'stores, 'cache, TS, More> JavaTreeGen<'stores, 'cache, TS, More> {
    pub fn _generate_file<'b: 'stores>(
        &mut self,
        name: &[u8],
        text: &'b [u8],
        cursor: tree_sitter::TreeCursor,
    ) -> FullNode<BasicGlobalData, Local> {
        todo!("handle Type inconsistences")
    }
}
impl<
        'stores,
        'cache,
        TS: JavaEnabledTypeStore,
        More: for<'a, 'b> self::More<SimpleStores<TS, &'b legion::World, &'a LabelStore>>,
    > JavaTreeGen<'stores, 'cache, TS, More>
{
    fn make_spacing(
        &mut self,
        spacing: Vec<u8>,
    ) -> Local {
        let kind = Type::Spaces;
        let interned_kind = self.stores.type_store.intern(kind);
        debug_assert_eq!(kind, self.stores.type_store.resolve(interned_kind));
        let bytes_len = spacing.len();
        let spacing = std::str::from_utf8(&spacing).unwrap().to_string();
        let line_count = spacing
            .matches("\n")
            .count()
            .to_u16()
            .expect("too many newlines");
        let spacing_id = self.stores.label_store.get_or_insert(spacing.clone());
        let hbuilder: hashed::HashesBuilder<SyntaxNodeHashs<u32>> =
            hashed::HashesBuilder::new(Default::default(), &interned_kind, &spacing, 1);
        let hsyntax = hbuilder.most_discriminating();
        let hashable = &hsyntax;

        let eq = |x: EntryRef| {
            let t = x.get_component::<TS::Ty>();
            if t != Ok(&interned_kind) {
                return false;
            }
            let l = x.get_component::<LabelIdentifier>();
            if l != Ok(&spacing_id) {
                return false;
            }
            true
        };

        let insertion = self.stores.node_store.prepare_insertion(&hashable, eq);

        let mut hashs = hbuilder.build();
        hashs.structt = 0;
        hashs.label = 0;

        let compressed_node = if let Some(id) = insertion.occupied_id() {
            id
        } else {
            let vacant = insertion.vacant();
            let bytes_len = compo::BytesLen(bytes_len.try_into().unwrap());
            let a = (interned_kind, spacing_id, bytes_len, hashs, BloomSize::None);
            if line_count == 0 {
                NodeStore::insert_after_prepare(vacant, a)
            } else {
                let a = a.concat((compo::LineCount(line_count),));
                NodeStore::insert_after_prepare(vacant, a)
            }
        };
        Local {
            compressed_node,
            metrics: SubTreeMetrics {
                size: 1,
                height: 0,
                size_no_spaces: 0,
                hashs,
                line_count,
            },
            ana: Default::default(),
            mcc: Mcc::new(&Type::Spaces),
            role: None,
            precomp_queries: Default::default(),
        }
    }

    pub fn tree_sitter_parse(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_java::language();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(text, None).unwrap();
        if tree.root_node().has_error() {
            Err(tree)
        } else {
            Ok(tree)
        }
    }

    pub fn generate_file<'b: 'stores>(
        &mut self,
        name: &[u8],
        text: &'b [u8],
        cursor: tree_sitter::TreeCursor,
    ) -> FullNode<BasicGlobalData, Local> {
        let mut global = Global::from(TextedGlobalData::new(Default::default(), text));
        let mut init = self.init_val(text, &TNode(cursor.node()));
        let mut xx = TTreeCursor(cursor);

        let spacing = get_spacing(
            init.padding_start,
            init.start_byte,
            text,
            init.indentation(),
        );
        if let Some(spacing) = spacing {
            global.down();
            // init.start_byte = 0;
            global.set_sum_byte_length(init.start_byte);
            init.push(FullNode {
                global: global.into(),
                local: self.make_spacing(spacing),
            });
            global.right();
        }
        let mut stack = init.into();

        self.gen(text, &mut stack, &mut xx, &mut global);

        let mut acc = stack.finalize();

        if has_final_space(&0, global.sum_byte_length(), text) {
            let spacing = get_spacing(
                global.sum_byte_length(),
                text.len(),
                text,
                acc.indentation(),
            );
            if let Some(spacing) = spacing {
                global.right();
                acc.push(FullNode {
                    global: global.into(),
                    local: self.make_spacing(spacing),
                });
            }
        }
        let label = Some(std::str::from_utf8(name).unwrap().to_owned());
        let full_node = self.make(&mut global, acc, label);

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
        if !ANA {
            return None;
        }
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
}

impl<'stores, 'cache, TS, More> TreeGen for JavaTreeGen<'stores, 'cache, TS, More>
where
    TS: JavaEnabledTypeStore,
    More: for<'a, 'b> self::More<SimpleStores<TS, &'b legion::World, &'a LabelStore>>,
{
    type Acc = Acc;
    type Global = SpacedGlobalData<'stores>;
    fn make(
        &mut self,
        global: &mut <Self as TreeGen>::Global,
        mut acc: <Self as TreeGen>::Acc,
        label: Option<String>,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let interned_kind = JavaEnabledTypeStore::intern(&self.stores.type_store, acc.simple.kind);
        let own_line_count = label.as_ref().map_or(0, |l| {
            l.matches("\n").count().to_u16().expect("too many newlines")
        });
        let metrics = acc.metrics.finalize(&interned_kind, &label, own_line_count);

        let hashable = &metrics.hashs.most_discriminating();

        let label_id = label.as_ref().map(|label| {
            // Some notable type can contain very different labels,
            // they might benefit from a particular storing (like a blob storage, even using git's object database )
            // eg. acc.simple.kind == Type::Comment and acc.simple.kind.is_literal()
            self.stores.label_store.get_or_insert(label.as_str())
        });
        let eq = eq_node(&interned_kind, label_id.as_ref(), &acc.simple.children);

        let insertion = self.stores.node_store.prepare_insertion(hashable, eq);

        let local = if let Some(compressed_node) = insertion.occupied_id() {
            let md = self.md_cache.get(&compressed_node).unwrap();
            let ana = md.ana.clone();
            let metrics = md.metrics;
            let precomp_queries = md.precomp_queries;
            let mcc = md.mcc.clone();
            Local {
                compressed_node,
                metrics,
                ana,
                mcc,
                role: acc.role.current,
                precomp_queries,
            }
        } else {
            make_partial_ana(
                acc.simple.kind,
                &mut acc.ana,
                &label,
                &acc.simple.children,
                &mut self.stores.label_store,
                &insertion,
            );
            let metrics = metrics.map_hashs(|h| h.build());
            let byte_len = (acc.end_byte - acc.start_byte).try_into().unwrap();
            let bytes_len = compo::BytesLen(byte_len);
            let vacant = insertion.vacant();
            let node_store: &legion::World = vacant.1 .1;
            let stores = SimpleStores {
                type_store: self.stores.type_store.clone(),
                label_store: &self.stores.label_store,
                node_store,
            };
            acc.precomp_queries |= self
                .more
                .match_precomp_queries(&stores, &acc, label.as_deref());
            let children_is_empty = acc.simple.children.is_empty();

            let mut dyn_builder = dyn_builder::EntityBuilder::new();
            dyn_builder.add(bytes_len);

            let current_role = Option::take(&mut acc.role.current);
            acc.role.add_md(&mut dyn_builder);
            if Mcc::persist(&acc.simple.kind) {
                dyn_builder.add(acc.mcc.clone());
            }
            if More::ENABLED {
                add_md_precomp_queries(&mut dyn_builder, acc.precomp_queries);
            }
            add_md_ref_ana(&mut dyn_builder, children_is_empty, acc.ana.as_ref());
            let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
            hashs.persist(&mut dyn_builder);

            if acc.simple.children.len() != acc.no_space.len() {
                dyn_builder.add(compo::NoSpacesCS(acc.no_space.into_boxed_slice()));
            }
            acc.simple.add_primary(&mut dyn_builder, interned_kind, label_id);

            let compressed_node =
                NodeStore::insert_built_after_prepare(vacant, dyn_builder.build());

            // TODO see if possible to only keep md in md_cache, but would need a generational cache I think
            self.md_cache.insert(
                compressed_node,
                MD {
                    metrics: metrics.clone(),
                    ana: acc.ana.clone(),
                    mcc: acc.mcc.clone(),
                    precomp_queries: acc.precomp_queries.clone(),
                },
            );
            Local {
                compressed_node,
                metrics,
                ana: acc.ana,
                mcc: acc.mcc,
                role: current_role,
                precomp_queries: acc.precomp_queries,
            }
        };

        let full_node = FullNode {
            global: global.into(),
            local,
        };
        full_node
    }
}

pub fn add_md_precomp_queries(
    dyn_builder: &mut impl EntityBuilder,
    precomp_queries: PrecompQueries,
) {
    if precomp_queries > 0 {
        dyn_builder.add(compo::Precomp(precomp_queries));
    } else {
        dyn_builder.add(compo::PrecompFlag);
    }
}

pub fn add_md_ref_ana(
    dyn_builder: &mut impl EntityBuilder,
    children_is_empty: bool,
    ana: Option<&PartialAnalysis>,
) {
    if children_is_empty {
        dyn_builder.add(BloomSize::None);
    } else {
        macro_rules! bloom_aux {
            ( $t:ty ) => {{
                type B = $t;
                let it = ana.as_ref().unwrap().solver.iter_refs();
                let it = BulkHasher::<_, <B as BF<[u8]>>::S, <B as BF<[u8]>>::H>::from(it);
                let bloom = B::from(it);
                dyn_builder.add(B::SIZE);
                dyn_builder.add(bloom);
            }};
        }
        macro_rules! bloom {
            ( $t:ty ) => {{
                bloom_aux!(Bloom::<&'static [u8], $t>);
            }};
        }
        match ana.as_ref().map(|x| x.estimated_refs_count()).unwrap_or(0) {
            x if x > 2048 => {
                dyn_builder.add(BloomSize::Much);
            }
            x if x > 1024 => bloom!([u64; 64]),
            x if x > 512 => bloom!([u64; 32]),
            x if x > 256 => bloom!([u64; 16]),
            x if x > 150 => bloom!([u64; 8]),
            x if x > 100 => bloom!([u64; 4]),
            x if x > 30 => bloom!([u64; 2]),
            x if x > 15 => bloom!(u64),
            x if x > 8 => bloom!(u32),
            x if x > 0 => bloom!(u16),
            _ => {
                dyn_builder.add(BloomSize::None);
            } // TODO use the following after having tested the previous, already enough changes for now
              // 2048.. => {
              //     dyn_builder.add(BloomSize::Much);
              // }
              // 1024.. => bloom!([u64; 64]),
              // 512.. => bloom!([u64; 32]),
              // 256.. => bloom!([u64; 16]),
              // 150.. => bloom!([u64; 8]),
              // 100.. => bloom!([u64; 4]),
              // 32.. => bloom!([u64; 2]),
              // 16.. => bloom!(u64),
              // 8.. => bloom!(u32),
              // 1.. => bloom!(u16),
              // 0 => {
              //     dyn_builder.add(BloomSize::None);
              // }
        }
    }
}

fn make_partial_ana(
    kind: Type,
    ana: &mut Option<PartialAnalysis>,
    label: &Option<String>,
    children: &[legion::Entity],
    label_store: &mut LabelStore,
    insertion: &PendingInsert,
) {
    if !ANA {
        *ana = None;
        return;
    }
    *ana = partial_ana_extraction(kind, ana.take(), &label, children, label_store, insertion)
        .map(|ana| ana_resolve(kind, ana, label_store));
}

fn ana_resolve(kind: Type, ana: PartialAnalysis, label_store: &LabelStore) -> PartialAnalysis {
    if kind == Type::ClassBody
        || kind.is_type_declaration()
        || kind == Type::MethodDeclaration
        || kind == Type::ConstructorDeclaration
    {
        log::trace!("refs in {kind:?}");
        for x in ana.display_refs(label_store) {
            log::trace!("    {}", x);
        }
        log::trace!("decls in {kind:?}");
        for x in ana.display_decls(label_store) {
            log::trace!("    {}", x);
        }
        let ana = ana.resolve();
        log::trace!("refs in {kind:?} after resolution");

        for x in ana.display_refs(label_store) {
            log::trace!("    {}", x);
        }
        ana
    } else if kind == Type::Program {
        log::debug!("refs in {kind:?}");
        for x in ana.display_refs(label_store) {
            log::debug!("    {}", x);
        }
        log::debug!("decls in {kind:?}");
        for x in ana.display_decls(label_store) {
            log::debug!("    {}", x);
        }
        let ana = ana.resolve();
        log::debug!("refs in {kind:?} after resolve");
        for x in ana.display_refs(label_store) {
            log::debug!("    {}", x);
        }
        // TODO assert that ana.solver.refs does not contains mentions to ?.this
        ana
    } else {
        ana
    }
}

fn partial_ana_extraction(
    kind: Type,
    ana: Option<PartialAnalysis>,
    label: &Option<String>,
    children: &[legion::Entity],
    label_store: &mut LabelStore,
    insertion: &PendingInsert,
) -> Option<PartialAnalysis> {
    let is_possibly_empty = |kind| {
        kind == Type::ArgumentList
            || kind == Type::FormalParameters
            || kind == Type::AnnotationArgumentList
            || kind == Type::SwitchLabel
            || kind == Type::Modifiers
            || kind == Type::BreakStatement
            || kind == Type::ContinueStatement
            || kind == Type::Wildcard
            || kind == Type::ConstructorBody
            || kind == Type::InterfaceBody
            || kind == Type::SwitchBlock
            || kind == Type::ClassBody
            || kind == Type::EnumBody
            || kind == Type::ModuleBody
            || kind == Type::AnnotationTypeBody
            || kind == Type::TypeArguments
            || kind == Type::ArrayInitializer
            || kind == Type::ReturnStatement
            || kind == Type::ForStatement
            || kind == Type::RequiresModifier
            || kind == Type::ERROR
    };
    let mut make = |label| {
        Some(PartialAnalysis::init(&kind, label, |x| {
            label_store.get_or_insert(x)
        }))
    };
    if kind == Type::Program {
        ana
    } else if kind.is_comment() {
        None
    } else if let Some(label) = label.as_ref() {
        let label = if kind.is_literal() {
            kind.literal_type()
        } else {
            label.as_str()
        };
        make(Some(label))
    } else if kind.is_primitive() {
        let node = insertion.resolve::<TIdN<NodeIdentifier>>(children[0]);
        let ty = node.get_type();
        let label = ty.to_str();
        make(Some(label))
    } else if let Some(ana) = ana {
        // nothing to do, resolutions at the end of post ?
        Some(ana)
    } else if kind == Type::Static
        || kind == Type::Public
        || kind == Type::Asterisk
        || kind == Type::Dimensions
        || kind == Type::Block
        || kind == Type::ElementValueArrayInitializer
        || kind == Type::PackageDeclaration
        || kind == Type::TypeParameter
    {
        make(None)
    } else if is_possibly_empty(kind) {
        if kind == Type::ArgumentList
            || kind == Type::FormalParameters
            || kind == Type::AnnotationArgumentList
        {
            if !children
                .iter()
                .all(|x| !insertion.resolve::<TIdN<NodeIdentifier>>(*x).has_children())
            {
                // eg. an empty body/block/paramlist/...
                log::error!("{:?} should only contains leafs", &kind);
            }
            make(None)
        // } else if kind == Type::SwitchLabel || kind == Type::Modifiers {
        //     // TODO decls or refs ?
        //     None
        } else {
            None
        }
    } else {
        if !children.is_empty()
            && children
                .iter()
                .all(|x| !insertion.resolve::<TIdN<NodeIdentifier>>(*x).has_children())
        {
            // eg. an empty body/block/paramlist/...
            log::error!("{:?} should only contains leafs", kind);
        }
        None
    }
}

impl<'stores, 'cache, TS: JavaEnabledTypeStore, More> hyper_ast::types::NodeStore<NodeIdentifier>
    for JavaTreeGen<'stores, 'cache, TS, More>
{
    type R<'a>
        = HashedNodeRef<'a, NodeIdentifier>
    where
        Self: 'a,
        'stores: 'a;

    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        self.stores.node_store.resolve(*id)
    }
}

impl<
        'stores,
        'cache,
        TS: JavaEnabledTypeStore,
        More: for<'a, 'b> self::More<SimpleStores<TS, &'b legion::World, &'a LabelStore>>,
    > NodeStoreExt<HashedNode> for JavaTreeGen<'stores, 'cache, TS, More>
where
    TS::Ty: TypeTrait,
{
    #[allow(unused)]
    fn build_then_insert(
        &mut self,
        i: <HashedNode as hyper_ast::types::Stored>::TreeId,
        t: AnyType, //<HashedNode as types::Typed>::Type,
        l: Option<<HashedNode as types::Labeled>::Label>,
        cs: Vec<<HashedNode as types::Stored>::TreeId>,
    ) -> <HashedNode as types::Stored>::TreeId {
        todo!();
        // if t.is_spaces() {
        //     //     // TODO improve ergonomics
        //     //     // should ge spaces as label then reconstruct spaces and insert as done with every other nodes
        //     //     // WARN it wont work for new spaces (l parameter is not used, and label do not return spacing)
        //     let spacing = self
        //         .stores
        //         .label_store
        //         .resolve(&l.unwrap())
        //         .as_bytes()
        //         .to_vec();
        //     self.make_spacing(spacing);
        //     return i;
        // }
        let mut acc: Acc = {
            let kind = t;
            let kind = todo!();
            Acc {
                labeled: l.is_some(),
                start_byte: 0,
                end_byte: 0,
                metrics: Default::default(),
                ana: None,
                mcc: Mcc::new(&kind),
                padding_start: 0,
                indentation: vec![],
                simple: BasicAccumulator {
                    kind,
                    children: vec![],
                },
                no_space: vec![],
                role: Default::default(),
                precomp_queries: Default::default(),
            }
        };
        for c in cs {
            let local = {
                // print_tree_syntax(&self.stores.node_store, &self.stores.label_store, &c);
                // println!();
                let md = self.md_cache.get(&c);
                let (ana, metrics, mcc) = if let Some(md) = md {
                    let ana = md.ana.clone();
                    let metrics = md.metrics;
                    let mcc = md.mcc.clone();
                    (ana, metrics, mcc)
                } else {
                    let node: HashedNodeRef<_> = self.stores.node_store.resolve(c);
                    let hashs = SyntaxNodeHashs {
                        structt: WithHashs::hash(&node, &SyntaxNodeHashsKinds::Struct),
                        label: WithHashs::hash(&node, &SyntaxNodeHashsKinds::Label),
                        syntax: WithHashs::hash(&node, &SyntaxNodeHashsKinds::Syntax),
                    };
                    let kind: TS::Ty = todo!(); //node.get_type();
                    let metrics = SubTreeMetrics {
                        size: node.size().to_u32().unwrap(),
                        height: node.height().to_u32().unwrap(),
                        size_no_spaces: node.size_no_spaces().to_u32().unwrap(),
                        hashs,
                        line_count: node.line_count().to_u16().unwrap(),
                    };
                    let mcc = node
                        .get_component::<Mcc>()
                        .map_or(Mcc::new(&kind), |x| x.clone());
                    (None, metrics, mcc)
                };
                Local {
                    compressed_node: c,
                    metrics,
                    ana,
                    mcc,
                    role: acc.role.current,
                    precomp_queries: todo!(),
                }
            };
            let global = BasicGlobalData::default();
            let full_node = FullNode { global, local };
            acc.push(full_node);
        }
        let post = {
            let node_store = &mut self.stores.node_store;
            let label_store = &mut self.stores.label_store;
            let label_id = l;
            let label = label_id.map(|l| label_store.resolve(&l));

            let interned_kind =
                JavaEnabledTypeStore::intern(&self.stores.type_store, acc.simple.kind);
            let own_line_count = label.as_ref().map_or(0, |l| {
                l.matches("\n").count().to_u16().expect("too many newlines")
            });
            let metrics = acc.metrics.finalize(&interned_kind, &label, own_line_count);

            let hsyntax = metrics.hashs.most_discriminating();
            let hashable = &hsyntax;

            let eq = eq_node(&interned_kind, label_id.as_ref(), &acc.simple.children);

            let insertion = node_store.prepare_insertion(&hashable, eq);

            let local = if let Some(id) = insertion.occupied_id() {
                let md = self.md_cache.get(&id).unwrap();
                let ana = md.ana.clone();
                let metrics = md.metrics;
                let precomp_queries = md.precomp_queries;
                let mcc = md.mcc.clone();
                Local {
                    compressed_node: id,
                    metrics,
                    ana,
                    mcc,
                    role: acc.role.current,
                    precomp_queries,
                }
            } else {
                let metrics = metrics.map_hashs(|h| h.build());
                let bytes_len = compo::BytesLen((acc.end_byte - acc.start_byte) as u32);

                let vacant = insertion.vacant();
                let node_store: &legion::World = vacant.1 .1;
                let stores = SimpleStores {
                    type_store: self.stores.type_store.clone(),
                    node_store,
                    label_store: &self.stores.label_store,
                };

                acc.precomp_queries |= self.more.match_precomp_queries(&stores, &acc, label);
                let children_is_empty = acc.simple.children.is_empty();

                let mut dyn_builder = dyn_builder::EntityBuilder::new();
                dyn_builder.add(bytes_len);

                let current_role = Option::take(&mut acc.role.current);
                acc.role.add_md(&mut dyn_builder);
                if Mcc::persist(&acc.simple.kind) {
                    dyn_builder.add(acc.mcc.clone());
                }
                if let Some(label_id) = label_id {
                    dyn_builder.add(label_id);
                }
                if More::ENABLED {
                    add_md_precomp_queries(&mut dyn_builder, acc.precomp_queries);
                }
                add_md_ref_ana(&mut dyn_builder, children_is_empty, acc.ana.as_ref());
                let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
                hashs.persist(&mut dyn_builder);
                if !children_is_empty {
                    if acc.simple.children.len() != acc.no_space.len() {
                        dyn_builder.add(compo::NoSpacesCS(acc.no_space.into_boxed_slice()));
                    }
                }
                acc.simple.add_primary(&mut dyn_builder, interned_kind, label_id);
                let compressed_node =
                    NodeStore::insert_built_after_prepare(vacant, dyn_builder.build());

                // TODO see if possible to only keep md in md_cache, but would need a generational cache I think
                self.md_cache.insert(
                    compressed_node,
                    MD {
                        metrics: metrics.clone(),
                        ana: acc.ana.clone(),
                        mcc: acc.mcc.clone(),
                        precomp_queries: acc.precomp_queries.clone(),
                    },
                );
                Local {
                    compressed_node,
                    metrics,
                    ana: acc.ana,
                    mcc: acc.mcc,
                    role: current_role,
                    precomp_queries: todo!(),
                }
            };
            local
        };
        post.compressed_node
    }
}
