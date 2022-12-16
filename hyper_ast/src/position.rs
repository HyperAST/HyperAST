use core::fmt;
use std::{
    fmt::{Debug, Display},
    io::stdout,
    path::{Path, PathBuf},
};

use num::ToPrimitive;

use crate::{
    nodes::{print_tree_syntax, IoOut},
    store::{defaults::NodeIdentifier, SimpleStores},
    types::{
        self, Children, IterableChildren, LabelStore, Labeled, Tree, Type, Typed, WithChildren,
    },
};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Position {
    file: PathBuf,
    offset: usize,
    len: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            file: PathBuf::default(),
            offset: 0,
            len: 0,
        }
    }
}

impl Position {
    pub fn new(file: PathBuf, offset: usize, len: usize) -> Self {
        Self { file, offset, len }
    }
    pub fn inc_path(&mut self, s: &str) {
        self.file.push(s);
    }
    pub fn inc_offset(&mut self, x: usize) {
        self.offset += x;
    }
    pub fn set_len(&mut self, x: usize) {
        self.len = x;
    }
    pub fn range(&self) -> std::ops::Range<usize> {
        self.offset..(self.offset + self.len)
    }
    pub fn file(&self) -> &Path {
        &self.file
    }
}
impl Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("file", &self.file)
            .field("offset", &self.offset)
            .field("len", &self.len)
            .finish()
    }
}
impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"offset\":{},\"len\":{},\"file\":{:?}}}",
            &self.offset, &self.len, &self.file
        )
    }
}

pub fn extract_file_postion(stores: &SimpleStores, parents: &[NodeIdentifier]) -> Position {
    if parents.is_empty() {
        Position::default()
    } else {
        let p = parents[parents.len() - 1];
        let b = stores.node_store.resolve(p);
        // println!("type {:?}", b.get_type());
        // if !b.has_label() {
        //     panic!("{:?} should have a label", b.get_type());
        // }
        let l = stores.label_store.resolve(b.get_label());

        let mut r = extract_file_postion(stores, &parents[..parents.len() - 1]);
        r.inc_path(l);
        r
    }
}

pub fn extract_position(
    stores: &SimpleStores,
    parents: &[NodeIdentifier],
    offsets: &[usize],
) -> Position {
    if parents.is_empty() {
        return Position::default();
    }
    let p = parents[parents.len() - 1];
    let o = offsets[offsets.len() - 1];

    let b = stores.node_store.resolve(p);
    let c = {
        let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
        v.iter()
            .map(|x| {
                let b = stores.node_store.resolve(*x);
                // println!("{:?}", b.get_type());
                b.get_bytes_len(0) as usize
            })
            .sum()
    };
    if b.get_type() == Type::Program {
        let mut r = extract_file_postion(stores, parents);
        r.inc_offset(c);
        r
    } else {
        let mut r = extract_position(
            stores,
            &parents[..parents.len() - 1],
            &offsets[..offsets.len() - 1],
        );
        r.inc_offset(c);
        r
    }
}

pub trait TreePath<IdN> {
    fn node(&self) -> Option<&IdN>;
    fn offset(&self) -> Option<&usize>;
    fn pop(&mut self) -> Option<(IdN, usize)>;
    fn goto(&mut self, node: IdN, i: usize);
    fn inc(&mut self, node: IdN);
    fn dec(&mut self, node: IdN);
    fn check(&self, stores: &SimpleStores) -> Result<(), ()>;
}

#[derive(Clone, Debug)]
pub struct StructuralPosition {
    pub(crate) nodes: Vec<NodeIdentifier>,
    pub(crate) offsets: Vec<usize>,
}

impl TreePath<NodeIdentifier> for StructuralPosition {
    fn node(&self) -> Option<&NodeIdentifier> {
        self.nodes.last()
    }

    fn offset(&self) -> Option<&usize> {
        self.offsets.last()
    }

    fn pop(&mut self) -> Option<(NodeIdentifier, usize)> {
        Some((self.nodes.pop()?, self.offsets.pop()?))
    }

