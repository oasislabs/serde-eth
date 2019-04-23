use serde::de;

use std::io::{Cursor, Read, Seek, SeekFrom};

use super::{
    error::{Category, Error, Result},
    eth,
};

pub struct Deserializer<R> {
    read: R,
}

impl<R: Read + Seek> Deserializer<R> {
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

    fn read_char(&mut self, scope_offset: u64) -> Result<char> {
        let bytes = self.read_byte_array(scope_offset, true)?;
        if bytes.len() > 4 {
            return Err(Error::parsing(
                "parsed char from byte array longer than 4 bytes",
            ));
        }

        match std::str::from_utf8(&bytes[..]) {
            Err(_) => Err(Error::parsing("parsed byte array cannot decode to a char")),
            Ok(s) => match s.chars().next() {
                Some(c) => Ok(c),
                None => Err(Error::parsing("parsed byte array cannot decode to a char")),
            },
        }
    }

    fn read_str(&mut self, scope_offset: u64) -> Result<String> {
        let bytes = self.read_byte_array(scope_offset, true)?;
        match std::str::from_utf8(&bytes[..]) {
            Err(_) => Err(Error::parsing("parsed byte array cannot decode to a char")),
            Ok(s) => Ok(s.to_string()),
        }
    }

    fn read_byte_array(&mut self, scope_offset: u64, consume_all: bool) -> Result<Vec<u8>> {
        let bytes_offset = self.read_uint(64)?;
        let offset = (bytes_offset - scope_offset) << 1;

        self.read
            .seek(SeekFrom::Current(offset as i64))
            .map_err(Error::io)?;

        let len = self.read_uint(64)?;
        let read_bytes = len << 1;
        // only read multiple of 64 bytes
        let base = (read_bytes >> 6) << 6;
        let remain = if read_bytes - base == 0 { 0 } else { 1 };
        let read_len = base + (remain << 6);
        let mut read_data = vec![0; read_len as usize];
        self.must_read(&mut read_data[..])?;

        let data = eth::decode_bytes(&read_data, len as usize)?;

        // after reading the relevant data the reader need to seek back
        // to the next available offset after the bytes that have been read

        if !consume_all {
            let seek_back_len = -((offset as i64) + (read_len as i64) + 64);
            self.read
                .seek(SeekFrom::Current(seek_back_len))
                .map_err(Error::io)?;
        }

        Ok(data)
    }

    fn read_uint(&mut self, size: usize) -> Result<u64> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        eth::decode_uint(&bytes, size)
    }

    fn read_int(&mut self, size: usize) -> Result<i64> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        eth::decode_int(&bytes, size)
    }
}

impl<'de, 'a, R: Read + Seek> de::Deserializer<'de> for &'a mut Deserializer<R> {
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
        let value = self.read_int(8)?;
        visitor.visit_i8(value as i8)
    }

    fn deserialize_i16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.read_int(16)?;
        visitor.visit_i16(value as i16)
    }

    fn deserialize_i32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.read_int(32)?;
        visitor.visit_i32(value as i32)
    }

    fn deserialize_i64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.read_int(64)?;
        visitor.visit_i64(value as i64)
    }

    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.read_uint(8)?;
        visitor.visit_u8(value as u8)
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.read_uint(16)?;
        visitor.visit_u16(value as u16)
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.read_uint(32)?;
        visitor.visit_u32(value as u32)
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.read_uint(64)?;
        visitor.visit_u64(value as u64)
    }

    fn deserialize_f32<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_f64<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let c = self.read_char(32)?;
        visitor.visit_char(c)
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.read_str(32)?;
        visitor.visit_str(&s)
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.read_str(32)?;
        visitor.visit_string(s)
    }

    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let b = self.read_byte_array(32, true)?;
        visitor.visit_bytes(&b[..])
    }

    #[inline]
    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let b = self.read_byte_array(32, true)?;
        visitor.visit_byte_buf(b)
    }

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

pub fn from_reader<'de, R: Read + Seek, T: de::Deserialize<'de>>(read: R) -> Result<T> {
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

    #[test]
    fn test_parse_char() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 6100000000000000000000000000000000000000000000000000000000000000",
                'a',
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000002\
                 c3a9000000000000000000000000000000000000000000000000000000000000",
                'é',
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000002\
                 c3b8000000000000000000000000000000000000000000000000000000000000",
                'ø',
            ),
        ];

        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_string() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000005\
                 68656c6c6f000000000000000000000000000000000000000000000000000000",
                "hello".to_string(),
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000000",
                "".to_string(),
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 000000000000000000000000000000000000000000000000000000000000005d\
                 736f6d65206c6f6e6720737472696e6720746861742074616b6573206d6f7265\
                 207468616e20333220627974657320736f2077652063616e2073656520686f77\
                 206574682061626920656e636f646573206c6f6e6720737472696e6773000000",
                "some long string that takes more than 32 bytes so we can see how eth abi \
                 encodes long strings"
                    .to_string(),
            ),
        ];

        test_parse_ok(tests);
    }
}
