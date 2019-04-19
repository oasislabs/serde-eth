use serde::ser;
use std::io;
use super::error::{Error, Result};

pub struct Serializer<W> {
    writer: W
}

impl<W> Serializer<W>
where
    W: io::Write
{
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer{ writer: writer }
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Compound<'a, W>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Compound<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_unit_struct(
        self,
        name: &'static str
    ) -> Result<Self::Ok>
    {
        Err(Error::not_implemented())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str
    ) -> Result<Self::Ok>
    {
        Err(Error::not_implemented())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn serialize_seq(
        self,
        len: Option<usize>
    ) -> Result<Self::SerializeSeq>
    {
        Err(Error::not_implemented())
    }

    fn serialize_tuple(
        self,
        len: usize
    ) -> Result<Self::SerializeTuple>
    {
        Err(Error::not_implemented())
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleStruct>
    {
        Err(Error::not_implemented())
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleVariant>
    {
        Err(Error::not_implemented())
    }

    fn serialize_map(
        self,
        len: Option<usize>
    ) -> Result<Self::SerializeMap>
    {
        Err(Error::not_implemented())
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize
    ) -> Result<Self::SerializeStruct>
    {
        Err(Error::not_implemented())
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize
    ) -> Result<Self::SerializeStruct>
    {
        Err(Error::not_implemented())
    }
}

pub struct Compound<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
}

impl<'a, W> ser::SerializeSeq for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a, W> ser::SerializeTuple for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a, W> ser::SerializeTupleStruct for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a, W> ser::SerializeTupleVariant for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a, W> ser::SerializeMap for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a, W> ser::SerializeStruct for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a, W> ser::SerializeStructVariant for Compound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}


