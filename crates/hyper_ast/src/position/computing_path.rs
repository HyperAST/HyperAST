//! Gather most of the common behaviors used to compute a path from an offset
use num::ToPrimitive;

use crate::types::{Childrn, HyperAST, WithChildren, WithSerialization};

/// must be in a file
pub fn resolve_range<'store, HAST>(
    root: HAST::IdN,
    start: usize,
    end: Option<usize>,
    stores: &'store HAST,
) -> (HAST::IdN, Vec<usize>)
where
    HAST: HyperAST,
    for<'t> <HAST as crate::types::AstLending<'t>>::RT: WithSerialization,
    HAST::IdN: crate::types::NodeId<IdN = HAST::IdN>,
    HAST::IdN: Copy,
{
    let mut offset = 0;
    let mut x = root;
    let mut offsets = vec![];
    'main: loop {
        let b = stores.resolve(&x);
        if let Some(cs) = b.children() {
            for (y, child_id) in cs.iter_children().enumerate() {
                let b = stores.resolve(&child_id);

                let len = b.try_bytes_len().unwrap_or(0).to_usize().unwrap();
                if offset + len < start {
                    // not yet reached something
                } else if end.map_or(true, |end| offset + len <= end) {
                    break 'main;
                } else {
                    offsets.push(y);
                    x = child_id;
                    break;
                }
                offset += len;
            }
        } else {
            break;
        };
    }
    (x, offsets)
}
