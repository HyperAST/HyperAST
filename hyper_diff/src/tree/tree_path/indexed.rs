use super::*;

/// advanced iterator used to get back path as Idx from compressed path
#[derive(Clone)]
pub struct IntoIter<Idx> {
    is_even: bool,
    side: bool,
    slice: Box<[u8]>,
    adv: usize,
    phantom: PhantomData<*const Idx>,
}

impl<Idx: PrimInt> IntoIter<Idx> {
    pub fn new(bits: Box<[u8]>) -> Self {
        Self {
            is_even: (bits[0] & 1) == 1,
            side: true,
            slice: bits,
            adv: 0,
            phantom: PhantomData,
        }
    }
}

impl<Idx: PrimInt> Iterator for IntoIter<Idx> {
    type Item = Idx;

    fn next(&mut self) -> Option<Self::Item> {
        if self.adv >= self.slice.len() {
            return None;
        }
        if self.is_even && self.slice.len() - self.adv == 1 && self.side {
            self.adv += 1;
            return None;
        }
        let mut c = num_traits::zero();
        loop {
            let a = &self.slice[self.adv];
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
                self.adv += 1;
            }
            self.side = !self.side;
            if br {
                break;
            }
            if self.is_even && self.slice.len() - self.adv == 1 && self.side {
                self.adv += 1;
                return Some(c);
            }
        }
        Some(c)
    }
}

#[test]
fn identity() {
    let v = vec![
        1, 4684, 68, 46, 84, 684, 68, 46, 846, 4460, 0, 00, 8, 0, 8, 0, 0, 0, 1, 12, 1, 2, 1,
        21, 2, 1, 2, 12, 1,
    ];
    let path = CompressedTreePath::<u16>::from(v.clone());
    let bits = path.as_bytes();
    let res: Vec<_> = IntoIter::<u16>::new(bits.into()).collect();
    assert_eq!(v, res);
}
