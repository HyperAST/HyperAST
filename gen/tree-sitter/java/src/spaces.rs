use std::panic;

use bitvec::{
    array::BitArray, bits, order::Lsb0, prelude::BitVec, slice::BitRefIter, store::BitStore,
};

use crate::vec_map_store::{ArrayOffset, Convertible, VecMapStore};

pub(crate) struct Spaces<Small: BitStore> {
    data: BitArray<Lsb0, Small>,
}

// just a field with Box<...> is 8 bytes
// an enum with Box<...> and 2 fields or 1 + other stuff is 16 bytes

#[derive(Debug)]
pub(crate) struct SpacesStore<T: BitStore + ArrayOffset, const U: usize> {
    medium: VecMapStore<BitArray<Lsb0, [T; U]>, T>,
    large: VecMapStore<BitVec<Lsb0, T>, T>,
}

impl<T: BitStore + ArrayOffset + Clone> SpacesStore<T, 4> {
    pub fn store(&mut self, s: &[u8]) -> Spaces<T> {
        Spaces::from(self, s)
    }

    pub fn store_relativized(&mut self, parent: &[u8], s: &[u8]) -> Spaces<T> {
        Spaces::realtivised(self, parent, s)
    }

    pub fn get(&mut self, s: &Spaces<T>) -> String {
        s.get_as_iter(self, |it| Spaces::bit_iter_to_str(it, "0"))
    }

    pub fn get_unrelativized(&mut self, parent: &[u8], s: &Spaces<T>) -> String {
        s.get_as_iter(self, |it| {
            Spaces::bit_iter_to_str(it, std::str::from_utf8(parent).unwrap())
        })
        .replace("0", std::str::from_utf8(parent).unwrap())
    }
}

enum Space {
    S,
    N,
    T,
    O,
    End,
}

type State = Space;

impl<Small: BitStore + ArrayOffset + Copy> Spaces<Small> {
    fn bit_iter_to_str(mut it: BitRefIter<Lsb0, Small>, parent: &str) -> String {
        let mut r = String::new();
        it.next();
        let mut s = State::S;
        for x in it {
            if *x {
                match s {
                    State::S => r.push(' '),
                    State::O => r.push_str(parent),
                    State::N => r.push('\n'),
                    State::T => r.push('\t'),
                    State::End => panic!(),
                }
                s = State::S;
            } else {
                match s {
                    State::S => s = State::O,
                    State::O => s = State::N,
                    State::N => s = State::T,
                    State::T => s = State::End,
                    State::End => break,
                }
            }
        }
        r
    }

    fn get_as_iter<'a, T, F: FnOnce(BitRefIter<Lsb0, Small>) -> T>(
        &'a self,
        spaces_store: &'a SpacesStore<Small, 4>,
        f: F,
    ) -> T {
        let data: &'a BitArray<Lsb0, Small> = &self.data;
        if data[0] {
            // inline
            f(data.iter().by_ref())
        } else if data[1] {
            // normal
            let mut id = data.clone();
            id.set(0, false);
            id.set(1, false);
            let i = id.as_raw_slice()[0].to() >> 2;
            let x = spaces_store.medium.resolve(&Convertible::from(i));
            f(x.iter().by_ref())
        } else {
            // big
            let mut id = data.clone();
            id.set(0, false);
            id.set(1, false);
            let i = id.as_raw_slice()[0].to() >> 2;
            let x = spaces_store.large.resolve(&Convertible::from(i));
            f(x.iter().by_ref())
        }
    }
}

impl<Small: BitStore + ArrayOffset + Clone> Spaces<Small> {
    fn from(spaces_store: &mut SpacesStore<Small, 4>, x: &[u8]) -> Self {
        let mut r: BitVec<Lsb0, Small> = BitVec::new();
        r.push(true);
        for x in x {
            match x {
                b' ' => r.extend_from_bitslice(bits![1]),
                b'0' => r.extend_from_bitslice(bits![0, 1]),
                b'\n' => r.extend_from_bitslice(bits![0, 0, 1]),
                b'\t' => r.extend_from_bitslice(bits![0, 0, 0, 1]),
                _ => panic!(),
            };
        }
        Spaces::try_inline(spaces_store, r)
    }

