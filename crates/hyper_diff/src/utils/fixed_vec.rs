use std::cmp;
use std::iter;
use std::mem;
use std::ops::{Bound, Deref, DerefMut, RangeBounds};
use std::ptr;
use std::slice;

// extra traits
use std::borrow::{Borrow, BorrowMut};
use std::fmt;
use std::hash::{Hash, Hasher};

#[cfg(feature = "std")]
use std::io;

use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod Error {
    use std::any::Any;
    use std::error::Error;
    use std::fmt;

    /// Error value indicating insufficient capacity
    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
    pub struct CapacityError<T = ()> {
        element: T,
    }

    impl<T> CapacityError<T> {
        /// Create a new `CapacityError` from `element`.
        pub const fn new(element: T) -> CapacityError<T> {
            CapacityError { element: element }
        }

        /// Extract the overflowing element
        pub fn element(self) -> T {
            self.element
        }

        /// Convert into a `CapacityError` that does not carry an element.
        pub fn simplify(self) -> CapacityError {
            CapacityError { element: () }
        }
    }

    const CAPERROR: &'static str = "insufficient capacity";

    impl<T: Any> Error for CapacityError<T> {}

    impl<T> fmt::Display for CapacityError<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", CAPERROR)
        }
    }

    impl<T> fmt::Debug for CapacityError<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}: {}", "CapacityError", CAPERROR)
        }
    }
}

use Error::*;

/// A vector with a fixed capacity.
///
/// The `FixedVec` is a vector backed by a fixed size boxed array. It keeps track of
/// the number of initialized elements. The `FixedVec<T>` is parameterized
/// by `T` for the element type and `CAP` for the maximum capacity.
///
/// `CAP` is of type `usize` but is range limited to `u32::MAX`; attempting to create larger
/// arrayvecs with larger capacity will panic.
///
/// The vector is a contiguous value (storing the elements inline) that you can store directly on
/// the stack if needed.
///
/// It offers a simple API but also dereferences to a slice, so that the full slice API is
/// available. The FixedVec can be converted into a by value iterator.
pub struct FixedVec<T> {
    xs: Box<[MaybeUninit<T>]>,
    len: usize,
}

impl<T> Drop for FixedVec<T> {
    fn drop(&mut self) {
        self.clear();

        // MaybeUninit inhibits array's drop
    }
}

macro_rules! panic_oob {
    ($method_name:expr, $index:expr, $len:expr) => {
        panic!(
            concat!(
                "FixedVec::",
                $method_name,
                ": index {} is out of bounds in vector of length {}"
            ),
            $index, $len
        )
    };
}

impl<T> FixedVec<T> {
    /// Create a new empty `FixedVec`.
    pub fn new(capcity: usize) -> FixedVec<T> {
        FixedVec {
            xs: Box::new_uninit_slice(capcity),
            len: 0,
        }
        // unsafe {
        //     FixedVec { xs: MaybeUninit::uninit().assume_init(), len: 0 }
        // }
    }

