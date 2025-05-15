///! fully compress all subtrees from a Java CST
use crate::types::JavaEnabledTypeStore;
use crate::{
    TNode,
    types::{TStore, Type},
};
// use bevy_ecs::world::EntryRef;
use hyperast::store::{
    defaults::LabelIdentifier,
    nodes::{
        EntityBuilder,
        bevy_ecs::{Component, HashedNodeRef, dyn_builder, eq_node, eq_space},
    },
};
use hyperast::tree_gen::utils_ts::TTreeCursor;
use hyperast::tree_gen::{
    self, Parents, PreResult, SubTreeMetrics, TreeGen, WithByteRange,
    parser::{Node, TreeCursor},
};
use hyperast::tree_gen::{
    GlobalData as _, StatsGlobalData, TextedGlobalData, TotalBytesGlobalData as _,
};
use hyperast::tree_gen::{NoOpMore, RoleAcc, add_md_precomp_queries};
use hyperast::{
    cyclomatic::Mcc,
    full::FullNode,
    hashed::{HashedNode, IndexingHashBuilder, MetaDataHashsBuilder},
    types::{self, AnyType, NodeStoreExt, Role, TypeTrait, WithHashs, WithStats},
};
use hyperast::{
    hashed::{self, SyntaxNodeHashs},
    nodes::Space,
    tree_gen::{
        AccIndentation, Accumulator, BasicAccumulator, Spaces, ZippedTreeGen, compute_indentation,
        get_spacing, has_final_space,
    },
    types::LabelStore as LabelStoreTrait,
};
use num::ToPrimitive;
use std::marker::PhantomData;
use std::{collections::HashMap, fmt::Debug, vec};

use bevy_ecs::NodeStore;
use hyperast::store::nodes::bevy_ecs;

pub type SimpleStores<TS> = hyperast::store::SimpleStores<TS, NodeStore>;

#[cfg(feature = "impact")]
mod reference_analysis;
// use reference_analysis::build_ana;
#[cfg(feature = "impact")]
pub use reference_analysis::add_md_ref_ana;

#[cfg(feature = "impact")]
pub use crate::impact::partial_analysis::PartialAnalysis;
#[cfg(not(feature = "impact"))]
#[derive(Debug, Clone)]
pub struct PartialAnalysis;
impl PartialAnalysis {
    pub fn init<F: FnMut(&str) -> LabelIdentifier>(
        kind: &Type,
        label: Option<&str>,
        mut intern_label: F,
    ) -> Self {
        Self
    }
}

// pub type EntryR<'a> = EntryRef<'a>;

pub type NodeIdentifier = hyperast::store::nodes::bevy_ecs::NodeIdentifier;

// pub struct HashedNodeRef<'a>(EntryRef<'a>);

pub type FNode = FullNode<StatsGlobalData, Local>;

// TODO try to use a const generic for spaceless generation ?
// SPC: consider spaces ie. add them to the HyperAST,
// NOTE there is a big issue with the byteLen of subtree then.
// just provide a view abstracting spaces (see attempt in hyper_diff)
pub struct JavaTreeGen<
    'stores,
    'cache,
    TS = TStore,
    S = SimpleStores<TS>,
    More = (),
    const HIDDEN_NODES: bool = true,
