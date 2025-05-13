//! Gather most of the common behaviors used to compute positions in an HyperAST

use super::{Position, StructuralPosition, TreePath};
use crate::types::{
    Children, Childrn, HyperAST, HyperType, LabelStore, Labeled, WithChildren, WithSerialization,
};
use crate::{PrimInt, types::WithStats};
use num::ToPrimitive;
use std::path::PathBuf;

/// precondition: root node do not contain a File node
/// TODO make whole thing more specific to a path in a tree
pub fn compute_range<'store, It, HAST>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: HAST,
) -> (usize, usize, HAST::IdN)
where
    HAST: HyperAST,
    HAST::IdN: Copy,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
    for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithSerialization,
    It: Iterator,
    It::Item: PrimInt,
{
    let mut offset = 0;
    let mut x = root;
    for o in offsets {
        let cs = stores.resolve(&x);
        let Some(cs) = cs.children() else {
            break;
        };
        for y in 0..o.to_usize().unwrap() {
            let id = &cs[num::cast(y).unwrap()];
            let b = stores.resolve(id);

            offset += b.try_bytes_len().unwrap_or(0).to_usize().unwrap();
        }
        let Some(a) = cs.get(num::cast(o).unwrap()) else {
            break;
        };
        x = *a;
    }
    let b = stores.resolve(&x);

    let len = b.try_bytes_len().unwrap_or(0).to_usize().unwrap();
    (offset, offset + len, x)
}

pub fn compute_position<HAST, It>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: HAST,
) -> (Position, HAST::IdN)
where
    It::Item: Clone,
    HAST::IdN: Clone,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
    HAST: HyperAST,
    for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithSerialization + WithChildren,
    It: Iterator<Item = HAST::Idx>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path = vec![];
    for o in &mut *offsets {
        let b = stores.resolve(&x);

        let t = stores.resolve_type(&x);

        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }

        let Some(cs) = b.children() else { break };
        if !t.is_directory() {
            for y in cs.before(o.clone()).iter_children() {
                let b = stores.resolve(&y);
                offset += b.try_bytes_len().unwrap().to_usize().unwrap();
            }
        }
        let Some(a) = cs.get(o) else { break };
        x = a.clone();
    }
    assert!(offsets.next().is_none());
    let b = stores.resolve(&x);
    let t = stores.resolve_type(&x);
    if t.is_directory() || t.is_file() {
        let l = stores.label_store().resolve(b.get_label_unchecked());
        path.push(l);
    }

    let len = if !t.is_directory() {
        b.try_bytes_len().unwrap().to_usize().unwrap()
    } else {
        0
    };
    let file = PathBuf::from_iter(path.iter());
    (Position::new(file, offset, len), x)
}

pub fn compute_position_and_nodes<'store, HAST, It: Iterator>(
    root: HAST::IdN,
    offsets: &mut It,
    stores: &'store HAST,
) -> (Position, Vec<HAST::IdN>)
where
    It::Item: Clone,
    HAST::IdN: Clone,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
    HAST: HyperAST,
    for<'t> <HAST as crate::types::AstLending<'t>>::RT:
        WithSerialization + WithChildren<ChildIdx = It::Item>,
{
    let mut offset = 0;
    let mut x = root;
    let mut path_ids = vec![];
    let mut path = vec![];
    for o in &mut *offsets {
        let b = stores.resolve(&x);

        let t = stores.resolve_type(&x);
        if t.is_directory() || t.is_file() {
            let l = stores.label_store().resolve(b.get_label_unchecked());
            path.push(l);
        }

        let Some(cs) = b.children() else {
            break;
        };
        if !t.is_directory() {
            for y in cs.before(o.clone()).iter_children() {
                let b = stores.resolve(&y);
                offset += b
                    .try_bytes_len()
                    .ok_or_else(|| MissingByteLenError(stores.resolve_type(&x)))
                    .unwrap()
                    .to_usize()
                    .unwrap();
            }
        }
        let Some(a) = cs.get(o) else { break };
        x = a.clone();
        path_ids.push(x.clone());
    }
    assert!(offsets.next().is_none());
    let b = stores.resolve(&x);
    let t = stores.resolve_type(&x);
    if t.is_directory() || t.is_file() {
        let l = stores.label_store().resolve(b.get_label_unchecked());
        path.push(l);
    }

    let len = if !t.is_directory() {
        b.try_bytes_len()
            .ok_or_else(|| MissingByteLenError(stores.resolve_type(&x)))
            .unwrap()
            .to_usize()
            .unwrap()
    } else {
        0
    };
    let file = PathBuf::from_iter(path.iter());
    path_ids.reverse();
    (Position::new(file, offset, len), path_ids)
}

