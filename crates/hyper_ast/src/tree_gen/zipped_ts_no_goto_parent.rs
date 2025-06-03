#![allow(unused)]
use super::{P, parser::Visibility, utils_ts::*};
use crate::store::nodes::compo;
use crate::store::{
    SimpleStores,
    nodes::{
        DefaultNodeStore as NodeStore,
        legion::{NodeIdentifier, dyn_builder, eq_node},
    },
};
use crate::tree_gen::{
    self, Accumulator, BasicAccumulator, BasicGlobalData, GlobalData, Parents, PreResult,
    SpacedGlobalData, SubTreeMetrics, TextedGlobalData, TotalBytesGlobalData as _, WithByteRange,
    has_final_space,
    parser::{Node as _, TreeCursor},
};
use crate::{
    filter::BloomSize,
    full::FullNode,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    types::{HyperType, LabelStore as _},
};
use legion::world::EntryRef;
use num::ToPrimitive as _;

///! fully compress all subtrees from a cpp CST
use std::{collections::HashMap, fmt::Debug, str::from_utf8, vec};

pub type LabelIdentifier = crate::store::labels::DefaultLabelIdentifier;

pub struct TsTreeGen<'store, 'cache, TS, More = (), const HIDDEN_NODES: bool = false> {
    pub line_break: Vec<u8>,
    pub stores: &'store mut SimpleStores<TS>,
    pub md_cache: &'cache mut MDCache,
    pub more: More,
}

pub type MDCache = HashMap<NodeIdentifier, DD>;

pub struct DD {
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
}

impl<T> From<Local<T>> for DD {
    fn from(x: Local<T>) -> Self {
        DD { metrics: x.metrics }
    }
}

pub type Global<'a> = SpacedGlobalData<'a>;

#[derive(Debug, Clone)]
pub struct Local<T> {
    pub compressed_node: NodeIdentifier,

    // # debug
    pub _ty: T,

    // # directly bubbling derived data
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    // # by providing store could also fetch the ones not there
}

impl<T> Local<T> {
    fn acc(self, acc: &mut Acc<T>) {
        acc.simple.push(self.compressed_node);
        acc.metrics.acc(self.metrics);
    }
}

pub struct Acc<T> {
    // # primary
    simple: BasicAccumulator<T, NodeIdentifier>,
    labeled: bool,
    // size: u32,
    // hash: u32,
    // # debug
    _next: T,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>, // contains important hash
    // support
    start_byte: usize,
    end_byte: usize,
    padding_start: usize,
}

pub type FNode<T> = FullNode<BasicGlobalData, Local<T>>;
impl<T> Accumulator for Acc<T> {
    type Node = FNode<T>;
    fn push(&mut self, full_node: Self::Node) {
        full_node.local.acc(self);
    }
}

impl<T> WithByteRange for Acc<T> {
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

impl<T: Debug> Debug for Acc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Acc")
            .field("simple", &self.simple)
            .field("labeled", &self.labeled)
            .field("start_byte", &self.start_byte)
            .field("end_byte", &self.end_byte)
            .field("metrics", &self.metrics)
            .field("padding_start", &self.padding_start)
            .finish()
    }
}

impl<T> tree_gen::WithChildren<NodeIdentifier> for Acc<T> {
    fn children(&self) -> &[NodeIdentifier] {
        &self.simple.children
    }
}

impl<'acc, T> tree_gen::WithLabel for &'acc Acc<T> {
    type L = &'acc str;
}

impl<'store, 'cache, 's, TS: TsEnableTS>
    TsTreeGen<'store, 'cache, TS, tree_gen::NoOpMore<TS, Acc<TS::Ty2>>, true>
where
    TS::Ty2: TsType,
{
    pub fn new(stores: &'store mut SimpleStores<TS>, md_cache: &'cache mut MDCache) -> Self {
        Self {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
            more: Default::default(),
        }
    }
}