> {
    // TODO replace with Arc<[u8]>
    pub line_break: Vec<u8>,
    pub stores: &'stores mut S,
    pub md_cache: &'cache mut MDCache,
    pub more: More,
    pub _p: PhantomData<TS>,
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

// Enables static reference analysis
const ANA: bool = false;

impl From<Local> for MD {
    fn from(x: Local) -> Self {
        MD {
            metrics: x.metrics,
            precomp_queries: x.precomp_queries,
        }
    }
}

pub type Global<'a> = hyperast::tree_gen::SpacedGlobalData<'a, StatsGlobalData>;

type PrecompQueries = u16;

#[derive(Debug, Clone)]
pub struct Local {
    pub compressed_node: NodeIdentifier,
    // * metadata: computation results from concrete code of node and its children
    // they can be qualitative metadata, e.g. a hash or they can be quantitative e.g. lines of code
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub role: Option<Role>,
    pub precomp_queries: PrecompQueries,
}

impl Local {
    fn acc<Scope>(self, acc: &mut Acc<Scope>) {
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

pub struct Acc<Scope = hyperast::scripting::Acc> {
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
    prepro: Option<Scope>,
}

impl<Scope> Accumulator for Acc<Scope> {
    type Node = FNode;
    fn push(&mut self, full_node: Self::Node) {
        full_node.local.acc(self);
    }
}

impl<Scope> AccIndentation for Acc<Scope> {
    fn indentation(&self) -> &Spaces {
        &self.indentation
    }
}

impl<Scope> WithByteRange for Acc<Scope> {
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

/// Implements [ZippedTreeGen] to offer a visitor for Java generation
impl<'stores, 'cache, TS, More, const HIDDEN_NODES: bool> ZippedTreeGen
    for JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, More, HIDDEN_NODES>
where
    TS: JavaEnabledTypeStore + 'static + hyperast::types::RoleStore<Role = Role, IdF = u16>,
    TS::Ty: Component,
    More: tree_gen::Prepro<SimpleStores<TS>>
        + tree_gen::PreproTSG<SimpleStores<TS>, Acc = Acc<More::Scope>>,
{
    // type Node1 = SimpleNode1<NodeIdentifier, String>;
    type Stores = SimpleStores<TS>;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b, HIDDEN_NODES>;

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn init_val(&mut self, text: &[u8], node: &Self::Node<'_>) -> <Self as TreeGen>::Acc {
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
        let mcc = Mcc::new(&kind);
        let prepro = if More::USING {
            Some(self.more.preprocessing(kind).unwrap())
        } else {
            None
        };
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
            prepro,
        }
    }

    fn pre_skippable(
        &mut self,
        text: &Self::Text,
        cursor: &Self::TreeCursor<'_>,
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> PreResult<<Self as TreeGen>::Acc> {
        let node = &cursor.node();
        let kind = TS::obtain_type(node);
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
                if let Ok(r) = r.try_into() {
                    acc.role.current = Some(r);
                } else {
                    log::error!("cannot convert role: {}", r)
                }
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
        let parent_indentation = &stack.parent().unwrap().indentation();
        let kind = TS::obtain_type(node);
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
        let prepro = if More::USING {
            Some(self.more.preprocessing(kind).unwrap())
        } else {
            None
        };
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
            prepro,
        }
    }

    fn acc(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        full_node: <<Self as TreeGen>::Acc as Accumulator>::Node,
    ) {
        // let id = full_node.local.compressed_node;
        // let ty = parent.simple.kind;
        parent.push(full_node);
        if let Some(_p) = &mut parent.prepro {
            unimplemented!("for now use the legion builder if you need preprocessing scripts");
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
            let id = local.compressed_node;
            parent.push(FullNode {
                global: global.simple(),
                local,
            });

            if let Some(p) = &mut parent.prepro {
                unimplemented!("for now use the legion builder if you need preprocessing scripts");
            }
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
    hyperast::tree_gen::utils_ts::tree_sitter_parse(text, &crate::language())
}

impl<'stores, 'cache, TS: JavaEnabledTypeStore, X>
    JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, NoOpMore<X, Acc>, true>
{
    pub fn new(stores: &'stores mut SimpleStores<TS>, md_cache: &'cache mut MDCache) -> Self {
        Self {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
            more: Default::default(),
            _p: Default::default(),
        }
    }
}

impl<'stores, 'cache, 'acc, TS, More>
    JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, More, true>
{
    pub fn without_hidden_nodes(
        self,
    ) -> JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, More, false> {
        JavaTreeGen {
            line_break: self.line_break,
            stores: self.stores,
            md_cache: self.md_cache,
            more: self.more,
            _p: self._p,
        }
    }
}

impl<'stores, 'cache, 'acc, TS: JavaEnabledTypeStore + 'static, More, const HIDDEN_NODES: bool>
    JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, More, HIDDEN_NODES>
{
    pub fn _generate_file<'b: 'stores>(
        &mut self,
        name: &[u8],
        text: &'b [u8],
        cursor: tree_sitter::TreeCursor,
    ) -> FullNode<StatsGlobalData, Local>
    where
        More: tree_gen::Prepro<SimpleStores<TS>, Scope = hyperast::scripting::Acc>
            + tree_gen::PreproTSG<SimpleStores<TS>, Acc = Acc<More::Scope>>,
    {
        todo!()
    }
}

impl<'stores, 'cache, 'acc, TS: JavaEnabledTypeStore + 'static, More>
    JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, More, true>
{
    pub fn with_preprocessing(
        stores: &'stores mut SimpleStores<TS>,
        md_cache: &'cache mut MDCache,
        more: More,
    ) -> Self {
        Self {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
            more: more.into(),
            _p: Default::default(),
        }
    }
}

impl<'stores, 'cache, 'acc, TS: JavaEnabledTypeStore + 'static, More, const HIDDEN_NODES: bool>
    JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, More, HIDDEN_NODES>
{
    pub fn with_more<M>(
        self,
        more: M,
    ) -> JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, M, HIDDEN_NODES> {
        JavaTreeGen {
            line_break: self.line_break,
            stores: self.stores,
            md_cache: self.md_cache,
            more: more,
            _p: self._p,
        }
    }

    pub fn with_line_break(self, line_break: Vec<u8>) -> Self {
        JavaTreeGen {
            line_break: self.line_break,
            stores: self.stores,
            md_cache: self.md_cache,
            more: self.more,
            _p: self._p,
        }
    }
}

impl<'stores, 'cache, TS, More, const HIDDEN_NODES: bool>
    JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, More, HIDDEN_NODES>
where
    TS: JavaEnabledTypeStore<Ty2 = Type>
        + 'static
        + hyperast::types::RoleStore<Role = Role, IdF = u16>,
    TS::Ty: Component,
    More: tree_gen::Prepro<SimpleStores<TS>>
        + tree_gen::PreproTSG<SimpleStores<TS>, Acc = Acc<More::Scope>>,
{
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

        let eq = eq_space(&interned_kind, &spacing_id);
        // |x: bevy_ecs::entity::EntityRef| {
        //     let t = x.get_component::<TS::Ty>();
        //     if t != Ok(&interned_kind) {
        //         return false;
        //     }
        //     let l = x.get_component::<LabelIdentifier>();
        //     if l != Ok(&spacing_id) {
        //         return false;
        //     }
        //     true
        // };

        let insertion = self.stores.node_store.prepare_insertion(&hashable, eq);

        let mut hashs = hbuilder.build();
        hashs.structt = 0;
        hashs.label = 0;

        let compressed_node = if let Some(id) = insertion.occupied_id() {
            id
        } else {
            let mut vacant = insertion.vacant3();
            // let mut dyn_builder = dyn_builder::EntityBuilder::new();
            let dyn_builder = vacant.world();
            dyn_builder.insert(interned_kind);
            // dyn_builder.add(compo::BytesLen(bytes_len.try_into().unwrap()));
            dyn_builder.insert(spacing_id);
            dyn_builder.insert(hashs);
            // dyn_builder.insert(BloomSize::None);
            // if line_count != 0 {
            //     dyn_builder.add(compo::LineCount(line_count));
            // }
            if More::USING {
                todo!()
                // let prepro = self.more.preprocessing(Type::Spaces).unwrap();
                // let subtr = hyperast::scripting::Subtr(kind, &dyn_builder);
                // use hyperast::scripting::Finishable;
                // let ss = prepro
                //     .finish_with_label(self.more.scripts(), &subtr, &spacing)
                //     .unwrap();
                // dyn_builder.add(ss);
            };

            dyn_builder.id()
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
        text: &'stores [u8],
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
            let local = self.make_spacing(spacing);
            let id = local.compressed_node;
            init.push(FullNode {
                global: global.simple(),
                local,
            });
            if let Some(p) = &mut init.prepro {
                unimplemented!("for now use the legion builder if you need preprocessing scripts");
            }
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
                let local = self.make_spacing(spacing);
                let id = local.compressed_node;
                acc.push(FullNode {
                    global: global.simple(),
                    local,
                });
                if let Some(p) = &mut acc.prepro {
                    unimplemented!(
                        "for now use the legion builder if you need preprocessing scripts"
                    );
                }
            }
        }
        let label = Some(std::str::from_utf8(name).unwrap().to_owned());

        use hyperast::types::HyperType;
        if !acc.simple.kind.is_file() {
            log::warn!("ignoring parsing error at the root of the file");
            acc.simple.kind = Type::Program;
        }

        let full_node = self.make(&mut global, acc, label);

        #[cfg(feature = "impact")]
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
        #[cfg(feature = "impact")]
        {
            build_ana(kind, label_store)
        }
        #[cfg(not(feature = "impact"))]
        {
            None
        }
    }
}

impl<'stores, 'cache, TS, More, const HIDDEN_NODES: bool> TreeGen
    for JavaTreeGen<'stores, 'cache, TS, SimpleStores<TS>, More, HIDDEN_NODES>
where
    TS: JavaEnabledTypeStore + 'static + hyperast::types::RoleStore<Role = Role, IdF = u16>,
    TS::Ty: Component,
    More: tree_gen::Prepro<SimpleStores<TS>>
        + tree_gen::PreproTSG<SimpleStores<TS>, Acc = Acc<More::Scope>>,
{
    type Acc = Acc<More::Scope>;
    type Global = Global<'stores>;
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

        let label_id = label.as_ref().map(|label| {
            // Some notable type can contain very different labels,
            // they might benefit from a particular storing (like a blob storage, even using git's object database )
            // eg. acc.simple.kind == Type::Comment and acc.simple.kind.is_literal()
            self.stores.label_store.get_or_insert(label.as_str())
        });
        let eq = eq_node(&interned_kind, label_id.as_ref(), &acc.simple.children);

        #[cfg(feature = "subtree-stats")]
        self.stores
            .node_store
            .inner
            .stats()
            .add_height_non_dedup(metrics.height);
        // &metrics.hashs.structt,

        let insertion = self.stores.node_store.prepare_insertion(hashable, eq);

        let local = if let Some(compressed_node) = insertion.occupied_id() {
            let md = self.md_cache.get(&compressed_node).unwrap();
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
            // let bytes_len = compo::BytesLen(byte_len);
            let mut vacant = insertion.vacant3();
            // let node_store: &_ = vacant;
            // let stores = hyperast::store::SimpleStores {
            //     type_store: self.stores.type_store.clone(),
            //     label_store: &self.stores.label_store,
            //     node_store,
            // };
            if More::ENABLED {
                todo!()
                // acc.precomp_queries |=
                //     self.more
                //         .match_precomp_queries(stores, &acc, label.as_deref());
            }
            let children_is_empty = acc.simple.children.is_empty();

            let mut dyn_builder = vacant.world();
            // let mut dyn_builder = dyn_builder::EntityBuilder::new();

            if More::ENABLED {
                todo!()
                // tree_gen::add_md_precomp_queries(&mut dyn_builder, acc.precomp_queries);
            }
            if More::GRAPHING {
                todo!()
                // self.more
                //     .compute_tsg(stores, &acc, label.as_deref())
                //     .unwrap();
            }

            let current_role = Option::take(&mut acc.role.current);
            // acc.role.add_md(&mut dyn_builder);

            // #[cfg(feature = "subtree-stats")]
            // vacant
            //     .1
            //     .1
            //     .stats()
            //     .add_height_dedup(metrics.height, metrics.hashs);
            let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
            hashs.persist(&mut dyn_builder);

            if acc.simple.children.len() != acc.no_space.len() {
                let children = acc.no_space;
                // tree_gen::add_cs_no_spaces(&mut dyn_builder, children);
            }

            acc.simple
                .add_primary(&mut dyn_builder, interned_kind, label_id);

            if More::USING {
                todo!()
                // let subtr = hyperast::scripting::Subtr(kind, &*dyn_builder);
                // use hyperast::scripting::Finishable;
                // let ss = if let Some(label) = &label {
                //     acc.prepro
                //         .unwrap()
                //         .finish_with_label(self.more.scripts(), &subtr, label)
                //         .unwrap()
                // } else {
                //     acc.prepro
                //         .unwrap()
                //         .finish(self.more.scripts(), &subtr)
                //         .unwrap()
                // };
                // dyn_builder.add(ss);
            }

            let compressed_node = dyn_builder.id();
            // let compressed_node =
            //     NodeStore::insert_built_after_prepare(vacant, dyn_builder.build());

            // TODO see if possible to only keep md in md_cache, but would need a generational cache I think
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
