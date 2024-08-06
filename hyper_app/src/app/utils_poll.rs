use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::Hasher;

/// Much simpler than [`Buffered`]...
/// would need a bench to see if [`Buffered`] should be completly removed.
/// But  because of the T: std::marker::Send + 'static I need to impl my how Ser/De
// #[serde(default)]
// #[serde(bound (deserialize = "T: Default + Deserialize<'de>"))]
pub struct Buffered2<T: std::marker::Send + 'static, U = T> {
    content: Option<U>,
    // #[serde(skip)]
    promise: Option<poll_promise::Promise<T>>,
}

pub type Buffered3<U> = Buffered2<ehttp::Result<super::types::Resource<U>>, U>;

pub type Buffered4<U> = Buffered2<ehttp::Result<super::types::Resource<U>>, PreHashed<U>>;

pub type Buffered5<U, E> =
    Buffered2<ehttp::Result<super::types::Resource<Result<U, E>>>, Result<PreHashed<U>, E>>;

#[derive(Deserialize, Serialize)]
pub struct PreHashed<T> {
    pub value: T,
    hash: u64,
}
impl<T> PreHashed<T> {
    pub(crate) fn h(&self) -> u64 {
        self.hash
    }
}

impl<T> std::hash::Hash for PreHashed<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<T: std::hash::Hash> From<T> for PreHashed<T> {
    fn from(value: T) -> Self {
        let mut state = std::hash::DefaultHasher::new();
        value.hash(&mut state);
        let hash = state.finish();
        Self { value, hash }
    }
}

impl<T> std::ops::Deref for PreHashed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for PreHashed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: std::marker::Send + 'static, U> Default for Buffered2<T, U> {
    fn default() -> Self {
        Self {
            content: None,
            promise: None,
        }
    }
}

impl<'de, T: std::marker::Send + 'static, U: serde::Deserialize<'de>> serde::Deserialize<'de>
    for Buffered2<T, U>
{
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let content = Option::<U>::deserialize(d)?;
        Ok(Buffered2 {
            content,
            promise: None,
        })
    }
}

impl<T: std::marker::Send + 'static, U: Serialize> Serialize for Buffered2<T, U> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.content {
            None => s.serialize_none(),
            Some(t) => s.serialize_some(t),
        }
    }
}

impl<T: std::marker::Send + 'static, U: Debug> Debug for Buffered2<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = &mut f.debug_struct("Buffered2");
        if let Some(prom) = &self.promise {
            f = if prom.ready().is_some() {
                f.field("promise", &"ready")
            } else {
                f.field("promise", &"waiting")
            }
        }
        f = f.field("content", &self.content);
        f.finish()
    }
}

impl<T: std::marker::Send + 'static> Buffered2<T> {
    pub fn try_poll(&mut self) -> bool {
        if let Some(prom) = self.promise.take() {
            match prom.try_take() {
                Ok(ready) => {
                    self.content = Some(ready);
                    return true;
                }
                Err(prom) => self.promise = Some(prom),
            }
        }
        false
    }
}

impl<T: std::marker::Send + 'static, U> Buffered2<T, U> {
    pub fn try_poll_with(&mut self, mut f: impl FnMut(T) -> U) -> bool {
        if let Some(prom) = self.promise.take() {
            match prom.try_take() {
                Ok(ready) => {
                    self.content = Some(f(ready));
                    return true;
                }
                Err(prom) => self.promise = Some(prom),
            }
        }
        false
    }

    pub fn get_mut(&mut self) -> Option<&mut U> {
        self.content.as_mut()
    }

    pub fn get(&self) -> Option<&U> {
        self.content.as_ref()
    }

    // can be both waiting and holding content
    pub fn is_waiting(&self) -> bool {
        self.promise.is_some()
    }

    pub fn is_present(&self) -> bool {
        self.content.is_some()
    }

    pub fn buffer(&mut self, waiting: poll_promise::Promise<T>) {
        self.promise = Some(waiting)
    }

    pub fn take(&mut self) -> Option<U> {
        self.content.take()
    }
}

#[derive(Default, Deserialize, Serialize)]
pub enum Buffered<T: std::marker::Send + 'static> {
    #[default]
    Empty,
    #[serde(skip)]
    Init(poll_promise::Promise<T>),
    Single(T),
    #[serde(skip)]
    Waiting {
        content: T,
        waiting: poll_promise::Promise<T>,
    },
}

