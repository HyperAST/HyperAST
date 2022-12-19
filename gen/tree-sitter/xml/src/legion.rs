///! fully compress all subtrees from an Xml CST
use std::{fmt::Debug, io::stdout, vec};

use legion::world::EntryRef;
use tuples::CombinConcat;

use hyper_ast::{
    filter::BloomSize,
    full::FullNode,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    // impact::{element::RefsEnum, elements::*, partial_analysis::PartialAnalysis},
    nodes::{self, IoOut, Space},
    store::{
        labels::LabelStore,
        nodes::legion::{NoSpacesCS, PendingInsert},
        SimpleStores,
    },
    store::{
        nodes::legion::{compo, NodeIdentifier, CS},
        nodes::DefaultNodeStore as NodeStore,
    },
    tree_gen::{
        compute_indentation, get_spacing, has_final_space, parser::Node as _, AccIndentation,
        Accumulator, BasicAccumulator, BasicGlobalData, GlobalData, SpacedGlobalData, Spaces,
        SubTreeMetrics, TextedGlobalData, TreeGen, ZippedTreeGen,
    },
    types::{LabelStore as _, Type},
};

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

impl<'a> ZippedTreeGen for XmlTreeGen<'a> {
    // type Node1 = SimpleNode1<NodeIdentifier, String>;
    type Stores = SimpleStores;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b>;

    fn stores(&mut self) -> &mut Self::Stores {
        &mut self.stores
    }

    fn init_val(&mut self, text: &[u8], node: &Self::Node<'_>) -> Self::Acc {
        let type_store = &mut self.stores().type_store;
        let kind = type_store.get_xml(node.kind());
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

impl<'a> XmlTreeGen<'a> {
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
                height: 1,
                hashs,
                size_no_spaces: 0,
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
        let language = tree_sitter_xml::language();
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
        full_node
    }

    fn build_ana(&mut self, kind: &Type) -> Option<PartialAnalysis> {
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

impl<'stores> TreeGen for XmlTreeGen<'stores> {
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

        let label_id = label
            .as_ref()
            .map(|label| label_store.get_or_insert(label.as_str()));
        let eq = eq_node(&acc.simple.kind, label_id.as_ref(), &acc.simple.children);

        let insertion = node_store.prepare_insertion(&hashable, eq);

        // let ana = None as Option<PartialAnalysis>;

        // let ana = match ana {
        //     Some(ana) => Some(ana), // TODO partial ana resolution such as deps in pom.xml
        //     None => None,
        // };

        let local = if let Some(compressed_node) = insertion.occupied_id() {
            let ana = None;
            let hashs = hbuilder.build();
            let metrics = SubTreeMetrics {
                size,
                height,
                hashs,
                size_no_spaces,
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
            );

            let metrics = SubTreeMetrics {
                size,
                height,
                hashs,
                size_no_spaces,
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

fn compress(
    label_id: Option<LabelIdentifier>,
    _ana: &Option<PartialAnalysis>,
    simple: BasicAccumulator<Type, NodeIdentifier>,
    no_space: Vec<NodeIdentifier>,
    bytes_len: compo::BytesLen,
    size: u32,
    height: u32,
    size_no_spaces: u32,
    insertion: PendingInsert,
    hashs: SyntaxNodeHashs<u32>,
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
    let base = (simple.kind.clone(), hashs, bytes_len);
    match (label_id, 0) {
        (None, _) => children_dipatch!(base,),
        (Some(label), _) => children_dipatch!(base, (label,),),
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
/// TODO partialana
impl PartialAnalysis {
    pub(crate) fn refs_count(&self) -> usize {
        0 //TODO
    }
    pub(crate) fn refs(&self) -> impl Iterator<Item = Vec<u8>> {
        vec![vec![0_u8]].into_iter() //TODO
    }
}
