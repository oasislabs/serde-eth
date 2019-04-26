use serde::de;

use std::{
    collections::HashMap,
    error::Error as stdError,
    io::{Cursor, Read, Seek, SeekFrom},
    vec::Vec,
};

use super::{
    custom_de::EthFixedAccess,
    error::{Category, Error, Result, TupleHint},
    eth,
};

#[derive(Debug, Clone)]
pub enum BaseType {
    Static,
    Dynamic,
}

#[derive(Debug)]
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
        self.types.iter().any(|t| match t {
            BaseType::Dynamic => true,
            _ => false,
        })
    }
}

macro_rules! array_stack {
    ( $de:expr, $x:expr ) => {{
        if ($de.remaining_size as usize) < $x {
            return Err(Error::message(
                "deserializer does not have enough space to \
                 allocate a vector in the stack",
            ));
        }

        [0 as u8; $x]
    }};
}

macro_rules! vec_heap {
    ( $de:expr, $x:expr ) => {{
        if ($de.remaining_size as usize) < $x {
            return Err(Error::message(
                "deserializer does not have enough space to \
                 allocate a vector in the heap",
            ));
        }

        vec![0 as u8; $x]
    }};
}

pub struct Deserializer<'r, R> {
    /// Remaining number of bytes that will be read at most from the reader
    /// or allocated in memory. Useful to avoid overflows or attempts to
    /// have the deserializer to allocate too much memory
    remaining_size: u64,

    /// Counts the number of tuples seen by the deserializer. Because
    /// tuple deserialization is ambiguous, in case of a failed attempt
    /// to deserialize a tuple, the counter can be used as an identifier
    /// for the tuple to provide a hint on what the deserializer thinks,
    /// the tuple should be deserialized to.
    tuple_counter: u64,

    /// A deserializer for custom types if required.
    current_custom_deserializer: Option<eth::Fixed>,

    /// A reader seeker that has the content deserialized by the
    /// deserializer
    read: &'r mut RefReadSeek<R>,

    /// Hints that the Deserializer takes into consideration when
    /// deserializing tuples. It can be used to attempt to deserialize
    /// the same tuple either as a fixed array or a dynamically sized
    /// tuple
    tuple_hints: HashMap<u64, BaseType>,

    /// Keeps track of the current scope that is being deserialized.
    /// Every sequence, or dynamically sized tuple adds a scope, and it's
    /// treated as a stack
    scope: Vec<Scope>,
}

impl<'r, R: Read + Seek> Deserializer<'r, R> {
    pub fn new(read: &'r mut RefReadSeek<R>) -> Self {
        Deserializer::with_hints(read, HashMap::new())
    }

