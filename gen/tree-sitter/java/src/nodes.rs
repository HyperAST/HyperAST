use std::fmt::{Debug, Display, Write};

use rusted_gumtree_core::tree::tree::Type;

pub type TypeIdentifier = Type;

type Label = Vec<u8>;

pub struct SimpleNode1<Child, Label> {
    pub(crate) kind: TypeIdentifier,
    pub(crate) label: Option<Label>,
    pub(crate) children: Vec<Child>,
}

pub type LabelIdentifier = u32;
pub type NodeIdentifier = u32;
pub type HashSize = u32;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub enum Space {
    Space,
    LineBreak,
    // NewLine,
    // CariageReturn,
    Tabulation,
    ParentIndentation,
}

#[derive(Debug)]
pub(crate) enum CompressedNode<NodeId, LabelId> {
    Type(Type),
    Label { label: LabelId, kind: Type },
    Children2 { children: [NodeId; 2], kind: Type },
    Children { children: Box<[NodeId]>, kind: Type },
    Spaces(Box<[Space]>),
}

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
    pub(crate) fn new(kind: TypeIdentifier, label: Option<L>, children: Vec<N>) -> Self {
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

impl<N, L> rusted_gumtree_core::tree::tree::Typed for CompressedNode<N, L> {
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

impl<N, L: Eq> rusted_gumtree_core::tree::tree::Labeled for CompressedNode<N, L> {
    type Label = L;

    fn get_label(&self) -> &L {
        match self {
            CompressedNode::Label { label, kind: _ } => label,
            _ => panic!(),
        }
    }
}

impl<N: Eq + Clone, L> rusted_gumtree_core::tree::tree::WithChildren for CompressedNode<N, L> {
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
        };
        todo!()
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
impl<N, L> rusted_gumtree_core::tree::tree::Node for CompressedNode<N, L> {}
impl<N: Eq, L> rusted_gumtree_core::tree::tree::Stored for CompressedNode<N, L> {
    type TreeId = N;
}

impl<N: Eq + Clone, L: Eq> rusted_gumtree_core::tree::tree::Tree for CompressedNode<N, L> {
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
            Space::LineBreak => write!(f, "n"),
            // Spaces::NewLine => write!(f, "n"),
            // Spaces::CariageReturn => write!(f, "r"),
            Space::Tabulation => write!(f, "t"),
            Space::ParentIndentation => write!(f, "0"),
        }
        // match self {
        //     Spaces::Space => write!(f, " "),
        //     Spaces::LineBreak => write!(f, "\n"),
        //     // Spaces::NewLine => write!(f, "n"),
        //     // Spaces::CariageReturn => write!(f, "r"),
        //     Spaces::Tabulation => write!(f, "\t"),
        //     Spaces::ParentIndentation => panic!(),
        // }
    }
}

impl Space {
    pub(crate) fn fmt<W: Write>(&self, w: &mut W, p: &str) -> std::fmt::Result {
        match self {
            Space::Space => write!(w, " "),
            Space::LineBreak => write!(w, "\n"),
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
            Space::LineBreak => write!(f, "n"),
            // Spaces::NewLine => write!(f, "n"),
            // Spaces::CariageReturn => write!(f, "r"),
            Space::Tabulation => write!(f, "t"),
            Space::ParentIndentation => write!(f, "0"),
        }
    }
}
const LB: char = '\n';
impl Space {
    pub(crate) fn format_indentation(spaces: &[u8]) -> Vec<Space> {
        spaces
            .iter()
            .map(|x| match *x as char {
                ' ' => Space::Space,
                LB => Space::LineBreak,
                '\t' => Space::Tabulation,
                _ => panic!(),
            })
            .collect()
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
