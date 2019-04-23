use serde::de;

use std::io::{Cursor, Read};

use super::{
    error::{Category, Error, Result},
    eth,
};

pub struct Deserializer<R> {
    read: R,
}

impl<R: Read> Deserializer<R> {
    pub fn new(read: R) -> Self {
        Deserializer { read: read }
    }

    pub fn must_read(&mut self, bytes: &mut [u8]) -> Result<()> {
        let mut total_read_bytes: usize = 0;

        while total_read_bytes < bytes.len() {
            let read_bytes = self
                .read
                .read(&mut bytes[total_read_bytes..])
                .map_err(Error::io)?;
            if read_bytes == 0 {
                break;
            }

            total_read_bytes += read_bytes;
        }

        if total_read_bytes == bytes.len() {
            Ok(())
        } else {
            Err(Error::message("insufficient bytes read from reader"))
        }
    }

    pub fn end(&mut self) -> Result<()> {
        let mut bytes = [0 as u8; 1];
        let res = self.must_read(&mut bytes);

        match res {
            Ok(_) => Err(Error::parsing("input has not been processed completely")),
            Err(err) => match err.classify() {
                Category::Data => Ok(()),
                _ => Err(Error::parsing("failed to verify if reader is empty")),
            },
        }
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_bool(&bytes)?;
        visitor.visit_bool(value)
    }

    fn deserialize_i8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_int(&bytes, 8)?;
        visitor.visit_i8(value as i8)
    }

    fn deserialize_i16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_int(&bytes, 16)?;
        visitor.visit_i16(value as i16)
    }

    fn deserialize_i32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_int(&bytes, 32)?;
        visitor.visit_i32(value as i32)
    }

    fn deserialize_i64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_int(&bytes, 64)?;
        visitor.visit_i64(value as i64)
    }

    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_uint(&bytes, 8)?;
        visitor.visit_u8(value as u8)
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_uint(&bytes, 16)?;
        visitor.visit_u16(value as u16)
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_uint(&bytes, 32)?;
        visitor.visit_u32(value as u32)
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_uint(&bytes, 64)?;
        visitor.visit_u64(value as u64)
    }

    fn deserialize_f32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_f64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    #[inline]
    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    /// Parses a `null` as a None, and any other values as a `Some(...)`.
    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    /// Parses a newtype struct as the underlying value.
    #[inline]
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        name: &str,
        visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_tuple<V: de::Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_map<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    /// Parses an enum as an object like `{"$KEY":$VALUE}`, where $VALUE is either a straight
    /// value, a `[..]`, or a `{..}`.
    #[inline]
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_identifier<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }
}

pub fn from_reader<'de, R: Read, T: de::Deserialize<'de>>(read: R) -> Result<T> {
    let mut de = Deserializer::new(read);
    let value = de::Deserialize::deserialize(&mut de)?;

    de.end()?;
    Ok(value)
}

pub fn from_str<'a, T: de::Deserialize<'a>>(s: &'a str) -> Result<T> {
    from_reader(Cursor::new(s))
}

#[cfg(test)]
mod tests {

    use super::from_str;
    use crate::error::Result;
    use serde::{de, ser};
    use std::{error::Error, fmt::Debug};

    fn test_parse_ok<T: Clone + Debug + PartialEq + ser::Serialize + de::DeserializeOwned>(
        tests: &[(&str, T)],
    ) {
        for (s, value) in tests {
            let v: T = from_str(s).unwrap();
            assert_eq!(v, value.clone());
        }
    }

    fn test_parse_error<T: Clone + Debug + PartialEq + ser::Serialize + de::DeserializeOwned>(
        tests: &[(&str, &str)],
    ) {
        for (s, expected) in tests {
            let res: Result<T> = from_str(s);

            match res {
                Ok(_) => assert_eq!("expected error".to_string(), expected.to_string()),
                Err(err) => assert_eq!(err.description().to_string(), expected.to_string()),
            }
        }
    }

    #[test]
    fn test_parse_bool() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                false,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000001",
                true,
            ),
        ];

        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_bool_error() {
        let tests = &[
            (
                "000000000000000000000000000000000000000000000000000000000000000000",
                "input has not been processed completely",
            ),
            (
                "1000000000000000000000000000000000000000000000000000000000000000",
                "Cannot decode bool",
            ),
            (
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "invalid character",
            ),
            ("0", "insufficient bytes read from reader"),
            ("", "insufficient bytes read from reader"),
        ];

        test_parse_error::<bool>(tests);
    }

    #[test]
    fn test_parse_u8() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                0x00 as u8,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000001",
                0x01 as u8,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000010",
                0x10 as u8,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000080",
                0x80 as u8,
            ),
            (
                "00000000000000000000000000000000000000000000000000000000000000ff",
                0xff as u8,
            ),
        ];
        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_u8_error() {
        let tests = &[
            (
                "1111111111111111111111111111111111111111111111111111111111111111",
                "expected error",
            ),
            (
                "2222222222222222222222222222222222222222222222222222222222222222",
                "expected error",
            ),
            (
                "0x0000000000000000000000000000000000000000000000000000000000000000",
                "invalid character",
            ),
            ("0", "insufficient bytes read from reader"),
            ("", "insufficient bytes read from reader"),
        ];

        test_parse_error::<u8>(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_parse_i8() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                0x00 as i8,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000001",
                0x01 as i8,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000010",
                0x10 as i8,
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
                0x80 as i8,
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                0xff as i8,
            ),
        ];
        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_u16() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                0x0000 as u16,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000100",
                0x0100 as u16,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000001000",
                0x1000 as u16,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000008000",
                0x8000 as u16,
            ),
            (
                "000000000000000000000000000000000000000000000000000000000000ffff",
                0xffff as u16,
            ),
        ];
        test_parse_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_parse_i16() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                0x0000 as i16,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000100",
                0x0100 as i16,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000001000",
                0x1000 as i16,
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8000",
                0x8000 as i16,
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                0xffff as i16,
            ),
        ];
        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_u32() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                0x00000000 as u32,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000001000000",
                0x01000000 as u32,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000010000000",
                0x10000000 as u32,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000080000000",
                0x80000000 as u32,
            ),
            (
                "00000000000000000000000000000000000000000000000000000000ffffffff",
                0xffffffff as u32,
            ),
        ];
        test_parse_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_parse_i32() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                0x00000000 as i32,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000001000000",
                0x01000000 as i32,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000010000000",
                0x10000000 as i32,
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffff80000000",
                0x80000000 as i32,
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                0xffffffff as i32,
            ),
        ];
        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_u64() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                0x0000000000000000 as u64,
            ),
            (
                "0000000000000000000000000000000000000000000000000100000000000000",
                0x0100000000000000 as u64,
            ),
            (
                "0000000000000000000000000000000000000000000000001000000000000000",
                0x1000000000000000 as u64,
            ),
            (
                "0000000000000000000000000000000000000000000000008000000000000000",
                0x8000000000000000 as u64,
            ),
            (
                "000000000000000000000000000000000000000000000000ffffffffffffffff",
                0xffffffffffffffff as u64,
            ),
        ];
        test_parse_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_parse_i64() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000000",
                0x0000000000000000 as i64,
            ),
            (
                "0000000000000000000000000000000000000000000000000100000000000000",
                0x0100000000000000 as i64,
            ),
            (
                "0000000000000000000000000000000000000000000000001000000000000000",
                0x1000000000000000 as i64,
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffff8000000000000000",
                0x8000000000000000 as i64,
            ),
            (
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                0xffffffffffffffff as i64,
            ),
        ];
        test_parse_ok(tests);
    }

}