    fn goto(&mut self, node: NodeIdentifier, i: usize) {
        self.nodes.push(node);
        self.offsets.push(i + 1);
    }

    fn inc(&mut self, node: NodeIdentifier) {
        *self.nodes.last_mut().unwrap() = node;
        *self.offsets.last_mut().unwrap() += 1;
    }

    fn dec(&mut self, node: NodeIdentifier) {
        *self.nodes.last_mut().unwrap() = node;
        if let Some(offsets) = self.offsets.last_mut() {
            assert!(*offsets > 1);
            *offsets -= 1;
        }
    }

    fn check(&self, stores: &SimpleStores) -> Result<(), ()> {
        assert_eq!(self.offsets.len(), self.nodes.len());
        if self.nodes.is_empty() {
            return Ok(());
        }
        let mut i = self.nodes.len() - 1;

        while i > 0 {
            let e = self.nodes[i];
            let o = self.offsets[i] - 1;
            let p = self.nodes[i - 1];
            let b = stores.node_store.resolve(p);
            if !b.has_children() || Some(e) != b.child(&o.to_u16().expect("too big")) {
                return Err(());
            }
            i -= 1;
        }
        Ok(())
    }
}

impl StructuralPosition {
    pub fn make_position(&self, stores: &SimpleStores) -> Position {
        self.check(stores).unwrap();
        // let parents = self.parents.iter().peekable();
        let mut from_file = false;
        // let mut len = 0;
        let x = *self.node().unwrap();
        let b = stores.node_store.resolve(x);
        let t = b.get_type();
        // println!("t0:{:?}", t);
        let len = if let Some(y) = b.try_get_bytes_len(0) {
            if t != Type::Program {
                from_file = true;
            }
            y as usize
            // Some(x)
        } else {
            0
            // None
        };
        let mut offset = 0;
        let mut path = vec![];
        if self.nodes.is_empty() {
            let path = PathBuf::from_iter(path.iter().rev());
            return Position {
                file: path,
                offset,
                len,
            };
        }
        let mut i = self.nodes.len() - 1;
        if from_file {
            while i > 0 {
                let p = self.nodes[i - 1];
                let b = stores.node_store.resolve(p);
                let t = b.get_type();
                // println!("t1:{:?}", t);
                let o = self.offsets[i];
                let c: usize = {
                    let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
                    v.iter()
                        .map(|x| {
                            let b = stores.node_store.resolve(*x);
                            // println!("{:?}", b.get_type());
                            b.get_bytes_len(0) as usize
                        })
                        .sum()
                };
                offset += c;
                if t == Type::Program {
                    from_file = false;
                    i -= 1;
                    break;
                } else {
                    i -= 1;
                }
            }
        }
        if self.nodes.is_empty() {
        } else if !from_file
        // || (i == 0 && stores.node_store.resolve(self.nodes[i]).get_type() == Type::Program)
        {
            loop {
                let n = self.nodes[i];
                let b = stores.node_store.resolve(n);
                // println!("t2:{:?}", b.get_type());
                let l = stores.label_store.resolve(b.get_label());
                path.push(l);
                if i == 0 {
                    break;
                } else {
                    i -= 1;
                }
            }
        } else {
            let p = self.nodes[i - 1];
            let b = stores.node_store.resolve(p);
            let o = self.offsets[i];
            let c: usize = {
                let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
                v.iter()
                    .map(|x| {
                        let b = stores.node_store.resolve(*x);
                        // println!("{:?}", b.get_type());
                        b.get_bytes_len(0) as usize
                    })
                    .sum()
            };
            offset += c;
        }

        let path = PathBuf::from_iter(path.iter().rev());
        Position {
            file: path,
            offset,
            len,
        }
    }

    pub fn new(node: NodeIdentifier) -> Self {
        Self {
            nodes: vec![node],
            offsets: vec![0],
        }
    }
}

