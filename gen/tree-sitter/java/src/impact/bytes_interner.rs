use core::{iter::Enumerate, marker::PhantomData, slice};
use std::num::NonZeroU32;

use crate::compat::{DefaultHashBuilder, HashMap};

use num::ToPrimitive;
use string_interner::DefaultSymbol;
use string_interner::{symbol::SymbolU16, Symbol};

#[derive(Debug)]
pub struct BytesBackend<S = DefaultSymbol> {
    ends: Vec<usize>,
    buffer: Vec<u8>,
    marker: PhantomData<fn() -> S>,
}

/// Represents a `[from, to)` index into the `StringBackend` buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    from: usize,
    to: usize,
}

impl<S> PartialEq for BytesBackend<S>
where
    S: Symbol,
{
    fn eq(&self, other: &Self) -> bool {
        if self.ends.len() != other.ends.len() {
            return false;
        }
        for ((_, lhs), (_, rhs)) in self.into_iter().zip(other) {
            if lhs != rhs {
                return false;
            }
        }
        true
    }
}

impl<S> Eq for BytesBackend<S> where S: Symbol {}

impl<S> Clone for BytesBackend<S> {
    fn clone(&self) -> Self {
        Self {
            ends: self.ends.clone(),
            buffer: self.buffer.clone(),
            marker: Default::default(),
        }
    }
}

impl<S> Default for BytesBackend<S> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self {
            ends: Vec::default(),
            buffer: Default::default(),
            marker: Default::default(),
        }
    }
}

#[inline]
pub(crate) fn expect_valid_symbol<S>(index: usize) -> S
where
    S: Symbol,
{
    S::try_from_usize(index).expect("encountered invalid symbol")
}

impl<S> BytesBackend<S>
where
    S: Symbol,
{
    /// Returns the next available symbol.
    fn next_symbol(&self) -> S {
        expect_valid_symbol(self.ends.len())
    }

    /// Returns the string associated to the span.
    fn span_to_slice(&self, span: Span) -> &[u8] {
        &self.buffer[(span.from as usize)..(span.to as usize)]
    }

    /// Returns the span for the given symbol if any.
    fn symbol_to_span(&self, symbol: S) -> Option<Span> {
        let index = symbol.to_usize();
        self.ends.get(index).copied().map(|to| {
            let from = self.ends.get(index.wrapping_sub(1)).copied().unwrap_or(0);
            Span { from, to }
        })
    }

    /// Returns the span for the given symbol if any.
    unsafe fn symbol_to_span_unchecked(&self, symbol: S) -> Span {
        let index = symbol.to_usize();
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        let to = unsafe { *self.ends.get_unchecked(index) };
        let from = self.ends.get(index.wrapping_sub(1)).copied().unwrap_or(0);
        Span { from, to }
    }

    /// Pushes the given string into the buffer and returns its span.
    ///
    /// # Panics
    ///
    /// If the backend ran out of symbols.
    fn extend(&mut self, value: &[u8]) -> S {
        self.buffer.extend(value);
        let to = self.buffer.len();
        let symbol = self.next_symbol();
        self.ends.push(to);
        symbol
    }
}

impl<S> Backend for BytesBackend<S>
where
    S: Symbol,
{
    type Symbol = S;

    #[cfg_attr(feature = "inline-more", inline)]
    fn with_capacity(cap: usize) -> Self {
        let default_word_len = 20;
        Self {
            ends: Vec::with_capacity(cap),
            buffer: Vec::with_capacity(cap * default_word_len),
            marker: Default::default(),
        }
    }

    #[inline]
    fn intern(&mut self, value: &[u8]) -> Self::Symbol {
        self.extend(value)
    }

    #[inline]
    fn resolve(&self, symbol: Self::Symbol) -> Option<&[u8]> {
        self.symbol_to_span(symbol)
            .map(|span| self.span_to_slice(span))
    }

    fn shrink_to_fit(&mut self) {
        self.ends.shrink_to_fit();
        self.buffer.shrink_to_fit();
    }

    #[inline]
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &[u8] {
        // SAFETY: The function is marked unsafe so that the caller guarantees
        //         that required invariants are checked.
        unsafe { self.span_to_slice(self.symbol_to_span_unchecked(symbol)) }
    }
}

