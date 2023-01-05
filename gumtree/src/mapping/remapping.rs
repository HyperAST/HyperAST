use crate::tree::tree_path::{CompressedTreePath, IntoIter};

use num_traits::PrimInt;

use super::{CompressedMappingStore, Mree};

// struct ArrayCompressedMapping<Id,Idx> {
//     mm: [Option<Id>;8],
//     offsets: [Idx;8],
// }

pub struct Remapper<'ms, It: Iterator, Ms: CompressedMappingStore> {
    ms: &'ms Ms,
    source: It,
    node: Option<Ms::Id>,
    waiting: Vec<IntoIter<Ms::Idx>>,
}

impl<'ms, It, Ms: CompressedMappingStore> Iterator for Remapper<'ms, It, Ms>
where
    It: Iterator<Item = Ms::Idx> + Clone, // add bound to get an hash of what remains
    It::Item: PrimInt,
    Ms::Id: Clone,
{
    type Item = Ms::Idx;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut waiting) = self.waiting.pop() {
            if let Some(n) = waiting.next() {
                self.waiting.push(waiting);
                return Some(n);
            }
        }
        let n = self.source.next()?;
        let r = self.ms.resolve(self.node.clone()?);
        // TODO check if rest of path exists with a bloom filter here, else return None

        if let Some(child) = r.definitely_mapped(n) {
            self.node = child.0;
            self.waiting.push(child.1.into_iter());
            return self.next();
        }

        let child = r.maybe_mapped(n);
        for child in child {
            // If this whole stuff works, it is genius Xd
            // should focus on mm and add bloom filters to skip mree given a path, do this on nodes instead of children
            let mut new = Self::new(self.ms, child.0.clone(), self.source.clone());//TODO take owned slice, then give it back
            let n = new.next();
            if let Some(n) = n {
                self.source = new.source;
                self.waiting.extend(new.waiting);
                self.waiting
                    .push(CompressedTreePath::from(vec![n]).into_iter());
                self.waiting.push(child.1.into_iter());
                // self.waiting = Some(child.1.into_iter());
                // self.waiting2 = n;
                // self.waiting3 = new.waiting;
                return self.next();
            } else if self.source.clone().next().is_none() && self.ms.resolve(child.0).is_mapped() {
                self.waiting.push(child.1.into_iter());
                return self.next();
            }
        }
        return None;
    }
}

impl<'ms, It: Iterator, Ms: CompressedMappingStore> Remapper<'ms, It, Ms>
where
    It::Item: PrimInt,
{
    pub fn new(ms: &'ms Ms, root: Ms::Id, source: It) -> Self {
        Self {
            source,
            ms,
            node: Some(root),
            waiting: vec![],
        }
    }
}