impl From<(Vec<NodeIdentifier>, Vec<usize>, NodeIdentifier)> for StructuralPosition {
    fn from(mut x: (Vec<NodeIdentifier>, Vec<usize>, NodeIdentifier)) -> Self {
        assert_eq!(x.0.len() + 1, x.1.len());
        x.0.push(x.2);
        Self {
            nodes: x.0,
            offsets: x.1,
        }
    }
}
impl From<(Vec<NodeIdentifier>, Vec<usize>)> for StructuralPosition {
    fn from(x: (Vec<NodeIdentifier>, Vec<usize>)) -> Self {
        assert_eq!(x.0.len(), x.1.len());
        Self {
            nodes: x.0,
            offsets: x.1,
        }
    }
}
impl From<NodeIdentifier> for StructuralPosition {
    fn from(node: NodeIdentifier) -> Self {
        Self::new(node)
    }
}

// #[derive(Clone, Debug)]
// pub struct StructuralPositionWithIndentation {
//     pub(crate) nodes: Vec<NodeIdentifier>,
//     pub(crate) offsets: Vec<usize>,
//     pub(crate) indentations: Vec<Box<[Space]>>,
// }

pub struct StructuralPositionStore {
    pub nodes: Vec<NodeIdentifier>,
    parents: Vec<usize>,
    offsets: Vec<usize>,
    // ends: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct SpHandle(usize);

// struct IterStructuralPositions<'a> {
//     sps: &'a StructuralPositionStore,
//     ends: core::slice::Iter<'a, usize>,
// }

// impl<'a> Iterator for IterStructuralPositions<'a> {
//     type Item = StructuralPosition;

//     fn next(&mut self) -> Option<Self::Item> {
//         let x = *self.ends.next()?;
//         let it = ExploreStructuralPositions::new(self.sps, x);
//         // let r = Position;
//         todo!()
//     }
// }

#[derive(Clone, Debug)]
pub struct Scout {
    root: usize,
    path: StructuralPosition,
}

impl TreePath<NodeIdentifier> for Scout {
    fn node(&self) -> Option<&NodeIdentifier> {
        self.path.node()
    }

    fn offset(&self) -> Option<&usize> {
        self.path.offset()
    }

    fn pop(&mut self) -> Option<(NodeIdentifier, usize)> {
        self.path.pop()
    }

    fn goto(&mut self, node: NodeIdentifier, i: usize) {
        self.path.goto(node, i)
    }

    fn inc(&mut self, node: NodeIdentifier) {
        self.path.inc(node)
    }

    fn dec(&mut self, node: NodeIdentifier) {
        self.path.dec(node)
    }

