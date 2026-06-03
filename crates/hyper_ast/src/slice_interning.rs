#![allow(unused)]
use num::ToPrimitive;

pub type DefaultInterner<T> = BasicInterner<T>;
pub type DefaultIdentifier = BasicIdentifier;

/// just use a vec to store slices,
/// do not even duplicate.
/// But tries to be clever about len of slices
#[derive(Default)]
pub struct BasicInterner<T> {
    backends: [Vec<T>; 4],
    // no need to go higher than max offsets in slices
    offsets_n: Vec<u16>,
}

impl<T> BasicInterner<T> {
    fn intern(&mut self, value: &[T]) -> BasicIdentifier
    where
        T: Copy,
    {
        match value.len() {
            0 => unreachable!(),
            x if x < 4 => {
                let i = self.backends[x].len();
                self.backends[x].extend(value);
                BasicIdentifier(i << 2 | x)
            }
            x => {
                let i = self.offsets_n.len();
                let ii = self.backends[0].len();
                let x = ii - i * 4 + x;
                // each slice must be bigger than 3,
                // so let only store the difference
                self.offsets_n.push(x.to_u16().unwrap());
                self.backends[0].extend(value);
                BasicIdentifier(i << 2)
            }
        }
    }
    fn resolver(&self, id: BasicIdentifier) -> &[T] {
        let i = id.offset();
        match id.len_value() {
            0 if i == 0 => {
                let r: usize = self.offsets_n[i].into();
                let r = r + i * 4;
                &self.backends[0][..r]
            }
            0 => {
                let l: usize = self.offsets_n[i - 1].into();
                let l = l + i * 4 - 4;
                let r: usize = self.offsets_n[i].into();
                let r = r + i * 4;
                &self.backends[0][l..r]
            }
            x if x < 4 => &self.backends[x][i * x..i * x + x],
            _ => unreachable!(),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct BasicIdentifier(usize);

impl BasicIdentifier {
    fn len_value(&self) -> usize {
        self.0 & 0x11
    }
    fn offset(&self) -> usize {
        self.0 >> 2
    }
}

#[derive(Default)]
pub struct MinimalInterner<T> {
    offsets_u16: Vec<u32>,
    backend_u16: Vec<u16>,
    offsets_u32: Vec<u32>,
    backend_u32: Vec<u32>,
    offsets_u64: Vec<u32>,
    backend_u64: Vec<u64>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> MinimalInterner<T> {
    fn intern(&mut self, value: &[T]) -> MinimalIdentifier
    where
        T: Copy + Into<usize>,
    {
        if value.iter().all(|x| x.clone().into() <= u16::MAX as usize) {
            let i = self.offsets_u16.len();
            self.backend_u16
                .extend(value.iter().map(|x| x.clone().into() as u16));
            let ii = self.backend_u16.len();
            self.offsets_u16.push(ii.to_u32().unwrap());
            MinimalIdentifier(i << 2 | 0)
        } else if value.iter().all(|x| x.clone().into() <= u32::MAX as usize) {
            let i = self.offsets_u32.len();
            self.backend_u32
                .extend(value.iter().map(|x| x.clone().into() as u32));
            let ii = self.backend_u32.len();
            self.offsets_u32.push(ii.to_u32().unwrap());
            MinimalIdentifier(i << 2 | 1)
        } else {
            let i = self.offsets_u64.len();
            self.backend_u64
                .extend(value.iter().map(|x| x.clone().into() as u64));
            let ii = self.backend_u64.len();
            self.offsets_u64.push(ii.to_u32().unwrap());
            MinimalIdentifier(i << 2 | 2)
        }
    }
    fn resolver(&self, id: MinimalIdentifier) -> impl Iterator<Item = T> + use<'_, T>
    where
        T: From<usize>,
    {
        struct It<'a, T> {
            inner: &'a MinimalInterner<T>,
            o: usize,
            left: usize,
            right: usize,
        }
        impl<'a, T: From<usize>> Iterator for It<'a, T> {
            type Item = T;

            fn next(&mut self) -> Option<Self::Item> {
                if self.left == self.right {
                    return None;
                }
                let r = match self.o {
                    0 => self.inner.backend_u16[self.left] as usize,
                    1 => self.inner.backend_u32[self.left] as usize,
                    2 => self.inner.backend_u64[self.left] as usize,
                    _ => unreachable!(),
                };
                self.left += 1;
                Some(r.into())
            }
        }
        let i = id.offset();
        let right = match id.len_value() {
            0 => self.offsets_u16[i],
            1 => self.offsets_u32[i],
            2 => self.offsets_u64[i],
            _ => unreachable!(),
        } as usize;
        let left = match id.len_value() {
            _ if i == 0 => 0,
            0 => self.offsets_u16[i - 1],
            1 => self.offsets_u32[i - 1],
            2 => self.offsets_u64[i - 1],
            _ => unreachable!(),
        } as usize;
        It {
            o: id.len_value(),
            left,
            right,
            inner: self,
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct MinimalIdentifier(usize);

impl MinimalIdentifier {
    fn len_value(&self) -> usize {
        self.0 & 0x11
    }
    fn offset(&self) -> usize {
        self.0 >> 2
    }
}
