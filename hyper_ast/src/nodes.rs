use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Write},
    hash::Hash,
    marker::PhantomData,
};

use num::ToPrimitive;

use crate::{
    impact::serialize::{Keyed, MySerialize},
    types::{HyperType, IterableChildren, MySlice, NodeId},
};

// pub type TypeIdentifier = Type;

pub trait RefContainer {
    type Result;
    fn check<U: MySerialize + Keyed<usize>>(&self, rf: U) -> Self::Result;
}

/// identifying data for a node in an HyperAST
// pub struct SimpleNode1<Child, Label> {
//     pub(crate) kind: TypeIdentifier,
//     pub(crate) label: Option<Label>,
//     pub(crate) children: Vec<Child>,
// }

// pub type DefaultLabelIdentifier = DefaultSymbol;
// pub type DefaultNodeIdentifier = legion::Entity;
pub type HashSize = u32;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub enum Space {
    Space,
    // LineBreak,
    NewLine,
    CariageReturn,
    Tabulation,
    ParentIndentation,
}

#[derive(Debug, Clone)]
pub enum CompressedNode<NodeId, LabelId, Type> {
    Type(Type),
    Label { label: LabelId, kind: Type },
    Children2 { children: [NodeId; 2], kind: Type },
    Children { children: Box<[NodeId]>, kind: Type },
    Spaces(LabelId), //Box<[Space]>),
}

// pub(crate) enum SimpNode<NodeId, LabelId> {
//     Type(Type),
//     Label { label: LabelId, kind: Type },
//     Children { children: Box<[NodeId]>, kind: Type },
//     Spaces(Box<[Space]>),
// }

// mod type_baggable_nodes {
//     use std::marker::PhantomData;

//     struct Keyword<Type> {
//         kind: Type,
//     }

//     struct UnsizedNode<Type, NodeId, LabelId> {
//         // kind: Type,
//         _phantom: PhantomData<*const (Type, NodeId, LabelId)>,
//         bytes: [u8],
//         // children: [MyUnion<LabelId,NodeId>],
//     }
// }

// #[repr(C)]
// union MyUnion<NodeId, LabelId> {
//     node: std::mem::ManuallyDrop<NodeId>,
//     label: std::mem::ManuallyDrop<LabelId>,
// }

impl<N: PartialEq, L: PartialEq, T: PartialEq> PartialEq for CompressedNode<N, L, T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Type(l0), Self::Type(r0)) => l0 == r0,
            (
                Self::Label {
                    label: l_label,
                    kind: l_kind,
                },
                Self::Label {
                    label: r_label,
                    kind: r_kind,
                },
            ) => l_label == r_label && l_kind == r_kind,
            (
                Self::Children2 {
                    children: l_children,
                    kind: l_kind,
                },
                Self::Children2 {
                    children: r_children,
                    kind: r_kind,
                },
            ) => l_children == r_children && l_kind == r_kind,
            (
                Self::Children {
                    children: l_children,
                    kind: l_kind,
                },
                Self::Children {
                    children: r_children,
                    kind: r_kind,
                },
            ) => l_children == r_children && l_kind == r_kind,
            (Self::Spaces(l0), Self::Spaces(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl<N: Eq, L: Eq, T: Eq> Eq for CompressedNode<N, L, T> {}

impl<N, L, T> CompressedNode<N, L, T> {
    pub fn new(kind: T, label: Option<L>, children: Vec<N>) -> Self {
        if children.len() > 2 {
            Self::Children {
                kind,
                children: children.into_boxed_slice(),
            }
        } else if children.len() == 2 {
            let mut it = children.into_iter();
            Self::Children2 {
                kind,
                children: [it.next().unwrap(), it.next().unwrap()],
            }
        } else if children.len() > 0 {
            // TODO Children2 Optional child2 might be better
            Self::Children {
                kind,
                children: children.into_boxed_slice(),
            }
        } else if let Some(label) = label {
            Self::Label { kind, label }
        } else {
            Self::Type(kind)
        }
    }
}

// CompressedNode

impl<N, L, T: HyperType + Copy + Hash + Eq + Send + Sync> crate::types::Typed
    for CompressedNode<N, L, T>
{
    type Type = T;

    fn get_type(&self) -> T {
        match self {
            CompressedNode::Type(kind) => *kind,
            CompressedNode::Label { label: _, kind } => *kind,
            CompressedNode::Children2 { children: _, kind } => *kind,
            CompressedNode::Children { children: _, kind } => *kind,
            CompressedNode::Spaces(_) => todo!("what is the generic version of Type::Spaces ?"),
        }
    }
}

impl<N, L: Eq, T> crate::types::Labeled for CompressedNode<N, L, T> {
    type Label = L;

    fn get_label_unchecked(&self) -> &L {
        match self {
            CompressedNode::Label { label, kind: _ } => label,
            _ => panic!(),
        }
    }

    fn try_get_label<'a>(&'a self) -> Option<&'a L> {
        todo!()
    }
}

