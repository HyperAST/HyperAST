use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Write},
    io::stdout,
    marker::PhantomData,
};

use string_interner::{DefaultSymbol, Symbol};

use crate::{types::Type, impact::serialize::{MySerialize, Keyed}};

pub type TypeIdentifier = Type;

type Label = Vec<u8>;

pub trait RefContainer {
    type Result;
    fn check<U: MySerialize+Keyed<usize>>(&self, rf: U) -> Self::Result;
}

/// identifying data for a node in an HyperAST
pub struct SimpleNode1<Child, Label> {
    pub(crate) kind: TypeIdentifier,
    pub(crate) label: Option<Label>,
    pub(crate) children: Vec<Child>,
}

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
    Spaces(Box<[Space]>),
}

pub(crate) enum SimpNode<NodeId, LabelId> {
    Type(Type),
    Label { label: LabelId, kind: Type },
    Children { children: Box<[NodeId]>, kind: Type },
    Spaces(Box<[Space]>),
}

mod TypeBaggableNodes {
    use std::marker::PhantomData;

    struct Keyword<Type> {
        kind: Type,
    }

    struct UnsizedNode<Type, NodeId, LabelId> {
        // kind: Type,
        _phantom: PhantomData<*const (Type, NodeId, LabelId)>,
        bytes: [u8],
        // children: [MyUnion<LabelId,NodeId>],
    }
}

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
    type ChildIdx = u8;
    fn child_count(&self) -> u8 {
        match self {
            CompressedNode::Children2 {
                children: _,
                kind: _,
            } => 2,
            CompressedNode::Children { children, kind: _ } => children.len() as u8,
            _ => 0,
        }
    }

    fn get_child(&self, idx: &Self::ChildIdx) -> N {
        match self {
            CompressedNode::Children2 { children, kind: _ } if *idx == 0 => children[0].clone(),
            CompressedNode::Children2 { children, kind: _ } if *idx == 1 => children[1].clone(),
            CompressedNode::Children { children, kind: _ } => children[*idx as usize].clone(),
            _ => panic!(),
        }
    }

    fn get_child_rev(&self, idx: &Self::ChildIdx) -> Self::TreeId {
        match self {
            CompressedNode::Children2 { children, kind: _ } if *idx == 1 => children[0].clone(),
            CompressedNode::Children2 { children, kind: _ } if *idx == 0 => children[1].clone(),
            CompressedNode::Children { children, kind: _ } => {
                children[children.len() - 1 - (*idx as usize)].clone()
            }
            _ => panic!(),
        }
    }

    // fn descendants_count(&self) -> Self::TreeId {
    //     match self {
    //         CompressedNode::Children2 {
    //             children: _,
    //             kind: _,
    //         } => todo!(),
    //         CompressedNode::Children {
    //             children: _,
    //             kind: _,
    //         } => todo!(),
    //         _ => 0,
    //     }
    // }

    fn get_children<'a>(&'a self) -> &'a [Self::TreeId] {
        match self {
            CompressedNode::Children2 { children, kind: _ } => &*children,
            CompressedNode::Children { children, kind: _ } => &*children,
            _ => &[],
        }
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
    pub(crate) fn fmt<W: Write>(&self, w: &mut W, p: &str) -> std::fmt::Result {
        match self {
            Space::Space => write!(w, " "),
            Space::NewLine => write!(w, "\n"),
            Space::CariageReturn => write!(w, "\r"),
            Space::Tabulation => write!(w, "\t"),
            Space::ParentIndentation => write!(w, "{}", p),
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
                    log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                    panic!("{} {:?}", x as u8, spaces)
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
    pub(crate) fn replace_indentation(indentation: &[Space], spaces: &[Space]) -> Vec<Space> {
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

trait DisplayTreeStruct<IdN: Clone, IdL>: Display {
    fn node(&self, id: &IdN) -> CompressedNode<IdN, IdL>;

    fn print_tree_structure(&self, id: &IdN) {
        match self.node(id) {
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
                for id in children.iter() {
                    self.print_tree_structure(id);
                }
                print!(")");
            }
            CompressedNode::Children { kind, children } => {
                print!("({} ", kind.to_string());
                let children = children.clone();
                for id in children.iter() {
                    self.print_tree_structure(id);
                }
                print!(")");
            }
            CompressedNode::Spaces(_) => (),
        };
    }
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "[{}]", todo!())
        } else {
            write!(f, "[{} drop scopes]", todo!())
        }
    }
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
            let children = children.clone();
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
            let children = children.clone();
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
            let children = children.clone();
            for id in children.iter() {
                print_tree_syntax(f, g, &id, out);
            }
            print!(")");
        }
        CompressedNode::Spaces(s) => {
            print!("(_ ");
            let a = &*s;
            a.iter().for_each(|a| print!("{:?}", a));
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
            let children = &(*children);
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
            let a = &*s;
            let mut b = String::new();
            a.iter()
                .for_each(|a| Space::fmt(a, &mut b, parent_indent).unwrap());
            out.write_str(&b).unwrap();
            Some(if b.contains("\n") {
                b
            } else {
                parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned()
            })
        }
    }
}
