use std::collections::HashMap;
use std::sync::Arc;

use tree_sitter::CaptureQuantifier as Quant;

use crate::auto::tsq_ser_meta::Conv;
use crate::no_fmt_legion::TsQueryTreeGen;
use crate::types::TStore;

use hyperast::store::SimpleStores;
use hyperast::store::nodes::legion::NodeIdentifier;
use hyperast::types::{HyperAST, Labeled};

mod preprocess;

mod recursive;
pub mod recursive2;

mod iterative;

// pub mod steped;

#[doc(hidden)]
pub mod utils;

// for now just uses the root types
// TODO implement approaches based on probabilitic sets
pub(crate) struct QuickTrigger<T> {
    pub(crate) root_types: Arc<[T]>,
}

pub struct PreparedMatcher<Ty, C = Conv<Ty>> {
    pub(crate) quick_trigger: QuickTrigger<Ty>,
    pub(crate) patterns: Arc<[Pattern<Ty>]>,
    pub captures: Arc<[Capture]>,
    pub(crate) quantifiers: Arc<[HashMap<usize, tree_sitter::CaptureQuantifier>]>,
    #[allow(unused)] // TODO remove entire file, now there is the port of the original
    converter: C,
}

#[derive(Debug)]
pub struct Capture {
    pub name: String,
}

impl<Ty: std::fmt::Debug, C> std::fmt::Debug for PreparedMatcher<Ty, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedMatcher")
            .field("quick_trigger", &self.quick_trigger.root_types)
            .field("patterns", &self.patterns)
            .finish()
    }
}
impl<Ty, C> PreparedMatcher<Ty, C> {
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn capture_index_for_name(&self, name: &str) -> Option<u32> {
        dbg!(self.captures.len());
        dbg!(&self.captures[..10.min(self.captures.len())]);
        dbg!(name);
        self.captures
            .iter()
            .position(|x| x.name == name)
            .map(|x| x as u32)
    }

    pub fn capture_quantifiers(
        &self,
        index: usize,
    ) -> (impl std::ops::Index<usize, Output = tree_sitter::CaptureQuantifier> + '_) {
        // struct A([tree_sitter::CaptureQuantifier]);
        // impl std::ops::Index<usize> for &A {
        //     type Output = tree_sitter::CaptureQuantifier;

        //     fn index(&self, index: usize) -> &Self::Output {
        //         self.0.get(index).unwrap_or(&Quant::Zero)
        //     }
        // }
        // let left = self.quantifiers_skips[index];
        // let right = self.quantifiers_skips.get(index + 1).copied().unwrap();
        // let s = &self.quantifiers[left..right];
        // let s: &A = unsafe { std::mem::transmute(s) };
        // s
        struct A(HashMap<usize, tree_sitter::CaptureQuantifier>);
        impl std::ops::Index<usize> for &A {
            type Output = tree_sitter::CaptureQuantifier;

            fn index(&self, index: usize) -> &Self::Output {
                self.0.get(&index).unwrap_or(&Quant::Zero)
            }
        }
        let s = &self.quantifiers[index];
        let s: &A = unsafe { std::mem::transmute(s) };
        s
    }
}

