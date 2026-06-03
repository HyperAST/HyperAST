///! fully compress all subtrees from a tree-sitter query CST
use std::{collections::HashMap, fmt::Debug};

use hyperast::store::nodes::legion::eq_node;
use hyperast::types::{HyperType, WithSerialization};

use hyperast::store::nodes::compo;
use hyperast::{
    full::FullNode,
    hashed::{self, IndexingHashBuilder, MetaDataHashsBuilder, SyntaxNodeHashs},
    store::{
        SimpleStores,
        nodes::{
            DefaultNodeStore as NodeStore, EntityBuilder,
            legion::{HashedNodeRef, NodeIdentifier},
        },
    },
    tree_gen::{
        AccIndentation, Accumulator, BasicAccumulator, BasicGlobalData, Parents, PreResult,
        SpacedGlobalData, Spaces, SubTreeMetrics, TextedGlobalData, TreeGen, WithByteRange,
        ZippedTreeGen,
        parser::{Node as _, TreeCursor},
        utils_ts::TTreeCursor,
    },
    types::LabelStore as _,
};
use num::ToPrimitive;

use crate::types::{TsQueryEnabledTypeStore, Type};
use crate::{TNode, types::TIdN};

pub type LabelIdentifier = hyperast::store::labels::DefaultLabelIdentifier;

pub struct TsQueryTreeGen<'store, 'cache, TS> {
    pub line_break: Vec<u8>,
    pub stores: &'store mut SimpleStores<TS>,
    pub md_cache: &'cache mut MDCache,
}

pub type MDCache = HashMap<NodeIdentifier, MD>;

// NOTE only keep compute intensive metadata (where space/time tradeoff is worth storing)
// eg. decls refs, maybe hashes but not size and height
// * metadata: computation results from concrete code of node and its children
// they can be qualitative metadata .eg a hash or they can be quantitative .eg lines of code
#[derive(Clone)]
pub struct MD {
    metrics: SubTreeMetrics<SyntaxNodeHashs<u32>>,
}

impl From<Local> for MD {
    fn from(x: Local) -> Self {
        MD { metrics: x.metrics }
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
}

pub use crate::tree_sitter_parse;

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
    padding_start: usize,
}

pub type FNode = FullNode<BasicGlobalData, Local>;
impl Accumulator for Acc {
    type Node = FNode;
    fn push(&mut self, full_node: Self::Node) {
        full_node.local.acc(self);
    }
}

