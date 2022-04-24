///! fully compress all subtrees from an Xml CST
use std::{fmt::Debug, vec};

use legion::world::EntryRef;
use tree_sitter::{Language, Parser, TreeCursor};
use tuples::CombinConcat;

use hyper_ast::{
    filter::BF,
    filter::{Bloom, BloomSize},
    full::FullNode,
    hashed::{self, NodeHashs, SyntaxNodeHashs},
    // impact::{element::RefsEnum, elements::*, partial_analysis::PartialAnalysis},
    nodes::{self, SimpleNode1, Space},
    store::labels::LabelStore,
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
        // label_for_cursor,
        AccIndentation,
        Accumulator,
        BasicAccumulator,
        Spaces,
        SubTreeMetrics,
        TreeGen,
    },
    types::{LabelStore as _, Tree as _, Type, Typed},
    utils::{self, clamp_u64_to_u32},
};

// pub type HashedNode<'a> = HashedCompressedNode<SyntaxNodeHashs<HashSize>,SymbolU32<&'a HashedNode>,LabelIdentifier>;

extern "C" {
    fn tree_sitter_html() -> Language;
}

pub type LabelIdentifier = hyper_ast::store::labels::DefaultLabelIdentifier;

pub struct XmlTreeGen {
    pub line_break: Vec<u8>,
    pub stores: SimpleStores,
}

#[derive(Debug)]
pub struct Global {
    pub(crate) depth: usize,
    pub(crate) position: usize,
}

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

// #[derive(Default, Debug, Clone, Copy)]
// pub struct SubTreeMetrics<U: NodeHashs> {
//     pub hashs: U,
//     pub size: u32,
//     pub height: u32,
// }

// impl<U: NodeHashs> SubTreeMetrics<U> {
//     pub fn acc(&mut self, other: Self) {
//         self.height = self.height.max(other.height);
//         self.size += other.size;
//         self.hashs.acc(&other.hashs);
//     }
// }

pub struct Acc {
    simple: BasicAccumulator<Type, NodeIdentifier>,
    label: Option<String>,
    start_byte: usize,
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: Option<PartialAnalysis>,
    padding_start: usize,
    indentation: Spaces,
}

// impl Acc {
//     pub(crate) fn new(kind: Type) -> Self {
//         Self {
//             simple: BasicAccumulator::new(kind),
//             metrics: Default::default(),
//             ana: Default::default(),
//             padding_start: 0,
//             indentation: Space::format_indentation(&"\n".as_bytes().to_vec()),
//         }
//     }
// }

pub type FNode = FullNode<Global, Local>;
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

pub struct SimpleStores {
    pub label_store: LabelStore,
    pub type_store: TypeStore,
    pub node_store: NodeStore,
}

impl Default for SimpleStores {
    fn default() -> Self {
        Self {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        }
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

impl<'a> TreeGen for XmlTreeGen {
    type Node1 = SimpleNode1<NodeIdentifier, String>;
    type Acc = Acc;
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
        sum_byte_length: usize,
    ) -> <Self as TreeGen>::Acc {
        let type_store = &mut self.stores().type_store;
        let parent_indentation = &stack.last().unwrap().indentation();
        let kind = node.kind();
        let kind = type_store.get(kind);
        // let kind = handle_wildcard_kind(kind, node);

        let indent = compute_indentation(
            &self.line_break,
            text,
            node.start_byte(),
            sum_byte_length,
            &parent_indentation,
        );
        let label = node
            .extract_label(text)
            .and_then(|x| Some(std::str::from_utf8(&x).unwrap().to_owned()));
        let ana = self.build_ana(&kind);
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            label,
            start_byte: node.start_byte(),
            metrics: Default::default(),
            ana,
            padding_start: sum_byte_length,
            indentation: indent,
        }
    }

    fn post(
        &mut self,
        parent: &mut <Self as TreeGen>::Acc,
        depth: usize,
        position: usize,
        text: &[u8],
        // node: &Self::Node<'_>,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let node_store = &mut self.stores.node_store;

        Self::handle_spacing(
            acc.padding_start,
            acc.start_byte,
            text,
            node_store,
            &(depth + 1),
            position,
            parent,
        );
        self.make(depth, position, acc)
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
        let label = node
            .extract_label(text)
            .and_then(|x| Some(std::str::from_utf8(&x).unwrap().to_owned()));
        let ana = self.build_ana(&kind);
        Acc {
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
            label,
            start_byte: node.start_byte(),
            metrics: Default::default(),
            ana,
            padding_start: 0,
            indentation: indent,
        }
    }
}

// /// make new types to handle wildcard precisely
// fn handle_wildcard_kind(kind: Type, node: &tree_sitter::Node) -> Type {
//     if kind == Type::Wildcard {
//         if node.child_by_field_name(b"extends").is_some() {
//             Type::WildcardExtends
//         } else if node.child_by_field_name(b"super").is_some() {
//             Type::WildcardSuper
//         } else {
//             kind
//         }
//     } else {
//         kind
//     }
// }

#[derive(PartialEq, Eq)]
enum Has {
    Down,
    Up,
    Right,
}