impl<'a, S> IntoIterator for &'a BytesBackend<S>
where
    S: Symbol,
{
    type Item = (S, &'a [u8]);
    type IntoIter = Iter<'a, S>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

pub struct Iter<'a, S> {
    backend: &'a BytesBackend<S>,
    start: usize,
    ends: Enumerate<slice::Iter<'a, usize>>,
}

impl<'a, S> Iter<'a, S> {
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new(backend: &'a BytesBackend<S>) -> Self {
        Self {
            backend,
            start: 0,
            ends: backend.ends.iter().enumerate(),
        }
    }
}

impl<'a, S> Iterator for Iter<'a, S>
where
    S: Symbol,
{
    type Item = (S, &'a [u8]);

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ends.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.ends.next().map(|(id, &to)| {
            let from = core::mem::replace(&mut self.start, to);
            (
                expect_valid_symbol(id),
                self.backend.span_to_slice(Span { from, to }),
            )
        })
    }
}

pub type DefaultBackend<S> = BytesBackend<S>;

/// Types implementing this trait may act as backends for the string interner.
///
/// The job of a backend is to actually store, manage and organize the interned
/// strings. Different backends have different trade-offs. Users should pick
/// their backend with hinsight of their personal use-case.
pub trait Backend: Default {
    /// The symbol used by the string interner backend.
    type Symbol: Symbol;

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
    fn intern(&mut self, value: &[u8]) -> Self::Symbol;

    /// Interns the given static string and returns its interned ref and symbol.
    ///
    /// # Note
    ///
    /// The backend must make sure that the returned symbol maps back to the
    /// original string in its [`resolve`](`Backend::resolve`) method.
    #[inline]
    fn intern_static(&mut self, value: &'static [u8]) -> Self::Symbol {
        // The default implementation simply forwards to the normal [`intern`]
        // implementation. Backends that can optimize for this use case should
        // implement this method.
        self.intern(value)
    }

    /// Shrink backend capacity to fit interned symbols exactly.
    fn shrink_to_fit(&mut self);

    /// Resolves the given symbol to its original string contents.
    fn resolve(&self, symbol: Self::Symbol) -> Option<&[u8]>;

    /// Resolves the given symbol to its original string contents.
    ///
    /// # Safety
    ///
    /// Does not perform validity checks on the given symbol and relies
    /// on the caller to be provided with a symbol that has been generated
    /// by the [`intern`](`Backend::intern`) or
    /// [`intern_static`](`Backend::intern_static`) methods of the same
    /// interner backend.
    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> &[u8];
}

use core::{
    fmt,
    fmt::{Debug, Formatter},
    hash::{BuildHasher, Hash, Hasher},
    iter::FromIterator,
};

/// Creates the `u64` hash value for the given value using the given hash builder.
fn make_hash<T>(builder: &impl BuildHasher, value: &T) -> u64
where
    T: ?Sized + Hash,
{
    let state = &mut builder.build_hasher();
    value.hash(state);
    state.finish()
}

/// Data structure to intern and resolve strings.
///
/// Caches strings efficiently, with minimal memory footprint and associates them with unique symbols.
/// These symbols allow constant time comparisons and look-ups to the underlying interned strings.
///
/// The following API covers the main functionality:
///
/// - [`StringInterner::get_or_intern`]: To intern a new string.
///     - This maps from `string` type to `symbol` type.
/// - [`StringInterner::resolve`]: To resolve your already interned strings.
///     - This maps from `symbol` type to `string` type.
pub struct BytesInterner<B = DefaultBackend<DefaultSymbol>, H = DefaultHashBuilder>
where
    B: Backend,
    H: BuildHasher,
{
    dedup: HashMap<<B as Backend>::Symbol, (), ()>,
    hasher: H,
    backend: B,
}

impl<B, H> Debug for BytesInterner<B, H>
where
    B: Backend + Debug,
    <B as Backend>::Symbol: Symbol + Debug,
    H: BuildHasher,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringInterner")
            .field("dedup", &self.dedup)
            .field("backend", &self.backend)
            .finish()
    }
}

