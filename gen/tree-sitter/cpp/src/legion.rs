///! fully compress all subtrees from a cpp CST
use std::{collections::HashMap, fmt::Debug, vec};

use crate::{types::TIdN, TNode};
use legion::world::EntryRef;
use num::ToPrimitive as _;
use tuples::CombinConcat;

use hyper_ast::{
    filter::BloomSize,
    full::FullNode,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    nodes::Space,
    store::{
        nodes::{
            legion::{
                compo::{self, NoSpacesCS, CS},
                eq_node, HashedNodeRef, NodeIdentifier, PendingInsert,
            },
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
    types::LabelStore as _,
};

use crate::types::{CppEnabledTypeStore, Type};

pub type LabelIdentifier = hyper_ast::store::labels::DefaultLabelIdentifier;

pub struct CppTreeGen<'store, 'cache, TS> {
    pub line_break: Vec<u8>,
    pub stores: &'store mut SimpleStores<TS>,
    pub md_cache: &'cache mut MDCache,
}

pub type MDCache = HashMap<NodeIdentifier, MD>;

// NOTE only keep compute intensive metadata (where space/time tradeoff is worth storing)
// eg. decls refs, maybe hashes but not size and height
// * metadata: computation results from concrete code of node and its children
// they can be qualitative metadata .eg a hash or they can be quantitative .eg lines of code
pub struct MD {
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: Option<PartialAnalysis>,
}

impl From<Local> for MD {
    fn from(x: Local) -> Self {
        MD {
            metrics: x.metrics,
            ana: x.ana,
        }
    }
}

pub type Global<'a> = SpacedGlobalData<'a>;

/// TODO temporary placeholder
#[derive(Debug, Clone, Default)]
pub struct PartialAnalysis {}

#[derive(Debug, Clone)]
pub struct Local {
    pub compressed_node: NodeIdentifier,
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub ana: Option<PartialAnalysis>,
}

impl Local {
    fn acc(self, acc: &mut Acc) {
        if self.metrics.size_no_spaces > 0 {
            acc.no_space.push(self.compressed_node)
        }
        acc.simple.push(self.compressed_node);
        acc.metrics.acc(self.metrics);

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
    fn begin_byte(&self) -> usize {
        self.start_byte
    }

    fn has_children(&self) -> bool {
        !self.simple.children.is_empty()
    }

    fn end_byte(&self) -> usize {
        self.end_byte
    }
}

/// enables recovering of hidden nodes from tree-sitter
#[cfg(not(debug_assertions))]
const HIDDEN_NODES: bool = true;
#[cfg(debug_assertions)]
static HIDDEN_NODES: bool = true;

#[repr(transparent)]
pub struct TTreeCursor<'a>(tree_sitter::TreeCursor<'a>);

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

impl<'a> Debug for TTreeCursor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TTreeCursor")
            .field(&self.0.node().kind())
            .finish()
    }
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
        if HIDDEN_NODES {
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
        if HIDDEN_NODES {
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

impl<'store, 'cache, TS: CppEnabledTypeStore> ZippedTreeGen for CppTreeGen<'store, 'cache, TS> {
    type Stores = SimpleStores<TS>;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b>;

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn init_val(&mut self, text: &[u8], node: &Self::Node<'_>) -> Self::Acc {
        let type_store = &mut self.stores().type_store;
        let kind = node.obtain_type(type_store); //type_store.get_cpp(node.kind());
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
        let node = cursor.node();
        let kind = node.obtain_type(type_store);
        if HIDDEN_NODES {
            if kind == Type::_ExpressionNotBinary
                || kind == Type::_FunctionDeclaratorSeq
                || kind == Type::ParameterListRepeat1
                || kind == Type::TranslationUnitRepeat1
                || kind.is_repeat()
            {
                return PreResult::Ignore;
            }
        }
        if node.0.is_missing() {
            return PreResult::Skip;
        }
        let acc = self.pre(text, &node, stack, global);
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
        // let kind = node.kind();
        // let kind = type_store.get_cpp(kind);
        let kind = node.obtain_type(type_store);
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
            debug_assert_ne!(parent.simple.children.len(), 0);
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

impl<'store, 'cache, TS: CppEnabledTypeStore> CppTreeGen<'store, 'cache, TS> {
    fn make_spacing(
        &mut self,
        spacing: Vec<u8>, //Space>,
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
            NodeStore::insert_after_prepare(
                vacant,
                (interned_kind, spacing_id, bytes_len, hashs, BloomSize::None),
            )
        };
        Local {
            compressed_node,
            metrics: SubTreeMetrics {
                size: 1,
                height: 1,
                hashs,
                size_no_spaces: 0,
                line_count,
            },
            ana: Default::default(),
        }
    }

    pub fn new(
        stores: &'store mut <Self as ZippedTreeGen>::Stores,
        md_cache: &'cache mut MDCache,
    ) -> CppTreeGen<'store, 'cache, TS> {
        CppTreeGen::<'store, 'cache, TS> {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
        }
    }

    pub fn tree_sitter_parse(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_cpp::language();
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
            init.start_byte = 0;
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

impl<'stores, 'cache, TS: CppEnabledTypeStore> TreeGen for CppTreeGen<'stores, 'cache, TS> {
    type Acc = Acc;
    type Global = SpacedGlobalData<'stores>;
    fn make(
        &mut self,
        global: &mut <Self as TreeGen>::Global,
        acc: <Self as TreeGen>::Acc,
        label: Option<String>,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let node_store = &mut self.stores.node_store;
        let label_store = &mut self.stores.label_store;
        let interned_kind = CppEnabledTypeStore::intern(&self.stores.type_store, acc.simple.kind);
        let own_line_count = label.as_ref().map_or(0, |l| {
            l.matches("\n").count().to_u16().expect("too many newlines")
        });
        let line_count = acc.metrics.line_count + own_line_count;
        let hashs = acc.metrics.hashs;
        let size = acc.metrics.size + 1;
        let height = acc.metrics.height + 1;
        let size_no_spaces = acc.metrics.size_no_spaces + 1;
        let hbuilder = hashed::HashesBuilder::new(hashs, &interned_kind, &label, size_no_spaces);
        let hsyntax = hbuilder.most_discriminating();
        let hashable = &hsyntax;

        let label_id = label
            .as_ref()
            .map(|label| label_store.get_or_insert(label.as_str()));
        let eq = eq_node(&interned_kind, label_id.as_ref(), &acc.simple.children);

        let insertion = node_store.prepare_insertion(&hashable, eq);

        let local = if let Some(compressed_node) = insertion.occupied_id() {
            let ana = None;
            let hashs = hbuilder.build();
            let metrics = SubTreeMetrics {
                size,
                height,
                hashs,
                size_no_spaces,
                line_count,
            };
            Local {
                compressed_node,
                metrics,
                ana,
            }
        } else {
            let ana = None;
            let hashs = hbuilder.build();
            let bytes_len = compo::BytesLen((acc.end_byte - acc.start_byte).try_into().unwrap());
            let compressed_node = if false {
                let base = (interned_kind, hashs, bytes_len);
                compress(
                    label_id,
                    &ana,
                    acc.simple,
                    acc.no_space,
                    size,
                    height,
                    size_no_spaces,
                    insertion,
                    base,
                )
            } else {
                // NOTE use of dyn_builder
                // TODO make it available through cargo feature or runtime config
                // - should most likely not change the behavior of the HyperAST, need tests
                // TODO make an even better API
                // - wrapping the builder,
                // - modularising computation and storage of metadata,
                // - checking some invariants when adding metadata,
                // - checking some invariants for indentifying data on debug builds
                // - tying up parts of accumulator (hyper_ast::tree_genBasicAccumulator) and builder (EntityBuilder).
                let mut dyn_builder =
                    hyper_ast::store::nodes::legion::dyn_builder::EntityBuilder::new();
                dyn_builder.add(interned_kind);
                dyn_builder.add(hashs.clone());
                dyn_builder.add(compo::BytesLen(
                    (acc.end_byte - acc.start_byte).try_into().unwrap(),
                ));

                if let Some(label_id) = label_id {
                    dyn_builder.add(label_id);
                }

                match acc.simple.children.len() {
                    0 => {
                        // dyn_builder.add(BloomSize::None);
                    }
                    x => {
                        let a = acc.simple.children.into_boxed_slice();
                        dyn_builder.add(compo::Size(size));
                        dyn_builder.add(compo::SizeNoSpaces(size_no_spaces));
                        dyn_builder.add(compo::Height(height));
                        dyn_builder.add(CS(a));
                        if x != acc.no_space.len() {
                            dyn_builder.add(NoSpacesCS(acc.no_space.into_boxed_slice()));
                        }
                    }
                }

                NodeStore::insert_built_after_prepare(insertion.vacant(), dyn_builder.build())
            };

            let metrics = SubTreeMetrics {
                size,
                height,
                hashs,
                size_no_spaces,
                line_count,
            };
            Local {
                compressed_node,
                metrics,
                ana,
            }
        };

        let full_node = FullNode {
            global: global.into(),
            local,
        };
        full_node
    }
}

fn compress<T: 'static + std::marker::Send + std::marker::Sync>(
    label_id: Option<LabelIdentifier>,
    _ana: &Option<PartialAnalysis>,
    simple: BasicAccumulator<Type, NodeIdentifier>,
    no_space: Vec<NodeIdentifier>,
    // bytes_len: compo::BytesLen,
    size: u32,
    height: u32,
    size_no_spaces: u32,
    insertion: PendingInsert,
    // hashs: SyntaxNodeHashs<u32,
    base: (T, SyntaxNodeHashs<u32>, compo::BytesLen),
) -> legion::Entity {
    let vacant = insertion.vacant();
    // let base = (CppEnabledTypeStore::intern_cpp(s,simple.kind), hashs, bytes_len);
    macro_rules! insert {
        ( $c0:expr, $($c:expr),* $(,)? ) => {{
            let c = $c0;
            $(
                let c = c.concat($c);
            )*
            NodeStore::insert_after_prepare(vacant, c)
        }};
    }
    macro_rules! children_dipatch {
        ( $c0:expr, $($c:expr),* $(,)? ) => {{
            let c = $c0;
            $(
                let c = c.concat($c);
            )*
            match simple.children.len() {
                0 => {
                    assert_eq!(1, size);
                    assert_eq!(1, height);
                    insert!(
                        c,
                        (BloomSize::None,)
                    )
                }
                x => {
                    let a = simple.children.into_boxed_slice();
                    let c = c.concat((compo::Size(size), compo::SizeNoSpaces(size_no_spaces), compo::Height(height), ));
                    let c = c.concat((CS(a),));
                    if x == no_space.len() {
                        insert!(c,)
                    } else {
                        let b = no_space.into_boxed_slice();
                        insert!(c, (NoSpacesCS(b),))
                    }
                }
            }}
        };
    }
    match (label_id, 0) {
        (None, _) => children_dipatch!(base,),
        (Some(label), _) => children_dipatch!(base, (label,),),
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
