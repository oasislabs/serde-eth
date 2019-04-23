use serde::ser;

use super::error::{Error, Result};

#[derive(Debug)]
pub enum SerializerType {
    H256,
    H160,
    U256,
}

impl SerializerType {
    pub fn get(name: &str) -> Option<SerializerType> {
        match name {
            "H256" => Some(SerializerType::H256),
            "H160" => Some(SerializerType::H160),
            "U256" => Some(SerializerType::U256),
            _ => None,
        }
    }
}

pub struct HashSerializer {
    /// offset keeps track of the current position
    offset: i8,

    /// offset_sign is the sign to move the offset forward.
    /// Use -1 for big endian arrays and 1 for little endian arrays
    offset_sign: i8,

    /// content that the serializer aggregates. Call `serialize` to
    /// have the hex serialization of the data
    content: [u8; 32],
}

impl HashSerializer {
    /// new_hash creates a new serializer for ethereum
    /// hash types. These are stored as u8 arrays with
    /// little endian byte order
    pub fn new_hash(len: usize) -> Self {
        if len != 20 && len != 32 {
            panic!("HashSerializer only supports H160, H256")
        }

        HashSerializer {
            offset: 32 - (len as i8),
            content: [0; 32],
            offset_sign: 1,
        }
    }

    /// new_uint creates a new serializer for ethereum
    /// unsigned types. These are stored as u64 arrays with
    /// big endian byte order
    pub fn new_uint(len: usize) -> Self {
        if len != 32 {
            panic!("HashSerializer only supports U256")
        }

        HashSerializer {
            offset: 31,
            content: [0; 32],
            offset_sign: -1,
        }
    }

    pub fn serialize(&self) -> String {
        hex::encode(self.content)
    }
}

impl<'a> ser::Serializer for &'a mut HashSerializer {
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

    fn serialize_i8(self, _value: i8) -> Result<Self::Ok> {
        Err(Error::not_implemented())
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
        self.content[self.offset as usize] = value;
        self.offset += self.offset_sign;
        Ok(())
    }

    fn serialize_u16(self, _value: u16) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_u32(self, _value: u32) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        self.content[self.offset as usize] = (value & 0x00ff) as u8;
        self.content[(self.offset + self.offset_sign) as usize] = ((value >> 8) & 0x00ff) as u8;
        self.content[(self.offset + 2 * self.offset_sign) as usize] =
            ((value >> 16) & 0x00ff) as u8;
        self.content[(self.offset + 3 * self.offset_sign) as usize] =
            ((value >> 24) & 0x00ff) as u8;
        self.content[(self.offset + 4 * self.offset_sign) as usize] =
            ((value >> 32) & 0x00ff) as u8;
        self.content[(self.offset + 5 * self.offset_sign) as usize] =
            ((value >> 40) & 0x00ff) as u8;
        self.content[(self.offset + 6 * self.offset_sign) as usize] =
            ((value >> 48) & 0x00ff) as u8;
        self.content[(self.offset + 7 * self.offset_sign) as usize] =
            ((value >> 56) & 0x00ff) as u8;
        self.offset += 8 * self.offset_sign;
        Ok(())
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

impl<'a> ser::SerializeSeq for &'a mut HashSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeTuple for &'a mut HashSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut HashSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut HashSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, _value: &T) -> Result<()> {
        Err(Error::not_implemented())
    }

    fn end(self) -> Result<()> {
        Err(Error::not_implemented())
    }
}

impl<'a> ser::SerializeMap for &'a mut HashSerializer {
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

impl<'a> ser::SerializeStruct for &'a mut HashSerializer {
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

impl<'a> ser::SerializeStructVariant for &'a mut HashSerializer {
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