    fn check(&self, stores: &SimpleStores) -> Result<(), ()> {
        self.path.check(stores)
    }
}

impl Scout {
    pub fn node_always(&self, x: &StructuralPositionStore) -> NodeIdentifier {
        if let Some(y) = self.path.node() {
            *y
        } else {
            x.nodes[self.root]
        }
    }
    pub fn offset_always(&self, x: &StructuralPositionStore) -> usize {
        if let Some(y) = self.path.offset() {
            *y
        } else {
            x.offsets[self.root]
        }
    }
    // pub fn try_node(&self) -> Result<NodeIdentifier, usize> {
    //     if let Some(y) = self.path.node() {
    //         Ok(*y)
    //     } else {
    //         Err(self.root)
    //     }
    // }
    pub fn has_parents(&self) -> bool {
        if self.path.nodes.is_empty() {
            self.root != 0
        } else {
            true
        }
    }
    // pub fn try_up(&mut self) -> Result<(), ()> {
    //     if self.path.nodes.is_empty() {
    //         Err(())
    //     } else {
    //         self.path.pop();
    //         Ok(())
    //     }
    // }
    pub fn up(&mut self, x: &StructuralPositionStore) -> Option<NodeIdentifier> {
        // println!("up {} {:?}", self.root, self.path);
        // if !self.path.offsets.is_empty() && self.path.offsets[0] == 0 {
        //     assert!(self.root == 0);
        // }
        if self.path.nodes.is_empty() {
            // let o = x.offsets[self.root];
            self.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            };
            // self.path = StructuralPosition::with_offset(x.nodes[self.root], o);
            assert_eq!(self.path.nodes.len(), self.path.offsets.len());
            if self.root == 0 {
                None
            } else {
                self.root = x.parents[self.root];
                Some(self.node_always(x))
            }
            // if o == 0 {
            //     assert!(self.path.offsets[0] == 0);
            //     assert!(self.root == 0);
            // }
        } else {
            self.path.pop();
            assert_eq!(self.path.nodes.len(), self.path.offsets.len());
            Some(self.node_always(x))
        }
        // if !self.path.offsets.is_empty() && self.path.offsets[0] == 0 {
        //     assert!(self.root == 0);
        // }
    }
    // pub fn goto(&mut self, node: NodeIdentifier, i: usize) {
    //     // println!("goto {} {:?}", self.root, self.path);
    //     self.path.nodes.push(node);
    //     self.path.offsets.push(i + 1);
    //     assert_eq!(self.path.nodes.len(), self.path.offsets.len());
    //     // if !self.path.offsets.is_empty() && self.path.offsets[0] == 0 {
    //     //     assert!(self.root == 0);
    //     // }
    // }
    // pub fn inc(&mut self, node: NodeIdentifier) -> usize {
    //     assert_eq!(self.path.nodes.len(), self.path.offsets.len());
    //     *self.path.nodes.last_mut().unwrap() = node;
    //     self.path.offsets.last_mut().unwrap().add_assign(1);
    //     // self.path.inc(node);
    //     self.path.offsets.last().unwrap() - 1
    // }
    // pub fn check_size(&self, stores: &SimpleStores) -> Result<(), ()> {
    //     assert_eq!(self.path.offsets.len(), self.path.nodes.len());
    //     if self.path.nodes.is_empty() {
    //         return Ok(());
    //     }
    //     let mut i = self.path.nodes.len() - 1;

    //     while i > 0 {
    //         let o = self.path.offsets[i] - 1;
    //         let p = self.path.nodes[i - 1];
    //         let b = stores.node_store.resolve(p);
    //         if !b.has_children() || o >= (b.child_count() as usize) {
    //             let s = b.child_count();
    //             if b.has_children() {
    //                 println!("error: {} {} {:?}", b.child_count(), o, p,);
    //             } else {
    //                 println!("error no children: {} {:?}", o, p,);
    //             }
    //             return Err(());
    //         }
    //         i -= 1;
    //     }
    //     Ok(())
    // }

    pub fn make_position(&self, sp: &StructuralPositionStore, stores: &SimpleStores) -> Position {
        self.check(stores).unwrap();
        // let parents = self.parents.iter().peekable();
        let mut from_file = false;
        // let mut len = 0;
        let x = self.node_always(sp);
        let b = stores.node_store.resolve(x);
        let t = b.get_type();
        // println!("t0:{:?}", t);
        let len = if let Some(y) = b.try_get_bytes_len(0) {
            if t != Type::Program {
                from_file = true;
            }
            y as usize
            // Some(x)
        } else {
            0
            // None
        };
        let mut offset = 0;
        let mut path = vec![];
        if self.path.nodes.is_empty() {
            return ExploreStructuralPositions::new(sp, self.root)
                .make_position_aux(stores, from_file, len, offset, path);
        }
        let mut i = self.path.nodes.len() - 1;
        if from_file {
            while i > 0 {
                let p = self.path.nodes[i - 1];
                let b = stores.node_store.resolve(p);
                let t = b.get_type();
                // println!("t1:{:?}", t);
                let o = self.path.offsets[i];
                let c: usize = {
                    let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
                    v.iter()
                        .map(|x| {
                            let b = stores.node_store.resolve(*x);
                            // println!("{:?}", b.get_type());
                            b.get_bytes_len(0) as usize
                        })
                        .sum()
                };
                offset += c;
                if t == Type::Program {
                    from_file = false;
                    i -= 1;
                    break;
                } else {
                    i -= 1;
                }
            }
        }
        if self.path.nodes.is_empty() {
        } else if !from_file
        // || (i == 0 && stores.node_store.resolve(self.path.nodes[i]).get_type() == Type::Program)
        {
            loop {
                from_file = false;
                let n = self.path.nodes[i];
                let b = stores.node_store.resolve(n);
                // println!("t2:{:?}", b.get_type());
                let l = stores.label_store.resolve(b.get_label());
                path.push(l);
                if i == 0 {
                    break;
                } else {
                    i -= 1;
                }
            }
        } else {
            let p = if i == 0 {
                sp.nodes[self.root]
            } else {
                self.path.nodes[i - 1]
            };
            let b = stores.node_store.resolve(p);
            let t = b.get_type();
            // println!("t3:{:?}", t);
            let o = self.path.offsets[i];
            let c: usize = {
                let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
                v.iter()
                    .map(|x| {
                        let b = stores.node_store.resolve(*x);
                        // println!("{:?}", b.get_type());
                        b.get_bytes_len(0) as usize
                    })
                    .sum()
            };
            offset += c;
            if t == Type::Program {
                from_file = false;
            } else {
            }
        }
        ExploreStructuralPositions::new(sp, self.root)
            .make_position_aux(stores, from_file, len, offset, path)
    }
}

