use serde::de;

use std::io::Read;

use super::{
    error::{Error, Result},
};

pub struct Deserializer<R> {
    read: R,
}

impl<R: Read> Deserializer<R> {
    pub fn new(read: R) -> Self {
        Deserializer {
            read: read,
        }
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_i8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_i16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_i32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_i64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_f32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_f64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    #[inline]
    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    /// Parses a `null` as a None, and any other values as a `Some(...)`.
    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    /// Parses a newtype struct as the underlying value.
    #[inline]
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(self, name: &str, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_tuple<V: de::Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_map<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    {
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
    ) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_identifier<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }

    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value>
    {
        Err(Error::not_implemented())
    }
}

pub fn from_value<T>(value: Value) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}

fn from_trait<'de, R, T>(read: R) -> Result<T>
where
    R: Read<'de>,
    T: de::Deserialize<'de>,
{
    let mut de = Deserializer::new(read);
    let value = try!(de::Deserialize::deserialize(&mut de));

    // Make sure the whole stream has been consumed.
    try!(de.end());
    Ok(value)
}

pub fn from_reader<R, T>(rdr: R) -> Result<T>
where
    R: io::Read,
    T: de::DeserializeOwned,
{
    from_trait(read::IoRead::new(rdr))
}

pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    from_trait(read::SliceRead::new(v))
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    from_trait(read::StrRead::new(s))
}


#[cfg(test)]
mod tests {

    use std::fmt::Debug;
    use serde::{ser, de};

    fn test_parse_ok<T>(tests: Vec<(&str, T)>)
    where
        T: Clone + Debug + PartialEq + ser::Serialize + de::DeserializeOwned,
    {
        for (s, value) in tests {
            let v: T = from_str(s).unwrap();
            assert_eq!(v, value.clone());

            let v: T = from_slice(s.as_bytes()).unwrap();
            assert_eq!(v, value.clone());

            // Make sure we can deserialize into a `Value`.
            let json_value: Value = from_str(s).unwrap();
            assert_eq!(json_value, to_value(&value).unwrap());

            // Make sure we can deserialize from a `&Value`.
            let v = T::deserialize(&json_value).unwrap();
            assert_eq!(v, value);

            // Make sure we can deserialize from a `Value`.
            let v: T = from_value(json_value.clone()).unwrap();
            assert_eq!(v, value);

            // Make sure we can round trip back to `Value`.
            let json_value2: Value = from_value(json_value.clone()).unwrap();
            assert_eq!(json_value2, json_value);

            // Make sure we can fully ignore.
            let twoline = s.to_owned() + "\n3735928559";
            let mut de = Deserializer::from_str(&twoline);
            IgnoredAny::deserialize(&mut de).unwrap();
            assert_eq!(0xDEAD_BEEF, u64::deserialize(&mut de).unwrap());

            // Make sure every prefix is an EOF error, except that a prefix of a
            // number may be a valid number.
            if !json_value.is_number() {
                for (i, _) in s.trim_end().char_indices() {
                    assert!(from_str::<Value>(&s[..i]).unwrap_err().is_eof());
                    assert!(from_str::<IgnoredAny>(&s[..i]).unwrap_err().is_eof());
                }
            }
        }
    }

    #[test]
    fn test_parse_bool() {
        
    }

}