pub trait ZippedTreeGen: TreeGen
where
    Self::Global: tree_gen::TotalBytesGlobalData,
{
    // # results
    // type Node1;
    type Stores;
    // # source
    type Text: ?Sized;
    type Node<'a>: tree_gen::parser::Node;
    type TreeCursor<'a>: tree_gen::parser::TreeCursor<N = Self::Node<'a>>;

    fn init_val(&mut self, text: &Self::Text, node: &Self::Node<'_>) -> Self::Acc;

    fn pre_skippable(
        &mut self,
        text: &Self::Text,
        cursor: &Self::TreeCursor<'_>,
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> PreResult<<Self as TreeGen>::Acc> {
        PreResult::Ok(self.pre(text, &cursor.node(), stack, global))
    }

    fn pre(
        &mut self,
        text: &Self::Text,
        node: &Self::Node<'_>,
        stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> <Self as TreeGen>::Acc;

    fn acc(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        full_node: <<Self as TreeGen>::Acc as Accumulator>::Node,
    ) {
        parent.push(full_node);
    }

    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        global: &mut Self::Global,
        text: &Self::Text,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node;

    fn stores(&mut self) -> &mut Self::Stores;

    fn r#gen(
        &mut self,
        text: &Self::Text,
        stack: &mut Parents<Self::Acc>,
        cursor: &mut Self::TreeCursor<'_>,
        global: &mut Self::Global,
    );
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum Has {
    Down,
    Up,
    Right,
}

impl<'store, 'cache, TS, More, const HIDDEN_NODES: bool> ZippedTreeGen
    for TsTreeGen<'store, 'cache, TS, More, HIDDEN_NODES>
where
    TS: TsEnableTS,
    TS::Ty2: TsType,
    More: for<'t> tree_gen::More<SimpleStores<TS>, Acc = Acc<TS::Ty2>>,
{
    type Stores = SimpleStores<TS>;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b, HIDDEN_NODES>;

    fn r#gen(
        &mut self,
        text: &Self::Text,
        stack: &mut Parents<Self::Acc>,
        cursor: &mut Self::TreeCursor<'_>,
        global: &mut Self::Global,
    ) {
        let mut pre_post = PrePost::new(cursor);
        let mut stack0 = vec![(cursor.node().kind().to_string(), Visibility::Visible)];
        let mut indent = 0;
        println!("{}", cursor.node().kind());
        while let Some((has, cursor, vis)) = pre_post.next() {
            if has == Has::Up || has == Has::Right {
                // post
                let (t, vis) = stack0.pop().unwrap();
                if vis == Visibility::Hidden {
                    println!("-. {}", t);
                } else {
                    println!("-| {}", t);
                }
            }
            if has == Has::Down || has == Has::Right {
                // pre
                let n = cursor.node();
                let t = n.kind().to_string();
                if vis == Visibility::Hidden {
                    println!("+. {}", t);
                } else {
                    println!("+| {}", t);
                }
                stack0.push((t, vis));
                // println!("{}", n);
            }
            // println!(
            //     "{has:?} {} {:?}",
            //     cursor.node().kind(),
            //     cursor.node().start_byte()..cursor.node().end_byte(),
            // );
        }
        panic!();

        // let starting_stack_height = stack.len();
        let mut cursor_stack = vec![(*cursor).clone()];
        let mut has = Has::Down;
        loop {
            println!();
            for e in 0..cursor_stack.len() {
                let x = stack.0[e].as_ref().unwrap();
                println!(
                    "{:20} {:15} {:5} {:?} {}",
                    cursor_stack[e].node().kind(),
                    x.simple.kind,
                    x.simple.kind.is_hidden(),
                    x.start_byte..x.end_byte,
                    from_utf8(&text[x.start_byte..x.end_byte]).unwrap()
                );
            }
            println!();
            // assert_eq!(stack.len() - starting_stack_height, cursor_stack.len());
            let mut cursor = if let Some(cursor) = cursor_stack.last() {
                cursor.clone()
            } else {
                break;
            };
            dbg!(
                cursor.0.node().kind(),
                cursor.0.node().id(),
                cursor.0.depth()
            );
            dbg!((cursor_stack.len(), stack.len()));
            if let Some(visibility) = (has != Has::Up)
                .then(|| cursor.goto_first_child_extended())
                .flatten()
            {
                dbg!(cursor.node().kind());
                cursor_stack.push(cursor);
                let cursor = cursor_stack.last_mut().unwrap();
                has = Has::Down;
                self._pre(global, text, cursor, stack, &mut has, visibility);
            } else {
                if let Some(visibility) = cursor.goto_next_sibling_extended() {
                    dbg!(cursor.node().kind());
                    cursor_stack.pop().unwrap();
                    loop {
                        let c = cursor_stack.pop().unwrap();
                        if c.0.node().end_byte() < cursor.0.node().start_byte() {
                            self._post(stack, global, text);
                        } else {
                            cursor_stack.push(c);
                            break;
                        }
                    }
                    cursor_stack.push(cursor);
                    has = Has::Right;
                    let cursor = cursor_stack.last_mut().unwrap();
                    self._pre(global, text, cursor, stack, &mut has, visibility);
                    dbg!()
                } else {
                    loop {
                        let Some(c) = cursor_stack.pop() else {
                            return;
                        };
                        if c.0.node().end_byte() < cursor.0.node().start_byte() {
                        } else {
                            cursor_stack.push(c);
                            break;
                        }
                    }
                    dbg!();
                    has = Has::Up;
                    self._post(stack, global, text);
                }
            }
        }
    }

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn init_val(&mut self, _text: &[u8], node: &Self::Node<'_>) -> Self::Acc {
        let kind = TS::obtain_type(node);
        let labeled = node.has_label();
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            labeled,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            metrics: Default::default(),
            padding_start: 0,
            _next: TS::Ty2::spaces(),
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
        let Some(kind) = TS::try_obtain_type(&node) else {
            return PreResult::Skip;
        };
        if HIDDEN_NODES {
            // if kind.is_repeat() {
            //     return PreResult::Ignore;
            // }
        }
        if node.0.is_missing() {
            dbg!("missing");
            return PreResult::Skip;
        }
        let mut acc = self.pre(text, &node, stack, global);
        // TODO replace with wrapper
        if !stack
            .parent()
            .map_or(false, |a| a.simple.kind.is_supertype())
        {
            if let Some(r) = cursor.0.field_name() {
                if let Ok(r) = TryInto::<crate::types::Role>::try_into(r) {
                    log::warn!("not retrieving roles");
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
        _stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> <Self as TreeGen>::Acc {
        println!(
            "`{}`",
            from_utf8(&text[node.start_byte()..node.end_byte()]).unwrap()
        );
        dbg!(global.sum_byte_length()..node.start_byte());
        println!(
            "`{}`",
            from_utf8(&text[global.sum_byte_length()..node.start_byte()]).unwrap()
        );
        let kind = TS::obtain_type(node);
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            labeled: node.has_label(),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            metrics: Default::default(),
            padding_start: global.sum_byte_length(),
            _next: TS::Ty2::spaces(),
        }
    }

    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        global: &mut Self::Global,
        text: &[u8],
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let spacing = get_spacing(acc.padding_start, acc.start_byte, text);
        if let Some(spacing) = spacing {
            let local = self.make_spacing(spacing);
            debug_assert_ne!(parent.simple.children.len(), 0, "{:?}", parent.simple);
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

struct PrePost<'a, const HIDDEN_NODES: bool = true> {
    has: Has,
    stack: Vec<TTreeCursor<'a, HIDDEN_NODES>>,
}

impl<'a, const HIDDEN_NODES: bool> PrePost<'a, HIDDEN_NODES> {
    fn new(cursor: &TTreeCursor<'a, HIDDEN_NODES>) -> Self {
        Self {
            has: Has::Down,
            stack: vec![cursor.clone()],
        }
    }

    fn next(&mut self) -> Option<(Has, &TTreeCursor<'a, HIDDEN_NODES>, Visibility)> {
        // dbg!(self.stack.len());
        let Some(cursor) = self.stack.last() else {
            return None;
        };
        // dbg!(cursor.0.node().kind());
        // dbg!(cursor.0.node().start_byte()..cursor.0.node().end_byte());
        // dbg!(cursor.0.depth());
        let mut cursor = cursor.clone();
        if let Some(visibility) = (self.has != Has::Up)
            .then(|| cursor.goto_first_child_extended())
            .flatten()
        {
            self.stack.push(cursor);
            self.has = Has::Down;
            let cursor = self.stack.last_mut().unwrap();
            // dbg!(self.has, cursor.node().kind());
            // dbg!(cursor.0.node().start_byte()..cursor.0.node().end_byte());
            Some((self.has, cursor, visibility))
        } else {
            // dbg!(cursor.node().kind());
            if let Some(visibility) = cursor.goto_next_sibling_extended() {
                // dbg!(cursor.0.depth());
                let _ = self.stack.pop().unwrap();
                let c = self.stack.last_mut().unwrap();
                if c.node().end_byte() <= cursor.0.node().start_byte() {
                    let cursor = self.stack.last_mut().unwrap();
                    self.has = Has::Up;
                    // dbg!();
                    // dbg!(self.has, cursor.node().kind());
                    // dbg!(cursor.0.node().start_byte()..cursor.0.node().end_byte());
                    return Some((self.has, cursor, Visibility::Visible));
                }
                self.stack.push(cursor);
                let cursor = self.stack.last_mut().unwrap();
                self.has = Has::Right;
                // dbg!(self.has, cursor.node().kind());
                // dbg!(cursor.0.node().start_byte()..cursor.0.node().end_byte());
                Some((self.has, cursor, visibility))
            } else if let Some(c) = self.stack.pop() {
                if self.stack.is_empty() {
                    self.stack.push(c);
                    let cursor = self.stack.last_mut().unwrap();
                    Some((self.has, cursor, Visibility::Visible))
                } else {
                    let cursor = self.stack.last().unwrap();
                    if cursor.node().kind() == "class_specifier" {
                        dbg!();
                    }
                    self.has = Has::Up;
                    // dbg!(self.has, cursor.node().kind());
                    // dbg!(cursor.0.node().start_byte()..cursor.0.node().end_byte());
                    Some((self.has, cursor, Visibility::Visible))
                }
            } else {
                None
            }
        }
    }
}

impl<'store, 'cache, TS, More, const HIDDEN_NODES: bool>
    TsTreeGen<'store, 'cache, TS, More, HIDDEN_NODES>
where
    TS: TsEnableTS,
    TS::Ty2: TsType,
    More: for<'t> tree_gen::More<SimpleStores<TS>, Acc = Acc<TS::Ty2>>,
{
    fn make_spacing(&mut self, spacing: Vec<u8>) -> Local<TS::Ty2> {
        let kind = TS::Ty2::spaces();
        let interned_kind = TS::intern(kind);
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
            _ty: kind,
        }
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

        let spacing = get_spacing(init.padding_start, init.start_byte, text);
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
            let spacing = get_spacing(global.sum_byte_length(), text.len(), text);
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

    fn _pre(
        &mut self,
        global: &mut SpacedGlobalData<'store>,
        text: &[u8],
        cursor: &mut <Self as ZippedTreeGen>::TreeCursor<'_>,
        stack: &mut Parents<Acc<TS::Ty2>>,
        has: &mut Has,
        visibility: Visibility,
    ) {
        global.down();
        match self.pre_skippable(text, cursor, &stack, global) {
            PreResult::Skip => {
                stack.push(tree_gen::P::BothHidden);
                *has = Has::Up;
                global.up();
            }
            PreResult::Ignore => todo!(),
            PreResult::SkipChildren(_) => todo!(),
            PreResult::Ok(acc) => {
                global.set_sum_byte_length(acc.begin_byte());
                stack.push(tree_gen::P::Visible(acc))
            }
        }
    }

    fn _post(
        &mut self,
        stack: &mut Parents<Acc<TS::Ty2>>,
        global: &mut SpacedGlobalData<'store>,
        text: &[u8],
    ) {
        let acc = stack.pop().unwrap();
        let acc = match acc {
            P::ManualyHidden => todo!(),
            P::BothHidden => return,
            P::Hidden(_) => todo!(),
            P::Visible(acc) => acc,
        };
        dbg!(acc.simple.kind);
        global.set_sum_byte_length(acc.end_byte());
        global.up();
        let parent = stack.parent_mut().unwrap();
        let full_node = self.post(parent, global, text, acc);
        self.acc(parent, full_node);
    }
}

pub fn get_spacing(padding_start: usize, pos: usize, text: &[u8]) -> Option<Vec<u8>> {
    // TODO change debug assert to assert if you want to strictly enforce spaces, issues with other char leaking is often caused by "bad" grammar.
    if padding_start != pos {
        let spaces = &text[padding_start..pos];
        // let spaces = Space::format_indentation(spaces);
        let mut bslash = false;
        spaces.iter().for_each(|x| {
            if bslash && (*x == b'\n' || *x == b'\r') {
                bslash = false
            } else if *x == b'\\' {
                debug_assert!(!bslash);
                bslash = true
            } else {
                debug_assert!(
                    *x == b' ' || *x == b'\n' || *x == b'\t' || *x == b'\r',
                    "{} {} {:?}",
                    x,
                    padding_start,
                    std::str::from_utf8(&spaces).unwrap()
                )
            }
        });
        debug_assert!(
            !bslash,
            "{}",
            std::str::from_utf8(&&text[padding_start.saturating_sub(100)..pos + 50]).unwrap()
        );
        let spaces = spaces.to_vec();
        // let spaces = Space::replace_indentation(parent_indentation, &spaces);
        // TODO put back the relativisation later, can pose issues when computing len of a subtree (contextually if we make the optimisation)
        Some(spaces)
    } else {
        None
    }
}

pub trait TreeGen {
    /// Container holding data waiting to be added to the HyperAST
    /// Note: needs WithByteRange to handle hidden node properly, it allows to go back up without using the cursor. When Treesitter is "fixed" change that
    type Acc: Accumulator + WithByteRange;
    /// Container holding global data used during generation.
    ///
    /// Useful for transient data needed during generation,
    /// this way you avoid cluttering [TreeGen::Acc].
    ///
    /// WARN make sure it does not leaks contextual data in subtrees.
    type Global: GlobalData;
    fn make(
        &mut self,
        global: &mut Self::Global,
        acc: <Self as TreeGen>::Acc,
        label: Option<String>,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node;
}

impl<'stores, 'cache, TS, More, const HIDDEN_NODES: bool> TreeGen
    for TsTreeGen<'stores, 'cache, TS, More, HIDDEN_NODES>
where
    TS: TsEnableTS,
    TS::Ty2: TsType,
    More: for<'t> tree_gen::More<SimpleStores<TS>, Acc = Acc<TS::Ty2>>,
{
    type Acc = Acc<TS::Ty2>;
    type Global = SpacedGlobalData<'stores>;
    fn make(
        &mut self,
        global: &mut <Self as TreeGen>::Global,
        acc: <Self as TreeGen>::Acc,
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
            Local {
                compressed_node,
                metrics,
                _ty: kind,
            }
        } else {
            let metrics = metrics.map_hashs(|h| h.build());
            let byte_len = (acc.end_byte - acc.start_byte).try_into().unwrap();
            let bytes_len = compo::BytesLen(byte_len);
            let vacant = insertion.vacant();
            let mut dyn_builder = dyn_builder::EntityBuilder::new();

            let children_is_empty = acc.simple.children.is_empty();
            use crate::store::nodes::EntityBuilder;
            dyn_builder.add(bytes_len);

            let hashs = metrics.add_md_metrics(&mut dyn_builder, children_is_empty);
            hashs.persist(&mut dyn_builder);

            acc.simple
                .add_primary(&mut dyn_builder, interned_kind, label_id);

            let compressed_node =
                NodeStore::insert_built_after_prepare(vacant, dyn_builder.build());

            self.md_cache.insert(
                compressed_node,
                DD {
                    metrics: metrics.clone(),
                },
            );
            Local {
                compressed_node,
                metrics,
                _ty: kind,
            }
        };

        let full_node = FullNode {
            global: global.simple(),
            local,
        };
        full_node
    }
}
