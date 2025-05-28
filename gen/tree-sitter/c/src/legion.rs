use crate::TNode;
use crate::types::{CEnabledTypeStore, Type};
use hyperast::store::nodes::compo;
use hyperast::store::nodes::legion::dyn_builder;
use hyperast::tree_gen::utils_ts::TTreeCursor;
use hyperast::tree_gen::{
    self, NoOpMore, RoleAcc, TotalBytesGlobalData as _, add_md_precomp_queries,
};
use hyperast::tree_gen::{
    AccIndentation, Accumulator, BasicAccumulator, BasicGlobalData, GlobalData, Parents, PreResult,
    SpacedGlobalData, Spaces, SubTreeMetrics, TextedGlobalData, TreeGen, WithByteRange,
    ZippedTreeGen, compute_indentation, get_spacing, has_final_space,
    parser::{Node as _, TreeCursor},
};
use hyperast::types;
use hyperast::{
    filter::BloomSize,
    full::FullNode,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    nodes::Space,
    store::{
        SimpleStores,
        nodes::{
            DefaultNodeStore as NodeStore, EntityBuilder,
            legion::{NodeIdentifier, eq_node},
        },
    },
    types::{LabelStore as _, Role},
};
use legion::world::EntryRef;
use num::ToPrimitive as _;
///! fully compress all subtrees from a cpp CST
use std::{collections::HashMap, fmt::Debug, vec};

pub type LabelIdentifier = hyperast::store::labels::DefaultLabelIdentifier;

/// HIDDEN_NODES: enables recovering of hidden nodes from tree-sitter.
///   You should start without filtering out hidden nodes when intergrating/updating a grammar,
///   filtering hidden nodes adds complexity, thus might cause additional bugs
pub struct CTreeGen<'store, 'cache, TS, More = (), const HIDDEN_NODES: bool = true> {
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
    precomp_queries: PrecompQueries,
}

impl From<Local> for MD {
    fn from(x: Local) -> Self {
        MD {
            metrics: x.metrics,
            precomp_queries: x.precomp_queries,
        }
    }
}

pub type Global<'a> = SpacedGlobalData<'a>;

type PrecompQueries = u16;

#[derive(Debug, Clone)]
pub struct Local {
    pub compressed_node: NodeIdentifier,
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
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
    }
}

pub struct Acc {
    simple: BasicAccumulator<Type, NodeIdentifier>,
    no_space: Vec<NodeIdentifier>,
    labeled: bool,
    start_byte: usize,
    end_byte: usize,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
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

impl hyperast::tree_gen::WithChildren<NodeIdentifier> for Acc {
    fn children(&self) -> &[NodeIdentifier] {
        &self.simple.children
    }
}

impl hyperast::tree_gen::WithRole<Role> for Acc {
    fn role_at(&self, o: usize) -> Option<Role> {
        self.role
            .offsets
            .iter()
            .position(|x| *x as usize == o)
            .and_then(|x| self.role.roles.get(x))
            .cloned()
    }
}

impl<'acc> hyperast::tree_gen::WithLabel for &'acc Acc {
    type L = &'acc str;
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
            .field("padding_start", &self.padding_start)
            .field("indentation", &self.indentation)
            .finish()
    }
}

impl<'store, 'cache, TS, More, const HIDDEN_NODES: bool> ZippedTreeGen
    for CTreeGen<'store, 'cache, TS, More, HIDDEN_NODES>