impl<N: Eq + Clone + NodeId<IdN = N>, L, T> crate::types::WithChildren for CompressedNode<N, L, T> {
    type ChildIdx = u16;
    type Children<'a> = MySlice<N> where Self: 'a;
    // type Children<'a> = [N] where Self:'a;

    fn child_count(&self) -> Self::ChildIdx {
        match self {
            CompressedNode::Children2 {
                children: _,
                kind: _,
            } => 2,
            CompressedNode::Children { children, kind: _ } => children.len().to_u16().unwrap(),
            _ => 0,
        }
    }

    fn child(&self, idx: &Self::ChildIdx) -> Option<N> {
        match self {
            CompressedNode::Children2 { children, kind: _ } if *idx == 0 => {
                Some(children[0].as_id().clone())
            }
            CompressedNode::Children2 { children, kind: _ } if *idx == 1 => {
                Some(children[1].as_id().clone())
            }
            CompressedNode::Children { children, kind: _ } => {
                Some(children[*idx as usize].as_id().clone())
            }
            _ => None,
        }
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<N> {
        match self {
            CompressedNode::Children2 { children, kind: _ } if *idx == 1 => {
                Some(children[0].as_id().clone())
            }
            CompressedNode::Children2 { children, kind: _ } if *idx == 0 => {
                Some(children[1].as_id().clone())
            }
            CompressedNode::Children { children, kind: _ } => Some(
                children[children.len() - 1 - (*idx as usize)]
                    .as_id()
                    .clone(),
            ),
            _ => None,
        }
    }

    // fn children_unchecked<'a>(&'a self) -> &[N] {
    //     match self {
    //         CompressedNode::Children2 { children, kind: _ } => &*children,
    //         CompressedNode::Children { children, kind: _ } => &*children,
    //         _ => &[],
    //     }
    // }

    // fn get_children_cpy<'a>(&'a self) -> Vec<N> {
    //     match self {
    //         CompressedNode::Children2 { children, kind: _ } => children.to_vec(),
    //         CompressedNode::Children { children, kind: _ } => children.to_vec(),
    //         _ => vec![],
    //     }
    // }

    fn children<'a>(&'a self) -> Option<&'a <Self as crate::types::WithChildren>::Children<'a>> {
        fn f<'a, N, L, T>(x: &'a CompressedNode<N, L, T>) -> &'a [N] {
            match x {
                CompressedNode::Children2 { children, kind: _ } => &*children,
                CompressedNode::Children { children, kind: _ } => &**children,
                _ => &[],
            }
        }
        // TODO check if it work, not sure
        Some(f(self).into())
    }
}

impl<N, L, T> crate::types::Node for CompressedNode<N, L, T> {}
impl<N: NodeId + Eq, L, T> crate::types::Stored for CompressedNode<N, L, T> {
    type TreeId = N;
}

impl<N: NodeId<IdN = N> + Eq + Clone, L: Eq, T: Copy + Hash + Eq + HyperType + Send + Sync>
    crate::types::Tree for CompressedNode<N, L, T>
