use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Index,
    str::Utf8Error,
};

use num::ToPrimitive;
use string_interner::{symbol::SymbolU16, Symbol};

use crate::filter::default::VaryHasher;

pub trait MySerializePar {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Serialize a sequence element.
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>;

    /// Finish serializing a sequence.
    fn end(self) -> Result<Self::Ok, Self::Error>;
}
pub trait MySerializeSco {
    /// Must match the `Ok` type of our `Serializer`.
    type Ok;

    /// Must match the `Error` type of our `Serializer`.
    type Error: Error;

    /// Serialize a sequence element.
    fn serialize_object<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>;

    /// Finish serializing a sequence.
    fn end(self, s: &str) -> Result<Self::Ok, Self::Error>;
}

pub trait Keyed<T> {
    fn key(&self) -> T;
}

pub trait Error: Sized + std::error::Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display;
}

pub trait MySerializer: Sized {
    /// The output type produced by this `Serializer` during successful
    /// serialization. Most serializers that produce text or binary output
    /// should set `Ok = ()` and serialize into an [`io::Write`] or buffer
    /// contained within the `Serializer` instance. Serializers that build
    /// in-memory data structures may be simplified by using `Ok` to propagate
    /// the data structure around.
    ///
    /// [`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
    type Ok;

    /// The error type when some error occurs during serialization.
    type Error: Error;

    /// Type returned from [`serialize_seq`] for serializing the content of the
    /// sequence.
    ///
    /// [`serialize_seq`]: #tymethod.serialize_seq
    type SerializePar: MySerializePar<Ok = Self::Ok, Error = Self::Error>;
    type SerializeSco: MySerializeSco<Ok = Self::Ok, Error = Self::Error>;

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Display;

    fn serialize_par(self, len: Option<usize>) -> Result<Self::SerializePar, Self::Error>;

    fn serialize_sco(self, len: Option<usize>) -> Result<Self::SerializeSco, Self::Error>;
}

pub trait MySerialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: MySerializer;
}

pub struct Table<T> {
    offsets: Vec<u32>,
    choices: Vec<u16>,
    buf: Vec<T>,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self {
            offsets: Default::default(),
            choices: Default::default(),
            buf: Default::default(),
        }
    }
}

impl<T> Index<SymbolU16> for Table<T> {
    type Output = [T];

    fn index(&self, index: SymbolU16) -> &Self::Output {
        let index = index.to_usize();
        let o = self.offsets[index] as usize;
        let c = self.choices[index] as usize;
        &self.buf[o..o + c]
    }
}

impl<T> Table<T> {
    fn insert(&mut self, index: usize, v: Vec<T>) -> SymbolU16 {
        assert_ne!(v.len(), 0);
        if self.offsets.len() <= index {
            self.offsets.resize(index + 1, 0);
            self.choices.resize(index + 1, 0);
        }
        if self.offsets[index] != 0 {
            assert!(self.choices[index] == v.len().to_u16().unwrap());
            return SymbolU16::try_from_usize(index).unwrap();
        }
        self.offsets[index] = self.buf.len().to_u32().unwrap();
        self.choices[index] = v.len().to_u16().unwrap();
        self.buf.extend(v);
        SymbolU16::try_from_usize(index).unwrap()
    }
}

#[derive(Debug)]
pub struct CachedHasherError(String);

impl Display for CachedHasherError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl From<std::io::Error> for CachedHasherError {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}
impl From<Utf8Error> for CachedHasherError {
    fn from(e: Utf8Error) -> Self {
        Self(e.to_string())
    }
}

impl std::error::Error for CachedHasherError {}

impl Error for CachedHasherError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self(msg.to_string())
    }
}

/// Could simplify structurally, fusioning Auxilary serializers
pub struct CachedHasher<'a, I, S, H: VaryHasher<S>> {
    pub(crate) index: I,
    pub(crate) table: &'a mut Table<H>,
    pub(crate) phantom: PhantomData<(*const H,*const S)>,
}
impl<'a, I, S, H: VaryHasher<S>> CachedHasher<'a, I, S, H> {
    pub fn new(table: &'a mut Table<H>, index: I) -> Self {
        Self {
            index,
            table: table,
            phantom: PhantomData,
        }
    }
}
impl<H: VaryHasher<u8>> CachedHasher<'static, usize, u8, H> {
    pub fn once<T: MySerialize + Keyed<usize>>(x: T) -> Vec<u8> {
        let mut table = Default::default();
        let s = CachedHasher::<usize, u8, H> {
            index: x.key(),
            table: &mut table,
            phantom: PhantomData,
        };
        match x.serialize(s) {
            Ok(x) => table[x].iter().map(|x| x.finish()).collect(),
            Err(e) => {
                log::error!("error {} with hashing of {}",e, x.key());
                vec![]
            },
        }
        
    }
}