#[derive(Debug)]
pub struct Captured<IdN, Idx>(pub Vec<CaptureRes<IdN, Idx>>, usize);
impl<IdN, Idx> Captured<IdN, Idx> {
    pub fn by_capture_id(&self, id: CaptureId) -> Option<&CaptureRes<IdN, Idx>> {
        captures(&self.0, id).next()
    }
    pub fn pattern_index(&self) -> usize {
        self.1
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Pattern<Ty> {
    NamedNode {
        ty: Ty,
        children: Arc<[Pattern<Ty>]>,
    },
    SupNamedNode {
        sup: Ty,
        ty: Ty,
        children: Arc<[Pattern<Ty>]>,
    },
    AnonymousNode(Ty),
    Capture {
        name: CaptureId,
        pat: Arc<Pattern<Ty>>,
    },
    Predicated {
        predicate: Predicate,
        pat: Arc<Pattern<Ty>>,
    },
    AnyNode {
        children: Arc<[Pattern<Ty>]>,
    },
    List(Arc<[Pattern<Ty>]>),
    FieldDefinition {
        name: Field,
        pat: Arc<Pattern<Ty>>,
    },
    Dot,
    Quantified {
        quantifier: tree_sitter::CaptureQuantifier,
        pat: Arc<Pattern<Ty>>,
    },
    NegatedField(Field),
}

impl<Ty> Pattern<Ty> {
    pub(crate) fn unwrap_captures(&self) -> &Self {
        match self {
            Pattern::Capture { name: _, pat } => pat.unwrap_captures(),
            x => x,
        }
    }
    pub(crate) fn is_any_node(&self) -> bool {
        match self {
            Pattern::AnyNode { .. } => true,
            _ => false,
        }
    }

    pub(crate) fn is_anonymous(&self) -> bool {
        match self {
            Pattern::AnonymousNode { .. } => true,
            _ => false,
        }
    }

    pub(crate) fn is_optional_match(&self) -> bool {
        match self {
            Pattern::NamedNode { .. }
            | Pattern::SupNamedNode { .. }
            | Pattern::AnyNode { .. }
            | Pattern::Dot
            | Pattern::NegatedField(_)
            | Pattern::List(_)
            | Pattern::AnonymousNode(_) => false,
            Pattern::FieldDefinition { pat, .. } | Pattern::Capture { pat, .. } => {
                pat.is_optional_match()
            }
            Pattern::Predicated { .. } => todo!(),
            Pattern::Quantified { quantifier: q, .. } => {
                *q == Quant::Zero || *q == Quant::ZeroOrMore || *q == Quant::ZeroOrOne
            }
        }
    }
}

type Field = String;

type CaptureId = u32;

#[derive(Debug, Clone)]
pub(crate) enum Predicate<I = CaptureId> {
    Eq { left: I, right: I },
    EqString { left: I, right: String },
}

impl Predicate<String> {
    fn resolve_name(self, captures: &[Capture]) -> Predicate<CaptureId> {
        match self {
            Predicate::Eq { left, right } => {
                for i in 0..captures.len() {
                    if captures[i].name == left {
                        let left = i as u32;
                        for i in i..captures.len() {
                            if captures[i].name == right {
                                let right = i as u32;
                                return Predicate::Eq { left, right };
                            }
                        }
                    } else if captures[i].name == right {
                        let right = i as u32;
                        for i in i..captures.len() {
                            if captures[i].name == left {
                                let left = i as u32;
                                return Predicate::Eq { left, right };
                            }
                        }
                    }
                }
                panic!(
                    "{} and {} cannot be resolved in {:?}",
                    left, right, captures
                );
            }
            Predicate::EqString { left, right } => {
                for i in 0..captures.len() {
                    if captures[i].name == left {
                        let left = i as u32;
                        return Predicate::EqString { left, right };
                    }
                }
                panic!("{} cannot be resolved in {:?}", left, captures);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct MatchingRes<IdN = NodeIdentifier, Idx = u16> {
    matched: tree_sitter::CaptureQuantifier,
    pub(crate) captures: Vec<CaptureRes<IdN, Idx>>,
}

impl<IdN, Idx> MatchingRes<IdN, Idx> {
    fn zero() -> Self {
        Self {
            matched: Quant::Zero,
            captures: Default::default(),
        }
    }

    fn capture(&self, id: CaptureId) -> Option<&CaptureRes<IdN, Idx>> {
        captures(&self.captures, id).next()
    }
}

fn captures<IdN, Idx>(
    c: &[CaptureRes<IdN, Idx>],
    id: CaptureId,
) -> impl Iterator<Item = &CaptureRes<IdN, Idx>> {
    c.iter().filter(move |x| x.id == id)
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct CaptureRes<IdN = NodeIdentifier, Idx = u16> {
    pub id: CaptureId,
    pub match_node: IdN,
    pub path: Vec<Idx>,
}

impl CaptureRes {
    #[deprecated]
    pub fn try_label_old(self) -> Option<String> {
        unimplemented!("refactor code using that")
    }
}

impl<IdN, Idx> CaptureRes<IdN, Idx> {
    pub fn try_label<'store, HAST>(&self, store: &'store HAST) -> Option<&'store str>
    where
        HAST: HyperAST<IdN = IdN, Idx = Idx>,
    {
        use hyperast::types::LabelStore;
        use hyperast::types::NodeStore;
        let n = store.node_store().resolve(&self.match_node);
        let l = n.try_get_label()?;
        let l = store.label_store().resolve(l);
        Some(l)
    }
}

pub fn ts_query_store() -> SimpleStores<crate::types::TStore> {
    SimpleStores::default()
}

pub fn ts_query(text: &[u8]) -> (SimpleStores<crate::types::TStore>, legion::Entity) {
    let mut stores = ts_query_store();
    let query = ts_query2(&mut stores, text);
    (stores, query)
}

pub fn ts_query2(stores: &mut SimpleStores<TStore>, text: &[u8]) -> legion::Entity {
    let mut md_cache = Default::default();
    let mut query_tree_gen = TsQueryTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores,
        md_cache: &mut md_cache,
    };

    let tree = match crate::legion::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => {
            eprintln!("{}", t.root_node().to_sexp());
            t
        }
    };
    let full_node = query_tree_gen.generate_file(b"", text, tree.walk());

    full_node.local.compressed_node
}

pub fn ts_query2_with_label_hash(
    stores: &mut SimpleStores<TStore>,
    text: &[u8],
) -> Option<(legion::Entity, u32)> {
    let mut md_cache = Default::default();
    let mut query_tree_gen = TsQueryTreeGen::new(stores, &mut md_cache);

    let tree = match crate::legion::tree_sitter_parse(text) {
        Ok(t) => t,
        Err(t) => {
            dbg!(t.root_node().to_sexp());
            return None;
        }
    };
    // dbg!(tree.root_node().to_sexp());
    let full_node = query_tree_gen.generate_file(b"", text, tree.walk());
    // eprintln!(
    //     "{}",
    //     hyperast::nodes::SyntaxSerializer::new(stores, full_node.local.compressed_node)
    // );
    let r = (
        full_node.local.compressed_node,
        full_node.local.metrics.hashs.label,
    );
    Some(r)
}
