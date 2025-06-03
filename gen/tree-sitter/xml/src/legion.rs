///! fully compress all subtrees from an Xml CST
use std::{fmt::Debug, vec};

use legion::world::EntryRef;
use tuples::CombinConcat;

use hyperast::store::nodes::compo::{self, CS, NoSpacesCS};
use hyperast::{
    filter::BloomSize,
    full::FullNode,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    nodes::Space,
    store::{
        SimpleStores,
        nodes::{
            DefaultNodeStore as NodeStore,
            legion::{NodeIdentifier, PendingInsert, eq_node},
        },
    },
    tree_gen::{
        AccIndentation, Accumulator, BasicAccumulator, BasicGlobalData, GlobalData, Parents,
        PreResult, SpacedGlobalData, Spaces, SubTreeMetrics, TextedGlobalData, TreeGen,
        WithByteRange, ZippedTreeGen, compute_indentation, get_spacing, has_final_space,
        parser::{Node as _, TreeCursor},
    },
    types::LabelStore as _,
};

use crate::{
    TNode,
    types::{TStore, Type, XmlEnabledTypeStore},
};

pub type LabelIdentifier = hyperast::store::labels::DefaultLabelIdentifier;

pub struct XmlTreeGen<'stores, TS = TStore> {
    pub line_break: Vec<u8>,
    pub stores: &'stores mut SimpleStores<TS>,
}

pub type Global<'a> = SpacedGlobalData<'a>;

#[derive(Debug, Clone)]
pub struct Local {
    pub compressed_node: NodeIdentifier,
    pub metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
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

#[repr(transparent)]
#[derive(Clone)]
pub struct TTreeCursor<'a>(tree_sitter::TreeCursor<'a>);

impl<'a> Debug for TTreeCursor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TTreeCursor")
            .field(&self.0.node().kind())
            .finish()
    }
}
impl<'a> hyperast::tree_gen::parser::TreeCursor for TTreeCursor<'a> {
    type N = TNode<'a>;
    fn node(&self) -> TNode<'a> {
        TNode(self.0.node())
    }

    fn role(&self) -> Option<std::num::NonZeroU16> {
        self.0.field_id()
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

impl<'stores, TS: XmlEnabledTypeStore> ZippedTreeGen for XmlTreeGen<'stores, TS> {
    // type Node1 = SimpleNode1<NodeIdentifier, String>;
    type Stores = SimpleStores<TS>;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b>;

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
        if node.0.is_missing() {
            return PreResult::Skip;
        }
        let Some(kind) = TS::try_obtain_type(&node) else {
            return PreResult::Skip;
        };
        let mut acc = self.pre(text, &node, stack, global);
        if kind == Type::AttValue {
            acc.labeled = true;
            return PreResult::SkipChildren(acc);
        }
        log::warn!("not retrieving roles");
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
        // if global.sum_byte_length() < 400 {
        //     dbg!((kind,node.start_byte(),node.end_byte(),global.sum_byte_length(),indent.len()));
        // }
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
                global: global.simple(),
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

pub fn tree_sitter_parse_xml(text: &[u8]) -> Result<tree_sitter::Tree, tree_sitter::Tree> {
    hyperast::tree_gen::utils_ts::tree_sitter_parse(text, &crate::language())
}

impl<'a, TS> XmlTreeGen<'a, TS> {
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
}
impl<'a, TS: XmlEnabledTypeStore> XmlTreeGen<'a, TS> {
    fn make_spacing(&mut self, spacing: Vec<u8>) -> Local {
        let kind = Type::Spaces;
        let interned_kind = TS::intern(kind);
        let bytes_len = spacing.len();
        let spacing = std::str::from_utf8(&spacing).unwrap().to_string();
        use num::ToPrimitive;
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
            let t = x.get_component::<_>();
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
        }
    }

    pub fn new(stores: &mut SimpleStores<TS>) -> XmlTreeGen<TS> {
        XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores,
        }
    }

    pub fn generate_file(
        &mut self,
        name: &[u8],
        text: &'a [u8],
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
            init.start_byte = 0;
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

impl<'stores, TS: XmlEnabledTypeStore> TreeGen for XmlTreeGen<'stores, TS> {
    type Acc = Acc;
    type Global = SpacedGlobalData<'stores>;
    fn make(
        &mut self,
        global: &mut <Self as TreeGen>::Global,
        acc: <Self as TreeGen>::Acc,
        label: Option<String>,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let interned_kind = TS::intern(acc.simple.kind);
        let node_store = &mut self.stores.node_store;
        let label_store = &mut self.stores.label_store;
        let line_count = acc.metrics.line_count;
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

        // let ana = None as Option<PartialAnalysis>;

        // let ana = match ana {
        //     Some(ana) => Some(ana), // TODO partial ana resolution such as deps in pom.xml
        //     None => None,
        // };

        let local = if let Some(compressed_node) = insertion.occupied_id() {
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
            }
        } else {
            let hashs = hbuilder.build();
            let bytes_len = compo::BytesLen((acc.end_byte - acc.start_byte).try_into().unwrap());
            let compressed_node = compress(
                label_id,
                interned_kind,
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
                line_count,
            };
            Local {
                compressed_node,
                metrics,
            }
        };

        let full_node = FullNode {
            global: global.simple(),
            local,
        };
        full_node
    }
}

fn compress<Ty: std::marker::Send + std::marker::Sync + 'static>(
    label_id: Option<LabelIdentifier>,
    interned_kind: Ty,
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
    let base = (interned_kind, hashs, bytes_len);
    match (label_id, 0) {
        (None, _) => children_dipatch!(base,),
        (Some(label), _) => children_dipatch!(base, (label,),),
    }
}