where
    N::IdN: Copy,
{
    fn has_children(&self) -> bool {
        match self {
            CompressedNode::Children2 {
                children: _,
                kind: _,
            } => true,
            CompressedNode::Children {
                children: _,
                kind: _,
            } => true,
            _ => false,
        }
    }

    fn has_label(&self) -> bool {
        match self {
            CompressedNode::Label { label: _, kind: _ } => true,
            _ => false,
        }
    }
}

// impl Hash for CompressedNode {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         match self {
//             CompressedNode::Type(k) => k.hash(state),
//             CompressedNode::Label { kind, label } => {}
//             CompressedNode::Children2 {
//                 kind,
//                 child1,
//                 child2,
//             } => {
//                 let size = 0;
//                 let middle_hash = 0;

//                 let mut k = DefaultHasher::new();
//                 kind.hash(&mut k);
//                 state.write_u32(innerNodeHash(&(((k.finish() & 0xffff0000) >> 32) as u32), &0, &size, &middle_hash));
//             }
//             CompressedNode::Children { kind, children } => {
//                 kind.hash(state);
//                 // children.
//             }
//             CompressedNode::Spaces(s) => s.hash(state),
//         }
//     }
// }

// Spaces

impl Display for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Space::Space => write!(f, "s"),
            Space::NewLine => write!(f, "n"),
            Space::CariageReturn => write!(f, "r"),
            Space::Tabulation => write!(f, "t"),
            Space::ParentIndentation => write!(f, "0"),
        }
    }
}

impl Space {
    pub fn fmt<W: Write>(&self, w: &mut W, p: &str) -> std::fmt::Result {
        match self {
            Space::Space => write!(w, " "),
            Space::NewLine => write!(w, "\n"),
            Space::CariageReturn => write!(w, "\r"),
            Space::Tabulation => write!(w, "\t"),
            Space::ParentIndentation => write!(w, "{}", p),
        }
    }
}

impl Space {
    pub fn to_string(&self) -> &str {
        match self {
            Space::Space => " ",
            Space::NewLine => "\n",
            Space::CariageReturn => "\r",
            Space::Tabulation => "\t",
            Space::ParentIndentation => "0",
        }
    }
}
// impl Deref for Spaces {

// }

impl Debug for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Space::Space => write!(f, "s"),
            Space::NewLine => write!(f, "n"),
            Space::CariageReturn => write!(f, "r"),
            Space::Tabulation => write!(f, "t"),
            Space::ParentIndentation => write!(f, "0"),
        }
    }
}
const NL: char = '\n';
const CR: char = '\r';
impl Space {
    pub fn format_indentation(spaces: &[u8]) -> Vec<Space> {
        spaces
            .iter()
            .map(|x| match *x as char {
                ' ' => Space::Space,
                '\u{000C}' => Space::Space,
                NL => Space::NewLine,
                '\t' => Space::Tabulation,
                CR => Space::CariageReturn,
                x => {
                    log::debug!("{:?}", x);
                    // log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                    dbg!(std::str::from_utf8(spaces));
                    // panic!("{:?}", spaces)
                    Space::Space
                }
            })
            .collect()
    }
    pub fn try_format_indentation(spaces: &[u8]) -> Option<Vec<Space>> {
        let mut err = false;
        let r = spaces
            .iter()
            .map_while(|x| match *x as char {
                ' ' => Some(Space::Space),
                '\u{000C}' => Some(Space::Space),
                NL => Some(Space::NewLine),
                '\t' => Some(Space::Tabulation),
                CR => Some(Space::CariageReturn),
                x => {
                    log::debug!("{:?}", x);
                    err = true;
                    None
                }
            })
            .collect();
        if err {
            None
        } else {
            Some(r)
        }
    }
    /// TODO test with nssss, n -> n
    pub fn replace_indentation(indentation: &[Space], spaces: &[Space]) -> Vec<Space> {
        let mut r = vec![];
        let mut tmp = vec![];
        let mut i = 0;
        for x in spaces {
            tmp.push(*x);
            if i < indentation.len() && indentation[i] == *x {
                i += 1;
                if i == indentation.len() {
                    r.push(Space::ParentIndentation);
                    tmp.clear();
                }
            } else {
                i = 0;
                r.extend_from_slice(&*tmp);
                tmp.clear();
            }
        }
        r.extend_from_slice(&*tmp);
        r
    }