    pub fn with_hints(read: &'r mut RefReadSeek<R>, tuple_hints: HashMap<u64, BaseType>) -> Self {
        Deserializer {
            remaining_size: 1 << 24, // 16MB
            tuple_counter: 0,
            current_custom_deserializer: None,
            read,
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
        if self.remaining_size < bytes.len() as u64 {
            return Err(Error::message(
                "deserializer does not have remaining space to parse more data",
            ));
        }

        let mut total_read_bytes: usize = 0;

        while total_read_bytes < bytes.len() {
            let read_bytes = self.read.read(&mut bytes[total_read_bytes..])?;

            if read_bytes == 0 {
                break;
            }
            total_read_bytes += read_bytes;
        }

        if total_read_bytes == bytes.len() {
            self.remaining_size -= bytes.len() as u64;
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
        let mut read_data = vec_heap![self, (read_len as usize)];
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
        _offset: usize,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        // the cursor of the reader should already be at the beginning
        // of the tuple. Since a static tuple does not have offsets,
        // we can just take that address as offset
        let offset = self.seek(SeekFrom::Current(0))?;

        self.push_scope(Scope::new(offset as usize));
        let res = visitor.visit_seq(StaticTupleAccess::new(self, len as usize));
        let scope = self.scope.pop().unwrap();
        let _ = self.seek(SeekFrom::Current(scope.read_head as i64))?;

        match self.pop_scope() {
            Some(mut s) => {
                s.read_head += scope.read_head;
                self.push_scope(s);
            }
            None => {}
        }
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
        let scope = Scope::new(curr as usize);
        self.push_scope(scope);
        let res = visitor.visit_seq(DynamicTupleAccess::new(self, len as usize));
        let scope = self.scope.pop().unwrap();
        let _ = self.seek(SeekFrom::Current(scope.read_tail as i64))?;

        match self.pop_scope() {
            Some(mut s) => {
                s.read_head += scope.read_head;
                self.push_scope(s);
            }
            None => {}
        }

        res
    }

    fn read_custom_tuple<'de, V: de::Visitor<'de>>(
        &mut self,
        _offset: usize,
        _len: usize,
        t: eth::Fixed,
        visitor: V,
    ) -> Result<V::Value> {
        match self.pop_scope() {
            Some(mut scope) => {
                scope.types.push(BaseType::Static);
                scope.read_head += 64;
                self.push_scope(scope);
            }
            None => {}
        }

        let mut bytes = array_stack![self, 64];
        let _ = self.must_read(&mut bytes[..])?;
        let bytes = eth::decode_bytes(&bytes[..], 32)?;
        visitor.visit_seq(EthFixedAccess::new(bytes, t))
    }

    fn peek_uint(&mut self, size: usize) -> Result<u64> {
        let mut bytes = array_stack![self, 64];
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
        let mut bytes = array_stack![self, 64];
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
        let mut bytes = array_stack![self, 64];
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
        let mut bytes = array_stack![self, 64];
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
}

impl<'r, 'de, 'a, R: Read + Seek> de::Deserializer<'de> for &'a mut Deserializer<'r, R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
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

        let mut bytes = array_stack![self, 64];
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
        name: &str,
        visitor: V,
    ) -> Result<V::Value> {
        self.current_custom_deserializer = eth::Fixed::get(name);
        self.deserialize_tuple(1, visitor)
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

        if let Some(t) = self.current_custom_deserializer {
            self.current_custom_deserializer = None;
            return self.read_custom_tuple(base, len, t, visitor);
        }

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
        let new_offset = scope.offset + scope.read_head;

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
        let new_offset = scope.offset + scope.read_head;

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
        let new_offset = scope.offset + scope.read_head;

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

                let _ = read.seek(SeekFrom::Start(0))?;
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
    use crate::{error::Result, serde_tests};
    use serde::{de, ser};
    use std::{error::Error, fmt::Debug};

    fn test_parse_ok<T: Clone + Debug + PartialEq + ser::Serialize + de::DeserializeOwned>(
        tests: &[(T, &str)],
    ) {
        for (value, s) in tests {
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

    #[test]
    fn test_parse_string_error() {
        let tests = &[(
            "0000000000000000000000000000000000000000000000000000000000000020\
             000000000000000000000000000000000000000000000000000fffffffffffff",
            "deserializer does not have enough space to allocate a vector in the heap",
        )];

        test_parse_error::<String>(tests);
    }

    #[test]
    fn test_parse_h160() {
        test_parse_ok(&serde_tests::test_h160()[..]);
    }

    #[test]
    fn test_parse_address() {
        test_parse_ok(&serde_tests::test_address()[..]);
    }

    #[test]
    fn test_parse_h256() {
        test_parse_ok(&serde_tests::test_h256()[..]);
    }

    #[test]
    fn test_parse_u256() {
        test_parse_ok(&serde_tests::test_u256()[..]);
    }

    #[test]
    fn test_parse_bool() {
        test_parse_ok(&serde_tests::test_bool()[..]);
    }

    #[test]
    fn test_parse_u8() {
        test_parse_ok(&serde_tests::test_u8()[..]);
    }

    #[test]
    fn test_parse_i8() {
        test_parse_ok(&serde_tests::test_i8()[..]);
    }

    #[test]
    fn test_parse_u16() {
        test_parse_ok(&serde_tests::test_u16()[..]);
    }

    #[test]
    fn test_parse_i16() {
        test_parse_ok(&serde_tests::test_i16()[..]);
    }

    #[test]
    fn test_parse_u32() {
        test_parse_ok(&serde_tests::test_u32()[..]);
    }

    #[test]
    fn test_parse_i32() {
        test_parse_ok(&serde_tests::test_i32()[..]);
    }

    #[test]
    fn test_parse_u64() {
        test_parse_ok(&serde_tests::test_u64()[..]);
    }

    #[test]
    fn test_parse_i64() {
        test_parse_ok(&serde_tests::test_i64()[..]);
    }

    #[test]
    fn test_parse_char() {
        test_parse_ok(&serde_tests::test_char()[..]);
    }

    #[test]
    fn test_parse_string() {
        test_parse_ok(&serde_tests::test_string()[..]);
    }

    #[test]
    fn test_parse_option() {
        test_parse_ok(&serde_tests::test_option()[..]);
    }

    #[test]
    fn test_parse_unit() {
        test_parse_ok(&serde_tests::test_unit()[..]);
    }

    #[test]
    fn test_parse_tuple_mixed() {
        test_parse_ok(&serde_tests::test_tuple_mixed()[..]);
    }

    #[test]
    fn test_parse_tuple_string() {
        test_parse_ok(&serde_tests::test_tuple_string()[..]);
    }

    #[test]
    fn test_parse_tuple_u8() {
        test_parse_ok(&serde_tests::test_tuple_u8()[..]);
    }

    #[test]
    fn test_parse_seq_int() {
        test_parse_ok(&serde_tests::test_seq_int()[..]);
    }

    #[test]
    fn test_parse_str_seq() {
        test_parse_ok(&serde_tests::test_str_seq()[..]);
    }

    #[test]
    fn test_parse_multiseq() {
        test_parse_ok(&serde_tests::test_multiseq()[..]);
    }

    #[test]
    fn test_parse_simple_struct() {
        test_parse_ok(&serde_tests::test_simple_struct()[..]);
    }

    #[test]
    fn test_parse_complex_struct() {
        test_parse_ok(&serde_tests::test_complex_struct()[..]);
    }

    #[test]
    fn test_parse_composed_struct() {
        test_parse_ok(&serde_tests::test_composed_struct()[..]);
    }

    #[test]
    fn test_parse_string_composed_struct() {
        test_parse_ok(&serde_tests::test_string_composed_struct()[..]);
    }

    #[test]
    fn test_parse_reversed_composed_struct() {
        test_parse_ok(&serde_tests::test_reversed_composed_struct()[..]);
    }
}
