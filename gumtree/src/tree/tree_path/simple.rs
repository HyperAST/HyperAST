use super::*;

#[derive(Clone)]
pub struct SimpleTreePath<Idx> {
    vec: Vec<Idx>,
}

impl<Idx: PrimInt> IntoIterator for SimpleTreePath<Idx> {
    type Item = Idx;

    type IntoIter = std::vec::IntoIter<Idx>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<Idx: PrimInt> TreePath for SimpleTreePath<Idx> {
    type ItemIterator<'a> = IterSimple<'a, Idx> where Idx: 'a;
    fn iter(&self) -> Self::ItemIterator<'_> {
        IterSimple {
            internal: self.vec.iter(),
        }
    }

    fn extend(self, path: &[Idx]) -> Self {
        let mut vec = self.vec;
        vec.extend_from_slice(path);
        Self { vec }
    }
}
impl<Idx: Clone> From<&[Idx]> for SimpleTreePath<Idx> {
    fn from(value: &[Idx]) -> Self {
        Self {
            vec: value.to_vec(),
        }
    }
}
impl<Idx: Clone> From<Vec<Idx>> for SimpleTreePath<Idx> {
    fn from(vec: Vec<Idx>) -> Self {
        Self { vec }
    }
}

impl<Idx: Debug> Debug for SimpleTreePath<Idx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.vec.fmt(f)
    }
}

/// dumb wrapper to avoid problems with iterators typing
pub struct IterSimple<'a, Idx: 'a> {
    internal: core::slice::Iter<'a, Idx>,
}

impl<'a, Idx: 'a + Copy> Iterator for IterSimple<'a, Idx> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        self.internal.next().and_then(|x| Some(*x))
    }
}