    // pub(crate) fn replace_indentation(indentation: &[Spaces], spaces: &[Spaces]) -> Vec<Spaces> {
    //     if spaces.len() < indentation.len() {
    //         return spaces.to_vec();
    //     }
    //     if indentation.len() == 0 {
    //         assert!(false);
    //     }
    //     let mut it = spaces.windows(indentation.len());
    //     let mut r: Vec<Spaces> = vec![];
    //     let mut tmp: &[Spaces] = &[];
    //     loop {
    //         match it.next() {
    //             Some(x) => {
    //                 if x == indentation {
    //                     r.push(Spaces::ParentIndentation);
    //                     for _ in 0..indentation.len()-1 {
    //                         it.next();
    //                     }
    //                     tmp = &[];
    //                 } else {
    //                     if tmp.len()>0 {
    //                         r.push(tmp[0]);
    //                     }
    //                     tmp = x;
    //                 }
    //             }
    //             None => {
    //                 r.extend(tmp);
    //                 return r
    //             },
    //         }
    //     }
    // }
}

pub struct IoOut<W: std::io::Write> {
    stream: W,
}

impl<W: std::io::Write> From<W> for IoOut<W> {
    fn from(stream: W) -> Self {
        Self { stream }
    }
}

impl<W: std::io::Write> std::fmt::Write for IoOut<W> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.stream
            .write_all(s.as_bytes())
            .map_err(|_| std::fmt::Error)
    }
}

pub type StructureSerializer<'a, 'b, IdN, HAST> =
    SimpleSerializer<'a, IdN, HAST, true, false, false, false>;
pub type LabelSerializer<'a, 'b, IdN, HAST> =
    SimpleSerializer<'a, IdN, HAST, true, true, false, false>;
pub type IdsSerializer<'a, 'b, IdN, HAST> =
    SimpleSerializer<'a, IdN, HAST, false, false, true, false>;
pub type SyntaxSerializer<'a, 'b, IdN, HAST, const SPC: bool = false> =
    SimpleSerializer<'a, IdN, HAST, true, true, false, true>;
pub type SyntaxWithIdsSerializer<'a, 'b, IdN, HAST, const SPC: bool = false> =
    SimpleSerializer<'a, IdN, HAST, true, true, true, SPC>;

pub struct SimpleSerializer<
    'a,
    IdN,
    HAST,
    const TY: bool = true,
    const LABELS: bool = false,
    const IDS: bool = false,
    const SPC: bool = false,
> {
    stores: &'a HAST,
    root: IdN,
}

impl<'store, IdN, HAST, const TY: bool, const LABELS: bool, const IDS: bool, const SPC: bool>
    SimpleSerializer<'store, IdN, HAST, TY, LABELS, IDS, SPC>
{
    pub fn new(stores: &'store HAST, root: IdN) -> Self {
        Self { stores, root }
    }
}

impl<'store, IdN, HAST, const TY: bool, const LABELS: bool, const IDS: bool, const SPC: bool>
    Display for SimpleSerializer<'store, IdN, HAST, TY, LABELS, IDS, SPC>
where
    IdN: NodeId<IdN = IdN> + Debug,
    HAST: crate::types::NodeStore<IdN>,
    HAST: crate::types::LabelStore<str>,
    HAST: crate::types::TypeStore<HAST::R<'store>>,
    HAST::R<'store>:
        crate::types::Labeled<Label = HAST::I>
        + crate::types::WithChildren<TreeId = IdN>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.serialize(&self.root, f)
    }
}

impl<'store, IdN, HAST, const TY: bool, const LABELS: bool, const IDS: bool, const SPC: bool>
    SimpleSerializer<'store, IdN, HAST, TY, LABELS, IDS, SPC>