    /// Return the number of elements in the `FixedVec`.
    ///
    /// ```
    /// use fixed_vec::FixedVec;
    ///
    /// let mut array = FixedVec::from([1, 2, 3]);
    /// array.pop();
    /// assert_eq!(array.len(), 2);
    /// ```
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the `FixedVec` is empty.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::from([1]);
    /// array.pop();
    /// assert_eq!(array.is_empty(), true);
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the capacity of the `FixedVec`.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let array = FixedVec::from([1, 2, 3]);
    /// assert_eq!(array.capacity(), 3);
    /// ```
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        self.xs.len()
    }

    /// Return true if the `FixedVec` is completely filled to its capacity, false otherwise.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::<_, 1>::new();
    /// assert!(!array.is_full());
    /// array.push(1);
    /// assert!(array.is_full());
    /// ```
    pub const fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Returns the capacity left in the `FixedVec`.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::from([1, 2, 3]);
    /// array.pop();
    /// assert_eq!(array.remaining_capacity(), 1);
    /// ```
    pub const fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len()
    }

    /// Push `element` to the end of the vector.
    ///
    /// ***Panics*** if the vector is already full.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::<_, 2>::new();
    ///
    /// array.push(1);
    /// array.push(2);
    ///
    /// assert_eq!(&array[..], &[1, 2]);
    /// ```
    pub fn push(&mut self, element: T) {
        self.try_push(element).unwrap()
    }

    /// Push `element` to the end of the vector.
    ///
    /// Return `Ok` if the push succeeds, or return an error if the vector
    /// is already full.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::<_, 2>::new();
    ///
    /// let push1 = array.try_push(1);
    /// let push2 = array.try_push(2);
    ///
    /// assert!(push1.is_ok());
    /// assert!(push2.is_ok());
    ///
    /// assert_eq!(&array[..], &[1, 2]);
    ///
    /// let overflow = array.try_push(3);
    ///
    /// assert!(overflow.is_err());
    /// ```
    pub fn try_push(&mut self, element: T) -> Result<(), CapacityError<T>> {
        if self.len() < self.capacity() {
            unsafe {
                self.push_unchecked(element);
            }
            Ok(())
        } else {
            Err(CapacityError::new(element))
        }
    }

    unsafe fn push_unchecked(&mut self, element: T) {
        let len = self.len();
        debug_assert!(len < self.capacity());
        ptr::write(self.as_mut_ptr().add(len), element);
        self.set_len(len + 1);
    }

    /// Shortens the vector, keeping the first `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater than the vector’s current length this has no
    /// effect.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::from([1, 2, 3, 4, 5]);
    /// array.truncate(3);
    /// assert_eq!(&array[..], &[1, 2, 3]);
    /// array.truncate(4);
    /// assert_eq!(&array[..], &[1, 2, 3]);
    /// ```
    pub fn truncate(&mut self, new_len: usize) {
        unsafe {
            let len = self.len();
            if new_len < len {
                self.set_len(new_len);
                let tail = slice::from_raw_parts_mut(self.as_mut_ptr().add(new_len), len - new_len);
                ptr::drop_in_place(tail);
            }
        }
    }

    /// Remove all elements in the vector.
    pub fn clear(&mut self) {
        self.truncate(0)
    }

    /// Get pointer to where element at `index` would be
    unsafe fn get_unchecked_ptr(&mut self, index: usize) -> *mut T {
        self.as_mut_ptr().add(index)
    }

    /// Insert `element` at position `index`.
    ///
    /// Shift up all elements after `index`.
    ///
    /// It is an error if the index is greater than the length or if the
    /// arrayvec is full.
    ///
    /// ***Panics*** if the array is full or the `index` is out of bounds. See
    /// `try_insert` for fallible version.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::<_, 2>::new();
    ///
    /// array.insert(0, "x");
    /// array.insert(0, "y");
    /// assert_eq!(&array[..], &["y", "x"]);
    ///
    /// ```
    pub fn insert(&mut self, index: usize, element: T) {
        self.try_insert(index, element).unwrap()
    }

    /// Insert `element` at position `index`.
    ///
    /// Shift up all elements after `index`; the `index` must be less than
    /// or equal to the length.
    ///
    /// Returns an error if vector is already at full capacity.
    ///
    /// ***Panics*** `index` is out of bounds.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::<_, 2>::new();
    ///
    /// assert!(array.try_insert(0, "x").is_ok());
    /// assert!(array.try_insert(0, "y").is_ok());
    /// assert!(array.try_insert(0, "z").is_err());
    /// assert_eq!(&array[..], &["y", "x"]);
    ///
    /// ```
    pub fn try_insert(&mut self, index: usize, element: T) -> Result<(), CapacityError<T>> {
        if index > self.len() {
            panic_oob!("try_insert", index, self.len())
        }
        if self.len() == self.capacity() {
            return Err(CapacityError::new(element));
        }
        let len = self.len();

        // follows is just like Vec<T>
        unsafe {
            // infallible
            // The spot to put the new value
            {
                let p: *mut _ = self.get_unchecked_ptr(index);
                // Shift everything over to make space. (Duplicating the
                // `index`th element into two consecutive places.)
                ptr::copy(p, p.offset(1), len - index);
                // Write it in, overwriting the first copy of the `index`th
                // element.
                ptr::write(p, element);
            }
            self.set_len(len + 1);
        }
        Ok(())
    }

    /// Remove the last element in the vector and return it.
    ///
    /// Return `Some(` *element* `)` if the vector is non-empty, else `None`.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::<_, 2>::new();
    ///
    /// array.push(1);
    ///
    /// assert_eq!(array.pop(), Some(1));
    /// assert_eq!(array.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len() == 0 {
            return None;
        }
        unsafe {
            let new_len = self.len() - 1;
            self.set_len(new_len);
            Some(ptr::read(self.as_ptr().add(new_len)))
        }
    }

    /// Remove the element at `index` and swap the last element into its place.
    ///
    /// This operation is O(1).
    ///
    /// Return the *element* if the index is in bounds, else panic.
    ///
    /// ***Panics*** if the `index` is out of bounds.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::from([1, 2, 3]);
    ///
    /// assert_eq!(array.swap_remove(0), 1);
    /// assert_eq!(&array[..], &[3, 2]);
    ///
    /// assert_eq!(array.swap_remove(1), 2);
    /// assert_eq!(&array[..], &[3]);
    /// ```
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.swap_pop(index)
            .unwrap_or_else(|| panic_oob!("swap_remove", index, self.len()))
    }

    /// Remove the element at `index` and swap the last element into its place.
    ///
    /// This is a checked version of `.swap_remove`.  
    /// This operation is O(1).
    ///
    /// Return `Some(` *element* `)` if the index is in bounds, else `None`.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut array = FixedVec::from([1, 2, 3]);
    ///
    /// assert_eq!(array.swap_pop(0), Some(1));
    /// assert_eq!(&array[..], &[3, 2]);
    ///
    /// assert_eq!(array.swap_pop(10), None);
    /// ```
    pub fn swap_pop(&mut self, index: usize) -> Option<T> {
        let len = self.len();
        if index >= len {
            return None;
        }
        self.swap(index, len - 1);
        self.pop()
    }

    // /// Remove the element at `index` and shift down the following elements.
    // ///
    // /// The `index` must be strictly less than the length of the vector.
    // ///
    // /// ***Panics*** if the `index` is out of bounds.
    // ///
    // /// ```
    // /// use arrayvec::FixedVec;
    // ///
    // /// let mut array = FixedVec::from([1, 2, 3]);
    // ///
    // /// let removed_elt = array.remove(0);
    // /// assert_eq!(removed_elt, 1);
    // /// assert_eq!(&array[..], &[2, 3]);
    // /// ```
    // pub fn remove(&mut self, index: usize) -> T {
    //     self.pop_at(index)
    //         .unwrap_or_else(|| panic_oob!("remove", index, self.len()))
    // }

    // /// Remove the element at `index` and shift down the following elements.
    // ///
    // /// This is a checked version of `.remove(index)`. Returns `None` if there
    // /// is no element at `index`. Otherwise, return the element inside `Some`.
    // ///
    // /// ```
    // /// use arrayvec::FixedVec;
    // ///
    // /// let mut array = FixedVec::from([1, 2, 3]);
    // ///
    // /// assert!(array.pop_at(0).is_some());
    // /// assert_eq!(&array[..], &[2, 3]);
    // ///
    // /// assert!(array.pop_at(2).is_none());
    // /// assert!(array.pop_at(10).is_none());
    // /// ```
    // pub fn pop_at(&mut self, index: usize) -> Option<T> {
    //     if index >= self.len() {
    //         None
    //     } else {
    //         self.drain(index..index + 1).next()
    //     }
    // }

    /// Set the vector’s length without dropping or moving out elements
    ///
    /// This method is `unsafe` because it changes the notion of the
    /// number of “valid” elements in the vector. Use with care.
    ///
    /// This method uses *debug assertions* to check that `length` is
    /// not greater than the capacity.
    pub unsafe fn set_len(&mut self, length: usize) {
        debug_assert!(length <= self.capacity());
        self.len = length;
    }

    /// Copy all elements from the slice and append to the `FixedVec`.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut vec: FixedVec<usize, 10> = FixedVec::new();
    /// vec.push(1);
    /// vec.try_extend_from_slice(&[2, 3]).unwrap();
    /// assert_eq!(&vec[..], &[1, 2, 3]);
    /// ```
    ///
    /// # Errors
    ///
    /// This method will return an error if the capacity left (see
    /// [`remaining_capacity`]) is smaller then the length of the provided
    /// slice.
    ///
    /// [`remaining_capacity`]: #method.remaining_capacity
    pub fn try_extend_from_slice(&mut self, other: &[T]) -> Result<(), CapacityError>
    where
        T: Copy,
    {
        if self.remaining_capacity() < other.len() {
            return Err(CapacityError::new(()));
        }

        let self_len = self.len();
        let other_len = other.len();

        unsafe {
            let dst = self.get_unchecked_ptr(self_len);
            ptr::copy_nonoverlapping(other.as_ptr(), dst, other_len);
            self.set_len(self_len + other_len);
        }
        Ok(())
    }

    /// Return the inner fixed size array, if it is full to its capacity.
    ///
    /// Return an `Ok` value with the array if length equals capacity,
    /// return an `Err` with self otherwise.
    pub fn into_inner(self) -> Result<Box<[T]>, Self> {
        if self.len() < self.capacity() {
            Err(self)
        } else {
            unsafe { Ok(self.into_inner_unchecked()) }
        }
    }

    /// Return the inner fixed size array.
    ///
    /// Safety:
    /// This operation is safe if and only if length equals capacity.
    pub unsafe fn into_inner_unchecked(self) -> Box<[T]> {
        debug_assert_eq!(self.len(), self.capacity());
        let self_ = ManuallyDrop::new(self);
        // let array = ptr::read(self_.as_ptr() as *const [T; CAP]);
        let array = ptr::read(self_.as_ptr() as *const Box<[T]>); // TODO check that
        array
    }

    /// Returns the FixedVec, replacing the original with a new empty FixedVec.
    ///
    /// ```
    /// use arrayvec::FixedVec;
    ///
    /// let mut v = FixedVec::from([0, 1, 2, 3]);
    /// assert_eq!([0, 1, 2, 3], v.take().into_inner().unwrap());
    /// assert!(v.is_empty());
    /// ```
    pub fn take(&mut self) -> Self {
        mem::replace(self, Self::new(self.capacity()))
    }

    fn as_ptr(&self) -> *const T {
        self.xs.as_ptr() as _
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.xs.as_mut_ptr() as _
    }

    /// Return a slice containing all elements of the vector.
    fn as_slice(&self) -> &[T] {
        let len = self.len();
        unsafe {
            slice::from_raw_parts(self.as_ptr(), len)
        }
    }

    /// Return a mutable slice containing all elements of the vector.
    fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.len();
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr(), len)
        }
    }

    // /// Return a raw pointer to the vector's buffer.
    // pub fn as_ptr(&self) -> *const T {
    //     FixedVecImpl::as_ptr(self)
    // }

    // /// Return a raw mutable pointer to the vector's buffer.
    // pub fn as_mut_ptr(&mut self) -> *mut T {
    //     FixedVecImpl::as_mut_ptr(self)
    // }
}
impl<T> Deref for FixedVec<T> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for FixedVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

