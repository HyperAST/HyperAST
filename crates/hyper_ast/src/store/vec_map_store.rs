use std::{
    cell::{Ref, RefCell},
    collections::hash_map::DefaultHasher,
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    rc::Rc,
};

use crate::{
    compat::{DefaultHashBuilder, HashMap},
    utils::make_hash,
};

pub struct VecHasher<T: Hash> {
    state: u64,
    node_table: Rc<RefCell<Vec<T>>>,
    default: DefaultHasher,
}

impl<T: Hash> Hasher for VecHasher<T> {
    fn write_u8(&mut self, i: u8) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write_u16(&mut self, i: u16) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write_u32(&mut self, i: u32) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write_u64(&mut self, i: u64) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write_usize(&mut self, i: usize) {
        self.node_table.borrow()[i as usize].hash(&mut self.default);
        self.state = self.default.finish();
    }
    fn write(&mut self, _bytes: &[u8]) {
        // for &byte in bytes {
        //     self.state = self.state.rotate_left(8) ^ u64::from(byte);
        // }
        panic!("should not have been called")
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

pub(crate) struct BuildVecHasher<T: Hash> {
    node_table: Rc<RefCell<Vec<T>>>,
}

impl<T: Hash> BuildHasher for BuildVecHasher<T> {
    type Hasher = VecHasher<T>;
    fn build_hasher(&self) -> VecHasher<T> {
        VecHasher {
            state: 0,
            node_table: self.node_table.clone(),
            default: DefaultHasher::new(),
        }
    }
}

pub trait Convertible: Copy + Debug {
    fn from(x: usize) -> Self;
    fn to(&self) -> usize;
}

impl Convertible for u8 {
    fn from(x: usize) -> Self {
        x as u8
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

impl Convertible for u16 {
    fn from(x: usize) -> Self {
        x as u16
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

impl Convertible for u32 {
    fn from(x: usize) -> Self {
        x as u32
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

impl Convertible for u64 {
    fn from(x: usize) -> Self {
        x as u64
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

impl Convertible for usize {
    fn from(x: usize) -> Self {
        x
    }
    fn to(&self) -> usize {
        *self as usize
    }
}

pub trait ArrayOffset: Convertible {
    fn offseted_hash<H: Hasher>(&self, state: &mut H);
}

impl ArrayOffset for u8 {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(*self);
    }
}
impl ArrayOffset for u16 {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(*self);
    }
}
impl ArrayOffset for u32 {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(*self);
    }
}
impl ArrayOffset for u64 {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(*self);
    }
}
impl ArrayOffset for usize {
    fn offseted_hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(*self);
    }
}

// pub struct VecMapStore<T: Hash, I: ArrayOffset> {
//     hash_table: HashSet<VecMapStoreEntry<I>, BuildVecHasher<T>>,
//     node_table: Rc<RefCell<Vec<T>>>,
//     counter: ConsistentCounter,
//     dedup: HashMap<<B as Backend>::Symbol, (), ()>;
// }

pub trait Symbol<T>: Copy + Eq {}
pub trait VecSymbol<T>: Symbol<T> {
    /// Creates a symbol from a `usize`.
    ///
    /// Returns `None` if `index` is out of bounds for the symbol.
    fn try_from_usize(index: usize) -> Option<Self>;

    /// Returns the `usize` representation of `self`.
    fn to_usize(self) -> usize;
}

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

impl<T> VecSymbol<T> for SymbolU32<T> {
    fn try_from_usize(index: usize) -> Option<Self> {
        use string_interner::Symbol;
        string_interner::symbol::SymbolU32::try_from_usize(index).and_then(|internal| {
            Some(Self {
                internal,
                _phantom: PhantomData,
            })
        })
    }

    fn to_usize(self) -> usize {
        use string_interner::Symbol;
        self.internal.to_usize()
    }
}

pub trait AsNodeEntityRef {
    type Ref<'a>
    where
        Self: 'a;
    fn eq(&self, other: &Self::Ref<'_>) -> bool;
}
pub trait AsNodeEntityRefSelf: AsNodeEntityRef {
    fn as_ref(&self) -> Self::Ref<'_>;
}

impl AsNodeEntityRef for Box<[u8]> {
    type Ref<'a> = &'a [u8];

    fn eq(&self, other: &Self::Ref<'_>) -> bool {
        AsNodeEntityRefSelf::as_ref(self) == *other
    }
}

impl AsNodeEntityRefSelf for Box<[u8]> {
    fn as_ref(&self) -> Self::Ref<'_> {
        AsRef::as_ref(self)
    }
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

impl<T: AsNodeEntityRefSelf, S: VecSymbol<T>> Backend<T> for VecBackend<T, S>
// where T: for<'a> AsNodeEntityRef<Ref<'a>=&'a T> ,
{
    type Symbol = S;

    fn with_capacity(cap: usize) -> Self {
        Self {
            internal: Vec::with_capacity(cap),
            phantom: PhantomData,
        }
    }

    fn intern(&mut self, node: T) -> Self::Symbol {
        let s = Self::Symbol::try_from_usize(self.internal.len())
            .expect("not enough symbol, you should take a bigger set");
        self.internal.push(node);
        s
    }

    fn shrink_to_fit(&mut self) {
        self.internal.shrink_to_fit()
    }

    fn resolve(&self, symbol: Self::Symbol) -> Option<T::Ref<'_>> {
        self.internal.get(symbol.to_usize()).map(|x| x.as_ref())
    }

    unsafe fn resolve_unchecked(&self, symbol: Self::Symbol) -> T::Ref<'_> {
        self.internal.get_unchecked(symbol.to_usize()).as_ref()
    }

    fn len(&self) -> usize {
        self.internal.len()
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
        let a = &dedup.len();
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