// impl From<StructuralPosition> for Scout {
//     fn from(x: StructuralPosition) -> Self {
//         Self { root: 0, path: x }
//     }
// }

impl From<(StructuralPosition, usize)> for Scout {
    fn from((path, root): (StructuralPosition, usize)) -> Self {
        let path = if !path.offsets.is_empty() && path.offsets[0] == 0 {
            assert_eq!(root, 0);
            StructuralPosition {
                nodes: path.nodes[1..].to_owned(),
                offsets: path.offsets[1..].to_owned(),
            }
        } else {
            path
        };
        Self { path, root }
    }
}

pub struct ExploreStructuralPositions<'a> {
    sps: &'a StructuralPositionStore,
    i: usize,
}
/// precondition: root node do not contain a File node
/// TODO make whole thing more specific to a path in a tree
pub fn compute_range<It: Iterator>(
    root: NodeIdentifier,
    offsets: &mut It,
    stores: &SimpleStores,
) -> (usize, usize, NodeIdentifier)
where
    It::Item: ToPrimitive,
{
    let mut offset = 0;
    let mut x = root;
    for o in offsets {
        // dbg!(offset);
        let b = stores.node_store.resolve(x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            for y in 0..o.to_usize().unwrap() {
                let b = stores.node_store.resolve(cs[y]);
                offset += b.try_get_bytes_len(0).unwrap_or(0).to_usize().unwrap();
            }
            // if o.to_usize().unwrap() >= cs.len() {
            //     // dbg!("fail");
            // }
            if let Some(a) = cs.get(o.to_u16().unwrap()) {
                x = *a;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    let b = stores.node_store.resolve(x);
    (
        offset,
        offset + b.try_get_bytes_len(0).unwrap_or(0).to_usize().unwrap(),
        x,
    )
}

pub fn compute_position<'store, T, NS, LS, It: Iterator>(
    root: T::TreeId,
    offsets: &mut It,
    node_store: &'store NS,
    label_store: &'store LS,
) -> (Position, T::TreeId)
where
    It::Item: Clone,
    T::TreeId: Clone,
    NS: 'store + types::NodeStore<T::TreeId, R<'store> = T>,
    T: types::Tree<Type = types::Type, Label = LS::I, ChildIdx = It::Item>
        + types::WithSerialization,
    LS: types::LabelStore<str>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path = vec![];
    for o in &mut *offsets {
        // dbg!(offset);
        let b = node_store.resolve(&x);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());

        let t = b.get_type();

        if t.is_directory() || t.is_file() {
            let l = label_store.resolve(b.get_label());
            path.push(l);
        }

        if let Some(cs) = b.children() {
            let cs = cs.clone();
            if !t.is_directory() {
                for y in cs.before(o.clone()).iter_children() {
                    let b = node_store.resolve(y);
                    offset += b.try_bytes_len().unwrap().to_usize().unwrap();
                }
            } else {
                // for y in 0..o.to_usize().unwrap() {
                //     let b = node_store.resolve(cs[y]);
                //     println!("{:?}",b.get_type());
                // }
            }
            // if o.to_usize().unwrap() >= cs.len() {
            //     // dbg!("fail");
            // }
            if let Some(a) = cs.get(o) {
                x = a.clone();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    assert!(offsets.next().is_none());
    let b = node_store.resolve(&x);
    let t = b.get_type();
    if t.is_directory() || t.is_file() {
        let l = label_store.resolve(b.get_label());
        path.push(l);
    }

    let len = if !t.is_directory() {
        b.try_bytes_len().unwrap().to_usize().unwrap()
    } else {
        0
    };
    let path = PathBuf::from_iter(path.iter());
    (
        Position {
            file: path,
            offset,
            len,
        },
        x,
    )
}

impl<'a> ExploreStructuralPositions<'a> {
    pub fn make_position(self, stores: &SimpleStores) -> Position {
        self.sps.check(stores).unwrap();
        // let parents = self.parents.iter().peekable();
        let mut from_file = false;
        // let mut len = 0;
        let len = if let Some(x) = self.peek_node() {
            let b = stores.node_store.resolve(x);
            let t = b.get_type();
            if let Some(y) = b.try_get_bytes_len(0) {
                if t != Type::Program {
                    from_file = true;
                }
                y as usize
                // Some(x)
            } else {
                0
                // None
            }
        } else {
            0
            // None
        };
        let offset = 0;
        let path = vec![];
        self.make_position_aux(stores, from_file, len, offset, path)
    }

    fn make_position_aux(
        mut self,
        stores: &'a SimpleStores,
        from_file: bool,
        len: usize,
        mut offset: usize,
        mut path: Vec<&'a str>,
    ) -> Position {
        // println!(
        //     "it: {:?},{:?},{:?}",
        //     &it.sps.nodes, &it.sps.offsets, &it.sps.parents
        // );
        if from_file {
            while let Some(p) = self.peek_parent_node() {
                // println!("i: {}", it.i);
                assert_ne!(p, self.peek_node().unwrap());
                assert_eq!(p, self.sps.nodes[self.sps.parents[self.i - 1]]);
                assert_eq!(self.peek_node().unwrap(), self.sps.nodes[self.i - 1]);
                // println!("nodes: {}, parents:{}, offsets:{}",it.sps.nodes.len(),it.sps.parents.len(),it.sps.offsets.len());
                let b = stores.node_store.resolve(p);
                let t = b.get_type();
                // println!("T0:{:?}", t);
                // let o = it.sps.offsets[it]
                // println!("nodes: ({})", it.sps.nodes.len());
                // println!("offsets: ({}) {:?}", it.sps.offsets.len(), &it.sps.offsets);
                // println!("parents: ({}) {:?}", it.sps.parents.len(), &it.sps.parents);
                // println!(
                //     "o: {}, o p: {}",
                //     it.peek_offset().unwrap(),
                //     it.sps.offsets[it.sps.parents[it.i - 1]]
                // );
                let o = self.peek_offset().unwrap();
                if self.peek_node().unwrap() != b.children().unwrap()[o - 1] {
                    print_tree_syntax(
                        |x| {
                            stores
                                .node_store
                                .resolve(*x)
                                .into_compressed_node()
                                .unwrap()
                        },
                        |x| stores.label_store.resolve(x).to_string(),
                        &p,
                        &mut Into::<IoOut<_>>::into(stdout()),
                    );
                    if self.peek_node().unwrap() != b.children().unwrap()[o - 1] {
                        log::error!("backtrace: {}", std::backtrace::Backtrace::force_capture());
                    }
                    assert_eq!(
                        self.peek_node().unwrap(),
                        b.children().unwrap()[o - 1],
                        "p:{:?} b.cs:{:?} o:{} o p:{} i p:{}",
                        p,
                        b.children().unwrap(),
                        self.peek_offset().unwrap(),
                        self.sps.offsets[self.sps.parents[self.i - 1]],
                        self.sps.parents[self.i - 1],
                    );
                }
                let c: usize = {
                    let v: Vec<_> = b.children().unwrap().before(o.to_u16().unwrap() - 1).into();
                    v.iter()
                        .map(|x| {
                            let b = stores.node_store.resolve(*x);
                            // println!("{:?}", b.get_type());
                            // println!("T1:{:?}", b.get_type());
                            b.get_bytes_len(0) as usize
                        })
                        .sum()
                };
                offset += c;
                if t == Type::Program {
                    self.next();
                    break;
                } else {
                    self.next();
                }
            }
        }
        for p in self {
            let b = stores.node_store.resolve(p);
            // println!("type {:?}", b.get_type());
            // if !b.has_label() {
            //     panic!("{:?} should have a label", b.get_type());
            // }
            let l = stores.label_store.resolve(b.get_label());
            // println!("value: {}",l);
            // path = path.join(path)
            path.push(l)
        }
        let path = PathBuf::from_iter(path.iter().rev());
        Position {
            file: path,
            offset,
            len,
        }
    }

    fn peek_parent_node(&self) -> Option<NodeIdentifier> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.nodes[self.sps.parents[i]];
        Some(r)
    }
    fn peek_offset(&self) -> Option<usize> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.offsets[i];
        Some(r)
    }
    fn peek_node(&self) -> Option<NodeIdentifier> {
        if self.i == 0 {
            return None;
        }
        let i = self.i - 1;
        let r = self.sps.nodes[i];
        Some(r)
    }

    fn new(sps: &'a StructuralPositionStore, x: usize) -> Self {
        Self { sps, i: x + 1 }
    }
}

impl<'a> Iterator for ExploreStructuralPositions<'a> {
    type Item = NodeIdentifier;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == 0 {
            return None;
        } //println!("next: {} {}", self.i, self.sps.parents[self.i - 1]);
        let i = self.i - 1;
        let r = self.sps.nodes[i];
        if i > 0 {
            self.i = self.sps.parents[i] + 1;
        } else {
            self.i = i;
        }
        Some(r)
    }
}

