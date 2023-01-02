use crate::tree::tree_path::IntoIter;

use num_traits::PrimInt;

use super::{Mree, MS};

// struct ArrayCompressedMapping<Id,Idx> {
//     mm: [Option<Id>;8],
//     offsets: [Idx;8],
// }

struct Remapper<'ms, It: Iterator, Ms: MS> {
    ms: &'ms Ms,
    source: Option<It>,
    node: Option<Ms::Id>,
    waiting: Option<IntoIter<Ms::Idx>>,
    waiting2: Option<Ms::Idx>,
    waiting3: Option<IntoIter<Ms::Idx>>,
}

impl<'ms, It, Ms: MS> Iterator for Remapper<'ms, It, Ms>
where
    It: Iterator<Item = Ms::Idx> + Clone, // add bound to get an hash of what remains
    It::Item: PrimInt,
    Ms::Id: Clone,
{
    type Item = Ms::Idx;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut waiting) = self.waiting.take() {
            if let Some(n) = waiting.next() {
                self.waiting = Some(waiting);
                return Some(n);
            }
        }
        if let Some(waiting) = self.waiting3.take() {
            self.waiting = Some(waiting);
            return self.next();
        }
        let n = self.source.as_mut()?.next()?;
        let r = self.ms.resolve(self.node.clone()?);
        // TODO check if rest of path exists with a bloom filter here, else return None

        if let Some(child) = r.definitely_mapped(n) {
            self.node = child.0;
            self.waiting = Some(child.1.into_iter());
            return self.next();
        }

        let child = r.maybe_mapped(n);

        for child in child {
            // If this whole stuff works, it is genius Xd
            // should focus on mm and add bloom filters to skip mree given a path, do this on nodes instead of children
            let mut new = Self::new(self.ms, child.0, self.source.clone()?);
            let n = new.next();
            if n.is_some() {
                self.source = new.source;
                self.waiting = Some(child.1.into_iter());
                self.waiting2 = n;
                self.waiting3 = new.waiting;
                return self.next();
            }
        }
        return None;
    }
}

impl<'ms, It: Iterator, Ms: MS> Remapper<'ms, It, Ms>
where
    It::Item: PrimInt,
{
    pub fn new(ms: &'ms Ms, root: Ms::Id, source: It) -> Self {
        Self {
            source: Some(source),
            ms,
            node: Some(root),
            waiting: None,
            waiting2: None,
            waiting3: None,
        }
    }
}

fn f<Ms: MS>() {}
