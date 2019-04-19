use serde::ser;
use std::io;
use super::eth;
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

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        let encoded = eth::encode_bool(value);
        self
            .writer
            .write_all(&encoded.into_bytes())
            .map_err(Error::io)
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        let encoded = eth::encode_i64(value);
        self
            .writer
            .write_all(&encoded.into_bytes())
            .map_err(Error::io)
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        let encoded = eth::encode_u64(value);
        self
            .writer
            .write_all(&encoded.into_bytes())
            .map_err(Error::io)
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

#[inline]
pub fn to_writer<W, T: ?Sized>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ser::Serialize,
{
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)?;
    Ok(())
}

#[inline]
pub fn to_vec<T: ?Sized>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize,
{
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value)?;
    Ok(writer)
}

#[inline]
pub fn to_string<T: ?Sized>(value: &T) -> Result<String>
where
    T: ser::Serialize,
{
    let vec = to_vec(value)?;
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

#[cfg(test)]
mod tests {
    use super::to_string;
    use serde::ser;
    use std::fmt;

    fn test_encode_ok<T>(errors: &[(T, &str)])
    where
        T: PartialEq + fmt::Debug + ser::Serialize,
    {
        for &(ref value, out) in errors {
            let out = out.to_string();

            let s = to_string(value).unwrap();
            assert_eq!(s, out);
        }
    }

    #[test]
    fn test_write_bool() {
        let tests = &[
            (false, "0000000000000000000000000000000000000000000000000000000000000000"),
            (true,  "0000000000000000000000000000000000000000000000000000000000000001"),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_u8() {
        let tests = &[
            (0x00 as u8, "0000000000000000000000000000000000000000000000000000000000000000"),
            (0x01 as u8, "0000000000000000000000000000000000000000000000000000000000000001"),
            (0x10 as u8, "0000000000000000000000000000000000000000000000000000000000000010"),
            (0x80 as u8, "0000000000000000000000000000000000000000000000000000000000000080"),
            (0xff as u8, "00000000000000000000000000000000000000000000000000000000000000ff"),
        ];
        test_encode_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_write_i8() {
        let tests = &[
            (0x00 as i8, "0000000000000000000000000000000000000000000000000000000000000000"),
            (0x01 as i8, "0000000000000000000000000000000000000000000000000000000000000001"),
            (0x10 as i8, "0000000000000000000000000000000000000000000000000000000000000010"),
            (0x80 as i8, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80"),
            (0xff as i8, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_u16() {
        let tests = &[
            (0x0000 as u16, "0000000000000000000000000000000000000000000000000000000000000000"),
            (0x0100 as u16, "0000000000000000000000000000000000000000000000000000000000000100"),
            (0x1000 as u16, "0000000000000000000000000000000000000000000000000000000000001000"),
            (0x8000 as u16, "0000000000000000000000000000000000000000000000000000000000008000"),
            (0xffff as u16, "000000000000000000000000000000000000000000000000000000000000ffff"),
        ];
        test_encode_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_write_i16() {
        let tests = &[
            (0x0000 as i16, "0000000000000000000000000000000000000000000000000000000000000000"),
            (0x0100 as i16, "0000000000000000000000000000000000000000000000000000000000000100"),
            (0x1000 as i16, "0000000000000000000000000000000000000000000000000000000000001000"),
            (0x8000 as i16, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8000"),
            (0xffff as i16, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
        ];
        test_encode_ok(tests);
    }


}
