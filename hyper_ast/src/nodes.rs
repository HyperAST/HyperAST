use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Write},
};

use num::{ToPrimitive};

use crate::{
    impact::serialize::{Keyed, MySerialize},
    types::{Type, MySlice, IterableChildren},
};

pub type TypeIdentifier = Type;

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
pub enum CompressedNode<NodeId, LabelId> {
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

impl<N: PartialEq, L: PartialEq> PartialEq for CompressedNode<N, L> {
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

impl<N: Eq, L: Eq> Eq for CompressedNode<N, L> {}

impl<N, L> CompressedNode<N, L> {
    pub fn new(kind: TypeIdentifier, label: Option<L>, children: Vec<N>) -> Self {
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

impl<N, L> crate::types::Typed for CompressedNode<N, L> {
    type Type = Type;

    fn get_type(&self) -> Type {
        match self {
            CompressedNode::Type(kind) => *kind,
            CompressedNode::Label { label: _, kind } => *kind,
            CompressedNode::Children2 { children: _, kind } => *kind,
            CompressedNode::Children { children: _, kind } => *kind,
            CompressedNode::Spaces(_) => Type::Spaces,
        }
    }
}

impl<N, L: Eq> crate::types::Labeled for CompressedNode<N, L> {
    type Label = L;

    fn get_label(&self) -> &L {
        match self {
            CompressedNode::Label { label, kind: _ } => label,
            _ => panic!(),
        }
    }
}

impl<N: Eq + Clone, L> crate::types::WithChildren for CompressedNode<N, L> {
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

    fn child(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        match self {
            CompressedNode::Children2 { children, kind: _ } if *idx == 0 => {
                Some(children[0].clone())
            }
            CompressedNode::Children2 { children, kind: _ } if *idx == 1 => {
                Some(children[1].clone())
            }
            CompressedNode::Children { children, kind: _ } => Some(children[*idx as usize].clone()),
            _ => None,
        }
    }

    fn child_rev(&self, idx: &Self::ChildIdx) -> Option<Self::TreeId> {
        match self {
            CompressedNode::Children2 { children, kind: _ } if *idx == 1 => {
                Some(children[0].clone())
            }
            CompressedNode::Children2 { children, kind: _ } if *idx == 0 => {
                Some(children[1].clone())
            }
            CompressedNode::Children { children, kind: _ } => {
                Some(children[children.len() - 1 - (*idx as usize)].clone())
            }
            _ => None,
        }
    }

    // fn children_unchecked<'a>(&'a self) -> &[Self::TreeId] {
    //     match self {
    //         CompressedNode::Children2 { children, kind: _ } => &*children,
    //         CompressedNode::Children { children, kind: _ } => &*children,
    //         _ => &[],
    //     }
    // }

    // fn get_children_cpy<'a>(&'a self) -> Vec<Self::TreeId> {
    //     match self {
    //         CompressedNode::Children2 { children, kind: _ } => children.to_vec(),
    //         CompressedNode::Children { children, kind: _ } => children.to_vec(),
    //         _ => vec![],
    //     }
    // }

    fn children<'a>(&'a self) -> Option<&'a <Self as crate::types::WithChildren>::Children<'a>> {
        fn f<'a, N,L>(x: &'a CompressedNode<N,L>) -> &'a [N] {
            match x {
                CompressedNode::Children2 { children, kind: _ } => {
                    &*children
                }
                CompressedNode::Children { children, kind: _ } => {
                    &**children
                },
                _ => {
                    &[]
                }
            }
        }
        // TODO check if it work, not sure
        Some(f(self).into())
    }
}

impl<N, L> crate::types::Node for CompressedNode<N, L> {}
impl<N: Eq, L> crate::types::Stored for CompressedNode<N, L> {
    type TreeId = N;
}

impl<N: Eq + Clone, L: Eq> crate::types::Tree for CompressedNode<N, L> {
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
                    log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                    println!("{:?}", std::str::from_utf8(spaces));
                    panic!("{:?}", spaces)
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

// trait DisplayTreeStruct<IdN: Clone, IdL>: Display {
//     fn node(&self, id: &IdN) -> CompressedNode<IdN, IdL>;

//     fn print_tree_structure(&self, id: &IdN) {
//         match self.node(id) {
//             CompressedNode::Type(kind) => {
//                 print!("{}", kind.to_string());
//                 // None
//             }
//             CompressedNode::Label { kind, label: _ } => {
//                 print!("({})", kind.to_string());
//                 // None
//             }
//             CompressedNode::Children2 { kind, children } => {
//                 print!("({} ", kind.to_string());
//                 for id in children.iter() {
//                     self.print_tree_structure(id);
//                 }
//                 print!(")");
//             }
//             CompressedNode::Children { kind, children } => {
//                 print!("({} ", kind.to_string());
//                 let children = children.clone();
//                 for id in children.iter() {
//                     self.print_tree_structure(id);
//                 }
//                 print!(")");
//             }
//             CompressedNode::Spaces(_) => (),
//         };
//     }
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         if f.alternate() {
//             write!(f, "[{}]", todo!())
//         } else {
//             write!(f, "[{} drop scopes]", todo!())
//         }
//     }
// }
pub fn print_tree_ids<
    IdN: Debug,
    IdL,
    T: Borrow<CompressedNode<IdN, IdL>>,
    F: Copy + Fn(&IdN) -> T,
>(
    f: F,
    id: &IdN,
) {
    match f(id).borrow() {
        CompressedNode::Type(_) => {
            print!("[{:?}]", id);
            // None
        }
        CompressedNode::Label { label: _, .. } => {
            print!("{{{:?}}}", id);
            // None
        }
        CompressedNode::Children2 { children, .. } => {
            print!("({:?} ", id);
            for id in children {
                print_tree_ids(f, &id);
            }
            print!(")");
        }
        CompressedNode::Children { children, .. } => {
            print!("({:?} ", id);
            for id in children.iter() {
                print_tree_ids(f, &id);
            }
            print!(")");
        }
        CompressedNode::Spaces(_) => print!("{{{:?}}}", id),
    };
}
pub fn print_tree_structure<
    IdN,
    IdL,
    T: Borrow<CompressedNode<IdN, IdL>>,
    F: Copy + Fn(&IdN) -> T,
>(
    f: F,
    id: &IdN,
) {
    match f(id).borrow() {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label: _ } => {
            print!("({})", kind.to_string());
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children {
                print_tree_structure(f, &id);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children.iter() {
                print_tree_structure(f, &id);
            }
            print!(")");
        }
        CompressedNode::Spaces(_) => (),
    };
}

pub fn print_tree_labels<
    IdN,
    IdL,
    T: Borrow<CompressedNode<IdN, IdL>>,
    F: Copy + Fn(&IdN) -> T,
    G: Copy + Fn(&IdL) -> String,
>(
    f: F,
    g: G,
    id: &IdN,
) {
    match f(id).borrow() {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
            let s = g(label);
            if s.len() > 20 {
                print!("({}='{}...')", kind.to_string(), &s[..20]);
            } else {
                print!("({}='{}')", kind.to_string(), s);
            }
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children {
                print_tree_labels(f, g, &id);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children.iter() {
                print_tree_labels(f, g, &id);
            }
            print!(")");
        }
        CompressedNode::Spaces(_) => (),
    };
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

pub fn print_tree_syntax<
    IdN,
    IdL,
    T: Borrow<CompressedNode<IdN, IdL>>,
    F: Copy + Fn(&IdN) -> T,
    G: Copy + Fn(&IdL) -> String,
    W: std::fmt::Write,
>(
    f: F,
    g: G,
    id: &IdN,
    out: &mut W,
) {
    match f(id).borrow() {
        CompressedNode::Type(kind) => {
            print!("{}", kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
            let s = &g(label);
            if s.len() > 20 {
                print!("({}='{}...')", kind.to_string(), &s[..20]);
            } else {
                print!("({}='{}')", kind.to_string(), s);
            }
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children {
                print_tree_syntax(f, g, &id, out);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({} ", kind.to_string());
            for id in children.iter() {
                print_tree_syntax(f, g, &id, out);
            }
            print!(")");
        }
        CompressedNode::Spaces(s) => {
            let s = &g(s);
            print!("(_ ");
            // print!("{}",s);
            print!("{:?}", Space::format_indentation(s.as_bytes()));
            // a.iter().for_each(|a| print!("{:?}", a));
            print!(")");
        }
    };
}

pub fn print_tree_syntax_with_ids<
    IdN: Debug,
    IdL,
    T: Borrow<CompressedNode<IdN, IdL>>,
    F: Copy + Fn(&IdN) -> T,
    G: Copy + Fn(&IdL) -> String,
    W: std::fmt::Write,
>(
    f: F,
    g: G,
    id: &IdN,
    out: &mut W,
) {
    match f(id).borrow() {
        CompressedNode::Type(kind) => {
            print!("{:?}{}", id, kind.to_string());
            // None
        }
        CompressedNode::Label { kind, label } => {
            let s = &g(label);
            if s.len() > 20 {
                print!("({:?}{}='{}...')", id, kind.to_string(), &s[..20]);
            } else {
                print!("({:?}{}='{}')", id, kind.to_string(), s);
            }
            // None
        }
        CompressedNode::Children2 { kind, children } => {
            print!("({:?}{} ", id, kind.to_string());
            for id in children {
                print_tree_syntax_with_ids(f, g, &id, out);
            }
            print!(")");
        }
        CompressedNode::Children { kind, children } => {
            print!("({:?}{} ", id, kind.to_string());
            for id in children.iter() {
                print_tree_syntax_with_ids(f, g, &id, out);
            }
            print!(")");
        }
        CompressedNode::Spaces(s) => {
            let s = &g(s);
            print!("({:?}_ ", id);
            // let a = &*s;
            // print!("{}",s);
            print!("{:?}", Space::format_indentation(s.as_bytes()));
            // a.iter().for_each(|a| print!("{:?}", a));
            print!(")");
        }
    };
}

pub fn serialize<
    IdN,
    IdL,
    T: Borrow<CompressedNode<IdN, IdL>>,
    F: Copy + Fn(&IdN) -> T,
    G: Copy + Fn(&IdL) -> String,
    W: std::fmt::Write,
>(
    f: F,
    g: G,
    id: &IdN,
    out: &mut W,
    parent_indent: &str,
) -> Option<String> {
    match f(id).borrow() {
        CompressedNode::Type(kind) => {
            out.write_str(&kind.to_string()).unwrap();
            None
        }
        CompressedNode::Label { kind: _, label } => {
            let s = g(label);
            out.write_str(&s).unwrap();
            None
        }
        CompressedNode::Children2 { kind: _, children } => {
            let ind = serialize(f, g, &children[0], out, parent_indent)
                .unwrap_or(parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned());
            serialize(f, g, &children[1], out, &ind);
            None
        }
        CompressedNode::Children { kind: _, children } => {
            let mut it = children.iter();
            let mut ind = serialize(f, g, &it.next().unwrap(), out, parent_indent)
                .unwrap_or(parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned());
            for id in it {
                ind = serialize(f, g, &id, out, &ind)
                    .unwrap_or(parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned());
            }
            None
        }
        CompressedNode::Spaces(s) => {
            let s = g(s);
            // let a = &*s;
            let b:String = //s; //String::new();
            Space::format_indentation(s.as_bytes())
                .iter()
                .map(|x| x.to_string())
                .collect();
            // a.iter()
            //     .for_each(|a| Space::fmt(a, &mut b, parent_indent).unwrap());
            out.write_str(&b).unwrap();
            Some(if b.contains("\n") {
                b
            } else {
                parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned()
            })
        }
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
    NS: crate::types::NodeStore<IdN>,
    <NS as crate::types::NodeStore<IdN>>::R<'a>: crate::types::Tree<TreeId=IdN, Type = Type,Label = LS::I>,
    LS: crate::types::LabelStore<str>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let id = &self.id;
        json_serialize::<_, _, _, _, _, _, SPC>(
            |id| -> _ { self.node_store.resolve(id.clone()) },
            |id| -> _ { self.label_store.resolve(id).to_owned() },
            id,
            f,
            "\n",
        );
        Ok(())
    }
}

pub fn json_serialize<
    IdN,
    IdL,
    T: crate::types::Tree<TreeId = IdN, Type = Type, Label = IdL>,
    F: Copy + Fn(&IdN) -> T,
    G: Copy + Fn(&IdL) -> String,
    W: std::fmt::Write,
    const SPC: bool,
>(
    f: F,
    g: G,
    id: &IdN,
    out: &mut W,
    parent_indent: &str,
) -> Option<String> {
    let b = f(id);
    let kind = b.get_type();
    let label = b.try_get_label();
    let children = b.children();

    if kind == Type::Spaces {
        let s = g(label.unwrap());
        let b:String = //s; //String::new();
        Space::format_indentation(s.as_bytes())
            .iter()
            .map(|x| x.to_string())
            .collect();
        if SPC {
            // let a = &*s;
            // a.iter()
            //     .for_each(|a| Space::fmt(a, &mut b, parent_indent).unwrap());
            out.write_str("{\"kind\":\"").unwrap();
            // out.write_str(&kind.to_string()).unwrap();
            out.write_str(&"spaces").unwrap();
            out.write_str("\",\"label\":\"").unwrap();
            out.write_str(&escape(&b)).unwrap();
            out.write_str("\"}").unwrap();
        }
        return Some(if b.contains("\n") {
            b
        } else {
            parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned()
        });
    }

    match (label, children) {
        (None, None) => {
            out.write_str("\"").unwrap();
            out.write_str(&escape(&kind.to_string())).unwrap();
            out.write_str("\"").unwrap();
            None
        }
        (label, Some(children)) => {
            out.write_str("{\"kind\":\"").unwrap();
            out.write_str(&escape(&kind.to_string())).unwrap();
            if let Some(label) = label {
                out.write_str("\",\"label\":\"").unwrap();
                let s = g(label);
                out.write_str(&escape(&s)).unwrap();
            }
            if !children.is_empty() {
                out.write_str("\",\"children\":[").unwrap();
                let mut it = children.iter_children();
                let mut ind = json_serialize::<_, _, _, _, _, _, SPC>(
                    f,
                    g,
                    &it.next().unwrap(),
                    out,
                    parent_indent,
                )
                .unwrap_or(parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned());
                for id in it {
                    out.write_str(",").unwrap();
                    ind = json_serialize::<_, _, _, _, _, _, SPC>(f, g, &id, out, &ind).unwrap_or(
                        parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned(),
                    );
                }
                out.write_str("]}").unwrap();
            } else {
                out.write_str("\"}").unwrap();
            }
            None
        }
        (Some(label), None) => {
            out.write_str("{\"kind\":\"").unwrap();
            out.write_str(&escape(&kind.to_string())).unwrap();
            out.write_str("\",\"label\":\"").unwrap();
            let s = g(label);
            out.write_str(&escape(&s)).unwrap();
            out.write_str("\"}").unwrap();
            None
        }
    }
}