// pub fn print_tree_ids(node_store: &NodeStore, id: &NodeIdentifier) {
//     nodes::print_tree_ids(
//         |id| -> _ {
//             node_store
//                 .resolve(id.clone())
//                 .into_compressed_node()
//                 .unwrap()
//         },
//         id,
//     )
// }

// pub fn print_tree_structure(node_store: &NodeStore, id: &NodeIdentifier) {
//     nodes::print_tree_structure(
//         |id| -> _ {
//             node_store
//                 .resolve(id.clone())
//                 .into_compressed_node()
//                 .unwrap()
//         },
//         id,
//     )
// }

// pub fn print_tree_labels(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
//     nodes::print_tree_labels(
//         |id| -> _ {
//             node_store
//                 .resolve(id.clone())
//                 .into_compressed_node()
//                 .unwrap()
//         },
//         |id| -> _ { label_store.resolve(id).to_owned() },
//         id,
//     )
// }

// pub fn print_tree_syntax(node_store: &NodeStore, label_store: &LabelStore, id: &NodeIdentifier) {
//     nodes::print_tree_syntax(
//         |id| -> _ {
//             node_store
//                 .resolve(id.clone())
//                 .into_compressed_node()
//                 .unwrap()
//         },
//         |id| -> _ { label_store.resolve(id).to_owned() },
//         id,
//         &mut Into::<IoOut<_>>::into(stdout()),
//     )
// }

// pub fn print_tree_syntax_with_ids(
//     node_store: &NodeStore,
//     label_store: &LabelStore,
//     id: &NodeIdentifier,
// ) {
//     nodes::print_tree_syntax_with_ids(
//         |id| -> _ {
//             node_store
//                 .resolve(id.clone())
//                 .into_compressed_node()
//                 .unwrap()
//         },
//         |id| -> _ { label_store.resolve(id).to_owned() },
//         id,
//         &mut Into::<IoOut<_>>::into(stdout()),
//     )
// }

// pub fn serialize<W: std::fmt::Write>(
//     node_store: &NodeStore,
//     label_store: &LabelStore,
//     id: &NodeIdentifier,
//     out: &mut W,
//     parent_indent: &str,
// ) -> Option<String> {
//     nodes::serialize(
//         |id| -> _ {
//             node_store
//                 .resolve(id.clone())
//                 .into_compressed_node()
//                 .unwrap()
//         },
//         |id| -> _ { label_store.resolve(id).to_owned() },
//         id,
//         out,
//         parent_indent,
//     )
// }
