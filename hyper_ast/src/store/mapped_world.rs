use std::{
    fmt::Debug,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

use crate::{
    compat::{DefaultHashBuilder, HashMap},
    utils::make_hash,
};

pub trait Symbol<T>: Copy + Eq {}

pub struct SymbolU32<T> {
    internal: string_interner::symbol::SymbolU32,
    _phantom: PhantomData<*const T>,
}

impl<T> Debug for SymbolU32<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use string_interner::Symbol;
        write!(f, "${}", &self.internal.to_usize())
        // f.debug_struct("SymbolU32")
        //     .field("internal", &self.internal)
        //     .finish()
    }
}

impl<T> Clone for SymbolU32<T> {
    fn clone(&self) -> Self {
        Self {
            internal: self.internal.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<T> Copy for SymbolU32<T> {}

impl<T> PartialEq for SymbolU32<T> {
    fn eq(&self, other: &Self) -> bool {
        self.internal == other.internal
    }
}

impl<T> Eq for SymbolU32<T> {}

impl<T> Symbol<T> for SymbolU32<T> {}

pub trait AsNodeEntityRef {
    type Ref<'a>
    where
        Self: 'a;
    fn eq(&self, other: &Self::Ref<'_>) -> bool;
}
pub trait AsNodeEntityRefSelf: AsNodeEntityRef {
    fn as_ref(&self) -> Self::Ref<'_>;
}

/// Come from string-interner
/// Types implementing this trait may act as backends for the string interner.
///
/// The job of a backend is to actually store, manage and organize the interned
/// strings. Different backends have different trade-offs. Users should pick
/// their backend with hinsight of their personal use-case.
pub trait Backend<T: AsNodeEntityRef>: Default {
    /// The symbol used by the string interner backend.
    type Symbol: Symbol<T>;

    fn len(&self) -> usize;

    /// Creates a new backend for the given capacity.
    ///
    /// The capacity denotes how many strings are expected to be interned.
    fn with_capacity(cap: usize) -> Self;

    /// Interns the given string and returns its interned ref and symbol.
    ///
    /// # Note
    ///
    /// The backend must make sure that the returned symbol maps back to the
    /// original string in its [`resolve`](`Backend::resolve`) method.
    fn intern(&mut self, string: T) -> Self::Symbol;

    /// Shrink backend capacity to fit interned symbols exactly.
    fn shrink_to_fit(&mut self);

    /// Resolves the given symbol to its original string contents.
    fn resolve(&self, symbol: Self::Symbol) -> Option<T::Ref<'_>>;

    /// Resolves the given symbol to its original string contents.
    ///
    /// # Safety
    ///
    /// Does not perform validity checks on the given symbol and relies
    /// on the caller to be provided with a symbol that has been generated
    /// by the [`intern`](`Backend::intern`) or
    /// [`intern_static`](`Backend::intern_static`) methods of the same
    /// interner backend.
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> T::Ref<'_>;
}

pub struct VecBackend<T, S: Symbol<T>> {
    internal: Vec<T>,
    phantom: PhantomData<*const S>,
}

impl<T: AsNodeEntityRef + Debug, S: Symbol<T>> Debug for VecBackend<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecBackend")
            .field("internal", &self.internal)
            .finish()
    }
}

impl<T, S: Symbol<T>> Default for VecBackend<T, S> {
    fn default() -> Self {
        Self {
            internal: Default::default(),
            phantom: Default::default(),
        }
    }
}

pub type DefaultBackend<T, I> = VecBackend<T, I>;

pub struct VecMapStore<
    T: Hash + AsNodeEntityRef,
    I: Symbol<T>,
    B = DefaultBackend<T, I>,
    H = DefaultHashBuilder,
> where
    B: Backend<T>,
    H: BuildHasher,
{
    dedup: HashMap<I, (), ()>,
    hasher: H,
    backend: B,
    phantom: PhantomData<*const T>,
}

impl<T: Hash + AsNodeEntityRef, I: Symbol<T>, B, H> Default for VecMapStore<T, I, B, H>
where
    B: Backend<T>,
    H: BuildHasher + Default,
{
    fn default() -> Self {
        Self {
            dedup: Default::default(),
            hasher: Default::default(),
            backend: Default::default(),
            phantom: Default::default(),
        }
    }
}

impl<T: Hash + AsNodeEntityRef, I: Symbol<T>, B, H> VecMapStore<T, I, B, H>
where
    B: Backend<T, Symbol = I>,
    H: BuildHasher + Default,
{
    pub fn new() -> Self {
        Self {
            dedup: HashMap::default(),
            hasher: Default::default(),
            backend: B::default(),
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.backend.len()
    }
}

impl<T: Hash + Debug + AsNodeEntityRef, I: Symbol<T> + Debug, B, H> Debug
    for VecMapStore<T, I, B, H>
where
    B: Backend<T> + Debug,
    <B as Backend<T>>::Symbol: Symbol<T> + Debug,
    H: BuildHasher,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecMapStore")
            .field("dedup", &self.dedup)
            .field("backend", &self.backend)
            .finish()
    }
}

impl<T: Hash + Eq + AsNodeEntityRef, I: Symbol<T>, B> VecMapStore<T, I, B>
where
    B: Backend<T, Symbol = I>,
    for<'a> T::Ref<'a>: Hash + Eq,
{
    pub fn get<U: AsRef<T>>(&mut self, node: U) -> Option<I> {
        let node = node.as_ref();
        let Self {
            dedup,
            hasher,
            backend,
            ..
        } = self;
        let hash = make_hash(hasher, node);
        dedup
            .raw_entry()
            .from_hash(hash, |symbol| {
                // SAFETY: This is safe because we only operate on symbols that
                //         we receive from our backend making them valid.
                AsNodeEntityRef::eq(node, &unsafe { backend.resolve_unchecked(*symbol) })
            })
            .map(|(&symbol, &())| symbol)
    }
}

impl<T: Hash + Eq + AsNodeEntityRef, I: Symbol<T>, B, H> VecMapStore<T, I, B, H>
where
    B: Backend<T, Symbol = I>,
    H: BuildHasher,
    for<'a> T::Ref<'a>: Hash + Eq,
{
    pub fn get_or_intern_using(&mut self, node: T, intern_fn: fn(&mut B, T) -> I) -> I {
        let Self {
            dedup,
            hasher,
            backend,
            ..
        } = self;
        let hash = make_hash(hasher, &node);
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            // SAFETY: This is safe because we only operate on symbols that
            //         we receive from our backend making them valid.
            // node.eq(unsafe { backend.resolve_unchecked(*symbol) })
            let tmp = unsafe { backend.resolve_unchecked(*symbol) };
            if AsNodeEntityRef::eq(&node, &tmp) {
                true
            } else {
                false
            }
        });
        use crate::compat::hash_map::RawEntryMut;
        let (&mut symbol, &mut ()) = match entry {
            RawEntryMut::Occupied(occupied) => occupied.into_key_value(),
            RawEntryMut::Vacant(vacant) => {
                let symbol = intern_fn(backend, node);
                vacant.insert_with_hasher(hash, symbol, (), |symbol| {
                    // SAFETY: This is safe because we only operate on symbols that
                    //         we receive from our backend making them valid.
                    let node = unsafe { backend.resolve_unchecked(*symbol) };
                    make_hash(hasher, &node)
                })
            }
        };
        symbol
    }

    #[inline]
    pub fn get_or_intern(&mut self, node: T) -> I
where {
        self.get_or_intern_using(node, B::intern)
    }

    pub fn resolve(&self, id: &I) -> T::Ref<'_> {
        self.backend.resolve(*id).unwrap()
    }
}
