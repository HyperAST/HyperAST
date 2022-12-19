///! fully compress all subtrees from a Java CST
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    io::stdout,
    vec,
};

use hyper_ast::{
    cyclomatic::Mcc,
    full::FullNode,
    hashed::{HashedNode, IndexingHashBuilder, MetaDataHashsBuilder},
    nodes::IoOut,
    store::{
        labels::LabelStore,
        nodes::legion::{HashedNodeRef, NoSpacesCS, PendingInsert},
    },
    tree_gen::{
        BasicGlobalData, GlobalData, SpacedGlobalData, SubTreeMetrics, TextedGlobalData, TreeGen,
    },
    types::{self, NodeStoreExt, WithHashs, WithStats},
    utils::{self},
};
use legion::world::EntryRef;
use num::ToPrimitive;
use string_interner::DefaultSymbol;
use tuples::CombinConcat;

use hyper_ast::{
    filter::BF,
    filter::{Bloom, BloomSize},
    hashed::{self, SyntaxNodeHashs, SyntaxNodeHashsKinds},
    nodes::{self, Space},
    store::{
        nodes::legion::{compo, CS},
        nodes::DefaultNodeStore as NodeStore,
        SimpleStores,
    },
    tree_gen::parser::Node as _,
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, AccIndentation, Accumulator,
        BasicAccumulator, Spaces, ZippedTreeGen,
    },
    types::{
        LabelStore as LabelStoreTrait,
        Tree,
        // NodeStore as NodeStoreTrait,
        Type,
        Typed,
    },
};
// use hyper_ast::nodes::SimpleNode1;

use crate::impact::partial_analysis::PartialAnalysis;

pub use crate::impact::element::BulkHasher;

pub fn hash32<T: ?Sized + Hash>(t: &T) -> u32 {
    utils::clamp_u64_to_u32(&utils::hash(t))
}

pub type EntryR<'a> = EntryRef<'a>;

pub type NodeIdentifier = legion::Entity;

// pub struct HashedNodeRef<'a>(EntryRef<'a>);

pub type FNode = FullNode<BasicGlobalData, Local>;

pub type LabelIdentifier = DefaultSymbol;

// TODO try to use a const generic for space less generation ?
// SPC: consider spaces ie. add them to the HyperAST,
// NOTE there is a big issue with the byteLen of subtree then.
pub struct JavaTreeGen<'stores, 'cache> {
    pub line_break: Vec<u8>,
    pub stores: &'stores mut SimpleStores,
    pub md_cache: &'cache mut MDCache,
}

pub type MDCache = HashMap<NodeIdentifier, MD>;

// TODO only keep compute intensive metadata (where space/time tradeoff is worth storing)
// eg. decls refs, maybe hashes but not size and height
// * metadata: computation results from concrete code of node and its children
// they can be qualitative metadata .eg a hash or they can be quantitative .eg lines of code
pub struct MD {
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: Option<PartialAnalysis>,
    mcc: Mcc,
}

impl From<Local> for MD {
    fn from(x: Local) -> Self {
        MD {
            metrics: x.metrics,
            ana: x.ana,
            mcc: x.mcc,
        }
    }
}

pub type Global<'a> = SpacedGlobalData<'a>;

#[derive(Debug, Clone)]
pub struct Local {
    pub compressed_node: NodeIdentifier,
    // * metadata: computation results from concrete code of node and its children
    // they can be qualitative metadata .eg a hash or they can be quantitative .eg lines of code
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    pub ana: Option<PartialAnalysis>,
    pub mcc: Mcc,
}

