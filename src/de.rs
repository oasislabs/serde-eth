use serde::de;

use std::{
    collections::HashMap,
    error::Error as stdError,
    io::{Cursor, Read, Seek, SeekFrom},
    vec::Vec,
};

use super::{
    error::{Category, Error, Result, TupleHint},
    eth,
};

#[derive(Clone)]
pub enum BaseType {
    Static,
    Dynamic,
}

pub struct Scope {
    offset: usize,
    read_head: usize,
    read_tail: usize,
    types: Vec<BaseType>,
}

impl Scope {
    fn new(offset: usize) -> Self {
        Scope {
            offset: offset,
            read_head: 0,
            read_tail: 0,
            types: Vec::new(),
        }
    }

    fn has_dynamic_types(&self) -> bool {
        for t in &self.types {
            match t {
                BaseType::Dynamic => return true,
                _ => {}
            }
        }

        false
    }
}

pub struct Deserializer<'r, R> {
    tuple_counter: u64,
    read: &'r mut RefReadSeek<R>,
    tuple_hints: HashMap<u64, BaseType>,
    scope: Vec<Scope>,
}

impl<'r, R: Read + Seek> Deserializer<'r, R> {
    pub fn new(read: &'r mut RefReadSeek<R>) -> Self {
        Deserializer::with_hints(read, HashMap::new())
    }

    pub fn with_hints(read: &'r mut RefReadSeek<R>, tuple_hints: HashMap<u64, BaseType>) -> Self {
        Deserializer {
            tuple_counter: 0,
            read: read,
            tuple_hints: tuple_hints,
            scope: Vec::new(),
        }
    }

    pub fn push_scope(&mut self, scope: Scope) {
        self.scope.push(scope);
    }

    pub fn pop_scope(&mut self) -> Option<Scope> {
        self.scope.pop()
    }

    fn seek(&mut self, from: SeekFrom) -> Result<u64> {
        self.read.seek(from)
    }

    fn must_read(&mut self, bytes: &mut [u8]) -> Result<()> {
        let mut total_read_bytes: usize = 0;

        while total_read_bytes < bytes.len() {
            let read_bytes = self.read.read(&mut bytes[total_read_bytes..])?;

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

    fn read_char(&mut self) -> Result<char> {
        let bytes = self.read_byte_array()?;
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

    fn read_str(&mut self) -> Result<String> {
        let bytes = self.read_byte_array()?;
        match std::str::from_utf8(&bytes[..]) {
            Err(_) => Err(Error::parsing("parsed byte array cannot decode to a char")),
            Ok(s) => Ok(s.to_string()),
        }
    }

    fn read_byte_array(&mut self) -> Result<Vec<u8>> {
        let bytes_offset = self.read_uint_head(64)?;

        let offset = match self.pop_scope() {
            Some(mut scope) => {
                let offset = (bytes_offset << 1) + scope.offset as u64;
                scope.types.push(BaseType::Dynamic);
                self.push_scope(scope);
                offset
            }
            None => bytes_offset << 1,
        };

        let _ = self.seek(SeekFrom::Start(offset as u64))?;

        let len = self.read_uint_tail(64)?;
        let read_bytes = len << 1;

        // only read multiple of 64 bytes
        let base = (read_bytes >> 6) << 6;
        let remain = if read_bytes - base == 0 { 0 } else { 1 };
        let read_len = base + (remain << 6);
        let mut read_data = vec![0; read_len as usize];
        self.must_read(&mut read_data[..])?;

        // keep track of how much data is read for a particular scope
        match self.pop_scope() {
            Some(mut scope) => {
                scope.read_tail += 64 + read_len as usize;
                self.push_scope(scope);
            }
            None => {}
        };

        eth::decode_bytes(&read_data, len as usize)
    }

    fn read_static_size_tuple<'de, V: de::Visitor<'de>>(
        &mut self,
        offset: usize,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        let curr = self.seek(SeekFrom::Start(offset as u64))?;
        self.push_scope(Scope::new(curr as usize));
        let res = visitor.visit_seq(StaticTupleAccess::new(self, len as usize));
        let _ = self.scope.pop().unwrap();
        res
    }

    fn read_dynamic_size_tuple<'de, V: de::Visitor<'de>>(
        &mut self,
        offset: usize,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        let tuple_offset = self.read_uint_head(64)?;
        let offset = (tuple_offset << 1) + offset as u64;

        let curr = self.seek(SeekFrom::Start(offset as u64))?;
        self.push_scope(Scope::new(curr as usize));

        let res = visitor.visit_seq(DynamicTupleAccess::new(self, len as usize));
        let _ = self.scope.pop().unwrap();
        res
    }

    fn peek_uint(&mut self, size: usize) -> Result<u64> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        self.seek(SeekFrom::Current(-64))?;
        eth::decode_uint(&bytes, size)
    }

    fn deserialize_uint(&mut self, size: usize) -> Result<u64> {
        match self.pop_scope() {
            Some(mut scope) => {
                scope.types.push(BaseType::Static);
                self.push_scope(scope);
            }
            None => {}
        }

        return self.read_uint_head(size);
    }

    fn deserialize_int(&mut self, size: usize) -> Result<i64> {
        match self.pop_scope() {
            Some(mut scope) => {
                scope.types.push(BaseType::Static);
                self.push_scope(scope);
            }
            None => {}
        }

        return self.read_int_head(size);
    }

    fn read_uint_head(&mut self, size: usize) -> Result<u64> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        match self.pop_scope() {
            Some(mut scope) => {
                scope.read_head += 64;
                self.push_scope(scope);
            }
            None => {}
        };
        eth::decode_uint(&bytes, size)
    }

    fn read_uint_tail(&mut self, size: usize) -> Result<u64> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        match self.pop_scope() {
            Some(mut scope) => {
                scope.read_tail += 64;
                self.push_scope(scope);
            }
            None => {}
        };
        eth::decode_uint(&bytes, size)
    }