    fn realtivised(spaces_store: &mut SpacesStore<Small, 4>, parent: &[u8], x: &[u8]) -> Self {
        let mut r: BitVec<Lsb0, Small> = BitVec::new();
        r.push(true);
        let mut tmp: BitVec<Lsb0, Small> = BitVec::new();
        let mut i = 0;
        for x in x {
            let c = match x {
                b' ' => {
                    tmp.extend_from_bitslice(bits![1]);
                    b' '
                }
                b'0' => {
                    tmp.extend_from_bitslice(bits![0, 1]);
                    b'0'
                }
                b'\n' => {
                    tmp.extend_from_bitslice(bits![0, 0, 1]);
                    b'\n'
                }
                b'\t' => {
                    tmp.extend_from_bitslice(bits![0, 0, 0, 1]);
                    b'\t'
                }
                _ => panic!(),
            };
            if i < parent.len() && parent[i] == c {
                i += 1;
                if i == parent.len() {
                    r.extend_from_bitslice(bits![0, 1]);
                    tmp.clear();
                }
            } else {
                i = 0;
                r.extend_from_bitslice(&tmp);
                tmp.clear();
            }
        }
        Spaces::try_inline(spaces_store, r)
    }

    fn try_inline(spaces_store: &mut SpacesStore<Small, 4>, r: BitVec<Lsb0, Small>) -> Self {
        if r.len() <= std::mem::size_of::<Small>() * 8 {
            let a = r.as_raw_slice()[0];
            Self {
                data: BitArray::new(a), //from_bits_slice(r.as_bitslice()),
            }
        } else if r.len() <= std::mem::size_of::<u64>() * 8 {
            let b = r.as_raw_slice();
            let z = &Convertible::from(0);
            let a = [
                *b.get(0).unwrap_or(z),
                *b.get(1).unwrap_or(z),
                *b.get(2).unwrap_or(z),
                *b.get(3).unwrap_or(z),
            ];

            let i = spaces_store.medium.get_or_insert(BitArray::new(a));

            let mut data: BitArray<Lsb0, Small> =
                BitArray::new(Convertible::from(Convertible::to(&i) << 2));
            data.set(0, false);
            data.set(1, true);
            Self { data }
        } else {
            let i = spaces_store.large.get_or_insert(r);
            let mut data: BitArray<Lsb0, Small> =
                BitArray::new(Convertible::from(Convertible::to(&i) << 2));
            data.set(0, false);
            data.set(1, false);
            Self { data }
        }
    }
}

#[inline]
fn aux_eq(x: u8, c: char) -> usize {
    if x == c as u8 {
        1
    } else {
        0
    }
}

type BigSpacesIndentifier = u16;

