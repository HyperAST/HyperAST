pub mod serialize;

use std::marker::PhantomData;

use serialize::{CachedHasher, Keyed, MySerialize, Table};

use crate::filter::default::VaryHasher;

pub struct BulkHasher<'a, It, S, H>
where
    It: Iterator,
    H: VaryHasher<S>,
{
    table: Table<H>,
    it: It,
    branched: Vec<S>,
    phantom: PhantomData<(*const H, &'a ())>,
}

impl<'a, It, S, H> From<It> for BulkHasher<'a, It, S, H>
where
    It: Iterator,
    H: VaryHasher<S>,
{
    fn from(it: It) -> Self {
        Self {
            table: Default::default(),
            it,
            branched: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<'a, It, H> Iterator for BulkHasher<'a, It, u8, H>
where
    It: Iterator,
    It::Item: MySerialize + Keyed<usize>,
    H: VaryHasher<u8>,
{
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        if let Some(x) = self.branched.pop() {
            return Some(x);
        }
        let x = self.it.next()?;
        let s = CachedHasher::<usize, u8, H>::new(&mut self.table, x.key());
        let x = x.serialize(s).unwrap();
        let x = &self.table[x];
        self.branched = x.iter().map(VaryHasher::finish).collect();
        self.next()
    }
}

impl<'a, It, H> Iterator for BulkHasher<'a, It, u16, H>
where
    It: Iterator,
    It::Item: MySerialize + Keyed<usize>,
    H: VaryHasher<u16>,
{
    type Item = u16;
    fn next(&mut self) -> Option<u16> {
        if let Some(x) = self.branched.pop() {
            return Some(x);
        }
        let x = self.it.next()?;
        let s = CachedHasher::<usize, u16, H>::new(&mut self.table, x.key());
        let x = x.serialize(s).unwrap();
        let x = &self.table[x];
        self.branched = x.iter().map(VaryHasher::finish).collect();
        self.next()
    }
}