    fn read_int_head(&mut self, size: usize) -> Result<i64> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        match self.pop_scope() {
            Some(mut scope) => {
                scope.read_head += 64;
                self.push_scope(scope);
            }
            None => {}
        };
        eth::decode_int(&bytes, size)
    }

    fn read_int_tail(&mut self, size: usize) -> Result<i64> {
        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        match self.pop_scope() {
            Some(mut scope) => {
                scope.read_tail += 64;
                self.push_scope(scope);
            }
            None => {}
        };
        eth::decode_int(&bytes, size)
    }
}

impl<'r, 'de, 'a, R: Read + Seek> de::Deserializer<'de> for &'a mut Deserializer<'r, R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.pop_scope() {
            Some(mut scope) => {
                scope.types.push(BaseType::Static);
                self.push_scope(scope);
            }
            None => {}
        }

        let mut bytes = [0; 64];
        self.must_read(&mut bytes)?;
        let value = eth::decode_bool(&bytes)?;
        visitor.visit_bool(value)
    }

    fn deserialize_i8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.deserialize_int(8)?;
        visitor.visit_i8(value as i8)
    }

    fn deserialize_i16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.deserialize_int(16)?;
        visitor.visit_i16(value as i16)
    }

    fn deserialize_i32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.deserialize_int(32)?;
        visitor.visit_i32(value as i32)
    }

    fn deserialize_i64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.deserialize_int(64)?;
        visitor.visit_i64(value as i64)
    }

    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.deserialize_uint(8)?;
        visitor.visit_u8(value as u8)
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.deserialize_uint(16)?;
        visitor.visit_u16(value as u16)
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.deserialize_uint(32)?;
        visitor.visit_u32(value as u32)
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value = self.deserialize_uint(64)?;
        visitor.visit_u64(value as u64)
    }

    fn deserialize_f32<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_f64<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let c = self.read_char()?;
        visitor.visit_char(c)
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.read_str()?;
        visitor.visit_str(&s)
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.read_str()?;
        visitor.visit_string(s)
    }

    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let b = self.read_byte_array()?;
        visitor.visit_bytes(&b[..])
    }

    #[inline]
    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let b = self.read_byte_array()?;
        visitor.visit_byte_buf(b)
    }

    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        // we expect an option to be serialized as a dynamic sized array that can either
        // have 1 element or 0.
        let base = match self.pop_scope() {
            Some(mut scope) => {
                let offset = scope.offset;
                scope.types.push(BaseType::Dynamic);
                self.push_scope(scope);
                offset
            }
            None => 0,
        };

        let seq_offset = self.read_uint_head(64)?;
        let offset = (seq_offset << 1) + base as u64;

        let curr = self.seek(SeekFrom::Start(offset as u64))?;

        let len = self.read_uint_tail(64)?;
        if len == 0 {
            return visitor.visit_none();
        }

        self.push_scope(Scope::new(64 + curr as usize));

        let res = visitor.visit_some(&mut *self);
        let _ = self.scope.pop().unwrap();
        res
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let base = match self.pop_scope() {
            Some(mut scope) => {
                let offset = scope.offset;
                scope.types.push(BaseType::Dynamic);
                self.push_scope(scope);
                offset
            }
            None => 0,
        };

        let seq_offset = self.read_uint_head(64)?;
        let offset = (seq_offset << 1) + base as u64;

        let curr = self.seek(SeekFrom::Start(offset as u64))?;

        let len = self.read_uint_tail(64)?;
        self.push_scope(Scope::new(64 + curr as usize));

        let res = visitor.visit_seq(SeqAccess::new(self, len as usize));
        let _ = self.scope.pop().unwrap();
        res
    }

    fn deserialize_tuple<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        self.tuple_counter += 1;

        let base = match self.pop_scope() {
            Some(mut scope) => {
                let offset = scope.offset;
                scope.types.push(BaseType::Dynamic);
                self.push_scope(scope);
                offset
            }
            None => 0,
        };

        // for tuples the deserialization is ambiguous. If the user has passed
        // a hint, used the hint to deserialize the tuple with that index
        let hint = self.tuple_hints.get(&self.tuple_counter);

        match hint {
            Some(h) => match h {
                BaseType::Static => self.read_static_size_tuple(base, len, visitor),
                BaseType::Dynamic => self.read_dynamic_size_tuple(base, len, visitor),
            },
            None => {
                // in case there's no hint, the assumption is the following:
                // if the first integer in the tuple is multiple of 32 it could be an offset,
                // in which case it would be a dynamic sized tuple. In case it is not multiple
                // of 32, for sure it is not an offset, in which case can be safely
                // deserialized as a static sized tuple.
                let tuple_offset = self.peek_uint(64)?;

                if tuple_offset % 32 == 0 {
                    // This is just a guess, it can be that this fails, in which case
                    // an error with TupleHint will be raised so that the deserialization
                    // can be attempted again
                    self.read_dynamic_size_tuple(base, len, visitor)
                } else {
                    self.read_static_size_tuple(base, len, visitor)
                }
            }
        }
    }

    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        Err(Error::not_implemented())
    }

    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_tuple(_fields.len(), visitor)
    }

    #[inline]
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(EnumAccess::new(self))
    }

    fn deserialize_identifier<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }
}