// pub(crate) enum CompressedNode {
//     Type(u16),
//     Label {
//         label: u16,
//         kind: u16,
//     },
//     Children2 {
//         child1: u16,
//         child2: u16,
//         kind: u16,
//     },
//     Children {
//         // children: Box<[u16]>,
//     },
//     Spaces(Spaces<u16>),
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_size() {
        println!("{}", size_of::<Spaces<u16>>());
        println!("{}", size_of::<Spaces<u32>>());
        println!("{}", size_of::<Vec<u8>>());
        println!("{}", size_of::<std::mem::ManuallyDrop<Box<[u8]>>>());
        println!("{}", size_of::<std::mem::ManuallyDrop<u64>>());
        // println!("{}", size_of::<CompressedNode>());
        println!("{}", size_of::<u16>());
    }

    fn id(spaces_store: &mut SpacesStore<u16, 4>, input: &str) {
        let r = spaces_store.store(input.as_bytes());
        let output = spaces_store.get(&r);
        assert_eq!(input, output);
    }

    #[test]
    fn test_identity() {
        let mut spaces_store: SpacesStore<u16, 4> = SpacesStore {
            medium: VecMapStore::new(Default::default()),
            large: VecMapStore::new(Default::default()),
        };
        id(&mut spaces_store, "    ");
        id(&mut spaces_store, "\n    ");
        id(&mut spaces_store, "\t    ");
        id(&mut spaces_store, "\n\n\t\t");
        id(&mut spaces_store, "\n\n\t\t ");
        id(&mut spaces_store, "\n\n\t\t\t");
        id(&mut spaces_store, "\n\n\t\t\t");
        id(&mut spaces_store, "\n\n\t\t  ");
        id(&mut spaces_store, "\n\n\t\t  ");
        id(&mut spaces_store, "\n\n\t\t   ");
        id(&mut spaces_store, "\n\n\n\n\n\n\n\t\t   ");
        id(&mut spaces_store, "\n\n\n\n\n\n\n            ");
        id(&mut spaces_store, "\n\n\n\n\n\n\n                        ");
        id(
            &mut spaces_store,
            "\n\n\n\n\n\n\n                                    ",
        );
        id(
            &mut spaces_store,
            "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n0                                    ",
        );
        id(&mut spaces_store, "\n\n\t\t  ");
        println!("{:?}", &spaces_store);
    }

    fn rel(spaces_store: &mut SpacesStore<u16, 4>, parent: &str, input: &str) {
        let r = spaces_store.store_relativized(parent.as_bytes(), input.as_bytes());
        let output = spaces_store.get_unrelativized(&parent.as_bytes(), &r);
        assert_eq!(input, output);
    }

    #[test]
    fn test_rel_identity() {
        let mut spaces_store: SpacesStore<u16, 4> = SpacesStore {
            medium: VecMapStore::new(Default::default()),
            large: VecMapStore::new(Default::default()),
        };
        rel(&mut spaces_store, "\n", "\n    ");
        rel(&mut spaces_store, "\n", "    ");
        rel(&mut spaces_store, "\n    ", "\n    ");
        rel(&mut spaces_store, "\n", "\t    ");
        rel(&mut spaces_store, "\n", "\n\n\t\t");
        rel(&mut spaces_store, "\n", "\n\n\t\t ");
        rel(&mut spaces_store, "\n", "\n\n\t\t\t");
        rel(&mut spaces_store, "\n", "\n\n\t\t\t");
        rel(&mut spaces_store, "\n", "\n\n\t\t  ");
        rel(&mut spaces_store, "\n", "\n\n\t\t  ");
        rel(&mut spaces_store, "\n", "\n\n\t\t   ");
        rel(&mut spaces_store, "\n\t\t", "\n\n\n\n\n\n\n\t\t   ");
        rel(&mut spaces_store, "\n", "\n\n\n\n\n\n\n            ");
        rel(
            &mut spaces_store,
            "\n\n\n\n\n\n\n        ",
            "\n\n\n\n\n\n\n                        ",
        );
        rel(
            &mut spaces_store,
            "\n",
            "\n\n\n\n\n\n\n                                    ",
        );
        rel(
            &mut spaces_store,
            "\n",
            "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n                                    ",
        );
        rel(&mut spaces_store, "\n", "\n\n\t\t  ");
        println!("{:?}", &spaces_store);
    }

    fn rel2(
        spaces_store: &mut SpacesStore<u16, 4>,
        parent: &str,
        parent_new: &str,
        input: &str,
        expected: &str,
    ) {
        let r = spaces_store.store_relativized(parent.as_bytes(), input.as_bytes());
        println!("{}", spaces_store.get(&r));
        let output = spaces_store.get_unrelativized(&parent_new.as_bytes(), &r);
        assert_eq!(expected, output);
    }

    /// retarget indentation
    #[test]
    fn test_rel2_identity() {
        let mut spaces_store: SpacesStore<u16, 4> = SpacesStore {
            medium: VecMapStore::new(Default::default()),
            large: VecMapStore::new(Default::default()),
        };
        rel2(&mut spaces_store, "\n", "\n", "\n    ", "\n    ");
        rel2(&mut spaces_store, "\n", "\n", "    ", "    ");
        rel2(&mut spaces_store, "\n", "\n    ", "\n    ", "\n        ");
        rel2(&mut spaces_store, "\n", "\n", "\t    ", "\t    ");
        rel2(&mut spaces_store, "\n", "\n", "\n\n\t\t", "\n\n\t\t");
        rel2(&mut spaces_store, "\n", "\n", "\n\n\t\t ", "\n\n\t\t ");
        rel2(&mut spaces_store, "\n", "\n", "\n\n\t\t\t", "\n\n\t\t\t");
        rel2(&mut spaces_store, "\n", "\n", "\n\n\t\t\t", "\n\n\t\t\t");
        rel2(&mut spaces_store, "\n", "\n", "\n\n\t\t  ", "\n\n\t\t  ");
        rel2(&mut spaces_store, "\n", "\n", "\n\n\t\t  ", "\n\n\t\t  ");
        rel2(&mut spaces_store, "\n", "\n", "\n\n\t\t   ", "\n\n\t\t   ");
        rel2(
            &mut spaces_store,
            "\n\t\t",
            "\n\t\t",
            "\n\n\n\n\n\n\n\t\t    ",
            "\n\n\n\n\n\n\n\t\t    ",
        );
        rel2(
            &mut spaces_store,
            "\n\t\t",
            "\n    ",
            "\n\n\n\n\n\n\n\t\t    ",
            "\n\n\n\n\n\n\n        ",
        );
        rel2(
            &mut spaces_store,
            "\n",
            "\n",
            "\n\n\n\n\n\n\n            ",
            "\n\n\n\n\n\n\n            ",
        );
        rel2(
            &mut spaces_store,
            "\n\n\n\n\n\n        ",
            "\n\n\n\n\n\n\n        ",
            "\n\n\n\n\n\n                        ",
            "\n\n\n\n\n\n\n                        ",
        );
        rel2(
            &mut spaces_store,
            "\n",
            "\n",
            "\n\n\n\n\n\n\n                                    ",
            "\n\n\n\n\n\n\n                                    ",
        );
        rel2(
            &mut spaces_store,
            "\n",
            "\n",
            "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n                                    ",
            "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n                                    ",
        );
        rel2(&mut spaces_store, "\n", "\n", "\n\n\t\t  ", "\n\n\t\t  ");
        println!("{:?}", &spaces_store);
    }
}