impl Local {
    fn acc(self, acc: &mut Acc) {
        if self.metrics.size_no_spaces > 0 {
            acc.no_space.push(self.compressed_node)
        }
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
        self.0.child(i).map(TNode)
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

impl<'stores, 'cache> ZippedTreeGen for JavaTreeGen<'stores, 'cache> {
    // type Node1 = SimpleNode1<NodeIdentifier, String>;
    type Stores = SimpleStores;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b>;

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn init_val(&mut self, text: &[u8], node: &Self::Node<'_>) -> <Self as TreeGen>::Acc {
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
        }
    }

    fn pre(
        &mut self,
        text: &[u8],
        node: &Self::Node<'_>,
        stack: &Vec<Self::Acc>,
        global: &mut Self::Global,
    ) -> <Self as TreeGen>::Acc {
        let type_store = &mut self.stores().type_store;
        let parent_indentation = &stack.last().unwrap().indentation();
        let kind = node.kind();
        let kind = type_store.get(kind);
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
impl<'stores, 'cache> JavaTreeGen<'stores, 'cache> {
    fn make_spacing(
        &mut self,
        spacing: Vec<u8>, //Space>,
    ) -> Local {
        let bytes_len = spacing.len();
        let spacing = std::str::from_utf8(&spacing).unwrap().to_string();
        let spacing_id = self.stores.label_store.get_or_insert(spacing.clone());
        let hbuilder: hashed::Builder<SyntaxNodeHashs<u32>> =
            hashed::Builder::new(Default::default(), &Type::Spaces, &spacing, 1);
        let hsyntax = hbuilder.most_discriminating();
        let hashable = &hsyntax;

        let eq = |x: EntryRef| {
            let t = x.get_component::<Type>();
            if t != Ok(&Type::Spaces) {
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
                (Type::Spaces, spacing_id, bytes_len, hashs, BloomSize::None),
            )
        };
        Local {
            compressed_node,
            metrics: SubTreeMetrics {
                size: 1,
                height: 0,
                size_no_spaces: 0,
                hashs,
            },
            ana: Default::default(),
            mcc: Mcc::new(&Type::Spaces),
        }
    }

    pub fn new<'a, 'b>(
        stores: &'a mut SimpleStores,
        md_cache: &'b mut MDCache,
    ) -> JavaTreeGen<'a, 'b> {
        JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
        }
    }

    pub fn tree_sitter_parse(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_java::language();
        parser.set_language(language).unwrap();
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
            init.start_byte = 0;
            init.push(FullNode {
                global: global.into(),
                local: self.make_spacing(spacing),
            });
            global.right();
        }
        let mut stack = vec![init];

        self.gen(text, &mut stack, &mut xx, &mut global);

        let mut acc = stack.pop().unwrap();

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

pub fn eq_node<'a>(
    kind: &'a Type,
    label_id: Option<&'a LabelIdentifier>,
    children: &'a [NodeIdentifier],
) -> impl Fn(EntryRef) -> bool + 'a {
    move |x: EntryRef| {
        let t = x.get_component::<Type>();
        if t != Ok(kind) {
            return false;
        }
        let l = x.get_component::<LabelIdentifier>().ok();
        if l != label_id {
            return false;
        } else {
            let cs = x.get_component::<CS<legion::Entity>>();
            let r = match cs {
                Ok(CS(cs)) => cs.as_ref() == children,
                Err(_) => children.is_empty(),
            };
            if !r {
                return false;
            }
        }
        true
    }
}

impl<'stores, 'cache> TreeGen for JavaTreeGen<'stores, 'cache> {
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
        let hashs = acc.metrics.hashs;
        let size = acc.metrics.size + 1;
        let height = acc.metrics.height + 1;
        let size_no_spaces = acc.metrics.size_no_spaces + 1;
        let hbuilder = hashed::Builder::new(hashs, &acc.simple.kind, &label, size_no_spaces);
        let hsyntax = hbuilder.most_discriminating();
        let hashable = &hsyntax;

        let label_id = label.as_ref().map(|label| {
            // Some notable type can contain very different labels,
            // they might benefit from a particular storing (like a blob storage, even using git's object database )
            // eg. acc.simple.kind == Type::Comment and acc.simple.kind.is_literal()
            label_store.get_or_insert(label.as_str())
        });
        let eq = eq_node(&acc.simple.kind, label_id.as_ref(), &acc.simple.children);

        let insertion = node_store.prepare_insertion(&hashable, eq);

        let local = if let Some(compressed_node) = insertion.occupied_id() {
            let md = self.md_cache.get(&compressed_node).unwrap();
            let ana = md.ana.clone();
            let metrics = md.metrics;
            let mcc = md.mcc.clone();
            Local {
                compressed_node,
                metrics,
                ana,
                mcc,
            }
        } else {
            let ana = make_partial_ana(
                acc.simple.kind,
                acc.ana,
                label,
                &acc.simple.children,
                label_store,
                &insertion,
            );
            let hashs = hbuilder.build();
            let bytes_len = compo::BytesLen((acc.end_byte - acc.start_byte).try_into().unwrap());
            let mcc = acc.mcc;
            let compressed_node = compress(
                label_id,
                &ana,
                acc.simple,
                acc.no_space,
                bytes_len,
                size,
                height,
                size_no_spaces,
                insertion,
                hashs,
                mcc.clone(),
            );

            let metrics = SubTreeMetrics {
                size,
                height,
                size_no_spaces,
                hashs,
            };

            // TODO see if possible to only keep md in md_cache, but would need a generational cache I think
            self.md_cache.insert(
                compressed_node,
                MD {
                    metrics: metrics.clone(),
                    ana: ana.clone(),
                    mcc: mcc.clone(),
                },
            );
            Local {
                compressed_node,
                metrics,
                ana,
                mcc,
            }
        };