where
    TS: CEnabledTypeStore<Ty2 = Type>,
    More: tree_gen::Prepro<SimpleStores<TS>> + tree_gen::More<SimpleStores<TS>, Acc = Acc>,
{
    type Stores = SimpleStores<TS>;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b, HIDDEN_NODES>;

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn init_val(&mut self, text: &[u8], node: &Self::Node<'_>) -> Self::Acc {
        let kind = TS::obtain_type(node);
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
        let kind = TS::obtain_type(&node);
        if HIDDEN_NODES {
            if kind.is_repeat() {
                return PreResult::Ignore;
            } else if kind.is_hidden() {
                return PreResult::Ignore;
            }
        }
        if node.0.is_missing() {
            // must skip missing nodes, i.e., leafs added by tree-sitter to fix CST,
            // needed to avoid breaking invarient, as the node has no span:
            // `is_parent_hidden && parent.end_byte() <= acc.begin_byte()`
            return PreResult::Skip;
        }
        let mut acc = self.pre(text, &node, stack, global);
        // TODO replace with wrapper
        if !stack
            .parent()
            .map_or(false, |a| a.simple.kind.is_supertype())
        {
            if let Some(r) = cursor.0.field_name() {
                if let Ok(r) = r.try_into() {
                    acc.role.current = Some(r);
                } else {
                    log::error!("cannot convert role: {}", r)
                }
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
        let kind = TS::obtain_type(node);
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
            parent.push(FullNode {
                global: global.simple(),
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

impl<'store, 'cache, TS: CEnabledTypeStore> CTreeGen<'store, 'cache, TS, NoOpMore<TS, Acc>, true> {
    pub fn new(stores: &'store mut SimpleStores<TS>, md_cache: &'cache mut MDCache) -> Self {
        Self {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
            more: Default::default(),
        }
    }
}

impl<'store, 'cache, 'acc, TS, More> CTreeGen<'store, 'cache, TS, More, true> {
    pub fn without_hidden_nodes(self) -> CTreeGen<'store, 'cache, TS, More, false> {
        CTreeGen {
            line_break: self.line_break,
            stores: self.stores,
            md_cache: self.md_cache,
            more: self.more,
        }
    }
}

pub fn tree_sitter_parse(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
    hyperast::tree_gen::utils_ts::tree_sitter_parse(text, &crate::language())
}

impl<'store, 'cache, TS, More, const HIDDEN_NODES: bool>
    CTreeGen<'store, 'cache, TS, More, HIDDEN_NODES>
where
    TS: CEnabledTypeStore<Ty2 = Type>,
    More: tree_gen::Prepro<SimpleStores<TS>> + tree_gen::More<SimpleStores<TS>, Acc = Acc>,
{
    pub fn with_more<M>(self, more: M) -> CTreeGen<'store, 'cache, TS, M, HIDDEN_NODES> {
        CTreeGen {
            line_break: self.line_break,
            stores: self.stores,
            md_cache: self.md_cache,
            more,
        }
    }
    fn make_spacing(&mut self, spacing: Vec<u8>) -> Local {
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
            role: None,
            precomp_queries: Default::default(),
        }
    }

    pub fn tree_sitter_parse(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
        tree_sitter_parse(text)
    }

    pub fn generate_file(
        &mut self,
        name: &[u8],
        text: &'store [u8],
        cursor: tree_sitter::TreeCursor,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
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
                global: global.simple(),
                local: self.make_spacing(spacing),
            });
            global.right();
        }
        let mut stack = init.into();

        self.r#gen(text, &mut stack, &mut xx, &mut global);

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
                    global: global.simple(),
                    local: self.make_spacing(spacing),
                });
            }
        }
        let label = Some(std::str::from_utf8(name).unwrap().to_owned());
        let full_node = self.make(&mut global, acc, label);
        full_node
    }
}

impl<'store, 'cache, TS, More, const HIDDEN_NODES: bool> TreeGen
    for CTreeGen<'store, 'cache, TS, More, HIDDEN_NODES>
where
    TS: CEnabledTypeStore<Ty2 = Type>,
    More: tree_gen::Prepro<SimpleStores<TS>> + tree_gen::More<SimpleStores<TS>, Acc = Acc>,
    TS::Ty2: hyperast::tree_gen::utils_ts::TsType,
{
    type Acc = Acc;
    type Global = SpacedGlobalData<'store>;
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
            debug_assert_eq!(metrics.height, md.metrics.height);
            debug_assert_eq!(metrics.size, md.metrics.size);
            debug_assert_eq!(metrics.size_no_spaces, md.metrics.size_no_spaces);
            debug_assert_eq!(metrics.line_count, md.metrics.line_count);
            debug_assert_eq!(metrics.hashs.build(), md.metrics.hashs);
            let metrics = md.metrics;
            let precomp_queries = md.precomp_queries;
            Local {
                compressed_node,
                metrics,
                role: acc.role.current,
                precomp_queries,
            }
        } else {
            let metrics = metrics.map_hashs(|h| h.build());
            let byte_len = (acc.end_byte - acc.start_byte).try_into().unwrap();
            let bytes_len = compo::BytesLen(byte_len);
            let vacant = insertion.vacant();
            let node_store: &_ = vacant.1.1;
            let stores = SimpleStores {
                type_store: self.stores.type_store.clone(),
                label_store: &self.stores.label_store,
                node_store,
            };
            acc.precomp_queries |= self
                .more
                .match_precomp_queries(stores, &acc, label.as_deref());
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
                let children = acc.no_space;
                tree_gen::add_cs_no_spaces(&mut dyn_builder, children);
            }
            acc.simple
                .add_primary(&mut dyn_builder, interned_kind, label_id);

            let compressed_node =
                NodeStore::insert_built_after_prepare(vacant, dyn_builder.build());

            self.md_cache.insert(
                compressed_node,
                MD {
                    metrics: metrics.clone(),
                    precomp_queries: acc.precomp_queries.clone(),
                },
            );
            Local {
                compressed_node,
                metrics,
                role: current_role,
                precomp_queries: acc.precomp_queries,
            }
        };

        let full_node = FullNode {
            global: global.simple(),
            local,
        };
        full_node
    }
}
