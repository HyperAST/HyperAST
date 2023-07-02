//! Gather most of the common behaviors used to compute a path from an offset
use num::ToPrimitive;

use crate::{
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        nodes::HashedNodeRef,
    },
    types::{HyperAST, IterableChildren, NodeStore, WithChildren, WithSerialization},
};

/// must be in a file
pub fn resolve_range<'store, HAST>(
    root: HAST::IdN,
    start: usize,
    end: Option<usize>,
    stores: &'store HAST,
) -> (HAST::IdN, Vec<usize>)
where
    HAST:
        HyperAST<'store, T = HashedNodeRef<'store>, IdN = NodeIdentifier, Label = LabelIdentifier>,
{
    enum RangeStatus {
        Inside(usize),
        Outside(usize),
        Right(usize, usize),
        Left(usize, usize),
    }

    fn range_status(start: usize, offset: usize, len: usize, end: usize) -> RangeStatus {
        if start < offset {
            if offset + len < end {
                RangeStatus::Inside((offset - start) + (offset + len - end))
            } else if offset + len == end {
                RangeStatus::Inside((offset - start) + (offset + len - end))
            } else {
                RangeStatus::Right(offset - start, offset + len - end)
            }
        } else if start == offset {
            if offset + len < end {
                RangeStatus::Inside((offset - start) + (offset + len - end))
            } else if offset + len == end {
                RangeStatus::Inside(0)
            } else {
                RangeStatus::Right(offset - start, offset + len - end)
            }
        } else {
            if offset + len < end {
                RangeStatus::Left(start - offset, end - (offset + len))
            } else if offset + len == end {
                RangeStatus::Left(start - offset, end - (offset + len))
            } else {
                RangeStatus::Inside((start - offset) + (end - (offset + len)))
            }
        }
    }
    let mut offset = 0;
    // let mut parent_status = RangeStatus::Outside(0);
    // let mut prev_status = RangeStatus::Outside(0);
    // let mut prev = root;
    let mut x = root;
    let mut offsets = vec![];
    'main: loop {
        let b = stores.node_store().resolve(&x);
        // dbg!(offset);
        // dbg!(b.get_type());
        // dbg!(o.to_usize().unwrap());
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            for (y, child_id) in cs.iter_children().enumerate() {
                let b = stores.node_store().resolve(child_id);

                let len = b.try_bytes_len().unwrap_or(0).to_usize().unwrap();
                // let rs = range_status(start, offset, len, end);
                // dbg!(b.get_type(), start, offset, len, end);
                if offset + len < start {
                    // not yet reached something
                } else if end.map_or(true, |end| offset + len <= end) {
                    break 'main;
                } else {
                    offsets.push(y);
                    x = *child_id;
                    break;
                }
                offset += len;
            }
        } else {
            break;
        }
    }
    (x, offsets)
}
