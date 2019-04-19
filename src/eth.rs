pub(crate) fn encode_bool(value: bool) -> String {
    let abi_encoded = ethabi::encode(&[ethabi::Token::Bool(value)]);
    hex::encode(abi_encoded)
}

pub(crate) fn encode_i64(value: i64) -> String {
    let padded = pad_i64(value);
    let abi_encoded = ethabi::encode(&[ethabi::Token::Int(padded.into())]);
    hex::encode(abi_encoded)
}

pub(crate) fn encode_u64(value: u64) -> String {
    let padded = pad_u64(value);
    let abi_encoded = ethabi::encode(&[ethabi::Token::Uint(padded.into())]);
    hex::encode(abi_encoded)
}

pub(crate) fn encode_bytes(value: &[u8]) -> String {
    let abi_encoded = ethabi::encode(&[ethabi::Token::Bytes(value.into())]);
    hex::encode(abi_encoded)
}

pub(crate) struct DynamicSizedEncoding {
    offset: String,
    size: String,
    content: String,
}

impl DynamicSizedEncoding {
    pub(crate) fn offset(&self) -> &str {
        &self.offset
    }

    pub(crate) fn size(&self) -> &str {
        &self.size
    }

    pub(crate) fn content(&self) -> &str {
        &self.content
    }
}

pub(crate) fn encode_bytes_dynamic(value: &[u8]) -> DynamicSizedEncoding {
    let abi_encoded = ethabi::encode(&[ethabi::Token::Bytes(value.into())]);
    let hex_encoded = hex::encode(abi_encoded);

    // ignore head which is the offset set by default by ethabi
    let (offset, tail) = hex_encoded.split_at(64);
    let (size, content) = tail.split_at(64);
    DynamicSizedEncoding {
        offset: offset.to_string(),
        size: size.to_string(),
        content: content.to_string(),
    }
}

/// Converts u64 to right aligned array of 32 bytes.
fn pad_u64(value: u64) -> [u8; 32] {
    let mut padded = [0u8; 32];
    padded[24] = (value >> 56) as u8;
    padded[25] = (value >> 48) as u8;
    padded[26] = (value >> 40) as u8;
    padded[27] = (value >> 32) as u8;
    padded[28] = (value >> 24) as u8;
    padded[29] = (value >> 16) as u8;
    padded[30] = (value >> 8) as u8;
    padded[31] = value as u8;
    padded
}

/// Converts i64 to right aligned array of 32 bytes.
fn pad_i64(value: i64) -> [u8; 32] {
    if value >= 0 {
        return pad_u64(value as u64);
    }

    let mut padded = [0xffu8; 32];
    padded[24] = (value >> 56) as u8;
    padded[25] = (value >> 48) as u8;
    padded[26] = (value >> 40) as u8;
    padded[27] = (value >> 32) as u8;
    padded[28] = (value >> 24) as u8;
    padded[29] = (value >> 16) as u8;
    padded[30] = (value >> 8) as u8;
    padded[31] = value as u8;
    padded
}
