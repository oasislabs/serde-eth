use std::io;

use serde::ser::{self, SerializeSeq};

use super::{
    custom_ser,
    error::{Error, Result},
    eth,
};

pub struct Serializer<W> {
    writer: W,

    // current_custom_type is used to set the current state of any type whose serialization
    // is implemented in the serializer.
    current_custom_serializer: Option<eth::Fixed>,
}

impl<W: io::Write> Serializer<W> {
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer {
            writer: writer,
            current_custom_serializer: None,
        }
    }

    #[inline]
    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.current_custom_serializer = None;
        self.writer.write_all(bytes).map_err(Error::io)
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<'a, W: io::Write> ser::Serializer for &'a mut Serializer<W> {
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
        self.write(&encoded.into_bytes())
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
        self.write(&encoded.into_bytes())
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
        self.write(&encoded.into_bytes())
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
        self.writer
            .write_all(&encoded.into_bytes())
            .map_err(Error::io)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        let serializer = self.serialize_seq(Some(0))?;
        serializer.end()
    }

    fn serialize_some<T: ser::Serialize + ?Sized>(self, value: &T) -> Result<Self::Ok> {
        let mut serializer = self.serialize_seq(Some(1))?;
        serializer.serialize_element(value)?;
        serializer.end()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        self.current_custom_serializer = eth::Fixed::get(name);
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        self.current_custom_serializer = eth::Fixed::get(name);
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let root = match len {
            Some(l) => Node::Seq(Vec::with_capacity(l)),
            None => Node::Seq(Vec::new()),
        };

        Ok(RootCompound::Standard {
            writer: self,
            ser: NodeSerializer::new(root),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        let custom_serializer = self.current_custom_serializer;
        self.current_custom_serializer = None;

        match custom_serializer {
            Some(t) => match t {
                eth::Fixed::H256 => Ok(RootCompound::BigInteger {
                    writer: self,
                    ser: custom_ser::BasicEthSerializer::new_hash(32),
                }),
                eth::Fixed::H160 => Ok(RootCompound::BigInteger {
                    writer: self,
                    ser: custom_ser::BasicEthSerializer::new_hash(20),
                }),
                eth::Fixed::U256 => Ok(RootCompound::BigInteger {
                    writer: self,
                    ser: custom_ser::BasicEthSerializer::new_uint(32),
                }),
            },
            None => {
                let root = Node::Tuple(Vec::with_capacity(len));
                Ok(RootCompound::Standard {
                    writer: self,
                    ser: NodeSerializer::new(root),
                })
            }
        }
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_tuple(len)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::not_implemented())
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.serialize_tuple(len)
    }
}

enum Node {
    // Fixed sized types do not have headers we only need to
    // keep track of its value
    Fixed(String),

    // Dynamic sized types have an offset, size and content
    Dynamic(String),

    // A compound type will not have specific content
    // it will be generated by its children. It adds an
    // structure to the sequence by writing the offset and
    // the size of the vector when serialized
    Seq(Vec<Node>),

    // A compound type will not have specific content
    // it will be generated by its children. It does not
    // write structe information when serialized
    Tuple(Vec<Node>),
}

enum SerializationMode {
    Tuple,
    Sequence,
}

impl Node {
    fn serialize_simple(head: &str, tail: &str) -> String {
        let mut result = String::with_capacity(head.len() + tail.len());
        result.push_str(head);
        result.push_str(tail);
        result
    }

    fn serialize_nodes(nodes: Vec<Node>, mode: SerializationMode) -> String {
        let result = Node::serialize_compound_to_simple(nodes, mode);
        result.serialize()
    }

    fn aggregate_simple_nodes_in_sequence(nodes: Vec<Node>) -> Node {
        let mut head = String::from(eth::encode_u64(nodes.len() as u64));
        let mut tail = String::from("");
        let mut offset = Node::calculate_header_len_from_simple_nodes(&nodes);

        for node in nodes {
            match node {
                Node::Fixed(h) => head.push_str(&h),
                Node::Dynamic(t) => {
                    head.push_str(&eth::encode_u64(offset as u64));
                    tail.push_str(&t);
                    offset += t.len() >> 1;
                }
                Node::Seq(_) => unreachable!(),
                Node::Tuple(_) => unreachable!(),
            }
        }

        let serialized = Node::serialize_simple(&head, &tail);
        Node::Dynamic(serialized)
    }