// /// Create an `FixedVec` from an array.
// ///
// /// ```
// /// use arrayvec::FixedVec;
// ///
// /// let mut array = FixedVec::from([1, 2, 3]);
// /// assert_eq!(array.len(), 3);
// /// assert_eq!(array.capacity(), 3);
// /// ```
// impl<T> From<[T; CAP]> for FixedVec<T> {
//     fn from(array: [T; CAP]) -> Self {
//         let array = ManuallyDrop::new(array);
//         let mut vec = <FixedVec<T>>::new();
//         unsafe {
//             (&*array as *const [T; CAP] as *const [MaybeUninit<T>; CAP])
//                 .copy_to_nonoverlapping(&mut vec.xs as *mut [MaybeUninit<T>; CAP], 1);
//             vec.set_len(CAP);
//         }
//         vec
//     }
// }

impl<T> From<Vec<T>> for FixedVec<T> {
    fn from(array: Vec<T>) -> Self {
        let mut vec = <FixedVec<T>>::new(array.len());
        let a = array.into_iter();
        vec.extend(a);
        vec
    }
}

// /// Try to create an `FixedVec` from a slice. This will return an error if the slice was too big to
// /// fit.
// ///
// /// ```
// /// use arrayvec::FixedVec;
// /// use std::convert::TryInto as _;
// ///
// /// let array: FixedVec<_, 4> = (&[1, 2, 3] as &[_]).try_into().unwrap();
// /// assert_eq!(array.len(), 3);
// /// assert_eq!(array.capacity(), 4);
// /// ```
// impl<T> std::convert::TryFrom<&[T]> for FixedVec<T>
// where
//     T: Clone,
// {
//     type Error = CapacityError;

