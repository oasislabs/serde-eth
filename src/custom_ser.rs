use serde::ser;

use super::{
    error::{Error, Result},
    eth::Fixed,
};

pub struct BasicEthSerializer {
    /// offset keeps track of the current position
    offset: i8,

    /// offset_sign is the sign to move the offset forward.
    /// Use -1 for big endian arrays and 1 for little endian arrays
    offset_sign: i8,

    /// serializer_type is the type of serializer to do
    serializer_type: Fixed,

    /// content that the serializer aggregates. Call `serialize` to
    /// have the hex serialization of the data
    content: [u8; 32],
}

impl BasicEthSerializer {
    /// new_hash creates a new serializer for ethereum
    /// hash types. These are stored as u8 arrays with
    /// little endian byte order
    pub fn new_hash(len: usize) -> Self {
        if len != 20 && len != 32 {
            panic!("BasicEthSerializer only supports H160, H256")
        }

        BasicEthSerializer {
            offset: 32 - (len as i8),
            offset_sign: 1,
            serializer_type: if len == 20 { Fixed::H160 } else { Fixed::H256 },
            content: [0; 32],
        }
    }

    /// new_uint creates a new serializer for ethereum
    /// unsigned types. These are stored as u64 arrays with
    /// big endian byte order
    pub fn new_uint(len: usize) -> Self {
        if len != 32 {
            panic!("BasicEthSerializer only supports U256")
        }

        BasicEthSerializer {
            offset: 31,
            content: [0; 32],
            serializer_type: Fixed::U256,
            offset_sign: -1,
        }
    }

    pub fn serialize(&self) -> String {
        hex::encode(self.content)
    }
}

impl<'a> ser::Serializer for &'a mut BasicEthSerializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        match self.serializer_type {
            Fixed::U256 => panic!("received i8 when serializing U256"),
            Fixed::H256 | Fixed::H160 => {
                self.content[self.offset as usize] = value as u8;
                self.offset += self.offset_sign;
                Ok(())
            }
        }
    }

    fn serialize_i16(self, _value: i16) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_i32(self, _value: i32) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_i64(self, _value: i64) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        match self.serializer_type {
            Fixed::U256 => panic!("received u8 when serializing U256"),
            Fixed::H256 | Fixed::H160 => {
                self.content[self.offset as usize] = value;
                self.offset += self.offset_sign;
                Ok(())
            }
        }
    }

    fn serialize_u16(self, _value: u16) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        match self.serializer_type {
            Fixed::H256 | Fixed::H160 => panic!("received u32 when serializing H256,H160"),
            Fixed::U256 => {
                for byte_index in 0..4 {
                    let index = (self.offset + byte_index * self.offset_sign) as usize;
                    self.content[index] = ((value >> (8 * byte_index)) & 0x00ff) as u8;
                }
                self.offset += 4 * self.offset_sign;
                Ok(())
            }
        }
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        match self.serializer_type {
            Fixed::H256 | Fixed::H160 => panic!("received u64 when serializing H256,H160"),
            Fixed::U256 => {
                for byte_index in 0..8 {
                    let index = (self.offset + byte_index * self.offset_sign) as usize;
                    self.content[index] = ((value >> (8 * byte_index)) & 0x00ff) as u8;
                }
                self.offset += 8 * self.offset_sign;
                Ok(())
            }
        }
    }

    fn serialize_f32(self, __value: f32) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_f64(self, __value: f64) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_char(self, _value: char) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_str(self, _value: &str) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_some<T: ?Sized + ser::Serialize>(self, _value: &T) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::not_implemented())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::not_implemented())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::not_implemented())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::not_implemented())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::not_implemented())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::not_implemented())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeSeq for &'a mut BasicEthSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeTuple for &'a mut BasicEthSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut BasicEthSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut BasicEthSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeMap for &'a mut BasicEthSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + ser::Serialize>(&mut self, _key: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn serialize_value<T: ?Sized + ser::Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeStruct for &'a mut BasicEthSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut BasicEthSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}