where
    IdN: NodeId<IdN = IdN> + Debug,
    HAST: crate::types::NodeStore<IdN>,
    HAST: crate::types::LabelStore<str>,
    HAST: crate::types::TypeStore<HAST::R<'store>>,
    HAST::R<'store>: crate::types::Labeled<Label = HAST::I>
        + crate::types::WithChildren<TreeId = IdN>,
{
    // pub fn tree_syntax_with_ids(
    fn serialize(
        &self,
        id: &IdN,
        out: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        use crate::types::LabelStore;
        use crate::types::Labeled;
        use crate::types::NodeStore;
        use crate::types::WithChildren;
        let b = NodeStore::resolve(self.stores, id);
        // let kind = (self.stores.type_store(), b);
        let kind = self.stores.resolve_type(&b);
        let label = b.try_get_label();
        let children = b.children();

        if kind.is_spaces() {
            if SPC {
                let s = LabelStore::resolve(self.stores, &label.unwrap());
                let b: String = Space::format_indentation(s.as_bytes())
                    .iter()
                    .map(|x| x.to_string())
                    .collect();
                write!(out, "(")?;
                if IDS { write!(out, "{:?}", id) } else { Ok(()) }.and_then(|x| {
                    if TY {
                        write!(out, "_",)
                    } else {
                        Ok(x)
                    }
                })?;
                if LABELS {
                    write!(out, " {:?}", Space::format_indentation(b.as_bytes()))?;
                }
                write!(out, ")")?;
            }
            return Ok(());
        }

        let w_kind = |out: &mut std::fmt::Formatter<'_>| {
            if IDS { write!(out, "{:?}", id) } else { Ok(()) }.and_then(|x| {
                if TY {
                    write!(out, "{}", kind.to_string())
                } else {
                    Ok(x)
                }
            })
        };

        match (label, children) {
            (None, None) => {
                w_kind(out)?;
            }
            (label, Some(children)) => {
                if let Some(label) = label {
                    let s = LabelStore::resolve(self.stores, label);
                    if LABELS {
                        write!(out, " {:?}", Space::format_indentation(s.as_bytes()))?;
                    }
                }
                if !children.is_empty() {
                    let it = children.iter_children();
                    write!(out, "(")?;
                    w_kind(out)?;
                    for id in it {
                        self.serialize(&id, out)?;
                    }
                    write!(out, ")")?;
                }
            }
            (Some(label), None) => {
                write!(out, "(")?;
                w_kind(out)?;
                if LABELS {
                    let s = LabelStore::resolve(self.stores, label);
                    if s.len() > 20 {
                        write!(out, "='{}...'", &s[..20])?;
                    } else {
                        write!(out, "='{}'", s)?;
                    }
                }
                write!(out, ")")?;
            }
        }
        return Ok(());
    }
}

fn escape(src: &str) -> String {
    let mut escaped = String::with_capacity(src.len());
    let mut utf16_buf = [0u16; 2];
    for c in src.chars() {
        match c {
            ' ' => escaped += " ",
            '\x08' => escaped += "\\b",
            '\x0c' => escaped += "\\f",
            '\n' => escaped += "\\n",
            '\r' => escaped += "\\r",
            '\t' => escaped += "\\t",
            '"' => escaped += "\\\"",
            '\\' => escaped += "\\\\",
            c if c.is_ascii_graphic() => escaped.push(c),
            c => {
                let encoded = c.encode_utf16(&mut utf16_buf);
                for utf16 in encoded {
                    write!(&mut escaped, "\\u{:04X}", utf16).unwrap();
                }
            }
        }
    }
    escaped
}

pub struct Json;
pub struct Text;

pub trait Format {}
impl Format for Json {}
impl Format for Text {}

pub type JsonSerializer<'a, 'b, IdN, HAST, const SPC: bool> =
    IndentedSerializer<'a, 'b, IdN, HAST, Json, SPC>;
pub type TextSerializer<'a, 'b, IdN, HAST> = IndentedSerializer<'a, 'b, IdN, HAST, Text, true>;

pub struct IndentedSerializer<'a, 'b, IdN, HAST, Fmt: Format = Text, const SPC: bool = false> {
    stores: &'a HAST,
    root: IdN,
    root_indent: &'b str,
    phantom: PhantomData<Fmt>,
}

