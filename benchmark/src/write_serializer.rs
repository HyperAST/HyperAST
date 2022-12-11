use std::{fmt::Display, io::Write, str::Utf8Error};

use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize, Serializer,
};

pub struct WriteJson<'a, W: Write> {
    out: &'a mut W,
}
impl<'a, W: Write> From<&'a mut W> for WriteJson<'a, W> {
    fn from(out: &'a mut W) -> Self {
        Self { out }
    }
}

#[derive(Debug)]
pub struct WriteJsonError(String);

impl Display for WriteJsonError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl From<std::io::Error> for WriteJsonError {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}
impl From<Utf8Error> for WriteJsonError {
    fn from(e: Utf8Error) -> Self {
        Self(e.to_string())
    }
}

impl std::error::Error for WriteJsonError {}

impl serde::ser::Error for WriteJsonError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self(msg.to_string())
    }
}

pub struct WriteJsonSeq<'a, W: Write> {
    out: &'a mut W,
    first: bool,
}

impl<'a, W: Write> SerializeTuple for WriteJsonSeq<'a, W> {
    type Ok = ();

    type Error = WriteJsonError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            write!(self.out, ",")?;
        }
        value.serialize(WriteJson { out: self.out })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "]")?)
    }
}

impl<'a, W: Write> SerializeMap for WriteJsonSeq<'a, W> {
    type Ok = ();

    type Error = WriteJsonError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            write!(self.out, ",")?;
        }
        key.serialize(WriteJson { out: self.out })
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        write!(self.out, ":")?;
        value.serialize(WriteJson { out: self.out })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "}}")?)
    }
}

impl<'a, W: Write> SerializeStruct for WriteJsonSeq<'a, W> {
    type Ok = ();

    type Error = WriteJsonError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            write!(self.out, ",")?;
        }
        write!(self.out, "\"{}\":", key)?;
        value.serialize(WriteJson { out: self.out })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "}}")?)
    }
}

impl<'a, W: Write> SerializeStructVariant for WriteJsonSeq<'a, W> {
    type Ok = ();

    type Error = WriteJsonError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        write!(self.out, ",")?;
        write!(self.out, "\"{}\":", key)?;
        value.serialize(WriteJson { out: self.out })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W: Write> SerializeTupleStruct for WriteJsonSeq<'a, W> {
    type Ok = ();

    type Error = WriteJsonError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        write!(self.out, ",")?;
        value.serialize(WriteJson { out: self.out })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "]}}")?)
    }
}

impl<'a, W: Write> SerializeTupleVariant for WriteJsonSeq<'a, W> {
    type Ok = ();

    type Error = WriteJsonError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        write!(self.out, ",")?;
        value.serialize(WriteJson { out: self.out })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "]}}")?)
    }
}

impl<'a, W: Write> SerializeSeq for WriteJsonSeq<'a, W> {
    type Ok = ();

    type Error = WriteJsonError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        if self.first {
            self.first = false;
        } else {
            write!(self.out, ",")?;
        }
        value.serialize(WriteJson { out: self.out })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "]")?)
    }
}

impl<'a, W: Write> Serializer for WriteJson<'a, W> {
    type Ok = ();

    type Error = WriteJsonError;

    type SerializeSeq = WriteJsonSeq<'a, W>;

    type SerializeTuple = WriteJsonSeq<'a, W>;

    type SerializeTupleStruct = WriteJsonSeq<'a, W>;

    type SerializeTupleVariant = WriteJsonSeq<'a, W>;

    type SerializeMap = WriteJsonSeq<'a, W>;

    type SerializeStruct = WriteJsonSeq<'a, W>;

    type SerializeStructVariant = WriteJsonSeq<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "\"{}\"", v)?)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "\"{}\"", v)?)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let v = std::str::from_utf8(v)?;
        Ok(write!(self.out, "\"{}\"", v)?)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "null")?)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "")?)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "")?)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        write!(self.out, "[")?;
        Ok(WriteJsonSeq {
            out: self.out,
            first: true,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        write!(self.out, "[")?;
        Ok(WriteJsonSeq {
            out: self.out,
            first: true,
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        write!(self.out, "{{")?;
        Ok(WriteJsonSeq {
            out: self.out,
            first: true,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        write!(self.out, "{{")?;
        Ok(WriteJsonSeq {
            out: self.out,
            first: true,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

pub struct WritePartialJson<'a, W: Write> {
    out: &'a mut W,
}

impl<'a, W: Write> From<&'a mut W> for WritePartialJson<'a, W> {
    fn from(out: &'a mut W) -> Self {
        Self { out }
    }
}

impl<'a, W: Write> Serializer for WritePartialJson<'a, W> {
    type Ok = ();

    type Error = WriteJsonError;

    type SerializeSeq = WriteJsonSeq<'a, W>;

    type SerializeTuple = WriteJsonSeq<'a, W>;

    type SerializeTupleStruct = WriteJsonSeq<'a, W>;

    type SerializeTupleVariant = WriteJsonSeq<'a, W>;

    type SerializeMap = WriteJsonSeq<'a, W>;

    type SerializeStruct = WriteJsonSeq<'a, W>;

    type SerializeStructVariant = WriteJsonSeq<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "{}", v)?)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "\"{}\"", v)?)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "\"{}\"", v)?)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let v = std::str::from_utf8(v)?;
        Ok(write!(self.out, "\"{}\"", v)?)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "null")?)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(WriteJson { out: self.out })
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "")?)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(write!(self.out, "")?)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(WriteJson { out: self.out })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(WriteJsonSeq {
            out: self.out,
            first: false,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(WriteJsonSeq {
            out: self.out,
            first: false,
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(WriteJsonSeq {
            out: self.out,
            first: false,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(WriteJsonSeq {
            out: self.out,
            first: false,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}