//     fn try_from(slice: &[T]) -> Result<Self, Self::Error> {
//         if slice.len() < slice.len() {
//             Err(CapacityError::new(()))
//         } else {
//             let mut array = Self::new(todo!());
//             array.extend_from_slice(slice);
//             Ok(array)
//         }
//     }
// }

/// Iterate the `FixedVec` with references to each element.
///
/// ```
/// use arrayvec::FixedVec;
///
/// let array = FixedVec::from([1, 2, 3]);
///
/// for elt in &array {
///     // ...
/// }
/// ```
impl<'a, T: 'a> IntoIterator for &'a FixedVec<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterate the `FixedVec` with mutable references to each element.
///
/// ```
/// use arrayvec::FixedVec;
///
/// let mut array = FixedVec::from([1, 2, 3]);
///
/// for elt in &mut array {
///     // ...
/// }
/// ```
impl<'a, T: 'a> IntoIterator for &'a mut FixedVec<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Iterate the `FixedVec` with each element by value.
///
/// The vector is consumed by this operation.
///
/// ```
/// use arrayvec::FixedVec;
///
/// for elt in FixedVec::from([1, 2, 3]) {
///     // ...
/// }
/// ```
impl<T> IntoIterator for FixedVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        IntoIter { index: 0, v: self }
    }
}

