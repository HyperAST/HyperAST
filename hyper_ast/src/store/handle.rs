use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::num::NonZeroU32;

use controlled_option::Niche;

//-------------------------------------------------------------------------------------------------
// Handle from scope-graph crate

/// A handle to an instance of type `T` that was allocated from an [`??`][].
///
/// #### Safety
///
/// Because of the type parameter `T`, the compiler can ensure that you don't use a handle for one
/// type to index into an arena of another type.  However, if you have multiple arenas for the
/// _same type_, we do not do anything to ensure that you only use a handle with the corresponding
/// arena.
#[repr(transparent)]
pub struct Handle<T> {
    index: NonZeroU32,
    _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    fn new(index: NonZeroU32) -> Handle<T> {
        Handle {
            index,
            _phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub fn as_u32(self) -> u32 {
        self.index.get()
    }

    #[inline(always)]
    pub fn as_usize(self) -> usize {
        self.index.get() as usize
    }
}

impl<T> Niche for Handle<T> {
    type Output = u32;

    #[inline]
    fn none() -> Self::Output {
        0
    }

    #[inline]
    fn is_none(value: &Self::Output) -> bool {
        *value == 0
    }

    #[inline]
    fn into_some(value: Self) -> Self::Output {
        value.index.get()
    }

    #[inline]
    fn from_some(value: Self::Output) -> Self {
        Self::new(unsafe { NonZeroU32::new_unchecked(value) })
    }
}

// Normally we would #[derive] all of these traits, but the auto-derived implementations all
// require that T implement the trait as well.  We don't store any real instances of T inside of
// Handle, so our implementations do _not_ require that.

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Handle<T> {
        Handle::new(self.index)
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("index", &self.index)
            .finish()
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.index.partial_cmp(&other.index)
    }
}

// Handles are always Send and Sync, even if the underlying types are not.  After all, a handle is
// just a number!  And you _also_ need access to the Arena (which _won't_ be Send/Sync if T isn't)
// to dereference the handle.
unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}