impl<'store, 'b, IdN, HAST, Fmt: Format, const SPC: bool>
    IndentedSerializer<'store, 'b, IdN, HAST, Fmt, SPC>
{
    pub fn new(stores: &'store HAST, root: IdN) -> Self {
        Self {
            stores,
            root,
            root_indent: "\n",
            phantom: PhantomData,
        }
    }
}

impl<'store, 'b, IdN, HAST, const SPC: bool> Display
    for IndentedSerializer<'store, 'b, IdN, HAST, Text, SPC>
where
    IdN: NodeId<IdN = IdN>,
    HAST: crate::types::NodeStore<IdN>,
    HAST: crate::types::LabelStore<str>,
    HAST: crate::types::TypeStore<HAST::R<'store>>,
    HAST::R<'store>:
        crate::types::Labeled<Label = HAST::I> + crate::types::WithChildren<TreeId = IdN>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.serialize(&self.root, &self.root_indent, f) {
            Err(IndentedAlt::FmtError) => Err(std::fmt::Error),
            _ => Ok(()),
        }
    }
}

impl<'store, 'b, IdN, HAST, const SPC: bool> Display
    for IndentedSerializer<'store, 'b, IdN, HAST, Json, SPC>
where
    IdN: NodeId<IdN = IdN>,
    HAST: crate::types::NodeStore<IdN>,
    HAST: crate::types::LabelStore<str>,
    HAST: crate::types::TypeStore<HAST::R<'store>>,
    HAST::R<'store>:
        crate::types::Labeled<Label = HAST::I> + crate::types::WithChildren<TreeId = IdN>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.serialize(&self.root, &self.root_indent, f) {
            Err(IndentedAlt::FmtError) => Err(std::fmt::Error),
            _ => Ok(()),
        }
    }
}