    fn should_tuple_have_head(nodes: &Vec<Node>) -> bool {
        for node in nodes {
            match node {
                Node::Fixed(_) => continue,
                Node::Dynamic(_) => return true,
                Node::Seq(_) => unreachable!(),
                Node::Tuple(_) => unreachable!(),
            }
        }

        return false;
    }

    fn aggregate_simple_nodes_in_tuple(nodes: Vec<Node>) -> Node {
        let needs_head = Node::should_tuple_have_head(&nodes);
        let mut head = String::from("");
        let mut tail = String::from("");
        let mut offset = Node::calculate_header_len_from_simple_nodes(&nodes);

        for node in nodes {
            match node {
                Node::Fixed(h) => {
                    head.push_str(&h);
                }
                Node::Dynamic(t) => {
                    head.push_str(&eth::encode_u64(offset as u64));
                    tail.push_str(&t);
                    offset += t.len() >> 1;
                }
                Node::Seq(_) => unreachable!(),
                Node::Tuple(_) => unreachable!(),
            }
        }

        let serialized = Node::serialize_simple(&head, &tail);
        if needs_head {
            Node::Dynamic(serialized)
        } else {
            Node::Fixed(serialized)
        }
    }

    fn calculate_header_len_from_simple_nodes(nodes: &Vec<Node>) -> usize {
        let mut offset = 0 as usize;

        for node in nodes {
            match node {
                Node::Fixed(h) => offset += h.len() >> 1,
                Node::Dynamic(_) => offset += 32,
                Node::Seq(_) => unreachable!(),
                Node::Tuple(_) => unreachable!(),
            }
        }

        offset
    }

    fn serialize_compound_to_simple(nodes: Vec<Node>, mode: SerializationMode) -> Node {
        let mut simple_nodes = Vec::with_capacity(nodes.len());

        for node in nodes {
            let simple_node = Node::serialize_node_to_simple(node);
            simple_nodes.push(simple_node);
        }

        match mode {
            SerializationMode::Sequence => Node::aggregate_simple_nodes_in_sequence(simple_nodes),
            SerializationMode::Tuple => Node::aggregate_simple_nodes_in_tuple(simple_nodes),
        }
    }

    fn serialize_dynamic(content: String) -> String {
        let offset = 32;
        let mut new_content = eth::encode_u64(offset as u64);
        new_content.push_str(&content);
        new_content
    }

    fn serialize_node_to_simple(node: Node) -> Node {
        match node {
            Node::Seq(vec) => Node::serialize_compound_to_simple(vec, SerializationMode::Sequence),
            Node::Tuple(vec) => Node::serialize_compound_to_simple(vec, SerializationMode::Tuple),
            node => node,
        }
    }

    fn serialize(self) -> String {
        match self {
            Node::Fixed(head) => Node::serialize_simple(&head, ""),
            Node::Dynamic(content) => Node::serialize_dynamic(content),
            Node::Seq(vec) => Node::serialize_nodes(vec, SerializationMode::Sequence),
            Node::Tuple(vec) => Node::serialize_nodes(vec, SerializationMode::Tuple),
        }
    }

    fn push(&mut self, node: Node) {
        match self {
            Node::Fixed(_) => panic!("attempt to push node to Node::Fixed"),
            Node::Dynamic(_) => panic!("attempt to push node to Node::Dynamic"),
            Node::Seq(children) => children.push(node),
            Node::Tuple(children) => children.push(node),
        }
    }
}

pub enum NodeCompound<'a> {
    Standard {
        base: &'a mut NodeSerializer,
        ser: NodeSerializer,
    },
    BigInteger {
        base: &'a mut NodeSerializer,
        ser: custom_ser::BasicEthSerializer,
    },
}

