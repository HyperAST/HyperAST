///! fully compress all subtrees from an Xml CST
use std::{fmt::Debug, vec};

use legion::world::EntryRef;
use tuples::CombinConcat;

use hyper_ast::{
    filter::BF,
    filter::{Bloom, BloomSize},
    full::FullNode,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, NodeHashs, SyntaxNodeHashs},
    // impact::{element::RefsEnum, elements::*, partial_analysis::PartialAnalysis},
    nodes::{self, Space},
    store::{labels::LabelStore, SimpleStores},
    store::{
        nodes::legion::{HashedNodeRef, NodeIdentifier, CS},
        nodes::DefaultNodeStore as NodeStore,
        TypeStore,
    },
    tree_gen::{
        compute_indentation,
        get_spacing,
        has_final_space,
        parser::{Node as _, TreeCursor as _},
        try_compute_indentation,
        // label_for_cursor,
        AccIndentation,
        Accumulator,
        BasicAccumulator,
        BasicGlobalData,
        GlobalData,
        SpacedGlobalData,
        Spaces,
        SubTreeMetrics,
        TextedGlobalData,
        TreeGen,
        ZippedTreeGen,
    },
    types::{LabelStore as _, Tree as _, Type, Typed},
    utils::{self, clamp_u64_to_u32},
};
// use hyper_ast::nodes::SimpleNode1;

pub type LabelIdentifier = hyper_ast::store::labels::DefaultLabelIdentifier;

pub struct XmlTreeGen<'a> {
    pub line_break: Vec<u8>,
    pub stores: &'a mut SimpleStores,
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
        acc.simple.push(self.compressed_node);
        acc.metrics.acc(self.metrics);

        // TODO things with this.ana
    }
}