struct SeqAccess<'r, 'a, R> {
    len: usize,
    count: usize,
    de: &'a mut Deserializer<'r, R>,
}

impl<'r, 'a, R> SeqAccess<'r, 'a, R> {
    fn new(de: &'a mut Deserializer<'r, R>, len: usize) -> Self {
        SeqAccess {
            len: len,
            count: 0,
            de: de,
        }
    }
}

impl<'de, 'r, 'a, R: Read + Seek + 'r> de::SeqAccess<'de> for SeqAccess<'r, 'a, R> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        self.count += 1;
        if self.count > self.len {
            return Ok(None);
        }

        let res = seed.deserialize(&mut *self.de);

        let scope = self.de.pop_scope().unwrap();
        let new_offset = scope.offset + 64 * self.count;

        if self.count < self.len {
            // if there are still elements in the sequence, seek back to the next
            // items's head
            let _ = self.de.seek(SeekFrom::Start(new_offset as u64))?;
        }

        self.de.push_scope(scope);

        match res {
            Err(err) => Err(err),
            Ok(value) => Ok(Some(value)),
        }
    }
}

struct EnumAccess<'r, 'a, R> {
    de: &'a mut Deserializer<'r, R>,
}

impl<'r, 'a, R> EnumAccess<'r, 'a, R> {
    fn new(de: &'a mut Deserializer<'r, R>) -> Self {
        EnumAccess { de: de }
    }
}