        let full_node = FullNode {
            global: global.into(),
            local,
        };
        full_node
    }
}

fn compress(
    label_id: Option<LabelIdentifier>,
    ana: &Option<PartialAnalysis>,
    simple: BasicAccumulator<Type, NodeIdentifier>,
    no_space: Vec<NodeIdentifier>,
    bytes_len: compo::BytesLen,
    size: u32,
    height: u32,
    size_no_spaces: u32,
    insertion: PendingInsert,
    hashs: SyntaxNodeHashs<u32>,
    mcc: Mcc,
) -> legion::Entity {
    let vacant = insertion.vacant();
    macro_rules! insert {
        ( $c0:expr, $($c:expr),* $(,)? ) => {{
            let c = $c0;
            $(
                let c = c.concat($c);
            )*
            NodeStore::insert_after_prepare(vacant, c)
        }};
    }
    // NOTE needed as macro because I only implemented BulkHasher and Bloom for u8 and u16
    macro_rules! bloom {
        ( $t:ty ) => {{
            type B = $t;
            let it = ana.as_ref().unwrap().solver.iter_refs();
            let it = BulkHasher::<_, <B as BF<[u8]>>::S, <B as BF<[u8]>>::H>::from(it);
            let bloom = B::from(it);
            (B::SIZE, bloom)
        }};
    }
    macro_rules! bloom_dipatch {
        ( $($c:expr),+ $(,)? ) => {
            match ana.as_ref().map(|x| x.estimated_refs_count()).unwrap_or(0) {
                x if x > 2048 => {
                    insert!($($c),+, (BloomSize::Much,),)
                }
                x if x > 1024 => {
                    insert!($($c),+, bloom!(Bloom::<&'static [u8], [u64; 64]>))
                }
                x if x > 512 => {
                    insert!($($c),+, bloom!(Bloom::<&'static [u8], [u64; 32]>))
                }
                x if x > 256 => {
                    insert!($($c),+, bloom!(Bloom::<&'static [u8], [u64; 16]>))
                }
                x if x > 150 => {
                    insert!($($c),+, bloom!(Bloom::<&'static [u8], [u64; 8]>))
                }
                x if x > 100 => {
                    insert!($($c),+, bloom!(Bloom::<&'static [u8], [u64; 4]>))
                }
                x if x > 30 => {
                    insert!($($c),+, bloom!(Bloom::<&'static [u8], [u64; 2]>))
                }
                x if x > 15 => {
                    insert!($($c),+, bloom!(Bloom::<&'static [u8], u64>))
                }
                x if x > 8 => {
                    insert!($($c),+, bloom!(Bloom::<&'static [u8], u32>))
                }
                x if x > 0 => {
                    insert!($($c),+, bloom!(Bloom::<&'static [u8], u16>))
                }
                _ => insert!($($c),+, (BloomSize::None,)),
            }
        };
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
                    insert!(c, (BloomSize::None,))
                }
                // TODO try to reduce indirections
                // might need more data inline in child pointer to be worth the added contruction cost
                // might also benefit from using more data to choose between inlining childs or not
                // // WARN if you dont want to use the inlining you can comment then change children accessors
                // 1 => {
                //     let a = simple.children;
                //     bloom_dipatch!(
                //         $($c),+,
                //         (compo::Size(size), compo::Height(height),),
                //         (CSStaticCount(1), CS0([a[0]]),)
                //     )
                // }
                // 2 => {
                //     let a = simple.children;
                //     bloom_dipatch!(
                //         $($c),+,
                //         (compo::Size(size), compo::Height(height),),
                //         (CSStaticCount(2), CS0([a[0],a[1]]),)
                //     )
                // }
                // 3 => {
                //     let a = simple.children;
                //     let c = c.concat((compo::Size(size), compo::Height(height),));
                //     let c = c.concat((CSStaticCount(3), CS0([a[0],a[1],a[2]]),));
                //     bloom_dipatch!(
                //         c,
                //     )
                // }
                x => {
                    let a = simple.children.into_boxed_slice();
                    let c = c.concat((compo::Size(size), compo::SizeNoSpaces(size_no_spaces), compo::Height(height),));
                    let c = c.concat((CS(a),));
                    if x == no_space.len() {
                        bloom_dipatch!(c)
                    } else {
                        let b = no_space.into_boxed_slice();
                        bloom_dipatch!(c, (NoSpacesCS(b),))
                    }
                }
            }}
        };
    }
    let base = (simple.kind.clone(), hashs, bytes_len);
    match (label_id, mcc) {
        (None, mcc) if Mcc::persist(&simple.kind) => children_dipatch!(base, (mcc,),),
        (None, _) => children_dipatch!(base,),
        (Some(label), mcc) if Mcc::persist(&simple.kind) => children_dipatch!(base, (label, mcc,),),
        (Some(label), _) => children_dipatch!(base, (label,),),
    }
}

fn make_partial_ana(
    kind: Type,
    ana: Option<PartialAnalysis>,
    label: Option<String>,
    children: &[legion::Entity],
    label_store: &mut LabelStore,
    insertion: &PendingInsert,
) -> Option<PartialAnalysis> {
    ana.and_then(|ana| partial_ana_extraction(kind, ana, label, children, label_store, insertion))
        .map(|ana| ana_resolve(kind, ana, label_store))
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
    ana: PartialAnalysis,
    label: Option<String>,
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
            || kind == Type::Error
    };
    let mut make = |label| {
        Some(PartialAnalysis::init(&kind, label, |x| {
            label_store.get_or_insert(x)
        }))
    };
    if kind == Type::Program {
        Some(ana)
    } else if kind == Type::Comment {
        None
    } else if let Some(label) = label.as_ref() {
        let label = if kind.is_literal() {
            kind.literal_type()
        } else {
            label.as_str()
        };
        make(Some(label))
    } else if kind.is_primitive() {
        let node = insertion.resolve(children[0]);
        let label = node.get_type().to_string();
        make(Some(label.as_str()))
    } else if kind == Type::TS86
        || kind == Type::TS81
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
                .all(|x| !insertion.resolve(*x).has_children())
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
                .all(|x| !insertion.resolve(*x).has_children())
        {
            // eg. an empty body/block/paramlist/...
            log::error!("{:?} should only contains leafs", kind);
        }
        None
    }
}

impl<'stores, 'cache> hyper_ast::types::NodeStore<NodeIdentifier> for JavaTreeGen<'stores, 'cache> {
    type R<'a> = HashedNodeRef<'a> where Self: 'a, 'stores:'a;

    fn resolve(&self, id: &NodeIdentifier) -> Self::R<'_> {
        self.stores.node_store.resolve(*id)
    }
}

impl<'stores, 'cache> NodeStoreExt<HashedNode> for JavaTreeGen<'stores, 'cache> {
    fn build_then_insert(
        &mut self,
        i: <HashedNode as hyper_ast::types::Stored>::TreeId,
        t: <HashedNode as types::Typed>::Type,
        l: Option<<HashedNode as types::Labeled>::Label>,
        cs: Vec<<HashedNode as types::Stored>::TreeId>,
    ) -> <HashedNode as types::Stored>::TreeId {
        if t == Type::Spaces {
            //     // TODO improve ergonomics
            //     // should ge spaces as label then reconstruct spaces and insert as done with every other nodes
            //     // WARN it wont work for new spaces (l parameter is not used, and label do not return spacing)
            let spacing = self
                .stores
                .label_store
                .resolve(&l.unwrap())
                .as_bytes()
                .to_vec();
            self.make_spacing(spacing);
            return i;
        }
        let mut acc: Acc = {
            let kind = t;
            Acc {
                labeled: l.is_some(),
                start_byte: 0,
                end_byte: 0,
                metrics: Default::default(),
                ana: None,
                mcc: Mcc::new(&t),
                padding_start: 0,
                indentation: vec![],
                simple: BasicAccumulator {
                    kind,
                    children: vec![],
                },
                no_space: vec![],
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
                    let node = self.stores.node_store.resolve(c);
                    let hashs = SyntaxNodeHashs {
                        structt: WithHashs::hash(&node, &SyntaxNodeHashsKinds::Struct),
                        label: WithHashs::hash(&node, &SyntaxNodeHashsKinds::Label),
                        syntax: WithHashs::hash(&node, &SyntaxNodeHashsKinds::Syntax),
                    };
                    let kind = node.get_type();
                    let metrics = SubTreeMetrics {
                        size: node.size().to_u32().unwrap(),
                        height: node.height().to_u32().unwrap(),
                        size_no_spaces: node.size_no_spaces().to_u32().unwrap(),
                        hashs,
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
                }
            };
            let global = BasicGlobalData::default();
            let full_node = FullNode { global, local };
            acc.push(full_node);
        }
        let post = {
            let node_store = &mut self.stores.node_store;
            let label_store = &mut self.stores.label_store;

            let hashs = acc.metrics.hashs;
            let size = acc.metrics.size + 1;
            let height = acc.metrics.height + 1;
            let size_no_spaces = acc.metrics.size_no_spaces + 1;
            let label = l.map(|l| label_store.resolve(&l));
            let hbuilder = hashed::Builder::new(hashs, &acc.simple.kind, &label, size_no_spaces);
            let hsyntax = hbuilder.most_discriminating();
            let hashable = &hsyntax;

            let label_id = l;
            let eq = eq_node(&acc.simple.kind, label_id.as_ref(), &acc.simple.children);

            let insertion = node_store.prepare_insertion(&hashable, eq);

            let local = if let Some(id) = insertion.occupied_id() {
                let md = self.md_cache.get(&id).unwrap();
                let ana = md.ana.clone();
                let metrics = md.metrics;
                let mcc = md.mcc.clone();
                Local {
                    compressed_node: id,
                    metrics,
                    ana,
                    mcc,
                }
            } else {
                let ana = None;
                let hashs = hbuilder.build();
                let bytes_len = compo::BytesLen((acc.end_byte - acc.start_byte) as u32);

                let mcc = Mcc::new(&acc.simple.kind);

                let compressed_node = compress(
                    label_id,
                    &ana,
                    acc.simple,
                    acc.no_space,
                    bytes_len,
                    size,
                    height,
                    size_no_spaces,
                    insertion,
                    hashs,
                    mcc.clone(),
                );

                let metrics = SubTreeMetrics {
                    size,
                    height,
                    size_no_spaces,
                    hashs,
                };

                // TODO see if possible to only keep md in md_cache, but would need a generational cache I think
                self.md_cache.insert(
                    compressed_node,
                    MD {
                        metrics: metrics.clone(),
                        ana: ana.clone(),
                        mcc: mcc.clone(),
                    },
                );
                Local {
                    compressed_node,
                    metrics,
                    ana,
                    mcc,
                }
            };
            local
        };
        post.compressed_node
    }
}

pub fn print_tree_ids(node_store: &NodeStore, id: &NodeIdentifier) {
    nodes::print_tree_ids(
        |id| -> _ {
            node_store
                .resolve(id.clone())
                .into_compressed_node()
                .unwrap()
        },
        id,
    )
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

pub fn print_tree_syntax_with_ids(
    node_store: &NodeStore,
    label_store: &LabelStore,
    id: &NodeIdentifier,
) {
    nodes::print_tree_syntax_with_ids(
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

pub fn json_serialize<W: std::fmt::Write, const SPC: bool>(
    node_store: &NodeStore,
    label_store: &LabelStore,
    id: &NodeIdentifier,
    out: &mut W,
    parent_indent: &str,
) -> Option<String> {
    nodes::json_serialize::<_, _, _, _, _, _, SPC>(
        |id| -> _ { node_store.resolve(id.clone()) },
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

pub struct TreeJsonSerializer<'a, IdN, NS, LS, const SPC: bool = true> {
    node_store: &'a NS,
    label_store: &'a LS,
    id: IdN,
}
impl<'a, IdN, NS, LS, const SPC: bool> TreeJsonSerializer<'a, IdN, NS, LS, SPC> {
    pub fn new(node_store: &'a NS, label_store: &'a LS, id: IdN) -> Self {
        Self {
            node_store,
            label_store,
            id,
        }
    }
}
impl<'a, IdN, NS, LS, const SPC: bool> Display for TreeJsonSerializer<'a, IdN, NS, LS, SPC>
where
    NS: hyper_ast::types::NodeStore<IdN>,
    <NS as hyper_ast::types::NodeStore<IdN>>::R<'a>:
        hyper_ast::types::Tree<TreeId = IdN, Type = Type, Label = LS::I>,
    LS: hyper_ast::types::LabelStore<String>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let id = &self.id;
        nodes::json_serialize::<_, _, _, _, _, _, SPC>(
            |id| -> _ { self.node_store.resolve(id.clone()) },
            |id| -> _ { self.label_store.resolve(id).to_owned() },
            id,
            f,
            "\n",
        );
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