#[cfg(feature = "backends")]
impl Default for BytesInterner {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        BytesInterner::new()
    }
}

impl<B, H> Clone for BytesInterner<B, H>
where
    B: Backend + Clone,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher + Clone,
{
    fn clone(&self) -> Self {
        Self {
            dedup: self.dedup.clone(),
            hasher: self.hasher.clone(),
            backend: self.backend.clone(),
        }
    }
}

impl<B, H> PartialEq for BytesInterner<B, H>
where
    B: Backend + PartialEq,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.len() == rhs.len() && self.backend == rhs.backend
    }
}

impl<B, H> Eq for BytesInterner<B, H>
where
    B: Backend + Eq,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher,
{
}

impl<B, H> BytesInterner<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher + Default,
{
    /// Creates a new empty `StringInterner`.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new() -> Self {
        Self {
            dedup: HashMap::default(),
            hasher: Default::default(),
            backend: B::default(),
        }
    }

    /// Creates a new `StringInterner` with the given initial capacity.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            dedup: HashMap::with_capacity_and_hasher(cap, ()),
            hasher: Default::default(),
            backend: B::with_capacity(cap),
        }
    }
}

impl<B, H> BytesInterner<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher,
{
    /// Creates a new empty `StringInterner` with the given hasher.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_hasher(hash_builder: H) -> Self {
        BytesInterner {
            dedup: HashMap::default(),
            hasher: hash_builder,
            backend: B::default(),
        }
    }

    /// Creates a new empty `StringInterner` with the given initial capacity and the given hasher.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity_and_hasher(cap: usize, hash_builder: H) -> Self {
        BytesInterner {
            dedup: HashMap::with_capacity_and_hasher(cap, ()),
            hasher: hash_builder,
            backend: B::with_capacity(cap),
        }
    }

    /// Returns the number of strings interned by the interner.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn len(&self) -> usize {
        self.dedup.len()
    }

    /// Returns `true` if the string interner has no interned strings.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the symbol for the given string if any.
    ///
    /// Can be used to query if a string has already been interned without interning.
    #[inline]
    pub fn get<T>(&self, value: T) -> Option<<B as Backend>::Symbol>
    where
        T: AsRef<[u8]>,
    {
        let value = value.as_ref();
        let Self {
            dedup,
            hasher,
            backend,
        } = self;
        let hash = make_hash(hasher, value);
        dedup
            .raw_entry()
            .from_hash(hash, |symbol| {
                // SAFETY: This is safe because we only operate on symbols that
                //         we receive from our backend making them valid.
                value == unsafe { backend.resolve_unchecked(*symbol) }
            })
            .map(|(&symbol, &())| symbol)
    }

    /// Interns the given string.
    ///
    /// This is used as backend by [`get_or_intern`][1] and [`get_or_intern_static`][2].
    ///
    /// [1]: [`StringInterner::get_or_intern`]
    /// [2]: [`StringInterner::get_or_intern_static`]
    #[cfg_attr(feature = "inline-more", inline)]
    fn get_or_intern_using<T>(
        &mut self,
        value: T,
        intern_fn: fn(&mut B, T) -> <B as Backend>::Symbol,
    ) -> <B as Backend>::Symbol
    where
        T: Copy + Hash + AsRef<[u8]> + for<'a> PartialEq<&'a [u8]>,
    {
        let Self {
            dedup,
            hasher,
            backend,
        } = self;
        let hash = make_hash(hasher, value.as_ref());
        let entry = dedup.raw_entry_mut().from_hash(hash, |symbol| {
            // SAFETY: This is safe because we only operate on symbols that
            //         we receive from our backend making them valid.
            value == unsafe { backend.resolve_unchecked(*symbol) }
        });
        use crate::compat::hash_map::RawEntryMut;
        let (&mut symbol, &mut ()) = match entry {
            RawEntryMut::Occupied(occupied) => occupied.into_key_value(),
            RawEntryMut::Vacant(vacant) => {
                let symbol = intern_fn(backend, value);
                vacant.insert_with_hasher(hash, symbol, (), |symbol| {
                    // SAFETY: This is safe because we only operate on symbols that
                    //         we receive from our backend making them valid.
                    let value = unsafe { backend.resolve_unchecked(*symbol) };
                    make_hash(hasher, value)
                })
            }
        };
        symbol
    }

    /// Interns the given string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    #[inline]
    pub fn get_or_intern<T>(&mut self, value: T) -> <B as Backend>::Symbol
    where
        T: AsRef<[u8]>,
    {
        self.get_or_intern_using(value.as_ref(), B::intern)
    }

    /// Interns the given `'static` string.
    ///
    /// Returns a symbol for resolution into the original string.
    ///
    /// # Note
    ///
    /// This is more efficient than [`StringInterner::get_or_intern`] since it might
    /// avoid some memory allocations if the backends supports this.
    ///
    /// # Panics
    ///
    /// If the interner already interns the maximum number of strings possible
    /// by the chosen symbol type.
    #[inline]
    pub fn get_or_intern_static(&mut self, value: &'static [u8]) -> <B as Backend>::Symbol {
        self.get_or_intern_using(value, B::intern_static)
    }

    /// Shrink backend capacity to fit the interned strings exactly.
    pub fn shrink_to_fit(&mut self) {
        self.backend.shrink_to_fit()
    }

    /// Returns the string for the given symbol if any.
    #[inline]
    pub fn resolve(&self, symbol: <B as Backend>::Symbol) -> Option<&[u8]> {
        self.backend.resolve(symbol)
    }
}