impl<'a> ser::SerializeSeq for NodeCompound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        match self {
            NodeCompound::Standard { base: _, ser } => value.serialize(ser),
            NodeCompound::BigInteger { base: _, ser } => value.serialize(ser),
        }
    }

    fn end(self) -> Result<()> {
        match self {
            NodeCompound::Standard { base, ser } => {
                let node = ser.into_inner();
                base.push(node);
            }
            NodeCompound::BigInteger { base, ser } => {
                base.push(Node::Fixed(ser.serialize()));
            }
        }

        Ok(())
    }
}

impl<'a> ser::SerializeTuple for NodeCompound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for NodeCompound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for NodeCompound<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeMap for NodeCompound<'a> {
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

impl<'a> ser::SerializeStruct for NodeCompound<'a> {
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

impl<'a> ser::SerializeStructVariant for NodeCompound<'a> {
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

pub struct NodeSerializer {
    root: Node,

    // current_custom_type is used to set the current state of any type whose serialization
    // is implemented in the serializer.
    current_custom_serializer: Option<eth::Fixed>,
}

impl NodeSerializer {
    #[inline]
    fn new(node: Node) -> Self {
        NodeSerializer {
            root: node,
            current_custom_serializer: None,
        }
    }

    #[inline]
    fn into_inner(self) -> Node {
        self.root
    }

    #[inline]
    fn push(&mut self, node: Node) {
        self.current_custom_serializer = None;
        self.root.push(node);
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
        let serializer = self.serialize_seq(Some(0))?;
        serializer.end()
    }