// impl<T: Serialize + std::marker::Send + 'static> Serialize for Buffered<T> {
//     fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         match self {
//             Buffered::Empty => s.serialize_none(),
//             Buffered::Init(_) => s.serialize_none(),
//             Buffered::Single(content) => s.serialize_some(content),
//             Buffered::Waiting { content, waiting } => s.serialize_some(content),
//         }
//     }
// }

impl<T: Debug + std::marker::Send + 'static> Debug for Buffered<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Init(waiting) => f
                .debug_tuple("Init")
                .field(&waiting.ready().is_none())
                .finish(),
            Self::Single(content) => f.debug_tuple("Single").field(content).finish(),
            Self::Waiting { content, waiting } => f
                .debug_struct("Waiting")
                .field("content", content)
                .field("waiting", &waiting.ready().is_none())
                .finish(),
        }
    }
}

impl<T: std::marker::Send + 'static> Buffered<T> {
    pub fn try_poll(&mut self) -> bool {
        let this = std::mem::take(self);
        let (changed, new) = match this {
            Buffered::Init(waiting) => match waiting.try_take() {
                Ok(ready) => (true, Buffered::Single(ready)),
                Err(waiting) => (false, Buffered::Init(waiting)),
            },
            Buffered::Waiting { waiting, content } => match waiting.try_take() {
                Ok(ready) => (true, Buffered::Single(ready)),
                Err(waiting) => (false, Buffered::Waiting { content, waiting }),
            },
            Buffered::Empty => (false, Buffered::Empty),
            Buffered::Single(content) => (false, Buffered::Single(content)),
        };
        *self = new;
        changed
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            Buffered::Empty | Buffered::Init(_) => None,
            Buffered::Single(content) | Buffered::Waiting { content, .. } => Some(content),
        }
    }

    pub fn is_waiting(&self) -> bool {
        match self {
            Buffered::Init(_) | Buffered::Waiting { .. } => true,
            _ => false,
        }
    }

    pub fn buffer(&mut self, waiting: poll_promise::Promise<T>) {
        let this = std::mem::take(self);
        *self = match this {
            Buffered::Empty => Buffered::Init(waiting),
            Buffered::Init(waiting) => Buffered::Init(waiting),
            Buffered::Single(content) => Buffered::Waiting { content, waiting },
            Buffered::Waiting {
                content,
                waiting: _,
            } => {
                // cancel old promise ?
                Buffered::Waiting { content, waiting }
            }
        };
    }

    pub fn take(&mut self) -> Option<T> {
        let this = std::mem::take(self);
        let (content, rest) = match this {
            Buffered::Waiting { waiting, content } => (Some(content), Buffered::Init(waiting)),
            Buffered::Single(content) => (Some(content), Buffered::Empty),
            x => (None, x),
        };
        *self = rest;
        content
    }
}

#[derive(Serialize, Deserialize)]
pub struct MultiBuffered<T, U: std::marker::Send + 'static> {
    pub(crate) content: Option<T>,
    #[serde(skip)]
    pub(crate) waiting: VecDeque<poll_promise::Promise<U>>,
}

impl<T, U: std::marker::Send + 'static> Default for MultiBuffered<T, U> {
    fn default() -> Self {
        Self {
            content: Default::default(),
            waiting: Default::default(),
        }
    }
}

pub trait Accumulable<Rhs = Self> {
    fn acc(&mut self, rhs: Rhs) -> bool;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccumulableResult<T, E> {
    pub(crate) content: T,
    pub(crate) errors: E,
}

impl<T: Accumulable<U>, U, E: Accumulable<F>, F> Accumulable<Result<U, F>>
    for AccumulableResult<T, E>
{
    fn acc(&mut self, rhs: Result<U, F>) -> bool {
        match rhs {
            Ok(rhs) => self.content.acc(rhs),
            Err(err) => self.errors.acc(err),
        }
    }
}

impl Accumulable<String> for Vec<String> {
    fn acc(&mut self, rhs: String) -> bool {
        self.push(rhs);
        true
    }
}

pub struct MultiBuffered2<K: Eq + std::hash::Hash, V2: std::marker::Send + 'static, V = V2> {
    pub(crate) content: HashMap<K, V>,
    pub(crate) waiting: HashMap<K, poll_promise::Promise<V2>>,
}