impl<'de, 'r, 'a, R: Read + Seek + 'r> de::EnumAccess<'de> for EnumAccess<'r, 'a, R> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'r, 'a, R: Read + Seek + 'r> de::VariantAccess<'de> for EnumAccess<'r, 'a, R> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        de::Deserialize::deserialize(self.de)
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V: de::Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        de::Deserializer::deserialize_struct(self.de, "", fields, visitor)
    }
}

struct StaticTupleAccess<'r, 'a, R: 'r> {
    len: usize,
    count: usize,
    de: &'a mut Deserializer<'r, R>,
}

impl<'r, 'a, R> StaticTupleAccess<'r, 'a, R> {
    fn new(de: &'a mut Deserializer<'r, R>, len: usize) -> Self {
        StaticTupleAccess {
            len: len,
            count: 0,
            de: de,
        }
    }
}

impl<'de, 'r, 'a, R: Read + Seek + 'r> de::SeqAccess<'de> for StaticTupleAccess<'r, 'a, R> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        self.count += 1;
        if self.count > self.len {
            return Ok(None);
        }

        let res = seed.deserialize(&mut *self.de);

        let scope = self.de.pop_scope().unwrap();
        let new_offset = scope.offset + 64 * self.count;

        if self.count < self.len {
            // if there are still elements in the sequence, seek back to the next
            // items's head
            let _ = self.de.seek(SeekFrom::Start(new_offset as u64))?;
        }

        self.de.push_scope(scope);

        match res {
            Err(err) => Err(err),
            Ok(value) => Ok(Some(value)),
        }
    }
}

struct DynamicTupleAccess<'r, 'a, R: 'r> {
    len: usize,
    count: usize,
    de: &'a mut Deserializer<'r, R>,
}

impl<'r, 'a, R: Read + Seek + 'r> DynamicTupleAccess<'r, 'a, R> {
    fn new(de: &'a mut Deserializer<'r, R>, len: usize) -> Self {
        DynamicTupleAccess {
            len: len,
            count: 0,
            de: de,
        }
    }

    fn get_error(&mut self, error: Error) -> Error {
        match error.classify() {
            Category::Data => {
                let has_dynamic_types = match self.de.pop_scope() {
                    Some(scope) => {
                        let has_dynamic_types = scope.has_dynamic_types();
                        self.de.push_scope(scope);
                        has_dynamic_types
                    }
                    None => false,
                };

                let should_give_hint = error.description() == "insufficient bytes read from reader"
                    && !has_dynamic_types;

                if should_give_hint {
                    Error::hint(TupleHint::new(self.de.tuple_counter, false), error)
                } else {
                    error
                }
            }
            _ => error,
        }
    }
}

impl<'de, 'r, 'a, R: Read + Seek + 'a> de::SeqAccess<'de> for DynamicTupleAccess<'r, 'a, R> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        self.count += 1;
        if self.count > self.len {
            return Ok(None);
        }

        let res = seed.deserialize(&mut *self.de);
        match res {
            Ok(_) => {}
            Err(error) => return Err(self.get_error(error)),
        }

        let scope = self.de.pop_scope().unwrap();
        let new_offset = scope.offset + 64 * self.count;
        if self.count < self.len {
            // if there are still elements in the sequence, seek back to the next
            // items's head
            let _ = self.de.seek(SeekFrom::Start(new_offset as u64))?;
        }

        self.de.push_scope(scope);

        match res {
            Err(err) => Err(err),
            Ok(value) => Ok(Some(value)),
        }
    }
}

pub struct RefReadSeek<R> {
    read: R,
}

impl<R: Read + Seek> RefReadSeek<R> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize> {
        self.read.read(bytes).map_err(Error::io)
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<u64> {
        self.read.seek(offset).map_err(Error::io)
    }
}

pub fn from_reader<'de, R: Read + Seek, T: de::Deserialize<'de>>(read: R) -> Result<T> {
    let mut hints = HashMap::new();
    let mut read = RefReadSeek { read: read };

    loop {
        let mut de = Deserializer::with_hints(&mut read, hints.clone());
        let res = de::Deserialize::deserialize(&mut de);
        match res {
            Err(err) => {
                let hint = err.tuple_hint();
                if hint.is_none() {
                    return Err(err);
                }

                let hint = hint.unwrap();
                if hints.contains_key(&hint.index) {
                    return Err(err);
                }

                hints.insert(
                    hint.index,
                    if hint.is_dynamic {
                        BaseType::Dynamic
                    } else {
                        BaseType::Static
                    },
                );
                continue;
            }
            _ => {
                de.end()?;
                return res;
            }
        }
    }
}

