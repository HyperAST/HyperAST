use crate::types::{CppEnabledTypeStore, Type};
use crate::TNode;
use hyper_ast::store::nodes::legion::{dyn_builder, RawHAST};
use hyper_ast::tree_gen::{self, add_md_precomp_queries, RoleAcc, TotalBytesGlobalData as _};
use hyper_ast::types;
use hyper_ast::{
    filter::BloomSize,
    full::FullNode,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    nodes::Space,
    store::{
        nodes::{
            legion::{compo, eq_node, NodeIdentifier},
            DefaultNodeStore as NodeStore, EntityBuilder,
        },
        SimpleStores,
    },
    tree_gen::{
        compute_indentation, get_spacing, has_final_space,
        parser::{Node as _, TreeCursor, Visibility},
        AccIndentation, Accumulator, BasicAccumulator, BasicGlobalData, GlobalData, Parents,
        PreResult, SpacedGlobalData, Spaces, SubTreeMetrics, TextedGlobalData, TreeGen,
        WithByteRange, ZippedTreeGen,
    },
    types::{LabelStore as _, Role},
};
use legion::world::EntryRef;
use num::ToPrimitive as _;
///! fully compress all subtrees from a cpp CST
use std::{collections::HashMap, fmt::Debug, vec};

pub type LabelIdentifier = hyper_ast::store::labels::DefaultLabelIdentifier;

pub struct CppTreeGen<'store, 'cache, TS, More = ()> {
    pub line_break: Vec<u8>,
    pub stores: &'store mut SimpleStores<TS>,
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
    precomp_queries: PrecompQueries,
}

impl From<Local> for MD {
    fn from(x: Local) -> Self {
        MD {
            metrics: x.metrics,
            ana: x.ana,
            precomp_queries: x.precomp_queries,
        }
    }
}

pub type Global<'a> = SpacedGlobalData<'a>;

/// TODO temporary placeholder
#[derive(Debug, Clone, Default)]
pub struct PartialAnalysis {}

type PrecompQueries = u16;

#[derive(Debug, Clone)]
pub struct Local {
    pub compressed_node: NodeIdentifier,
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub ana: Option<PartialAnalysis>,
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

        // TODO things with this.ana
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
    padding_start: usize,
    indentation: Spaces,
    role: RoleAcc<crate::types::Role>,
    precomp_queries: PrecompQueries,
}

pub type FNode = FullNode<BasicGlobalData, Local>;
impl Accumulator for Acc {
    type Node = FNode;
    fn push(&mut self, full_node: Self::Node) {
        full_node.local.acc(self);
    }
}

impl AccIndentation for Acc {
    fn indentation<'a>(&'a self) -> &'a Spaces {
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

impl types::Typed for Acc {
    type Type = Type;

    fn get_type(&self) -> Self::Type {
        self.simple.kind
    }
}

impl hyper_ast::tree_gen::WithChildren<NodeIdentifier> for Acc {
    fn children(&self) -> &[NodeIdentifier] {
        &self.simple.children
    }
}

impl hyper_ast::tree_gen::WithRole<Role> for Acc {
    fn role_at(&self, o: usize) -> Option<Role> {
        self.role
            .offsets
            .iter()
            .position(|x| *x as usize == o)
            .and_then(|x| self.role.roles.get(x))
            .cloned()
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
            .field("padding_start", &self.padding_start)
            .field("indentation", &self.indentation)
            .finish()
    }
}

/// enables recovering of hidden nodes from tree-sitter
#[cfg(not(debug_assertions))]
const HIDDEN_NODES: bool = true;
#[cfg(debug_assertions)]
static HIDDEN_NODES: bool = true;