impl<'a> From<(&'a StructuralPositionStore, SpHandle)> for ExploreStructuralPositions<'a> {
    fn from((sps, x): (&'a StructuralPositionStore, SpHandle)) -> Self {
        Self::new(sps, x.0)
    }
}

impl StructuralPositionStore {
    pub fn push_up_scout(&self, s: &mut Scout) -> Option<NodeIdentifier> {
        s.up(self)
    }

    pub fn ends_positions(&self, stores: &SimpleStores, ends: &[SpHandle]) -> Vec<Position> {
        let mut r = vec![];
        for x in ends.iter() {
            let x = x.0;
            // let parents = self.parents.iter().peekable();
            let it = ExploreStructuralPositions::from((self, SpHandle(x)));
            r.push(it.make_position(stores));
        }
        r
    }

    /// would ease approximate comparisons with other ASTs eg. spoon
    /// the basic idea would be to take the position of the parent.
    /// would be better to directly use a relaxed comparison.
    pub fn to_relaxed_positions(&self, _stores: &SimpleStores) -> Vec<Position> {
        todo!()
    }

    pub fn check_with(&self, stores: &SimpleStores, scout: &Scout) -> Result<(), String> {
        scout.path.check(stores).map_err(|_| "bad path")?;
        if self.nodes.is_empty() {
            return Ok(());
        }
        let mut i = scout.root;
        if !scout.path.nodes.is_empty() {
            let e = scout.path.nodes[0];
            let p = self.nodes[i];
            let o = scout.path.offsets[0];
            if o == 0 {
                if i != 0 {
                    return Err(format!("bad offset"));
                }
                return Ok(());
            }
            let o = o - 1;
            let b = stores.node_store.resolve(p);
            if !b.has_children() || Some(e) != b.child(&o.to_u16().expect("too big")) {
                return Err(if b.has_children() {
                    format!("error on link: {} {} {:?}", b.child_count(), o, p,)
                } else {
                    format!("error no children on link: {} {:?}", o, p,)
                });
            }
        }

        while i > 0 {
            let e = self.nodes[i];
            let o = self.offsets[i] - 1;
            let p = self.nodes[self.parents[i]];
            let b = stores.node_store.resolve(p);
            if !b.has_children() || Some(e) != b.child(&o.to_u16().expect("too big")) {
                return Err(if b.has_children() {
                    format!("error: {} {} {:?}", b.child_count(), o, p,)
                } else {
                    format!("error no children: {} {:?}", o, p,)
                });
            }
            i -= 1;
        }
        Ok(())
    }