pub fn from_str<'a, T: de::Deserialize<'a>>(s: &'a str) -> Result<T> {
    from_reader(Cursor::new(s))
}

#[cfg(test)]
mod tests {

    use super::from_str;
    use crate::error::Result;
    use serde::{de, ser, Deserialize, Serialize};
    use std::{error::Error, fmt::Debug};

    fn test_parse_ok<T: Clone + Debug + PartialEq + ser::Serialize + de::DeserializeOwned>(
        tests: &[(&str, T)],
    ) {
        for (s, value) in tests {
            let v: T = from_str(s).unwrap();
            assert_eq!(v, value.clone());
        }
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct Simple {
        value1: String,
        value2: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct Complex {
        value: String,
        simple: Simple,
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

    #[test]
    fn test_parse_unit() {
        let tests = &[("", ())];
        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_option() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000000",
                None,
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000005\
                 68656c6c6f000000000000000000000000000000000000000000000000000000",
                Some("hello".to_string()),
            ),
        ];

        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_tuple_int() {
        let tests = &[(
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000001\
             3100000000000000000000000000000000000000000000000000000000000000",
            (1, "1".to_string()),
        )];

        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_tuple_string() {
        let tests = &[(
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000080\
             0000000000000000000000000000000000000000000000000000000000000001\
             3100000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000001\
             3200000000000000000000000000000000000000000000000000000000000000",
            ("1".to_string(), "2".to_string()),
        )];

        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_tuple_with_32int() {
        let tests = &[(
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000020",
            [0x20 as u8; 3],
        )];

        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_int_seq() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000003\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 0000000000000000000000000000000000000000000000000000000000000002\
                 0000000000000000000000000000000000000000000000000000000000000003",
                vec![1, 2, 3],
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000000",
                vec![],
            ),
        ];

        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_u8_fixed_seq() {
        let tests = &[(
            "0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000001",
            [1 as u8; 3],
        )];

        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_str_seq() {
        let tests = &[
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000003\
                 0000000000000000000000000000000000000000000000000000000000000060\
                 00000000000000000000000000000000000000000000000000000000000000a0\
                 00000000000000000000000000000000000000000000000000000000000000e0\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 3100000000000000000000000000000000000000000000000000000000000000\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 3200000000000000000000000000000000000000000000000000000000000000\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 3300000000000000000000000000000000000000000000000000000000000000",
                vec!["1".to_string(), "2".to_string(), "3".to_string()],
            ),
            (
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000000",
                vec![],
            ),
        ];
        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_multiseq() {
        let tests = &[(
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000120\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000080\
             0000000000000000000000000000000000000000000000000000000000000001\
             3100000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000001\
             3200000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000080\
             0000000000000000000000000000000000000000000000000000000000000001\
             3300000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000001\
             3400000000000000000000000000000000000000000000000000000000000000",
            vec![
                vec!["1".to_string(), "2".to_string()],
                vec!["3".to_string(), "4".to_string()],
            ],
        )];

        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_simple_struct() {
        let tests = &[(
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000080\
             0000000000000000000000000000000000000000000000000000000000000001\
             3100000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000001\
             3200000000000000000000000000000000000000000000000000000000000000",
            Simple {
                value1: "1".to_string(),
                value2: "2".to_string(),
            },
        )];
        test_parse_ok(tests);
    }

    #[test]
    fn test_parse_complex_struct() {
        let tests = &[(
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000080\
             0000000000000000000000000000000000000000000000000000000000000001\
             3100000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000080\
             0000000000000000000000000000000000000000000000000000000000000001\
             3200000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000001\
             3300000000000000000000000000000000000000000000000000000000000000",
            Complex {
                value: "1".to_string(),
                simple: Simple {
                    value1: "2".to_string(),
                    value2: "3".to_string(),
                },
            },
        )];

        test_parse_ok(tests);
    }

}