#[cfg(not(debug_assertions))]
const fn should_get_hidden_nodes() -> bool {
    HIDDEN_NODES
}
#[cfg(debug_assertions)]
pub(crate) fn should_get_hidden_nodes() -> bool {
    HIDDEN_NODES
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
#[allow(dead_code)] // NOTE: created by tree sitter
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

impl<'store, 'cache, TS, More> ZippedTreeGen for CppTreeGen<'store, 'cache, TS, More>
where
    TS: CppEnabledTypeStore,
    More: for<'a, 'b> tree_gen::More<RawHAST<'a, 'b, TS>, Acc>,
{
    type Stores = SimpleStores<TS>;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b>;

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn init_val(&mut self, text: &[u8], node: &Self::Node<'_>) -> Self::Acc {
        let kind = node.obtain_type();
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
        let node = cursor.node();
        let kind = node.obtain_type();
        if HIDDEN_NODES {
            if kind == Type::_FunctionDeclaratorSeq
                || kind == Type::ParameterListRepeat1
                || kind == Type::TranslationUnitRepeat1
                || kind == Type::_Declarator
                || kind.is_hidden()
                || kind.is_repeat()
            {
                return PreResult::Ignore;
            }
        }
        if node.0.is_missing() {
            return PreResult::Skip;
        }
        let mut acc = self.pre(text, &node, stack, global);
        // TODO replace with wrapper
        if !stack
            .parent()
            .map_or(false, |a| a.simple.kind.is_supertype())
        {
            if let Some(r) = cursor.0.field_name() {
                acc.role.current = r.try_into().ok();
            }
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
        let parent_indentation = &stack.parent().unwrap().indentation();
        let kind = node.obtain_type();
        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            global.sum_byte_length(),
            &parent_indentation,
        );
        // if global.sum_byte_length() < 400 {
        //     dbg!((kind,node.start_byte(),node.end_byte(),global.sum_byte_length(),indent.len()));
        // }
        Acc {
            labeled: node.has_label(),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            metrics: Default::default(),
            ana: self.build_ana(&kind),
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
            let local = self.make_spacing(spacing);
            // debug_assert_ne!(parent.simple.children.len(), 0, "{:?}", parent.simple);
            parent.push(FullNode {
                global: global.into(),
                local,
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

impl<'store, 'cache, TS> CppTreeGen<'store, 'cache, TS, ()> {
    pub fn new<'a, 'b>(
        stores: &'a mut SimpleStores<TS>,
        md_cache: &'b mut MDCache,
    ) -> CppTreeGen<'a, 'b, TS, ()> {
        CppTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
            more: (),
        }
    }
}

pub fn tree_sitter_parse(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    let language = crate::language();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(text, None).unwrap();
    if tree.root_node().has_error() {
        Err(tree)
    } else {
        Ok(tree)
    }
}

impl<'store, 'cache, TS, More> CppTreeGen<'store, 'cache, TS, More>
where
    TS: CppEnabledTypeStore,
    More: for<'a, 'b> tree_gen::More<RawHAST<'a, 'b, TS>, Acc>,
{
    fn make_spacing(
        &mut self,
        spacing: Vec<u8>, //Space>,
    ) -> Local {
        let kind = Type::Spaces;
        let interned_kind = TS::intern(kind);
        debug_assert_eq!(kind, TS::resolve(interned_kind));
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
            NodeStore::insert_after_prepare(
                vacant,
                (interned_kind, spacing_id, bytes_len, hashs, BloomSize::None),
            )
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
            role: None,
            precomp_queries: Default::default(),
        }
    }

    pub fn tree_sitter_parse(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        let language = crate::language();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(text, None).unwrap();
        if tree.root_node().has_error() {
            Err(tree)
        } else {
            Ok(tree)
        }
    }

    pub fn generate_file(
        &mut self,
        name: &[u8],
        text: &'store [u8],
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
        full_node
    }

    fn build_ana(&mut self, kind: &Type) -> Option<PartialAnalysis> {
        if kind == &Type::TranslationUnit {
            Some(PartialAnalysis {})
        } else {
            None
        }
    }
}

impl<'stores, 'cache, TS, More> TreeGen for CppTreeGen<'stores, 'cache, TS, More>
where
    TS: CppEnabledTypeStore,
    More: for<'a, 'b> tree_gen::More<RawHAST<'a, 'b, TS>, Acc>,
{
    type Acc = Acc;
    type Global = SpacedGlobalData<'stores>;
    fn make(
        &mut self,
        global: &mut <Self as TreeGen>::Global,
        mut acc: <Self as TreeGen>::Acc,
        label: Option<String>,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let kind = acc.simple.kind;
        let interned_kind = TS::intern(kind);
        let own_line_count = label.as_ref().map_or(0, |l| {
            l.matches("\n").count().to_u16().expect("too many newlines")
        });
        let metrics = acc.metrics.finalize(&interned_kind, &label, own_line_count);

        let hashable = &metrics.hashs.most_discriminating();

        let label_id = label
            .as_ref()
            .map(|label| self.stores.label_store.get_or_insert(label.as_str()));
        let eq = eq_node(&interned_kind, label_id.as_ref(), &acc.simple.children);

        let insertion = self.stores.node_store.prepare_insertion(hashable, eq);

        let local = if let Some(compressed_node) = insertion.occupied_id() {
            let md = self.md_cache.get(&compressed_node).unwrap();
            let ana = md.ana.clone();
            let metrics = md.metrics;
            let precomp_queries = md.precomp_queries;
            Local {
                compressed_node,
                metrics,
                ana,
                role: acc.role.current,
                precomp_queries,
            }
        } else {
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
            if More::ENABLED {
                add_md_precomp_queries(&mut dyn_builder, acc.precomp_queries);
            }

            let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
            hashs.persist(&mut dyn_builder);

            if acc.simple.children.len() != acc.no_space.len() {
                dyn_builder.add(compo::NoSpacesCS(acc.no_space.into_boxed_slice()));
            }
            acc.simple
                .add_primary(&mut dyn_builder, interned_kind, label_id);

            let compressed_node =
                NodeStore::insert_built_after_prepare(vacant, dyn_builder.build());

            self.md_cache.insert(
                compressed_node,
                MD {
                    metrics: metrics.clone(),
                    ana: acc.ana.clone(),
                    precomp_queries: acc.precomp_queries.clone(),
                },
            );
            Local {
                compressed_node,
                metrics,
                ana: acc.ana,
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

/// TODO partialana
impl PartialAnalysis {
    pub(crate) fn refs_count(&self) -> usize {
        0 //TODO
    }
    pub(crate) fn refs(&self) -> impl Iterator<Item = Vec<u8>> {
        vec![vec![0_u8]].into_iter() //TODO
    }
}