impl AccIndentation for Acc {
    fn indentation(&self) -> &Spaces {
        unimplemented!()
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
impl Debug for Acc {
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

impl<'store, TS: TsQueryEnabledTypeStore<HashedNodeRef<'store, NodeIdentifier>>> ZippedTreeGen
    for TsQueryTreeGen<'store, '_, TS>
{
    type Stores = SimpleStores<TS>;
    type Text = [u8];
    type Node<'b> = TNode<'b>;
    type TreeCursor<'b> = TTreeCursor<'b>;

    fn stores(&mut self) -> &mut Self::Stores {
        self.stores
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
        let mut acc = self.pre(text, &node, stack, global);
        if kind == Type::String {
            acc.labeled = true;
            return PreResult::SkipChildren(acc);
        }
        // TODO find better condition, for now using the alias
        // NOTE this targets _string in rule anonymous_node
        else if kind == Type::Identifier {
            acc.labeled = true;
            return PreResult::SkipChildren(acc);
        }
        log::trace!("not retrieving roles");
        PreResult::Ok(acc)
    }
    fn pre(
        &mut self,
        _text: &[u8],
        node: &Self::Node<'_>,
        _stack: &Parents<Self::Acc>,
        global: &mut Self::Global,
    ) -> <Self as TreeGen>::Acc {
        let kind = TS::obtain_type(node);
        Acc {
            labeled: node.has_label(),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            metrics: Default::default(),
            padding_start: global.sum_byte_length(),
            simple: BasicAccumulator {
                kind,
                children: vec![],
            },
        }
    }

    fn post(
        &mut self,
        _parent: &mut <Self as TreeGen>::Acc,
        global: &mut Self::Global,
        text: &[u8],
        acc: <Self as TreeGen>::Acc,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
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

impl<'store, 'cache, TS: TsQueryEnabledTypeStore<HashedNodeRef<'store, NodeIdentifier>>>
    TsQueryTreeGen<'store, 'cache, TS>
{
    pub fn new(
        stores: &'store mut <Self as ZippedTreeGen>::Stores,
        md_cache: &'cache mut MDCache,
    ) -> TsQueryTreeGen<'store, 'cache, TS> {
        TsQueryTreeGen::<'store, 'cache, TS> {
            line_break: "\n".as_bytes().to_vec(),
            stores,
            md_cache,
        }
    }

    pub fn generate_file(
        &mut self,
        name: &[u8],
        text: &'store [u8],
        cursor: tree_sitter::TreeCursor,
    ) -> <<Self as TreeGen>::Acc as Accumulator>::Node {
        let mut global = Global::from(TextedGlobalData::new(Default::default(), text));
        let init = self.init_val(text, &TNode(cursor.node()));
        let mut xx = TTreeCursor(cursor);

        let mut stack = init.into();

        self.r#gen(text, &mut stack, &mut xx, &mut global);

        let acc = stack.finalize();

        let label = Some(std::str::from_utf8(name).unwrap().to_owned());

        self.make(&mut global, acc, label)
    }
}

impl<'stores, TS: TsQueryEnabledTypeStore<HashedNodeRef<'stores, NodeIdentifier>>> TreeGen
    for TsQueryTreeGen<'stores, '_, TS>
{
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
        let interned_kind = TS::intern(acc.simple.kind);
        // let hashs = acc.metrics.hashs;
        // let size = acc.metrics.size + 1;
        // let height = acc.metrics.height + 1;
        // let line_count = acc.metrics.line_count;
        let size_no_spaces = acc.metrics.size_no_spaces + 1;
        // let hbuilder = hashed::HashesBuilder::new(hashs, &interned_kind, &label, size_no_spaces);
        // let hsyntax = hbuilder.most_discriminating();
        // let hashable = &hsyntax;

        let metrics = acc
            .metrics
            .finalize(&interned_kind, &label, size_no_spaces as u16);

        let hashable = &metrics.hashs.most_discriminating();

        let label_id = label
            .as_ref()
            .map(|label| label_store.get_or_insert(label.as_str()));
        let eq = eq_node(&interned_kind, label_id.as_ref(), &acc.simple.children);

        let insertion = node_store.prepare_insertion(&hashable, eq);

        let local = if let Some(compressed_node) = insertion.occupied_id() {
            let metrics = metrics.map_hashs(|h| h.build());
            // let hashs = hbuilder.build();
            // let metrics = SubTreeMetrics {
            //     size,
            //     height,
            //     hashs,
            //     size_no_spaces,
            //     line_count,
            // };
            Local {
                compressed_node,
                metrics,
            }
        } else {
            let byte_len = compo::BytesLen((acc.end_byte - acc.start_byte).try_into().unwrap());
            Self::insert_new_subtree(acc, interned_kind, metrics, label_id, insertion, byte_len)
        };

        FullNode {
            global: global.simple(),
            local,
        }
    }
}

impl<'stores, TS: TsQueryEnabledTypeStore<HashedNodeRef<'stores, NodeIdentifier>>>
    TsQueryTreeGen<'stores, '_, TS>
{
    pub fn build_then_insert(
        &mut self,
        _i: <hashed::HashedNode as hyperast::types::Stored>::TreeId,
        t: Type,
        l: Option<LabelIdentifier>,
        cs: Vec<NodeIdentifier>,
    ) -> NodeIdentifier {
        let (acc, byte_len) =
            Self::rebuild_acc(self.stores, t, l, cs, |c| self.md_cache.get(&c).cloned());
        let node_store = &mut self.stores.node_store;
        let label_store = &mut self.stores.label_store;
        let interned_kind = TS::intern(acc.simple.kind);
        // let hashs = acc.metrics.hashs;
        // let size = acc.metrics.size + 1;
        // let height = acc.metrics.height + 1;
        let line_count = acc.metrics.line_count;
        // let size_no_spaces = acc.metrics.size_no_spaces + 1;

        let label = l.map(|l| label_store.resolve(&l));
        // let hbuilder = hashed::HashesBuilder::new(hashs, &interned_kind, &label, size_no_spaces);
        // let hsyntax = hbuilder.most_discriminating();
        // let hashable = &hsyntax;

        let metrics = acc
            .metrics
            .finalize(&interned_kind, &label, line_count as u16);

        let hashable = &metrics.hashs.most_discriminating();

        let label_id = l;
        let eq = eq_node(&interned_kind, label_id.as_ref(), &acc.simple.children);

        let insertion = node_store.prepare_insertion(&hashable, eq);

        let local = if let Some(compressed_node) = insertion.occupied_id() {
            let metrics = metrics.map_hashs(|h| h.build());
            Local {
                compressed_node,
                metrics,
            }
        } else {
            let byte_len = compo::BytesLen(
                byte_len.to_u32().unwrap(), // (acc.end_byte - acc.start_byte).try_into().unwrap(),
            );
            Self::insert_new_subtree(acc, interned_kind, metrics, label_id, insertion, byte_len)
        };
        local.compressed_node
    }

    /// Try to build a node with the given type, label, and children.
    /// Cannot be used to build a new node.
    /// Only take self as shared ref and check if the would be created node would already exist
    pub fn try_build(
        stores: &'stores SimpleStores<TS>,
        _i: <hashed::HashedNode as hyperast::types::Stored>::TreeId,
        t: Type,
        l: Option<LabelIdentifier>,
        cs: Vec<NodeIdentifier>,
    ) -> Option<NodeIdentifier> {
        let (acc, _byte_len) = Self::rebuild_acc(stores, t, l, cs, |_| None);
        let node_store = &stores.node_store;
        let label_store = &stores.label_store;
        let interned_kind = TS::intern(acc.simple.kind);
        let size_no_spaces = acc.metrics.size_no_spaces + 1;

        let label = l.map(|l| label_store.resolve(&l));
        // let hbuilder = hashed::HashesBuilder::new(hashs, &interned_kind, &label, size_no_spaces);
        // let hsyntax = hbuilder.most_discriminating();
        // let hashable = &hsyntax;

        let metrics = acc
            .metrics
            .finalize(&interned_kind, &label, size_no_spaces as u16);
        let hashable = &metrics.hashs.most_discriminating();

        let label_id = l;
        let eq = eq_node(&interned_kind, label_id.as_ref(), &acc.simple.children);

        node_store.get(&hashable, eq)
    }

    fn insert_new_subtree(
        acc: Acc,
        interned_kind: <TS as hyperast::types::TypeStore>::Ty,
        metrics: SubTreeMetrics<hashed::HashesBuilder<SyntaxNodeHashs<u32>>>,
        label_id: Option<hyperast::store::labels::DefaultLabelIdentifier>,
        insertion: hyperast::store::nodes::legion::PendingInsert<'_>,
        byte_len: compo::BytesLen,
    ) -> Local {
        let metrics = metrics.map_hashs(|h| h.build());

        let mut dyn_builder = hyperast::store::nodes::legion::dyn_builder::EntityBuilder::new();
        dyn_builder.add(byte_len);

        let children_is_empty = acc.simple.children.is_empty();
        let hashs = metrics
            .clone()
            .add_md_metrics(&mut dyn_builder, children_is_empty);
        hashs.persist(&mut dyn_builder);
        acc.simple
            .add_primary(&mut dyn_builder, interned_kind, label_id);

        let compressed_node =
            NodeStore::insert_built_after_prepare(insertion.vacant(), dyn_builder.build());

        Local {
            compressed_node,
            metrics,
        }
    }

    fn rebuild_acc(
        stores: &SimpleStores<TS>,
        t: Type,
        l: Option<hyperast::store::labels::DefaultLabelIdentifier>,
        cs: Vec<legion::Entity>,
        md: impl Fn(NodeIdentifier) -> Option<MD>,
    ) -> (Acc, usize) {
        let mut acc: Acc = {
            let kind = t;
            Acc {
                labeled: l.is_some(),
                start_byte: 0,
                end_byte: 0,
                metrics: Default::default(),
                simple: BasicAccumulator {
                    kind,
                    children: vec![],
                },
                padding_start: 0,
            }
        };
        let mut byte_len = 0;
        for c in cs {
            let local = {
                let metrics = if let Some(md) = md(c) {
                    let metrics = md.metrics;
                    metrics
                } else {
                    use hyperast::hashed::SyntaxNodeHashsKinds;
                    use hyperast::types::WithHashs;
                    let (_, node) = stores
                        .node_store
                        .resolve_with_type::<TIdN<NodeIdentifier>>(&c);
                    let hashs = SyntaxNodeHashs {
                        structt: WithHashs::hash(&node, &SyntaxNodeHashsKinds::Struct),
                        label: WithHashs::hash(&node, &SyntaxNodeHashsKinds::Label),
                        syntax: WithHashs::hash(&node, &SyntaxNodeHashsKinds::Syntax),
                    };
                    byte_len += node.try_bytes_len().unwrap();
                    use hyperast::types::WithStats;
                    use num::ToPrimitive;
                    let metrics = SubTreeMetrics {
                        size: node.size().to_u32().unwrap(),
                        height: node.height().to_u32().unwrap(),
                        size_no_spaces: node.size_no_spaces().to_u32().unwrap(),
                        hashs,
                        line_count: node.line_count().to_u16().unwrap(),
                    };
                    metrics
                };
                Local {
                    compressed_node: c,
                    metrics,
                }
            };
            let global = BasicGlobalData::default();
            let full_node = FullNode { global, local };
            acc.push(full_node);
        }
        (acc, byte_len)
    }
}

pub struct PP<IdN, HAST, const SPC: bool = false> {
    stores: HAST,
    root: IdN,
    pub indent: &'static str,
}

impl<IdN, HAST, const SPC: bool> std::fmt::Display for PP<IdN, HAST, SPC>
where
    IdN: hyperast::types::NodeId<IdN = IdN>,
    HAST: hyperast::types::HyperAST<IdN = IdN>,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithSerialization,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithStats,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use hyperast::types::WithChildren;
        let kind = self.stores.resolve_type(&self.root);
        if !kind.is_file() {
            return match self.serialize(&self.root, "", "", f) {
                Err(hyperast::nodes::IndentedAlt::FmtError) => Err(std::fmt::Error),
                _ => Ok(()),
            };
        }
        let b = self.stores.resolve(&self.root);
        let children = b.children();
        let mut first = true;
        let mut ind = "";
        let Some(children) = children else {
            return Ok(());
        };
        for c in children {
            let kind = self.stores.resolve_type(&c);
            if !first && kind.as_static_str() == "named_node" {
                writeln!(f)?;
                writeln!(f)?;
            }
            match self.serialize(&c, "", ind, f) {
                Err(hyperast::nodes::IndentedAlt::FmtError) => return Err(std::fmt::Error),
                Err(hyperast::nodes::IndentedAlt::NoIndent) => {}
                Ok(_) => {}
            }
            if first {
                ind = self.indent;
                first = false;
            }
        }
        Ok(())
    }
}

impl<IdN, HAST, const SPC: bool> PP<IdN, HAST, SPC>
where
    IdN: hyperast::types::NodeId<IdN = IdN>,
    HAST: hyperast::types::HyperAST<IdN = IdN>,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithSerialization,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithStats,
{
    pub fn new(stores: HAST, root: IdN) -> Self {
        Self {
            stores,
            root,
            indent: "\n  ",
        }
    }
    fn serialize(
        &self,
        id: &IdN,
        parent_indent: &str,
        indent: &str,
        out: &mut std::fmt::Formatter<'_>,
    ) -> Result<String, hyperast::nodes::IndentedAlt> {
        use hyperast::nodes::IndentedAlt;
        use hyperast::types::Childrn;
        use hyperast::types::HyperType;
        use hyperast::types::LabelStore;
        use hyperast::types::Labeled;
        use hyperast::types::WithChildren;
        let b = self.stores.resolve(id);
        // let kind = (self.stores.type_store(), b);
        let kind = self.stores.resolve_type(id);
        let label = b.try_get_label();
        let children = b.children();

        match kind.as_static_str() {
            "program" => (),
            "identifier" | "@" | "#" | "predicate_type" => (),
            "parameters" => (),
            "/" => (), // for supertypes e.g. `(_literal/string_literal)`
            "quantifier" => (),
            "(" => (),
            ")" | "]" => {
                out.write_str(parent_indent)?;
            }
            "predicate" => {
                out.write_str(" ")?;
            }
            "string" | "capture" => {
                out.write_str(" ")?;
            }
            "named_node" => {
                out.write_str(indent)?;
            }
            _ => {
                out.write_str(indent)?;
            }
        }

        let r = match (label, children) {
            (None, None) => {
                out.write_str(&kind.to_string())?;
                let mut ind = indent.to_string();
                if kind.as_static_str() == "[" {
                    ind.push_str("  ")
                }
                match kind.as_static_str() {
                    "[" => Ok(ind),
                    _ => Err(IndentedAlt::NoIndent),
                }
                // out.write_str(&format!("{parent_indent:?}"))?;
                // let mut ind = indent.to_string();
                // if kind.as_static_str() == "(" {
                //     // ind.push_str("    ");
                //     //     // out.write_str(parent_indent)?;
                //     //     Err(IndentedAlt::NoIndent)
                //     Ok(ind)
                // } else if kind.as_static_str() == ")" {
                //     //     // out.write_str(parent_indent)?;
                //     Ok(ind)
                //     // Err(IndentedAlt::NoIndent)
                // } else {
                //     // ind.push_str("    ");
                //     Ok(ind)
                // }
                // Err(IndentedAlt::NoIndent)
            }
            (_, Some(children)) => {
                let mut parent_indent = parent_indent.to_string();
                // out.write_str("[")?;
                // out.write_str(&kind.to_string())?;
                // // out.write_str(&format!("{parent_indent:?}"))?;
                // out.write_str("]")?;

                // if kind.is_file() {
                // } else if kind.as_static_str() == "anonymous_node" {
                //     out.write_str(parent_indent)?;
                //     // write!(out, "~{}~", children.len())?;
                // } else if kind.as_static_str() == "parameters" {
                //     // out.write_str(" ")?;
                // } else if kind.as_static_str() == "predicate" {
                //     out.write_str(" ")?;
                // } else if kind.as_static_str() == "capture" {
                //     out.write_str("")?;
                // } else {
                //     out.write_str("___")?;
                //     out.write_str(&kind.to_string())?;
                //     out.write_str("___")?;
                //     out.write_str(parent_indent)?;
                // }
                let mut it = children;
                let mut ind = indent.to_string();

                match kind.as_static_str() {
                    "parameters" => ind = " ".to_string(),
                    // "named_node" if children.len() <= 3 => parent_indent = "".to_string(),
                    _ => (),
                }

                use hyperast::types::WithStats;
                // let len = it.len();
                let len = b.size();
                // let len = b.try_bytes_len().unwrap_or(0);
                for _i in 0..it.len() {
                    let id = it.next().unwrap();
                    match self.serialize(&id, &parent_indent, &ind, out) {
                        Ok(_ind) if kind.as_static_str() == "named_node" && len <= 4 => {
                            // Ok(_ind) if kind.as_static_str() == "named_node" && len <= 40 => {
                            parent_indent = "".to_string();
                            ind = "\n  ".to_string();
                            // parent_indent = ind;
                            // ind = _ind;
                        }
                        Ok(_ind) if kind.as_static_str() == "predicate" => {
                            // parent_indent = "c'".to_string();
                            parent_indent = "".to_string();
                            ind = "c".to_string();
                        }
                        Ok(_ind) if parent_indent.is_empty() && ind.is_empty() => {
                            parent_indent = "\n".to_string();
                            ind = format!("\n{_ind}");
                            // parent_indent = format!("{ind}d'");
                            // ind = format!("{_ind}d");
                        }
                        Ok(_ind) => {
                            parent_indent = ind;
                            ind = _ind;
                            // parent_indent = format!("{ind}d'");
                            // ind = format!("{_ind}d");
                        }
                        // Err(IndentedAlt::NoIndent) => ind = indent.to_string(),
                        Err(IndentedAlt::NoIndent) => (),
                        Err(e) => return Err(e),
                    };
                }
                // Ok(ind)
                Err(IndentedAlt::NoIndent)
                // Ok(indent.to_string())
            }
            (Some(label), None) => {
                // out.write_str("{")?;
                // out.write_str(&kind.to_string())?;
                // // out.write_str(&format!("{parent_indent:?}"))?;
                // out.write_str("}")?;
                let s = self.stores.label_store().resolve(label);
                out.write_str(s)?;

                // out.write_str("--")?;
                // out.write_str(&kind.to_string())?;
                // out.write_str("--")?;

                let mut ind = indent.to_string();
                if kind.as_static_str() == "identifier" {
                    ind.push_str("  ")
                }
                match kind.as_static_str() {
                    "identifier" => Ok(ind),
                    _ => Err(IndentedAlt::NoIndent),
                }
                // Ok(ind)
                // Err(IndentedAlt::NoIndent)
            }
        };
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pp_name_node() -> Result<(), ()> {
        let text = r#"
(block)
    "#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_name_nodes() -> Result<(), ()> {
        let text = r#"
(block)

(block)
        "#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_capture() -> Result<(), ()> {
        let text = r#"
(block)@a
        "#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_nested() -> Result<(), ()> {
        let text = r#"
(block
  (local_variable_declaration
    (variable_declarator
      (identifier)
      (string_literal)
    )
  )
)
"#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_nested2() -> Result<(), ()> {
        let text = r#"
(block
  (local_variable_declaration
    (variable_declarator
      (identifier)
      (string_literal)
    )
  )
  (local_variable_declaration)
)
"#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_predicate() -> Result<(), ()> {
        let text = r#"
(block
  (local_variable_declaration
    (variable_declarator
      (identifier) (#EQ? "input")
      (string_literal)
    )
  )
)"#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_predicate_eq() -> Result<(), ()> {
        let text = r#"
(block
  (local_variable_declaration
    (variable_declarator
      (identifier) @a
      (string_literal) @b
    ) (#eq? @a @b)
  )
)
    "#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_predicate_end() -> Result<(), ()> {
        let text = r#"
(block
  (local_variable_declaration
    (variable_declarator
      (identifier) @a
      (string_literal) @b
    )
  )
) (#eq? @a @b)
"#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_large() -> Result<(), ()> {
        let text = r#"
(block
  (local_variable_declaration
    (variable_declarator
      (identifier) (#EQ? "input")
      (string_literal)
    )
  )
  (local_variable_declaration
    (type_identifier)
    (variable_declarator
      (identifier) (#EQ? "close")
      (string_literal)
    )
  )
  (expression_statement
    (method_invocation
      (identifier) (#EQ? "assertNull")
    )
  )
)@_root

(block)
    "#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_choice() -> Result<(), ()> {
        let text = r#"
[
  (expression_statement)
  (local_variable_declaration)
] @x
            "#
        .trim();
        identity_check(text)
    }

    /// fails. I need to redo the serialization state machine
    #[test]
    fn test_pp_supertype() -> Result<(), ()> {
        let text = r#"
(_literal/string_literal)
        "#
        .trim();
        identity_check(text)
    }

    #[test]
    fn test_pp_more_complex() -> Result<(), ()> {
        let text = r#"
(block
  (expression_statement
    (method_invocation
      (identifier) (#EQ? "assertEquals")
    )
  )
  (expression_statement
    (method_invocation
      (identifier) (#EQ? "assertEquals")
      (argument_list
        (string_literal)
        (array_access
          (identifier) (#EQ? "result")
          (decimal_integer_literal)
        )
      )
    )
  )
) @_root
        "#
        .trim();
        identity_check(text)
    }

    fn identity_check(text: &str) -> Result<(), ()> {
        use crate::types::TStore;
        let mut query_store = SimpleStores::<TStore>::default();
        let mut md_cache = Default::default();
        let mut query_tree_gen = TsQueryTreeGen::new(&mut query_store, &mut md_cache);
        let tree = match crate::tree_sitter_parse(text.as_bytes()) {
            Ok(t) => t,
            Err(t) => {
                log::warn!("Error parsing query: {}", t.root_node().to_sexp());
                return Err(());
            }
        };
        let full_node = query_tree_gen.generate_file(b"", text.as_bytes(), tree.walk());
        let root = full_node.local.compressed_node;
        let p = PP::<_, _>::new(&query_store, root);
        println!("{p}");
        pretty_assertions::assert_str_eq!(text, p.to_string());

        Ok(())
    }
}