/// By-value iterator for `FixedVec`.
pub struct IntoIter<T> {
    index: usize,
    v: FixedVec<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.v.len() {
            None
        } else {
            unsafe {
                let index = self.index;
                self.index = index + 1;
                Some(ptr::read(self.v.get_unchecked_ptr(index)))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.v.len() - self.index;
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == self.v.len() {
            None
        } else {
            unsafe {
                let new_len = self.v.len() - 1;
                self.v.set_len(new_len);
                Some(ptr::read(self.v.get_unchecked_ptr(new_len)))
            }
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // panic safety: Set length to 0 before dropping elements.
        let index = self.index;
        let len = self.v.len();
        unsafe {
            self.v.set_len(0);
            let elements = slice::from_raw_parts_mut(self.v.get_unchecked_ptr(index), len - index);
            ptr::drop_in_place(elements);
        }
    }
}

impl<T> Clone for IntoIter<T>
where
    T: Clone,
{
    fn clone(&self) -> IntoIter<T> {
        let mut v = FixedVec::new(self.v.capacity() - self.index);
        v.extend_from_slice(&self.v[self.index..]);
        v.into_iter()
    }
}

impl<T> fmt::Debug for IntoIter<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(&self.v[self.index..]).finish()
    }
}

struct ScopeExitGuard<T, Data, F>
where
    F: FnMut(&Data, &mut T),
{
    value: T,
    data: Data,
    f: F,
}

impl<T, Data, F> Drop for ScopeExitGuard<T, Data, F>
where
    F: FnMut(&Data, &mut T),
{
    fn drop(&mut self) {
        (self.f)(&self.data, &mut self.value)
    }
}

/// Extend the `FixedVec` with an iterator.
///
/// ***Panics*** if extending the vector exceeds its capacity.
impl<T> Extend<T> for FixedVec<T> {
    /// Extend the `FixedVec` with an iterator.
    ///
    /// ***Panics*** if extending the vector exceeds its capacity.
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        unsafe { self.extend_from_iter::<_, true>(iter) }
    }
}

#[inline(never)]
#[cold]
fn extend_panic() {
    panic!("FixedVec: capacity exceeded in extend/from_iter");
}

impl<T> FixedVec<T> {
    /// Extend the arrayvec from the iterable.
    ///
    /// ## Safety
    ///
    /// Unsafe because if CHECK is false, the length of the input is not checked.
    /// The caller must ensure the length of the input fits in the capacity.
    pub(crate) unsafe fn extend_from_iter<I, const CHECK: bool>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = T>,
    {
        let take = self.capacity() - self.len();
        let len = self.len();
        let mut ptr = raw_ptr_add(self.as_mut_ptr(), len);
        let end_ptr = raw_ptr_add(ptr, take);
        // Keep the length in a separate variable, write it back on scope
        // exit. To help the compiler with alias analysis and stuff.
        // We update the length to handle panic in the iteration of the
        // user's iterator, without dropping any elements on the floor.
        let mut guard = ScopeExitGuard {
            value: &mut self.len,
            data: len,
            f: move |&len, self_len| {
                **self_len = len;
            },
        };
        let mut iter = iterable.into_iter();
        loop {
            if let Some(elt) = iter.next() {
                if ptr == end_ptr && CHECK {
                    extend_panic();
                }
                debug_assert_ne!(ptr, end_ptr);
                ptr.write(elt);
                ptr = raw_ptr_add(ptr, 1);
                guard.data += 1;
            } else {
                return; // success
            }
        }
    }