    pub fn check(&self, stores: &SimpleStores) -> Result<(), String> {
        assert_eq!(self.offsets.len(), self.parents.len());
        assert_eq!(self.nodes.len(), self.parents.len());
        if self.nodes.is_empty() {
            return Ok(());
        }
        let mut i = self.nodes.len() - 1;

        while i > 0 {
            let e = self.nodes[i];
            let o = self.offsets[i] - 1;
            let p = self.nodes[self.parents[i]];
            let b = stores.node_store.resolve(p);
            if !b.has_children() || Some(e) != b.child(&o.to_u16().expect("too big")) {
                return Err(if b.has_children() {
                    format!("error: {} {} {:?}", b.child_count(), o, p,)
                } else {
                    format!("error no children: {} {:?}", o, p,)
                });
            }
            i -= 1;
        }
        Ok(())
    }
}

impl StructuralPositionStore {
    pub fn push(&mut self, x: &mut Scout) -> SpHandle {
        assert_eq!(x.path.nodes.len(), x.path.offsets.len());
        if x.path.offsets.is_empty() {
            return SpHandle(x.root);
        }
        assert!(!x.path.offsets[1..].contains(&0), "{:?}", &x.path.offsets);
        if x.path.offsets[0] == 0 {
            assert!(x.root == 0, "{:?} {}", &x.path.offsets, &x.root);
            if x.path.offsets.len() == 1 {
                return SpHandle(0);
            }
            let l = x.path.nodes.len() - 2;
            let o = self.parents.len();
            self.nodes.extend(&x.path.nodes[1..]);

            self.parents.push(x.root);
            self.parents
                .extend((o..o + l).into_iter().collect::<Vec<_>>());

            self.offsets.extend(&x.path.offsets[1..]);
            x.root = self.nodes.len() - 1;
            x.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            }
        } else {
            let l = x.path.nodes.len() - 1;
            let o = self.parents.len();
            self.nodes.extend(x.path.nodes.clone());
            self.parents.push(x.root);
            self.parents
                .extend((o..o + l).into_iter().collect::<Vec<_>>());
            self.offsets.extend(&x.path.offsets);
            // self.ends.push(self.nodes.len() - 1);
            x.root = self.nodes.len() - 1;
            x.path = StructuralPosition {
                nodes: vec![],
                offsets: vec![],
            }
            // x.path = StructuralPosition::with_offset(x.path.current_node(), x.path.current_offset());
        }

        // if !x.path.offsets.is_empty() && x.path.offsets[0] == 0 {
        //     assert!(x.root == 0, "{:?} {}", &x.path.offsets, &x.root);
        // }

        assert!(
            self.offsets.is_empty() || !self.offsets[1..].contains(&0),
            "{:?}",
            &self.offsets
        );
        assert_eq!(self.offsets.len(), self.parents.len());
        assert_eq!(self.nodes.len(), self.parents.len());
        SpHandle(self.nodes.len() - 1)
    }
}

impl From<StructuralPosition> for StructuralPositionStore {
    fn from(x: StructuralPosition) -> Self {
        let l = x.nodes.len();
        assert!(!x.offsets[1..].contains(&0));
        let nodes = x.nodes;
        Self {
            nodes,
            parents: (0..l).into_iter().collect(),
            offsets: x.offsets,
            // ends: vec![],
        }
    }
}

impl Default for StructuralPositionStore {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            parents: Default::default(),
            offsets: Default::default(),
            // ends: Default::default(),
        }
    }
}