impl<IdN: Copy, Idx: PrimInt> StructuralPosition<IdN, Idx> {
    pub fn make_position<'store, HAST>(&self, stores: &'store HAST) -> Position
    where
        HAST: HyperAST<IdN = IdN, Idx = Idx>,
        for<'t> crate::types::LendT<'t, HAST>: WithSerialization,
        IdN: crate::types::NodeId<IdN = IdN>,
    {
        if cfg!(debug_assertions) {
            self.check(stores)
                .expect("a well formed structural position");
        }
        let mut from_file = false;
        let x = *self.node().unwrap();
        let b = stores.resolve(&x);

        let t = stores.resolve_type(&x);
        let len = if let Some(y) = b.try_bytes_len() {
            if !(t.is_file() || t.is_directory()) {
                from_file = true;
            }
            y as usize
        } else {
            0
        };
        let mut offset = 0;
        let mut path = vec![];
        if self.parents.is_empty() {
            let file = PathBuf::from_iter(path.iter().rev());
            return Position::new(file, offset, len);
        }
        let mut i = self.parents.len() - 1;
        if from_file {
            loop {
                if !(i > 0) {
                    break;
                }
                let p = self.parents[i - 1];
                let b = stores.resolve(&p);
                let t = stores.resolve_type(&p);
                let o = self.offsets[i];
                let c: usize = b
                    .children()
                    .unwrap()
                    .before(o - num::one())
                    .iter_children()
                    .map(|x| {
                        stores
                            .resolve(&x)
                            .try_bytes_len()
                            .ok_or_else(|| MissingByteLenError(stores.resolve_type(&x)))
                            .unwrap() as usize
                    })
                    .sum();
                offset += c;
                if t.is_file() {
                    from_file = false;
                    i -= 1;
                    break;
                } else {
                    debug_assert!(
                        !t.is_directory(),
                        "a file should have been crossed before reaching a dir"
                    );
                    i -= 1;
                }
            }
        }
        if self.parents.is_empty() {
        } else if !from_file {
            loop {
                let n = self.parents[i];
                let b = stores.resolve(&n);
                let l = stores.label_store().resolve(b.get_label_unchecked());
                path.push(l);
                if i == 0 {
                    break;
                } else {
                    i -= 1;
                }
            }
        } else {
            let p = self.parents[i - 1];
            let b = stores.resolve(&p);
            let o = self.offsets[i];
            let c: usize = b
                .children()
                .unwrap()
                .before(o - num::one())
                .iter_children()
                .map(|x| {
                    stores
                        .resolve(&x)
                        .try_bytes_len()
                        .ok_or_else(|| MissingByteLenError(stores.resolve_type(&x)))
                        .unwrap() as usize
                })
                .sum();
            offset += c;
        }

        let file = PathBuf::from_iter(path.iter().rev());
        Position::new(file, offset, len)
    }

    pub fn make_file_line_range<'store, HAST>(&self, stores: &'store HAST) -> (String, usize, usize)
    where
        HAST: HyperAST<IdN = IdN, Idx = Idx>,
        for<'t> crate::types::LendT<'t, HAST>: WithStats + WithSerialization,
        IdN: crate::types::NodeId<IdN = IdN>,
    {
        if cfg!(debug_assertions) {
            self.check(stores)
                .expect("a well formed structural position");
        }
        let mut from_file = false;
        let x = *self.node().unwrap();
        let b = stores.resolve(&x);

        let t = stores.resolve_type(&x);
        if !(t.is_file() || t.is_directory()) {
            from_file = true;
        }

        let len = b.line_count();
        let mut offset = 0;
        let mut path = vec![];
        if self.parents.is_empty() {
            let file = PathBuf::from_iter(path.iter().rev())
                .to_string_lossy()
                .to_string();
            return (file, offset, len);
        }
        let mut i = self.parents.len() - 1;
        if from_file {
            loop {
                if !(i > 0) {
                    break;
                }
                let p = self.parents[i - 1];
                let b = stores.resolve(&p);

                let o = self.offsets[i];
                let c: usize = b
                    .children()
                    .unwrap() // always have children as we are going up
                    .before(o - num::one())
                    .iter_children()
                    .map(|x| stores.resolve(&x).line_count())
                    .sum();
                offset += c;
                let t = stores.resolve_type(&p);
                if t.is_file() {
                    from_file = false;
                    i -= 1;
                    break;
                } else {
                    debug_assert!(
                        !t.is_directory(),
                        "a file should have been crossed before reaching a dir"
                    );
                    i -= 1;
                }
            }
        }
        if self.parents.is_empty() {
        } else if !from_file {
            loop {
                let n = self.parents[i];
                let b = stores.resolve(&n);
                let l = stores.label_store().resolve(b.get_label_unchecked());
                path.push(l);
                if i == 0 {
                    break;
                } else {
                    i -= 1;
                }
            }
        } else {
            if i == 0 {
                i += 1;
            }
            let p = self.parents[i - 1];
            let b = stores.resolve(&p);
            let o = self.offsets[i];

            // TODO make a debug assert node at offset o being that should correspond to prev node
            let c: usize = b
                .children()
                .unwrap() // always have children as we are going up
                .before(o - num::one())
                .iter_children()
                .map(|x| {
                    stores
                        .resolve(&x)
                        .try_bytes_len()
                        .ok_or_else(|| MissingByteLenError(stores.resolve_type(&x)))
                        .unwrap() as usize
                })
                .sum();
            offset += c;
        }

        let file = PathBuf::from_iter(path.iter().rev())
            .to_string_lossy()
            .to_string();
        (file, offset, len)
    }
}

/// Not an end-user error.
/// This error might be raised in case WithSerialization is missing the derived data
/// meaning:
///   depending on the type of node, the partiular derived data might not have been added
///   during the construction of the corresponding subtree
///   by Default a Directory does not have a length in bytes
///
/// TODO In order to deprecate this error, work has to be done
/// to only provide this accessor on non-directory subtrees
#[derive(Debug)]
pub(crate) struct MissingByteLenError<T: std::fmt::Debug>(T);