impl<B, H, T> FromIterator<T> for BytesInterner<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher + Default,
    T: AsRef<[u8]>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let (capacity, _) = iter.size_hint();
        let mut interner = Self::with_capacity(capacity);
        interner.extend(iter);
        interner
    }
}

impl<B, H, T> Extend<T> for BytesInterner<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    H: BuildHasher,
    T: AsRef<[u8]>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for s in iter {
            self.get_or_intern(s.as_ref());
        }
    }
}

impl<'a, B, H> IntoIterator for &'a BytesInterner<B, H>
where
    B: Backend,
    <B as Backend>::Symbol: Symbol,
    &'a B: IntoIterator<Item = (<B as Backend>::Symbol, &'a [u8])>,
    H: BuildHasher,
{
    type Item = (<B as Backend>::Symbol, &'a [u8]);
    type IntoIter = <&'a B as IntoIterator>::IntoIter;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        self.backend.into_iter()
    }
}

#[derive(Debug)]
pub struct BytesMap<S = SymbolU16> {
    slices: Vec<Option<(u32, NonZeroU32)>>,
    buffer: Vec<u8>,
    marker: PhantomData<fn() -> S>,
}

impl<S: Symbol> BytesMap<S> {
    /// Creates a new empty `StringInterner`.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new() -> Self {
        Self {
            slices: Default::default(),
            buffer: Default::default(),
            marker: Default::default(),
        }
    }

    /// Creates a new `StringInterner` with the given initial capacity.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            slices: Vec::with_capacity(cap),
            buffer: Vec::with_capacity(cap),
            marker: Default::default(),
        }
    }

    #[inline]
    pub fn insert<T>(&mut self, k: usize, v: T) -> S
    where
        T: AsRef<[u8]>,
    {
        let v = v.as_ref();
        let a = self.buffer.len();
        assert_ne!(0, v.len());
        self.buffer.extend_from_slice(v);

        self.slices.resize(k, None);
        self.slices[k] = Some((
            a.to_u32().unwrap(),
            NonZeroU32::try_from(v.len().to_u32().unwrap()).unwrap(),
        ));
        S::try_from_usize(k).unwrap()
    }

    /// Returns the string for the given symbol if any.
    #[inline]
    pub fn get(&self, symbol: S) -> Option<&[u8]> {
        let c = self.slices.get(symbol.to_usize())?;
        let c = (*c)?;
        let l: u32 = c.1.try_into().unwrap();
        Some(&self.buffer[(c.0 as usize)..(l as usize)])
    }
}