pub struct Acc {
    simple: BasicAccumulator<Type, NodeIdentifier>,
    labeled: bool,
    start_byte: usize,
    end_byte: usize,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: Option<PartialAnalysis>,
    padding_start: usize,
    indentation: Spaces,
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

impl<'a> ZippedTreeGen for XmlTreeGen<'a> {
    // type Node1 = SimpleNode1<NodeIdentifier, String>;
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
        global: &mut Self::Global,
    ) -> <Self as TreeGen>::Acc {
        let type_store = &mut self.stores().type_store;
        let parent_indentation = &stack.last().unwrap().indentation();
        let kind = node.kind();
        let kind = type_store.get_xml(kind);
        // let kind = handle_wildcard_kind(kind, node);

        let indent = try_compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            global.sum_byte_length(),
            &parent_indentation,
        );
        let labeled = node.has_label();
        let ana = self.build_ana(&kind);
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            labeled,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            metrics: Default::default(),
            ana,
            padding_start: global.sum_byte_length(),
            indentation: indent,
        }
    }

    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        global: &mut Self::Global,
        text: &[u8],
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let node_store = &mut self.stores.node_store;
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
            // parent.push(Self::make_spacing(spacing, node_store, global));
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

    fn init_val(&mut self, text: &[u8], node: &Self::Node<'_>) -> Self::Acc {
        let type_store = &mut self.stores().type_store;
        let kind = type_store.get(node.kind());

        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            0,
            &Space::format_indentation(&self.line_break),
        );
        let labeled = node.has_label();
        let ana = self.build_ana(&kind);
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            labeled,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            metrics: Default::default(),
            ana,
            padding_start: 0,
            indentation: indent,
        }
    }
}
impl<'a> TreeGen for XmlTreeGen<'a> {
    type Acc = Acc;
    type Global = SpacedGlobalData<'a>;
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
        let hbuilder = hashed::Builder::new(hashs, &acc.simple.kind, &label, size);
        let hsyntax = hbuilder.most_discriminating();
        let hashable = &hsyntax;
        let (ana, label) = if let Some(label) = label.as_ref() {
            assert!(acc.ana.is_none());
            if &acc.simple.kind == &Type::Comment {
                (None, Some(label_store.get_or_insert(label.as_str())))
            } else if acc.simple.kind.is_literal() {
                let tl = acc.simple.kind.literal_type();
                // let tl = label_store.get_or_insert(tl);
                (
                    Some(PartialAnalysis {}),
                    Some(label_store.get_or_insert(label.as_str())),
                )
            } else {
                let rf = label_store.get_or_insert(label.as_str());
                (Some(PartialAnalysis {}), Some(rf))
            }
        } else if acc.simple.kind.is_primitive() {
            let node = node_store.resolve(acc.simple.children[0]);
            let label = node.get_type().to_string();
            if let Some(ana) = acc.ana {
                todo!("{:?} {:?}", acc.simple.kind, ana)
            }
            // let rf = label_store.get_or_insert(label.as_str());
            (Some(PartialAnalysis {}), None)
        } else if let Some(ana) = acc.ana {
            // nothing to do, resolutions at the end of post ?
            (Some(ana), None)
        } else if acc.simple.kind == Type::TS86
            || acc.simple.kind == Type::TS81
            || acc.simple.kind == Type::Asterisk
            || acc.simple.kind == Type::Dimensions
            || acc.simple.kind == Type::Block
            || acc.simple.kind == Type::ElementValueArrayInitializer
        {
            (Some(PartialAnalysis {}), None)
        } else if acc.simple.kind == Type::ArgumentList
            || acc.simple.kind == Type::FormalParameters
            || acc.simple.kind == Type::AnnotationArgumentList
        {
            assert!(acc
                .simple
                .children
                .iter()
                .all(|x| { !node_store.resolve(*x).has_children() }));
            // TODO decls
            (Some(PartialAnalysis {}), None)
        } else if acc.simple.kind == Type::SwitchLabel || acc.simple.kind == Type::Modifiers {
            // TODO decls
            (None, None)
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
        {
            // TODO maybe do something later?
            (None, None)
        } else {
            assert!(
                acc.simple.children.is_empty()
                    || !acc
                        .simple
                        .children
                        .iter()
                        .all(|x| { !node_store.resolve(*x).has_children() }),
                "{:?}",
                &acc.simple.kind
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
                    Some(CS(cs)) => cs.as_ref() == &acc.simple.children,
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

        let hashs = hbuilder.build();

        let ana = match ana {
            Some(ana) => Some(ana), // TODO partialana resolution such as deps in pom.xml
            None => None,
        };
        let compressed_node = if let Some(id) = insertion.occupied_id() {
            id
        } else {
            let vacant = insertion.vacant();
            match label {
                None => {
                    macro_rules! insert {
                        ( $c:expr, $t:ty ) => {{
                            // let it = ana.as_ref().unwrap().solver.iter_refs();
                            // let it =
                            //     BulkHasher::<_, <$t as BF<[u8]>>::S, <$t as BF<[u8]>>::H>::from(it);
                            NodeStore::insert_after_prepare(
                                vacant,
                                // $c.concat((<$t>::SIZE, <$t>::from(it))),
                                $c
                            )
                        }};
                    }
                    match acc.simple.children.len() {
                        0 => {
                            assert_eq!(0, size);
                            assert_eq!(0, height);
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
                            let a = acc.simple.children.into_boxed_slice();
                            use hyper_ast::store::nodes::legion::compo;
                            let c = (
                                acc.simple.kind.clone(),
                                compo::Size(size),
                                compo::Height(height),
                                hashs,
                                CS(a),
                            );
                            match ana.as_ref().map(|x| x.refs_count()).unwrap_or(0) {
                                x if x > 1024 => NodeStore::insert_after_prepare(
                                    vacant,
                                    c.concat((BloomSize::Much,)),
                                ),
                                x if x > 512 => {
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
                    assert!(acc.simple.children.is_empty());
                    NodeStore::insert_after_prepare(
                        vacant,
                        (acc.simple.kind.clone(), hashs, label, BloomSize::None), // None not sure
                    )
                }
            }
        };

        let metrics = SubTreeMetrics {
            size,
            height,
            hashs,
        };

        let full_node = FullNode {
            global: global.into(),
            local: Local {
                compressed_node,
                metrics,
                ana,
            },
        };
        full_node
    }
}

impl<'a> XmlTreeGen<'a> {
    fn make_spacing(
        &mut self,
        spacing: Vec<u8>, //Space>,
                          // node_store: &mut NodeStore,
                          // global: &mut <Self as TreeGen>::Global,
    ) -> Local {
        // <<Self as TreeGen>::Acc as Accumulator>::Node {
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
                (Type::Spaces, spacing_id, hashs, BloomSize::None),
            )
        };

        Local {
            compressed_node,
            metrics: SubTreeMetrics {
                size: 1,
                height: 1,
                hashs,
            },
            ana: Default::default(),
        }
    }

    pub fn new(stores: &mut SimpleStores) -> XmlTreeGen {
        XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores,
        }
    }

    pub fn tree_sitter_parse(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_html::language();
        parser.set_language(language).unwrap();
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
        text: &'a [u8],
        cursor: tree_sitter::TreeCursor,
    ) -> FullNode<BasicGlobalData, Local> {
        let mut init = self.init_val(text, &TNode(cursor.node()));
        let mut xx = TTreeCursor(cursor);
        let mut global = Global::from(TextedGlobalData::new(Default::default(), text));

        let spacing = get_spacing(
            init.padding_start,
            init.start_byte,
            text,
            init.indentation(),
        );
        if let Some(spacing) = spacing {
            global.down();
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
            Some(PartialAnalysis {})
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
/// TODO partialana
impl PartialAnalysis {
    pub(crate) fn refs_count(&self) -> usize {
        0 //TODO
    }
    pub(crate) fn refs(&self) -> impl Iterator<Item = Vec<u8>> {
        vec![vec![0_u8]].into_iter() //TODO
    }
}