impl<'a, H: VaryHasher<u16>> CachedHasher<'a, usize, u16, H> {
    pub fn once<T: MySerialize + Keyed<usize>>(x: T) -> Vec<u16> {
        let mut table = Default::default();
        let s = CachedHasher::<usize, u16, H> {
            index: x.key(),
            table: &mut table,
            phantom: PhantomData,
        };
        match x.serialize(s) {
            Ok(x) => table[x].iter().map(|x| x.finish()).collect(),
            Err(e) => {
                log::error!("error {} with hashing of {}",e, x.key());
                vec![]
            },
        }
    }
}

impl<'a, H: 'a + VaryHasher<u8>> MySerializer for CachedHasher<'a, usize, u8, H> {
    type Ok = SymbolU16; // TODO use an u8 symbol

    type Error = CachedHasherError;

    type SerializePar = CachedHasherAux<'a, usize, u8, H>;
    type SerializeSco = CachedHasherAux<'a, usize, u8, H>;

    fn serialize_par(self, _: Option<usize>) -> Result<Self::SerializePar, Self::Error> {
        Ok(CachedHasherAux {
            index: self.index,
            table: self.table,
            acc: Default::default(),
            _phantom: PhantomData,
        })
    }

    fn serialize_sco(self, _: Option<usize>) -> Result<Self::SerializeSco, Self::Error> {
        Ok(CachedHasherAux {
            index: self.index,
            table: self.table,
            acc: Default::default(),
            _phantom: PhantomData,
        })
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Display,
    {
        let mut h = H::new(0);
        h.write(value.to_string().as_bytes());
        let x = self.table.insert(self.index, vec![h]);
        Ok(x)
    }
}

impl<'a, H: 'a + VaryHasher<u16>> MySerializer for CachedHasher<'a, usize, u16, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    type SerializePar = CachedHasherAux<'a, usize, u16, H>;
    type SerializeSco = CachedHasherAux<'a, usize, u16, H>;

    fn serialize_par(self, _: Option<usize>) -> Result<Self::SerializePar, Self::Error> {
        Ok(CachedHasherAux {
            index: self.index,
            table: self.table,
            acc: Default::default(),
            _phantom: PhantomData,
        })
    }

    fn serialize_sco(self, _: Option<usize>) -> Result<Self::SerializeSco, Self::Error> {
        Ok(CachedHasherAux {
            index: self.index,
            table: self.table,
            acc: Default::default(),
            _phantom: PhantomData,
        })
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Display,
    {
        let mut h = H::new(0);
        h.write(value.to_string().as_bytes());
        let x = self.table.insert(self.index, vec![h]);
        Ok(x)
    }
}

pub struct CachedHasherAux<'a, I, S, H: VaryHasher<S>> {
    index: I,
    table: &'a mut Table<H>,
    acc: Vec<H>,
    _phantom: PhantomData<*const S>
}

impl<'a, H: VaryHasher<u8>> MySerializePar for CachedHasherAux<'a, usize, u8, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>,
    {
        let x = value.serialize(CachedHasher::<_, _, H> {
            index: value.key(),
            table: self.table,
            phantom: PhantomData,
        })?;
        for x in &self.table[x] {
            let h = x.clone();
            self.acc.push(h);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.acc.is_empty() {
            Err(CachedHasherError::custom("empty element"))
        } else {
            Ok(self.table.insert(self.index, self.acc))
        }
    }
}

impl<'a, H: VaryHasher<u16>> MySerializePar for CachedHasherAux<'a, usize, u16, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>,
    {
        let x = value.serialize(CachedHasher::<_, _, H> {
            index: value.key(),
            table: self.table,
            phantom: PhantomData,
        })?;
        for x in &self.table[x] {
            let h = x.clone();
            self.acc.push(h);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.acc.is_empty() {
            Err(CachedHasherError::custom("empty element"))
        } else {
            Ok(self.table.insert(self.index, self.acc))
        }
    }
}
impl<'a, H: VaryHasher<u8>> MySerializeSco for CachedHasherAux<'a, usize, u8, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    fn serialize_object<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>,
    {
        let x = value.serialize(CachedHasher::<_, _, H> {
            index: value.key(),
            table: self.table,
            phantom: PhantomData,
        })?;
        for x in &self.table[x] {
            let h = x.clone();
            self.acc.push(h);
        }
        Ok(())
    }

    fn end(mut self, s: &str) -> Result<Self::Ok, Self::Error> {
        for h in &mut self.acc {
            h.write(s.as_bytes());
        }
        Ok(self.table.insert(self.index, self.acc))
    }
}
impl<'a, H: VaryHasher<u16>> MySerializeSco for CachedHasherAux<'a, usize, u16, H> {
    type Ok = SymbolU16;

    type Error = CachedHasherError;

    fn serialize_object<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: MySerialize + Keyed<usize>,
    {
        let x = value.serialize(CachedHasher::<_, _, H> {
            index: value.key(),
            table: self.table,
            phantom: PhantomData,
        })?;
        for x in &self.table[x] {
            let h = x.clone();
            self.acc.push(h);
        }
        Ok(())
    }

    fn end(mut self, s: &str) -> Result<Self::Ok, Self::Error> {
        for h in &mut self.acc {
            h.write(s.as_bytes());
        }
        Ok(self.table.insert(self.index, self.acc))
    }
}