impl XmlTreeGen {
    fn handle_spacing(
        padding_start: usize,
        pos: usize,
        text: &[u8],
        node_store: &mut NodeStore,
        depth: &usize,
        position: usize,
        parent: &mut <Self as TreeGen>::Acc,
    ) {
        let tmp = get_spacing(padding_start, pos, text, parent.indentation());
        if let Some(relativized) = tmp {
            let hsyntax = utils::clamp_u64_to_u32(&utils::hash(&relativized));
            let hashable = &hsyntax;

            let spaces = relativized.into_boxed_slice();

            let eq = |x: EntryRef| {
                let t = x.get_component::<Box<[Space]>>().ok();
                if t != Some(&spaces) {
                    return false;
                }
                true
            };

            let insertion = node_store.prepare_insertion(&hashable, eq);

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
                    (Type::Spaces, spaces, hashs, BloomSize::None),
                )
            };

            let full_spaces_node = FullNode {
                global: Global {
                    depth: *depth,
                    position,
                },
                local: Local {
                    compressed_node,
                    metrics: SubTreeMetrics {
                        size: 1,
                        height: 1,
                        hashs,
                    },
                    ana: Default::default(),
                },
            };
            parent.push(full_spaces_node);
        };
    }

    /// end of tree but not end of file,
    /// thus to be a bijection, we need to get the last spaces
    fn handle_final_space(
        depth: &usize,
        sum_byte_length: usize,
        text: &[u8],
        node_store: &mut NodeStore,
        position: usize,
        parent: &mut <Self as TreeGen>::Acc,
    ) {
        if has_final_space(depth, sum_byte_length, text) {
            Self::handle_spacing(
                sum_byte_length,
                text.len(),
                text,
                node_store,
                depth,
                position,
                parent,
            )
        }
    }

    pub fn new() -> Self {
        Self {
            line_break: "\n".as_bytes().to_vec(),
            stores: SimpleStores {
                label_store: LabelStore::new(),
                type_store: TypeStore {},
                node_store: NodeStore::new(),
            },
        }
    }

    pub fn generate_file(
        &mut self,
        name: &[u8],
        text: &[u8],
        cursor: TreeCursor,
    ) -> FullNode<Global, Local> {
        let mut init = self.init_val(text, &TNode(cursor.node()));
        init.label = Some(std::str::from_utf8(name).unwrap().to_owned());
        let mut stack = vec![init];
        let mut xx = TTreeCursor(cursor);
        let sum_byte_length = self.gen(text, &mut stack, &mut xx);
        let mut acc = stack.pop().unwrap();
        Self::handle_final_space(
            &0,
            sum_byte_length,
            text,
            &mut self.stores.node_store,
            acc.metrics.size as usize + 1,
            &mut acc,
        );
        let full_node = self.make(0, acc.metrics.size as usize, acc);
        full_node
    }

    pub fn main() {
        let mut parser = Parser::new();
        parser.set_language(unsafe { tree_sitter_html() }).unwrap();

        let text = {
            let source_code1 = "class A {void test() {}}";
            source_code1.as_bytes()
        };
        // let mut parser: Parser, old_tree: Option<&Tree>
        let tree = parser.parse(text, None).unwrap();
        let mut xml_tree_gen = XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: SimpleStores {
                label_store: LabelStore::new(),
                type_store: TypeStore {},
                node_store: NodeStore::new(),
            },
        };
        let _full_node = xml_tree_gen.generate_file(b"", text, tree.walk());

        // print_tree_structure(
        //     &xml_tree_gen.stores.node_store,
        //     &_full_node.local.compressed_node,
        // );

        let tree = parser.parse(text, Some(&tree)).unwrap();
        let _full_node = xml_tree_gen.generate_file(b"", text, tree.walk());
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
    fn make(
        &mut self,
        depth: usize,
        position: usize,
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let node_store = &mut self.stores.node_store;
        let label_store = &mut self.stores.label_store;
        let label = acc.label;
        let metrics = acc.metrics;
        let hashed_kind = &clamp_u64_to_u32(&utils::hash(&acc.simple.kind));
        let hashed_label = &clamp_u64_to_u32(&utils::hash(&label));
        let hsyntax = hashed::inner_node_hash(
            hashed_kind,
            hashed_label,
            &acc.metrics.size,
            &acc.metrics.hashs.syntax,
        );
        let hashable = &hsyntax; //(hlabel as u64) << 32 & hsyntax as u64;

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
                    Some(CS(cs)) => cs == &acc.simple.children,
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

        let hashs = SyntaxNodeHashs {
            structt: hashed::inner_node_hash(
                hashed_kind,
                &0,
                &acc.metrics.size,
                &acc.metrics.hashs.structt,
            ),
            label: hashed::inner_node_hash(
                hashed_kind,
                hashed_label,
                &acc.metrics.size,
                &acc.metrics.hashs.label,
            ),
            syntax: hsyntax,
        };

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
                            assert_eq!(0, metrics.size);
                            assert_eq!(0, metrics.height);
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
                            let a = acc.simple.children;
                            use hyper_ast::store::nodes::legion::compo;
                            let c = (
                                acc.simple.kind.clone(),
                                compo::Size(metrics.size + 1),
                                compo::Height(metrics.height + 1),
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
            size: metrics.size + 1,
            height: metrics.height + 1,
            hashs,
        };

        let full_node = FullNode {
            global: Global { depth, position },
            local: Local {
                compressed_node,
                metrics,
                ana,
            },
        };
        full_node
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