impl<'store, 'b, IdN, HAST, const SPC: bool> IndentedSerializer<'store, 'b, IdN, HAST, Text, SPC>
where
    IdN: NodeId<IdN = IdN>,
    HAST: crate::types::NodeStore<IdN>,
    HAST: crate::types::LabelStore<str>,
    HAST: crate::types::TypeStore<HAST::R<'store>>,
    HAST::R<'store>:
        crate::types::Labeled<Label = HAST::I> + crate::types::WithChildren<TreeId = IdN>,
{
    fn serialize(
        &self,
        id: &IdN,
        parent_indent: &str,
        out: &mut std::fmt::Formatter<'_>,
    ) -> Result<String, IndentedAlt> {
        use crate::types::LabelStore;
        use crate::types::Labeled;
        use crate::types::NodeStore;
        use crate::types::Typed;
        use crate::types::WithChildren;
        let b = NodeStore::resolve(self.stores, id);
        // let kind = (self.stores.type_store(), b);
        let kind = self.stores.resolve_type(&b);
        let label = b.try_get_label();
        let children = b.children();

        if kind.is_spaces() {
            let indent = if let Some(label) = label {
                let s = LabelStore::resolve(self.stores, label);
                let b: String = Space::format_indentation(s.as_bytes())
                    .iter()
                    .map(|x| x.to_string())
                    .collect();
                out.write_str(&b).unwrap();
                if b.contains("\n") {
                    b
                } else {
                    parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned()
                }
            } else {
                parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned()
            };
            return Ok(indent);
        }

        match (label, children) {
            (None, None) => {
                out.write_str(&kind.to_string()).unwrap();
                Err(IndentedAlt::NoIndent)
            }
            (label, Some(children)) => {
                if let Some(label) = label {
                    let s = LabelStore::resolve(self.stores, label);
                    dbg!(s);
                }
                if !children.is_empty() {
                    let mut it = children.iter_children();
                    let op = |alt| {
                        if alt == IndentedAlt::NoIndent {
                            Ok(parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned())
                        } else {
                            Err(alt)
                        }
                    };
                    let mut ind = self
                        .serialize(&it.next().unwrap(), parent_indent, out)
                        .or_else(op)?;
                    for id in it {
                        ind = self.serialize(&id, &ind, out).or_else(op)?;
                    }
                }
                Err(IndentedAlt::NoIndent)
            }
            (Some(label), None) => {
                let s = LabelStore::resolve(self.stores, label);
                out.write_str(&s).unwrap();
                Err(IndentedAlt::NoIndent)
            }
        }
    }
}
impl<'store, 'b, IdN, HAST, const SPC: bool> IndentedSerializer<'store, 'b, IdN, HAST, Json, SPC>
where
    IdN: NodeId<IdN = IdN>,
    HAST: crate::types::NodeStore<IdN>,
    HAST: crate::types::LabelStore<str>,
    HAST: crate::types::TypeStore<HAST::R<'store>>,
    HAST::R<'store>:
        crate::types::Labeled<Label = HAST::I> + crate::types::WithChildren<TreeId = IdN>,
{
    fn serialize(
        &self,
        id: &IdN,
        parent_indent: &str,
        out: &mut std::fmt::Formatter<'_>,
    ) -> Result<String, IndentedAlt> {
        use crate::types::LabelStore;
        use crate::types::Labeled;
        use crate::types::NodeStore;
        use crate::types::Typed;
        use crate::types::WithChildren;
        let b = NodeStore::resolve(self.stores, id);
        // let kind = (self.stores.type_store(), b);
        let kind = self.stores.resolve_type(&b);
        let label = b.try_get_label();
        let children = b.children();

        if kind.is_spaces() {
            let s = LabelStore::resolve(self.stores, &label.unwrap());
            let b:String = //s; //String::new();
        Space::format_indentation(s.as_bytes())
            .iter()
            .map(|x| x.to_string())
            .collect();
            if SPC {
                // let a = &*s;
                // a.iter()
                //     .for_each(|a| Space::fmt(a, &mut b, parent_indent).unwrap());
                out.write_str("{\"kind\":\"")?;
                // out.write_str(&kind.to_string())?;
                out.write_str(&"spaces")?;
                out.write_str("\",\"label\":\"")?;
                out.write_str(&escape(&b))?;
                out.write_str("\"}")?;
            }
            return Ok(if b.contains("\n") {
                b
            } else {
                parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned()
            });
        }

        match (label, children) {
            (None, None) => {
                out.write_str("\"")?;
                out.write_str(&escape(&kind.to_string()))?;
                out.write_str("\"")?;
                Err(IndentedAlt::NoIndent)
            }
            (label, Some(children)) => {
                out.write_str("{\"kind\":\"")?;
                out.write_str(&escape(&kind.to_string()))?;
                if let Some(label) = label {
                    out.write_str("\",\"label\":\"")?;
                    let s = LabelStore::resolve(self.stores, label);
                    out.write_str(&escape(&s))?;
                }
                if !children.is_empty() {
                    out.write_str("\",\"children\":[").unwrap();
                    let mut it = children.iter_children();
                    let mut ind = self
                        .serialize(&it.next().unwrap(), parent_indent, out)
                        .unwrap_or(
                            parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned(),
                        );
                    for id in it {
                        out.write_str(",").unwrap();
                        ind = self.serialize(&id, &ind, out).unwrap_or(
                            parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned(),
                        );
                    }
                    out.write_str("]}").unwrap();
                } else {
                    out.write_str("\"}").unwrap();
                }
                Err(IndentedAlt::NoIndent)
            }
            (Some(label), None) => {
                out.write_str("{\"kind\":\"").unwrap();
                out.write_str(&escape(&kind.to_string())).unwrap();
                out.write_str("\",\"label\":\"").unwrap();
                let s = LabelStore::resolve(self.stores, label);
                out.write_str(&escape(&s)).unwrap();
                out.write_str("\"}").unwrap();
                Err(IndentedAlt::NoIndent)
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum IndentedAlt {
    FmtError,
    NoIndent,
}
impl From<std::fmt::Error> for IndentedAlt {
    fn from(_: std::fmt::Error) -> Self {
        IndentedAlt::FmtError
    }
}
