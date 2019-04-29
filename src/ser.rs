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
    pub fn new(writer: W) -> Self {
        Serializer {
            writer: writer,
            current_custom_serializer: None,
        }
    }

    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.current_custom_serializer = None;
        self.writer.write_all(bytes).map_err(Error::io)
    }

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
        let mut tail = String::new();
        let mut offset = Node::calculate_header_len_from_simple_nodes(&nodes);

        for node in nodes {
            match node {
                Node::Fixed(h) => head.push_str(&h),
                Node::Dynamic(t) => {
                    head.push_str(&eth::encode_u64(offset as u64));
                    tail.push_str(&t);
                    offset += t.len() >> 1;
                }
                Node::Seq(_) | Node::Tuple(_) => unreachable!(),
            }
        }

        let serialized = Node::serialize_simple(&head, &tail);
        Node::Dynamic(serialized)
    }

    fn should_tuple_have_head(nodes: &Vec<Node>) -> bool {
        nodes.iter().any(|t| match t {
            Node::Fixed(_) => false,
            Node::Dynamic(_) => true,
            Node::Seq(_) | Node::Tuple(_) => unreachable!(),
        })
    }

    fn aggregate_simple_nodes_in_tuple(nodes: Vec<Node>) -> Node {
        let needs_head = Node::should_tuple_have_head(&nodes);
        let mut head = String::new();
        let mut tail = String::new();
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
                Node::Seq(_) | Node::Tuple(_) => unreachable!(),
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
        nodes.iter().fold(0, |acc, node| match node {
            Node::Fixed(h) => acc + (h.len() >> 1),
            Node::Dynamic(_) => acc + 32,
            Node::Seq(_) | Node::Tuple(_) => unreachable!(),
        })
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
    fn new(node: Node) -> Self {
        NodeSerializer {
            root: node,
            current_custom_serializer: None,
        }
    }

    fn into_inner(self) -> Node {
        self.root
    }

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

pub fn to_writer<W: io::Write, T: ?Sized + ser::Serialize>(writer: W, value: &T) -> Result<()> {
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)?;
    Ok(())
}

pub fn to_vec<T: ?Sized + ser::Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value)?;
    Ok(writer)
}

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
    use crate::serde_tests;
    use serde::Serialize;
    use std::fmt::Debug;

    fn test_encode_ok<T: PartialEq + Debug + Serialize>(errors: &[(T, &str)]) {
        for &(ref value, out) in errors {
            let out = out.to_string();
            let s = to_string(value).unwrap();
            assert_eq!(s, out);
        }
    }

    #[test]
    fn test_write_h160() {
        test_encode_ok(&serde_tests::test_h160()[..]);
    }

    #[test]
    fn test_write_address() {
        test_encode_ok(&serde_tests::test_address()[..]);
    }

    #[test]
    fn test_write_h256() {
        test_encode_ok(&serde_tests::test_h256()[..]);
    }

    #[test]
    fn test_write_u256() {
        test_encode_ok(&serde_tests::test_u256()[..]);
    }

    #[test]
    fn test_write_bool() {
        test_encode_ok(&serde_tests::test_bool()[..]);
    }

    #[test]
    fn test_write_u8() {
        test_encode_ok(&serde_tests::test_u8()[..]);
    }

    #[test]
    fn test_write_i8() {
        test_encode_ok(&serde_tests::test_i8()[..]);
    }

    #[test]
    fn test_write_u16() {
        test_encode_ok(&serde_tests::test_u16()[..]);
    }

    #[test]
    fn test_write_i16() {
        test_encode_ok(&serde_tests::test_i16()[..]);
    }

    #[test]
    fn test_write_u32() {
        test_encode_ok(&serde_tests::test_u32()[..]);
    }

    #[test]
    fn test_write_i32() {
        test_encode_ok(&serde_tests::test_i32()[..]);
    }

    #[test]
    fn test_write_u64() {
        test_encode_ok(&serde_tests::test_u64()[..]);
    }

    #[test]
    fn test_write_i64() {
        test_encode_ok(&serde_tests::test_i64()[..]);
    }

    #[test]
    fn test_write_char() {
        test_encode_ok(&serde_tests::test_char()[..]);
    }

    #[test]
    fn test_write_string() {
        test_encode_ok(&serde_tests::test_string()[..]);
    }

    #[test]
    fn test_write_option() {
        test_encode_ok(&serde_tests::test_option()[..]);
    }

    #[test]
    fn test_write_unit() {
        test_encode_ok(&serde_tests::test_unit()[..]);
    }

    #[test]
    fn test_write_tuple_mixed() {
        test_encode_ok(&serde_tests::test_tuple_mixed()[..]);
    }

    #[test]
    fn test_write_tuple_string() {
        test_encode_ok(&serde_tests::test_tuple_string()[..]);
    }

    #[test]
    fn test_write_tuple_u8() {
        test_encode_ok(&serde_tests::test_tuple_u8()[..]);
    }

    #[test]
    fn test_write_seq_int() {
        test_encode_ok(&serde_tests::test_seq_int()[..]);
    }

    #[test]
    fn test_write_str_seq() {
        test_encode_ok(&serde_tests::test_str_seq()[..]);
    }

    #[test]
    fn test_write_multiseq() {
        test_encode_ok(&serde_tests::test_multiseq()[..]);
    }

    #[test]
    fn test_write_simple_struct() {
        test_encode_ok(&serde_tests::test_simple_struct()[..]);
    }

    #[test]
    fn test_write_complex_struct() {
        test_encode_ok(&serde_tests::test_complex_struct()[..]);
    }

    #[test]
    fn test_write_composed_struct() {
        test_encode_ok(&serde_tests::test_composed_struct()[..]);
    }

    #[test]
    fn test_write_string_composed_struct() {
        test_encode_ok(&serde_tests::test_string_composed_struct()[..]);
    }

    #[test]
    fn test_write_reversed_composed_struct() {
        test_encode_ok(&serde_tests::test_reversed_composed_struct()[..]);
    }
}
