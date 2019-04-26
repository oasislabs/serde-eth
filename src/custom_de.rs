use serde::de;

use super::{
    error::{Error, Result},
    eth::Fixed,
};

struct EthFixedDeserializer {
    /// offset keeps track of the current position
    offset: i8,

    /// offset_sign is the sign to move the offset forward.
    /// Use -1 for big endian arrays and 1 for little endian arrays
    offset_sign: i8,

    /// expected number of elements to deserialize
    len: usize,

    /// serializer_type is the type of serializer to do
    serializer_type: Fixed,

    /// content that the serializer aggregates. Call `serialize` to
    /// have the hex serialization of the data
    content: Vec<u8>,
}

impl<'de> de::Deserializer<'de> for &mut EthFixedDeserializer {
    type Error = Error;

    fn deserialize_any<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_i8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.serializer_type {
            Fixed::U256 => panic!("received i8 when deserializing U256"),
            Fixed::H256 | Fixed::H160 => {
                let value = self.content[self.offset as usize];
                self.offset += self.offset_sign;
                visitor.visit_i8(value as i8)
            }
        }
    }

    fn deserialize_i16<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_i32<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_i64<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.serializer_type {
            Fixed::U256 => panic!("received u8 when deserializing U256"),
            Fixed::H256 | Fixed::H160 => {
                let value = self.content[self.offset as usize];
                self.offset += self.offset_sign;
                visitor.visit_u8(value)
            }
        }
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.serializer_type {
            Fixed::H256 | Fixed::H160 => panic!("received u64 when deserializing H256,H160"),
            Fixed::U256 => {
                let value = (self.content[self.offset as usize] as u64)
                    + ((self.content[(self.offset + self.offset_sign) as usize] as u64) << 8)
                    + ((self.content[(self.offset + 2 * self.offset_sign) as usize] as u64) << 16)
                    + ((self.content[(self.offset + 3 * self.offset_sign) as usize] as u64) << 24)
                    + ((self.content[(self.offset + 4 * self.offset_sign) as usize] as u64) << 32)
                    + ((self.content[(self.offset + 5 * self.offset_sign) as usize] as u64) << 40)
                    + ((self.content[(self.offset + 6 * self.offset_sign) as usize] as u64) << 48)
                    + ((self.content[(self.offset + 7 * self.offset_sign) as usize] as u64) << 56);
                self.offset += 8 * self.offset_sign;
                visitor.visit_u64(value)
            }
        }
    }

    fn deserialize_f32<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_f64<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_char<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_bytes<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_option<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    #[inline]
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &str,
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_seq<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_tuple<V: de::Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(EthTupleAccess::new(self))
    }

    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_map<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    #[inline]
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_identifier<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }
}

struct EthTupleAccess<'a> {
    count: usize,
    len: usize,
    de: &'a mut EthFixedDeserializer,
}

impl<'a> EthTupleAccess<'a> {
    pub fn new(de: &'a mut EthFixedDeserializer) -> Self {
        EthTupleAccess {
            count: 0,
            len: de.len,
            de: de,
        }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for EthTupleAccess<'a> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        if self.count >= self.len {
            return Ok(None);
        } else {
            self.count += 1;
            match seed.deserialize(&mut *self.de) {
                Ok(value) => Ok(Some(value)),
                Err(err) => Err(err),
            }
        }
    }
}

pub struct EthFixedAccess {
    count: usize,
    len: usize,
    de: EthFixedDeserializer,
}

impl EthFixedAccess {
    pub fn new(bytes: Vec<u8>, t: Fixed) -> Self {
        let (len, offset, offset_sign) = match t {
            Fixed::H256 => (32, 0, 1),
            Fixed::H160 => (20, 12, 1),
            Fixed::U256 => (4, 31, -1),
        };

        if bytes.len() < 32 {
            panic!("the expected number of bytes is 32")
        }

        EthFixedAccess {
            count: 0,
            len: 1,
            de: EthFixedDeserializer {
                len: len,
                offset: offset,
                offset_sign: offset_sign,
                serializer_type: t,
                content: bytes,
            },
        }
    }
}

impl<'de> de::SeqAccess<'de> for EthFixedAccess {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        if self.count >= self.len {
            return Ok(None);
        } else {
            self.count += 1;
            match seed.deserialize(&mut self.de) {
                Ok(value) => Ok(Some(value)),
                Err(err) => Err(err),
            }
        }
    }
}