    /// Extend the FixedVec with clones of elements from the slice;
    /// the length of the slice must be <= the remaining capacity in the arrayvec.
    pub(crate) fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        let take = self.capacity() - self.len();
        debug_assert!(slice.len() <= take);
        unsafe {
            let slice = if take < slice.len() {
                &slice[..take]
            } else {
                slice
            };
            self.extend_from_iter::<_, false>(slice.iter().cloned());
        }
    }
}

/// Rawptr add but uses arithmetic distance for ZST
unsafe fn raw_ptr_add<T>(ptr: *mut T, offset: usize) -> *mut T {
    if mem::size_of::<T>() == 0 {
        // Special case for ZST
        (ptr as usize).wrapping_add(offset) as _
    } else {
        ptr.add(offset)
    }
}

impl<T> Clone for FixedVec<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut array = FixedVec::new(self.xs.len());
        array.extend(self.iter().cloned());
        array
    }

    fn clone_from(&mut self, rhs: &Self) {
        // recursive case for the common prefix
        let prefix = cmp::min(self.len(), rhs.len());
        self[..prefix].clone_from_slice(&rhs[..prefix]);

        if prefix < self.len() {
            // rhs was shorter
            self.truncate(prefix);
        } else {
            let rhs_elems = &rhs[self.len()..];
            self.extend_from_slice(rhs_elems);
        }
    }
}

impl<T> Hash for FixedVec<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T> PartialEq for FixedVec<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T> PartialEq<[T]> for FixedVec<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &[T]) -> bool {
        **self == *other
    }
}

impl<T> Eq for FixedVec<T> where T: Eq {}

impl<T> Borrow<[T]> for FixedVec<T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T> BorrowMut<[T]> for FixedVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> AsRef<[T]> for FixedVec<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for FixedVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> fmt::Debug for FixedVec<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> PartialOrd for FixedVec<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other)
    }

    fn lt(&self, other: &Self) -> bool {
        (**self).lt(other)
    }

    fn le(&self, other: &Self) -> bool {
        (**self).le(other)
    }

    fn ge(&self, other: &Self) -> bool {
        (**self).ge(other)
    }

    fn gt(&self, other: &Self) -> bool {
        (**self).gt(other)
    }
}

impl<T> Ord for FixedVec<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (**self).cmp(other)
    }
}

#[cfg(feature = "std")]
/// `Write` appends written data to the end of the vector.
///
/// Requires `features="std"`.
impl<const CAP: usize> io::Write for FixedVec<u8> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let len = cmp::min(self.remaining_capacity(), data.len());
        let _result = self.try_extend_from_slice(&data[..len]);
        debug_assert!(_result.is_ok());
        Ok(len)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "serde")]
/// Requires crate feature `"serde"`
impl<T: Serialize> Serialize for FixedVec<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self)
    }
}

#[cfg(feature = "serde")]
/// Requires crate feature `"serde"`
impl<'de, T: Deserialize<'de>> Deserialize<'de> for FixedVec<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Error, SeqAccess, Visitor};
        use std::marker::PhantomData;

        struct FixedVecVisitor<'de, T: Deserialize<'de>>(PhantomData<(&'de (), [T; CAP])>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for FixedVecVisitor<'de, T> {
            type Value = FixedVec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array with no more than {} items")
            }

            fn visit_seq<SA>(self, mut seq: SA) -> Result<Self::Value, SA::Error>
            where
                SA: SeqAccess<'de>,
            {
                let mut values = FixedVec::<T>::new();

                while let Some(value) = seq.next_element()? {
                    if let Err(_) = values.try_push(value) {
                        return Err(SA::Error::invalid_length(CAP + 1, &self));
                    }
                }

                Ok(values)
            }
        }

        deserializer.deserialize_seq(FixedVecVisitor::<T>(PhantomData))
    }
}
