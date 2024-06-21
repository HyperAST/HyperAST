#[repr(C)]
pub(crate) struct Array<T> {
    pub(crate) contents: *mut T,
    pub(crate) size: u32,
    pub(crate) capacity: u32,
}

impl<T> Array<T> {
    pub(crate) fn len(&self) -> usize {
        self.size as usize
    }

    pub(crate) fn search_sorted_by<C: Ord, F: Fn(&T) -> C>(
        &self,
        f: F,
        needle: C,
    ) -> Option<usize> {
        unsafe { std::slice::from_raw_parts(self.contents, self.size as usize) }
            .binary_search_by_key(&needle, f)
            .ok()?
            .try_into()
            .ok()
    }
}

impl<T> Array<T> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        struct Iter<'a, T>(u32, &'a Array<T>);

        impl<'a, T> Iterator for Iter<'a, T> {
            type Item = &'a T;

            fn next(&mut self) -> Option<Self::Item> {
                if self.0 >= self.1.size {
                    return None;
                }
                let r = unsafe { self.1.contents.add(self.0 as usize) };

                self.0 += 1;

                Some(unsafe { &(*r) })
            }
        }

        assert!(self.size <= self.capacity);

        Iter(0, self)
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> std::ops::Index<I> for Array<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        // assert!(index < self.size as usize);
        // unsafe { self.contents.add(index).as_ref().unwrap() }
        let contents = unsafe { std::slice::from_raw_parts(self.contents, self.size as usize) };
        std::ops::Index::index(contents, index)
    }
}

impl<T> std::ops::IndexMut<usize> for Array<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.contents.add(index).as_mut().unwrap() }
    }
}

#[repr(C)]
pub(super) struct SymbolTable {
    characters: Array<std::ffi::c_char>,
    slices: Array<Slice>,
}

#[repr(C)]
pub(super) struct Slice {
    pub(super) offset: u32,
    pub(super) length: u32,
}

impl SymbolTable {
    pub(super) fn symbol_table_id_for_name(&self, name: &[std::ffi::c_char]) -> Option<usize> {
        for i in 0..self.slices.len() {
            let slice = &self.slices[i];
            if slice.length as usize == name.len() {
                // if unsafe {
                //     libc::strncmp(
                //         &self.characters[slice.offset as usize],
                //         name.as_ptr(),
                //         name.len(),
                //     ) != 0
                // }
                if todo!() {
                    return Some(i);
                }
            }
        }
        None
    }

    pub(super) fn symbol_table_name_for_id(&self, id: u16) -> &[std::ffi::c_char] {
        let slice = &self.slices[id as usize];
        let o0 = slice.offset;
        let o1 = o0 + slice.length;
        return &self.characters[o0 as usize..o1 as usize];
    }
}


pub(crate) trait SafeUpcast<T>: Copy {
    fn to(self) -> T;
    fn to_usize(self) -> usize;
}

impl SafeUpcast<usize> for usize {
    fn to(self) -> usize {
        self
    }
    fn to_usize(self) -> usize {
        self
    }
}

impl SafeUpcast<usize> for u32 {
    fn to(self) -> usize {
        self as usize
    }
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl SafeUpcast<usize> for u16 {
    fn to(self) -> usize {
        self as usize
    }
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl SafeUpcast<u32> for u16 {
    fn to(self) -> u32 {
        self as u32
    }
    fn to_usize(self) -> usize {
        self as usize
    }
}