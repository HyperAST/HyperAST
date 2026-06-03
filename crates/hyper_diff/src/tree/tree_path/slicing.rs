use super::*;
/// advanced iterator used to get back path as Idx from compressed path
#[derive(Clone)]
pub struct IntoIter<Idx> {
    is_even: bool,
    side: bool,
    slice: Box<[u8]>,
    phantom: PhantomData<*const Idx>,
}

impl<Idx: PrimInt> IntoIter<Idx> {
    pub fn new(bits: Box<[u8]>) -> Self {
        Self {
            is_even: (bits[0] & 1) == 1,
            side: true,
            slice: bits,
            phantom: PhantomData,
        }
    }
}

impl<Idx: PrimInt> Iterator for IntoIter<Idx> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            return None;
        }
        if self.is_even && self.slice.len() == 1 && self.side {
            self.slice = self.slice[1..].into();
            return None;
        }
        let mut c = num_traits::zero();
        loop {
            let a = &self.slice[0];
            let a = if self.side {
                (a & 0b11110000) >> 4
            } else {
                a & 0b00001111
            };
            let mut br = false;
            let b = if a == 16 - 1 {
                128
            } else if a == 16 - 2 {
                32
            } else if a == 16 - 3 {
                16 - 3
            } else {
                br = true;
                a
            };
            c = c + cast(b).unwrap();
            if self.side {
                self.slice = self.slice[1..].into();
            }
            self.side = !self.side;
            if br {
                break;
            }
            if self.is_even && self.slice.len() == 1 && self.side {
                self.slice = self.slice[1..].into();
                return Some(c);
            }
        }
        Some(c)
    }
}
