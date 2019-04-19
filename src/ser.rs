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

    type SerializeSeq = RootCompound<'a, W>;
    type SerializeTuple = RootCompound<'a, W>;
    type SerializeTupleStruct = RootCompound<'a, W>;
    type SerializeTupleVariant = RootCompound<'a, W>;
    type SerializeMap = RootCompound<'a, W>;
    type SerializeStruct = RootCompound<'a, W>;
    type SerializeStructVariant = RootCompound<'a, W>;

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

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.serialize_bytes(&value.to_string().into_bytes())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        let encoded = eth::encode_bytes(value);
        self
            .writer
            .write_all(&encoded.into_bytes())
            .map_err(Error::io)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(
        self,
        _: &'static str
    ) -> Result<Self::Ok>
    {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str
    ) -> Result<Self::Ok>
    {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _: &'static str,
        value: &T
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        value: &T
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(
        self,
        len: Option<usize>
    ) -> Result<Self::SerializeSeq>
    {
        let ser = match len {
            Some(l) => NodeSerializer::new_with_len(l),
            None => NodeSerializer::new(),
        };

        Ok(RootCompound{
            writer: &mut self.writer,
            ser: ser,
        })
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

enum Node {
    // Fixed sized types do not have headers we only need to
    // keep track of its value
    Fixed(String),

    // Dynamic sized types have an offset, size and content
    Dynamic(String),

    // A compound type will not have specific content
    // it will be generated by its children
    Compound(Vec<Node>),
}

impl Node {
    fn serialize_simple(head: &str, tail: &str) -> String {
        let mut result = String::with_capacity(head.len() + tail.len());
        result.push_str(head);
        result.push_str(tail);
        result
    }

    fn serialize_nodes(nodes: Vec<Node>) -> String {
        let result = Node::serialize_compound_to_simple(nodes);
        result.serialize()
    }

    fn aggregate_simple_nodes(nodes: Vec<Node>) -> Node {
        let mut head = String::from(eth::encode_u64(nodes.len() as u64));
        let mut tail = String::from("");
        // The initial offset for the content of the generated node
        // will be the header generated by the nodes.
        let mut offset = nodes.len() << 5;

        for node in nodes {
            match node {
                Node::Fixed(h) => head.push_str(&h),
                Node::Dynamic(t) => {
                    head.push_str(&eth::encode_u64(offset as u64));
                    tail.push_str(&t);
                    offset += t.len() >> 1;
                },
                Node::Compound(_) => unreachable!(),
            }
        }

        Node::Dynamic(Node::serialize_simple(&head, &tail))
    }

    fn serialize_compound_to_simple(nodes: Vec<Node>) -> Node {
        let mut simple_nodes = Vec::with_capacity(nodes.len());

        for node in nodes {
            let simple_node = Node::serialize_node_to_simple(node);
            simple_nodes.push(simple_node);
        }

        Node::aggregate_simple_nodes(simple_nodes)
    }

    fn serialize_dynamic(content: String) -> String {
        let offset = 32;
        let mut new_content = eth::encode_u64(offset as u64);
        new_content.push_str(&content);
        new_content
    }

    fn serialize_node_to_simple(node: Node) -> Node {
        if let Node::Compound(vec) = node {
            Node::serialize_compound_to_simple(vec)

        } else {
            node
        }
    }

    fn serialize(self) -> String {
        match self {
            Node::Fixed(head) => Node::serialize_simple(&head, ""),
            Node::Dynamic(content) => Node::serialize_dynamic(content),
            Node::Compound(vec) => Node::serialize_nodes(vec),
        }
    }

    fn push(&mut self, node: Node) {
        match self {
            Node::Fixed(_) => panic!("attempt to push node to Node::Fixed"),
            Node::Dynamic(_) => panic!("attempt to push node to Node::Dynamic"),
            Node::Compound(children) => children.push(node),
        }
    }
}

struct NodeCompound<'a> {
    base: &'a mut NodeSerializer,
    ser: NodeSerializer,
}

impl<'a> ser::SerializeSeq for NodeCompound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut self.ser)
    }

    fn end(self) -> Result<()> {
        let node = self.ser.into_inner();
        self.base.push(node);
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for NodeCompound<'a> {
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

impl<'a> ser::SerializeTupleStruct for NodeCompound<'a> {
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

impl<'a> ser::SerializeTupleVariant for NodeCompound<'a> {
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

impl<'a> ser::SerializeMap for NodeCompound<'a> {
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

impl<'a> ser::SerializeStruct for NodeCompound<'a> {
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

impl<'a> ser::SerializeStructVariant for NodeCompound<'a> {
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

struct NodeSerializer {
    root: Node,
}

impl NodeSerializer {
    #[inline]
    fn new() -> Self {
        NodeSerializer{ root: Node::Compound(Vec::new()) }
    }

    #[inline]
    fn new_with_len(len: usize) -> Self {
        NodeSerializer{ root: Node::Compound(Vec::with_capacity(len)) }
    }

    #[inline]
    fn into_inner(self) -> Node {
        self.root
    }
}

impl NodeSerializer {
    #[inline]
    fn push(&mut self, node: Node) {
        self.root.push(node)
    }
}

impl<'a> ser::Serializer for &'a mut NodeSerializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = NodeCompound<'a>;
    type SerializeTuple = NodeCompound<'a>;
    type SerializeTupleStruct = NodeCompound<'a>;
    type SerializeTupleVariant = NodeCompound<'a>;
    type SerializeMap = NodeCompound<'a>;
    type SerializeStruct = NodeCompound<'a>;
    type SerializeStructVariant = NodeCompound<'a>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        let encoded = eth::encode_bool(value);
        self.root.push(Node::Fixed(encoded));
        Ok(())
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
        self.root.push(Node::Fixed(encoded));
        Ok(())
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
        self.root.push(Node::Fixed(encoded));
        Ok(())
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok> {
        Err(Error::not_implemented())
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.serialize_bytes(&value.to_string().into_bytes())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        let encoded = eth::encode_bytes_dynamic(value);
        let mut content = String::with_capacity(encoded.size().len() + encoded.content().len());
        content.push_str(encoded.size());
        content.push_str(encoded.content());
        let node = Node::Dynamic(content);
        self.root.push(node);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(
        self,
        _: &'static str
    ) -> Result<Self::Ok>
    {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str
    ) -> Result<Self::Ok>
    {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _: &'static str,
        value: &T
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        value: &T
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(
        self,
        len: Option<usize>
    ) -> Result<Self::SerializeSeq>
    {
        let ser = match len {
            Some(l) => NodeSerializer::new_with_len(l),
            None => NodeSerializer::new(),
        };

        Ok(NodeCompound{
            base: self,
            ser: ser,
        })
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

pub struct RootCompound<'a, W: 'a> {
    writer: &'a mut W,
    ser: NodeSerializer,
}

impl<'a, W> ser::SerializeSeq for RootCompound<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(&mut self.ser)
    }

    fn end(self) -> Result<()> {
        let node = self.ser.into_inner();
        let encoded = node.serialize();
        self
            .writer
            .write_all(&encoded.into_bytes())
            .map_err(Error::io)
    }
}

impl<'a, W> ser::SerializeTuple for RootCompound<'a, W>
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

impl<'a, W> ser::SerializeTupleStruct for RootCompound<'a, W>
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

impl<'a, W> ser::SerializeTupleVariant for RootCompound<'a, W>
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

impl<'a, W> ser::SerializeMap for RootCompound<'a, W>
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

impl<'a, W> ser::SerializeStruct for RootCompound<'a, W>
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

impl<'a, W> ser::SerializeStructVariant for RootCompound<'a, W>
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

    #[test]
    fn test_write_u32() {
        let tests = &[
            (0x00000000 as u32, "0000000000000000000000000000000000000000000000000000000000000000"),
            (0x01000000 as u32, "0000000000000000000000000000000000000000000000000000000001000000"),
            (0x10000000 as u32, "0000000000000000000000000000000000000000000000000000000010000000"),
            (0x80000000 as u32, "0000000000000000000000000000000000000000000000000000000080000000"),
            (0xffffffff as u32, "00000000000000000000000000000000000000000000000000000000ffffffff"),
        ];
        test_encode_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_write_i32() {
        let tests = &[
            (0x00000000 as i32, "0000000000000000000000000000000000000000000000000000000000000000"),
            (0x01000000 as i32, "0000000000000000000000000000000000000000000000000000000001000000"),
            (0x10000000 as i32, "0000000000000000000000000000000000000000000000000000000010000000"),
            (0x80000000 as i32, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffff80000000"),
            (0xffffffff as i32, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_u64() {
        let tests = &[
            (0x0000000000000000 as u64, "0000000000000000000000000000000000000000000000000000000000000000"),
            (0x0100000000000000 as u64, "0000000000000000000000000000000000000000000000000100000000000000"),
            (0x1000000000000000 as u64, "0000000000000000000000000000000000000000000000001000000000000000"),
            (0x8000000000000000 as u64, "0000000000000000000000000000000000000000000000008000000000000000"),
            (0xffffffffffffffff as u64, "000000000000000000000000000000000000000000000000ffffffffffffffff"),
        ];
        test_encode_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_write_i64() {
        let tests = &[
            (0x0000000000000000 as i64, "0000000000000000000000000000000000000000000000000000000000000000"),
            (0x0100000000000000 as i64, "0000000000000000000000000000000000000000000000000100000000000000"),
            (0x1000000000000000 as i64, "0000000000000000000000000000000000000000000000001000000000000000"),
            (0x8000000000000000 as i64, "ffffffffffffffffffffffffffffffffffffffffffffffff8000000000000000"),
            (0xffffffffffffffff as i64, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_char() {
        let tests = &[
            ('a', "0000000000000000000000000000000000000000000000000000000000000020\
                   0000000000000000000000000000000000000000000000000000000000000001\
                   6100000000000000000000000000000000000000000000000000000000000000"),
            ('é', "0000000000000000000000000000000000000000000000000000000000000020\
                   0000000000000000000000000000000000000000000000000000000000000002\
                   c3a9000000000000000000000000000000000000000000000000000000000000"),
            ('ø', "0000000000000000000000000000000000000000000000000000000000000020\
                   0000000000000000000000000000000000000000000000000000000000000002\
                   c3b8000000000000000000000000000000000000000000000000000000000000"),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_string() {
        let tests = &[
            ("hello", "0000000000000000000000000000000000000000000000000000000000000020\
                       0000000000000000000000000000000000000000000000000000000000000005\
                       68656c6c6f000000000000000000000000000000000000000000000000000000"),
            ("",      "0000000000000000000000000000000000000000000000000000000000000020\
                       0000000000000000000000000000000000000000000000000000000000000000"),
            ("some long string that takes more than 32 bytes so we can see how eth abi \
              encodes long strings",
                      "0000000000000000000000000000000000000000000000000000000000000020\
                       000000000000000000000000000000000000000000000000000000000000005d\
                       736f6d65206c6f6e6720737472696e6720746861742074616b6573206d6f7265\
                       207468616e20333220627974657320736f2077652063616e2073656520686f77\
                       206574682061626920656e636f646573206c6f6e6720737472696e6773000000"),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_option() {
        let tests = &[
            (None, ""),
            (Some("hello"),  "0000000000000000000000000000000000000000000000000000000000000020\
                              0000000000000000000000000000000000000000000000000000000000000005\
                              68656c6c6f000000000000000000000000000000000000000000000000000000"),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_unit() {
        let tests = &[
            ((), ""),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_int_seq() {
        let tests = &[
            (vec![1, 2, 3], "0000000000000000000000000000000000000000000000000000000000000020\
                             0000000000000000000000000000000000000000000000000000000000000003\
                             0000000000000000000000000000000000000000000000000000000000000001\
                             0000000000000000000000000000000000000000000000000000000000000002\
                             0000000000000000000000000000000000000000000000000000000000000003"),
            (vec![],        "0000000000000000000000000000000000000000000000000000000000000020\
                             0000000000000000000000000000000000000000000000000000000000000000"),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_str_seq() {
        let tests = &[
            (vec!["1", "2", "3"],
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
                              3300000000000000000000000000000000000000000000000000000000000000"),
            (vec![],        "0000000000000000000000000000000000000000000000000000000000000020\
                             0000000000000000000000000000000000000000000000000000000000000000"),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_multiseq() {
        let tests = &[
            (vec![vec!["1", "2"], vec!["3", "4"]],
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
                             3400000000000000000000000000000000000000000000000000000000000000"),
            ];
        test_encode_ok(tests);
    }

}