    fn serialize_some<T: ?Sized + ser::Serialize>(self, value: &T) -> Result<Self::Ok> {
        let mut serializer = self.serialize_seq(Some(1))?;
        serializer.serialize_element(value)?;
        serializer.end()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        self.current_custom_serializer = eth::Fixed::get(name);
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        self.current_custom_serializer = eth::Fixed::get(name);
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let root = match len {
            Some(l) => Node::Seq(Vec::with_capacity(l)),
            None => Node::Seq(Vec::new()),
        };

        Ok(NodeCompound::Standard {
            base: self,
            ser: NodeSerializer::new(root),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        match &self.current_custom_serializer {
            Some(t) => match t {
                eth::Fixed::H256 => Ok(NodeCompound::BigInteger {
                    base: self,
                    ser: custom_ser::BasicEthSerializer::new_hash(32),
                }),
                eth::Fixed::H160 => Ok(NodeCompound::BigInteger {
                    base: self,
                    ser: custom_ser::BasicEthSerializer::new_hash(20),
                }),
                eth::Fixed::U256 => Ok(NodeCompound::BigInteger {
                    base: self,
                    ser: custom_ser::BasicEthSerializer::new_uint(32),
                }),
            },
            None => {
                let root = Node::Tuple(Vec::with_capacity(len));
                Ok(NodeCompound::Standard {
                    base: self,
                    ser: NodeSerializer::new(root),
                })
            }
        }
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        let root = Node::Tuple(Vec::with_capacity(len));
        Ok(NodeCompound::Standard {
            base: self,
            ser: NodeSerializer::new(root),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let root = Node::Tuple(Vec::with_capacity(len));
        Ok(NodeCompound::Standard {
            base: self,
            ser: NodeSerializer::new(root),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::not_implemented())
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        let root = Node::Tuple(Vec::with_capacity(len));
        Ok(NodeCompound::Standard {
            base: self,
            ser: NodeSerializer::new(root),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        let root = Node::Tuple(Vec::with_capacity(len));
        Ok(NodeCompound::Standard {
            base: self,
            ser: NodeSerializer::new(root),
        })
    }
}

pub enum RootCompound<'a, W: 'a> {
    Standard {
        writer: &'a mut Serializer<W>,
        ser: NodeSerializer,
    },
    BigInteger {
        writer: &'a mut Serializer<W>,
        ser: custom_ser::BasicEthSerializer,
    },
}

impl<'a, W: io::Write> ser::SerializeSeq for RootCompound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        match self {
            RootCompound::Standard { writer: _, ser } => value.serialize(ser),
            RootCompound::BigInteger { writer: _, ser } => value.serialize(ser),
        }
    }

    fn end(self) -> Result<()> {
        match self {
            RootCompound::Standard { writer, ser } => {
                let node = ser.into_inner();
                let encoded = node.serialize();
                writer.write(&encoded.into_bytes())
            }
            RootCompound::BigInteger { writer, ser } => writer.write(&ser.serialize().into_bytes()),
        }
    }
}

impl<'a, W: io::Write> ser::SerializeTuple for RootCompound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: io::Write> ser::SerializeTupleStruct for RootCompound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: io::Write> ser::SerializeTupleVariant for RootCompound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: io::Write> ser::SerializeMap for RootCompound<'a, W> {
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

impl<'a, W: io::Write> ser::SerializeStruct for RootCompound<'a, W> {
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

impl<'a, W: io::Write> ser::SerializeStructVariant for RootCompound<'a, W> {
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

#[inline]
pub fn to_writer<W: io::Write, T: ?Sized + ser::Serialize>(writer: W, value: &T) -> Result<()> {
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)?;
    Ok(())
}

#[inline]
pub fn to_vec<T: ?Sized + ser::Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value)?;
    Ok(writer)
}

#[inline]
pub fn to_string<T: ?Sized + ser::Serialize>(value: &T) -> Result<String> {
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
    use oasis_std::types::{Address, H160, H256, U256};
    use serde::Serialize;
    use std::fmt::Debug;

    fn test_encode_ok<T: PartialEq + Debug + Serialize>(errors: &[(T, &str)]) {
        for &(ref value, out) in errors {
            let out = out.to_string();
            let s = to_string(value).unwrap();
            assert_eq!(s, out);
        }
    }

    fn gen_u256(n: u64) -> U256 {
        let mut v = [0 as u8; 32];
        v[31] = (n & 0x00ff) as u8;
        v[30] = ((n >> 8) & 0x00ff) as u8;
        v[29] = ((n >> 16) & 0x00ff) as u8;
        v[28] = ((n >> 24) & 0x00ff) as u8;
        v[27] = ((n >> 32) & 0x00ff) as u8;
        v[26] = ((n >> 40) & 0x00ff) as u8;
        v[25] = ((n >> 48) & 0x00ff) as u8;
        v[24] = ((n >> 56) & 0x00ff) as u8;
        U256::from(v)
    }

    fn gen_h256(n: u64) -> H256 {
        let mut v = [0 as u8; 32];
        v[31] = (n & 0x00ff) as u8;
        v[30] = ((n >> 8) & 0x00ff) as u8;
        v[29] = ((n >> 16) & 0x00ff) as u8;
        v[28] = ((n >> 24) & 0x00ff) as u8;
        v[27] = ((n >> 32) & 0x00ff) as u8;
        v[26] = ((n >> 40) & 0x00ff) as u8;
        v[25] = ((n >> 48) & 0x00ff) as u8;
        v[24] = ((n >> 56) & 0x00ff) as u8;
        H256::from(v)
    }

    fn gen_h160(n: u64) -> H160 {
        let mut v = [0 as u8; 20];
        v[19] = (n & 0x00ff) as u8;
        v[18] = ((n >> 8) & 0x00ff) as u8;
        v[17] = ((n >> 16) & 0x00ff) as u8;
        v[16] = ((n >> 24) & 0x00ff) as u8;
        v[15] = ((n >> 32) & 0x00ff) as u8;
        v[14] = ((n >> 40) & 0x00ff) as u8;
        v[13] = ((n >> 48) & 0x00ff) as u8;
        v[12] = ((n >> 56) & 0x00ff) as u8;
        H160::from(v)
    }

    #[derive(Serialize, Debug, PartialEq)]
    struct Simple {
        value1: String,
        value2: String,
    }

    #[derive(Serialize, Debug, PartialEq)]
    struct Complex {
        value: String,
        simple: Simple,
    }

    #[derive(Serialize, Clone, Debug, PartialEq)]
    struct Composed {
        field: Vec<Vec<(String, (H256, [u32; 4]))>>,
    }

    #[derive(Serialize, Clone, Debug, PartialEq)]
    struct Composed2 {
        field: Vec<Vec<((H256, [u32; 4]), String)>>,
    }

    #[test]
    fn test_write_h160() {
        let tests = &[
            (
                gen_h160(0),
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                gen_h160(2),
                "0000000000000000000000000000000000000000000000000000000000000002",
            ),
            (
                gen_h160(15),
                "000000000000000000000000000000000000000000000000000000000000000f",
            ),
            (
                gen_h160(16),
                "0000000000000000000000000000000000000000000000000000000000000010",
            ),
            (
                gen_h160(1_000),
                "00000000000000000000000000000000000000000000000000000000000003e8",
            ),
            (
                gen_h160(100_000),
                "00000000000000000000000000000000000000000000000000000000000186a0",
            ),
            (
                gen_h160(u64::max_value()),
                "000000000000000000000000000000000000000000000000ffffffffffffffff",
            ),
        ];

        test_encode_ok(tests);
    }

    #[test]
    fn test_write_address() {
        let tests = &[
            (
                gen_h160(0) as Address,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                gen_h160(2) as Address,
                "0000000000000000000000000000000000000000000000000000000000000002",
            ),
            (
                gen_h160(15) as Address,
                "000000000000000000000000000000000000000000000000000000000000000f",
            ),
            (
                gen_h160(16) as Address,
                "0000000000000000000000000000000000000000000000000000000000000010",
            ),
            (
                gen_h160(1_000) as Address,
                "00000000000000000000000000000000000000000000000000000000000003e8",
            ),
            (
                gen_h160(100_000) as Address,
                "00000000000000000000000000000000000000000000000000000000000186a0",
            ),
            (
                gen_h160(u64::max_value()) as Address,
                "000000000000000000000000000000000000000000000000ffffffffffffffff",
            ),
        ];

        test_encode_ok(tests);
    }

    #[test]
    fn test_write_h256() {
        let tests = &[
            (
                gen_h256(0),
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                gen_h256(2),
                "0000000000000000000000000000000000000000000000000000000000000002",
            ),
            (
                gen_h256(15),
                "000000000000000000000000000000000000000000000000000000000000000f",
            ),
            (
                gen_h256(16),
                "0000000000000000000000000000000000000000000000000000000000000010",
            ),
            (
                gen_h256(1_000),
                "00000000000000000000000000000000000000000000000000000000000003e8",
            ),
            (
                gen_h256(100_000),
                "00000000000000000000000000000000000000000000000000000000000186a0",
            ),
            (
                gen_h256(u64::max_value()),
                "000000000000000000000000000000000000000000000000ffffffffffffffff",
            ),
        ];

        test_encode_ok(tests);
    }

    #[test]
    fn test_write_u256() {
        let tests = &[
            (
                gen_u256(0),
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                gen_u256(2),
                "0000000000000000000000000000000000000000000000000000000000000002",
            ),
            (
                gen_u256(15),
                "000000000000000000000000000000000000000000000000000000000000000f",
            ),
            (
                gen_u256(16),
                "0000000000000000000000000000000000000000000000000000000000000010",
            ),
            (
                gen_u256(1_000),
                "00000000000000000000000000000000000000000000000000000000000003e8",
            ),
            (
                gen_u256(100_000),
                "00000000000000000000000000000000000000000000000000000000000186a0",
            ),
            (
                gen_u256(u64::max_value()),
                "000000000000000000000000000000000000000000000000ffffffffffffffff",
            ),
        ];

        test_encode_ok(tests);
    }

    #[test]
    fn test_write_bool() {
        let tests = &[
            (
                false,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                true,
                "0000000000000000000000000000000000000000000000000000000000000001",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_u8() {
        let tests = &[
            (
                0x00 as u8,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                0x01 as u8,
                "0000000000000000000000000000000000000000000000000000000000000001",
            ),
            (
                0x10 as u8,
                "0000000000000000000000000000000000000000000000000000000000000010",
            ),
            (
                0x80 as u8,
                "0000000000000000000000000000000000000000000000000000000000000080",
            ),
            (
                0xff as u8,
                "00000000000000000000000000000000000000000000000000000000000000ff",
            ),
        ];
        test_encode_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_write_i8() {
        let tests = &[
            (
                0x00 as i8,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                0x01 as i8,
                "0000000000000000000000000000000000000000000000000000000000000001",
            ),
            (
                0x10 as i8,
                "0000000000000000000000000000000000000000000000000000000000000010",
            ),
            (
                0x80 as i8,
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80",
            ),
            (
                0xff as i8,
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_u16() {
        let tests = &[
            (
                0x0000 as u16,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                0x0100 as u16,
                "0000000000000000000000000000000000000000000000000000000000000100",
            ),
            (
                0x1000 as u16,
                "0000000000000000000000000000000000000000000000000000000000001000",
            ),
            (
                0x8000 as u16,
                "0000000000000000000000000000000000000000000000000000000000008000",
            ),
            (
                0xffff as u16,
                "000000000000000000000000000000000000000000000000000000000000ffff",
            ),
        ];
        test_encode_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_write_i16() {
        let tests = &[
            (
                0x0000 as i16,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                0x0100 as i16,
                "0000000000000000000000000000000000000000000000000000000000000100",
            ),
            (
                0x1000 as i16,
                "0000000000000000000000000000000000000000000000000000000000001000",
            ),
            (
                0x8000 as i16,
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8000",
            ),
            (
                0xffff as i16,
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_u32() {
        let tests = &[
            (
                0x00000000 as u32,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                0x01000000 as u32,
                "0000000000000000000000000000000000000000000000000000000001000000",
            ),
            (
                0x10000000 as u32,
                "0000000000000000000000000000000000000000000000000000000010000000",
            ),
            (
                0x80000000 as u32,
                "0000000000000000000000000000000000000000000000000000000080000000",
            ),
            (
                0xffffffff as u32,
                "00000000000000000000000000000000000000000000000000000000ffffffff",
            ),
        ];
        test_encode_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_write_i32() {
        let tests = &[
            (
                0x00000000 as i32,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                0x01000000 as i32,
                "0000000000000000000000000000000000000000000000000000000001000000",
            ),
            (
                0x10000000 as i32,
                "0000000000000000000000000000000000000000000000000000000010000000",
            ),
            (
                0x80000000 as i32,
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffff80000000",
            ),
            (
                0xffffffff as i32,
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_u64() {
        let tests = &[
            (
                0x0000000000000000 as u64,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                0x0100000000000000 as u64,
                "0000000000000000000000000000000000000000000000000100000000000000",
            ),
            (
                0x1000000000000000 as u64,
                "0000000000000000000000000000000000000000000000001000000000000000",
            ),
            (
                0x8000000000000000 as u64,
                "0000000000000000000000000000000000000000000000008000000000000000",
            ),
            (
                0xffffffffffffffff as u64,
                "000000000000000000000000000000000000000000000000ffffffffffffffff",
            ),
        ];
        test_encode_ok(tests);
    }

    #[allow(overflowing_literals)]
    #[test]
    fn test_write_i64() {
        let tests = &[
            (
                0x0000000000000000 as i64,
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                0x0100000000000000 as i64,
                "0000000000000000000000000000000000000000000000000100000000000000",
            ),
            (
                0x1000000000000000 as i64,
                "0000000000000000000000000000000000000000000000001000000000000000",
            ),
            (
                0x8000000000000000 as i64,
                "ffffffffffffffffffffffffffffffffffffffffffffffff8000000000000000",
            ),
            (
                0xffffffffffffffff as i64,
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_char() {
        let tests = &[
            (
                'a',
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 6100000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                'é',
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000002\
                 c3a9000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                'ø',
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000002\
                 c3b8000000000000000000000000000000000000000000000000000000000000",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_string() {
        let tests = &[
            (
                "hello",
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000005\
                 68656c6c6f000000000000000000000000000000000000000000000000000000",
            ),
            (
                "",
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                "some long string that takes more than 32 bytes so we can see how eth abi \
                 encodes long strings",
                "0000000000000000000000000000000000000000000000000000000000000020\
                 000000000000000000000000000000000000000000000000000000000000005d\
                 736f6d65206c6f6e6720737472696e6720746861742074616b6573206d6f7265\
                 207468616e20333220627974657320736f2077652063616e2073656520686f77\
                 206574682061626920656e636f646573206c6f6e6720737472696e6773000000",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_option() {
        let tests = &[
            (
                None,
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                Some("hello"),
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000005\
                 68656c6c6f000000000000000000000000000000000000000000000000000000",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_unit() {
        let tests = &[((), "")];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_tuple_int() {
        let tests = &[(
            (1, "1"),
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000001\
             3100000000000000000000000000000000000000000000000000000000000000",
        )];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_tuple_string() {
        let tests = &[(
            ("1", "2"),
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000080\
             0000000000000000000000000000000000000000000000000000000000000001\
             3100000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000001\
             3200000000000000000000000000000000000000000000000000000000000000",
        )];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_int_seq() {
        let tests = &[
            (
                vec![1, 2, 3],
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000003\
                 0000000000000000000000000000000000000000000000000000000000000001\
                 0000000000000000000000000000000000000000000000000000000000000002\
                 0000000000000000000000000000000000000000000000000000000000000003",
            ),
            (
                vec![],
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000000",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_u8_fixed_seq() {
        let tests = &[(
            [1 as u8; 3],
            "0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000001",
        )];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_str_seq() {
        let tests = &[
            (
                vec!["1", "2", "3"],
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
            ),
            (
                vec![],
                "0000000000000000000000000000000000000000000000000000000000000020\
                 0000000000000000000000000000000000000000000000000000000000000000",
            ),
        ];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_multiseq() {
        let tests = &[(
            vec![vec!["1", "2"], vec!["3", "4"]],
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
        )];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_simple_struct() {
        let tests = &[(
            Simple {
                value1: "1".to_string(),
                value2: "2".to_string(),
            },
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000040\
             0000000000000000000000000000000000000000000000000000000000000080\
             0000000000000000000000000000000000000000000000000000000000000001\
             3100000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000001\
             3200000000000000000000000000000000000000000000000000000000000000",
        )];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_complex_struct() {
        let tests = &[(
            Complex {
                value: "1".to_string(),
                simple: Simple {
                    value1: "2".to_string(),
                    value2: "3".to_string(),
                },
            },
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
        )];
        test_encode_ok(tests);
    }

    #[test]
    fn test_write_composed_struct() {
        let s = "string".to_string();
        let addr = [1u8; 32];
        let b = [2u32; 4];

        let tests = &[(
            Composed {
                field: vec![vec![(s, (addr.into(), b))]],
            },
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000020\
             00000000000000000000000000000000000000000000000000000000000000c0\
             0101010101010101010101010101010101010101010101010101010101010101\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000006\
             737472696e670000000000000000000000000000000000000000000000000000",
        )];

        test_encode_ok(tests);
    }

    #[test]
    fn test_write_composed2_struct() {
        let s = "string".to_string();
        let addr = [1u8; 32];
        let b = [2u32; 4];

        let tests = &[(
            Composed2 {
                field: vec![vec![((addr.into(), b), s)]],
            },
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000020\
             0101010101010101010101010101010101010101010101010101010101010101\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000002\
             0000000000000000000000000000000000000000000000000000000000000002\
             00000000000000000000000000000000000000000000000000000000000000c0\
             0000000000000000000000000000000000000000000000000000000000000006\
             737472696e670000000000000000000000000000000000000000000000000000",
        )];

        test_encode_ok(tests);
    }
}