//
impl<
        'de,
        K: Eq + std::hash::Hash + serde::Deserialize<'de>,
        V2: std::marker::Send + 'static,
        V: serde::Deserialize<'de>,
    > serde::Deserialize<'de> for MultiBuffered2<K, V2, V>
{
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let content = HashMap::<K, V>::deserialize(d)?;
        Ok(MultiBuffered2 {
            content,
            waiting: Default::default(),
        })
    }
}

impl<K: Eq + std::hash::Hash + Serialize, V2: std::marker::Send + 'static, V: Serialize> Serialize
    for MultiBuffered2<K, V2, V>
{
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.content.serialize(s)
    }
}

impl<K: Eq + std::hash::Hash, V2: std::marker::Send + 'static, V> Default
    for MultiBuffered2<K, V2, V>
{
    fn default() -> Self {
        Self {
            content: Default::default(),
            waiting: Default::default(),
        }
    }
}

impl<K: Eq + std::hash::Hash, V2: std::marker::Send + 'static, V> MultiBuffered2<K, V2, V> {
    pub fn try_poll_with(&mut self, key: &K, mut f: impl FnMut(V2) -> V) -> bool {
        if let Some((key, prom)) = self.waiting.remove_entry(key) {
            match prom.try_take() {
                Ok(content) => {
                    self.content.insert(key, f(content));
                    true
                }
                Err(prom) => {
                    self.waiting.insert(key, prom);
                    false
                }
            }
        } else {
            false
        }
    }
    pub fn try_poll_all_waiting(&mut self, mut f: impl FnMut(V2) -> V) -> bool {
        let mut b = false;
        if self.waiting.is_empty() {
            return false;
        }
        self.waiting = std::mem::take(&mut self.waiting)
            .into_iter()
            .filter_map(|(key, prom)| match prom.try_take() {
                Ok(content) => {
                    self.content.insert(key, f(content));
                    b |= true;
                    None
                }
                Err(prom) => Some((key, prom)),
            })
            .collect();
        b
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.content.get_mut(key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.content.get(key)
    }

    #[allow(unused)]
    pub fn is_waiting(&self, k: &K) -> bool {
        self.waiting.contains_key(k)
    }

    pub fn insert(&mut self, key: K, waiting: poll_promise::Promise<V2>) {
        self.waiting.insert(key, waiting);
    }

    pub(crate) fn len_local(&self) -> usize {
        self.content.len()
    }

    pub(crate) fn is_absent<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        !self.content.contains_key(k) && !self.waiting.contains_key(k)
    }
}

impl<T: Default, U: std::marker::Send + 'static> MultiBuffered<T, U> {
    pub fn try_poll(&mut self) -> bool
    where
        T: Accumulable<U>,
    {
        if let Some(front) = self.waiting.pop_front() {
            match front.try_take() {
                Ok(content) => {
                    if self.content.is_none() {
                        self.content = Some(Default::default())
                    }
                    let Some(c) = &mut self.content else {
                        unreachable!()
                    };
                    c.acc(content)
                }
                Err(front) => {
                    self.waiting.push_front(front);
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.content.as_mut()
    }

    #[allow(unused)]
    pub fn is_waiting(&self) -> bool {
        !self.waiting.is_empty()
    }

    pub fn buffer(&mut self, waiting: poll_promise::Promise<U>) {
        self.waiting.push_back(waiting)
    }
    #[allow(unused)]
    pub fn take(&mut self) -> Option<T> {
        self.content.take()
    }
}

pub(crate) fn try_fetch_remote_file<R>(
    file_result: &std::collections::hash_map::Entry<
        '_,
        super::types::FileIdentifier,
        super::code_tracking::RemoteFile,
    >,
    mut f: impl FnMut(&super::code_tracking::FetchedFile) -> R,
) -> Option<Result<R, String>> {
    if let std::collections::hash_map::Entry::Occupied(promise) = file_result {
        let promise = promise.get();
        if let Some(result) = promise.ready() {
            match result {
                Ok(resource) => {
                    // ui_resource(ui, resource);
                    if let Some(text) = &resource.content {
                        Some(Ok(f(text)))
                    } else {
                        None
                    }
                }
                Err(error) => Some(Err(error.to_string())),
            }
        } else {
            None
        }
    } else {
        None
    }
}
