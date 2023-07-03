//! Gather most of the common behaviors used to compute a path from an offset
use num::ToPrimitive;

use crate::types::{HyperAST, IterableChildren, NodeStore, WithChildren, WithSerialization};

/// must be in a file
pub fn resolve_range<'store, HAST>(
    root: HAST::IdN,
    start: usize,
    end: Option<usize>,
    stores: &'store HAST,
) -> (HAST::IdN, Vec<usize>)
where
    HAST: HyperAST<'store>,
    HAST::T: WithSerialization,
    HAST::IdN: Copy,
{
    let mut offset = 0;
    let mut x = root;
    let mut offsets = vec![];
    'main: loop {
        let b = stores.node_store().resolve(&x);
        if let Some(cs) = b.children() {
            let cs = cs.clone();
            for (y, child_id) in cs.iter_children().enumerate() {
                let b = stores.node_store().resolve(child_id);

                let len = b.try_bytes_len().unwrap_or(0).to_usize().unwrap();
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
