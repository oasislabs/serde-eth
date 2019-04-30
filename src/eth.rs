use super::error::Error;

pub(crate) fn decode_bool(bytes: &[u8]) -> Result<bool, Error> {
    let decoded = hex::decode(bytes).map_err(Error::hex_parsing)?;
    let tokens =
        ethabi::decode(&[ethabi::ParamType::Bool], &decoded[..]).map_err(Error::eth_parsing)?;
    if tokens.len() != 1 {
        return Err(Error::parsing("decoded unexpected number of tokens"));
    }

    match tokens
        .get(0)
        .expect("If token decoded successfully there should be one token in the decoded list")
    {
        ethabi::Token::Bool(b) => Ok(*b),
        _ => Err(Error::parsing("decoded unexpected type for boolean")),
    }
}

pub(crate) fn decode_uint(bytes: &[u8], size: usize) -> Result<u64, Error> {
    if size < 8 || size > 64 {
        return Err(Error::message(
            "an unsigned integer must be anumber between 8 and 64 bits",
        ));
    }

    let decoded = hex::decode(bytes).map_err(Error::hex_parsing)?;
    let tokens = ethabi::decode(&[ethabi::ParamType::Uint(size)], &decoded[..])
        .map_err(Error::eth_parsing)?;
    if tokens.len() != 1 {
        return Err(Error::parsing("decoded unexpected number of tokens"));
    }

    match tokens
        .get(0)
        .expect("If token decoded successfully there should be one token in the decoded list")
    {
        ethabi::Token::Uint(v) => verify_uint(v, size),
        _ => Err(Error::parsing("decoded unexpected type for uint")),
    }
}

pub(crate) fn decode_int(bytes: &[u8], size: usize) -> Result<i64, Error> {
    if size < 8 || size > 64 {
        return Err(Error::message(
            "an integer must be a number between 8 and 64 bits",
        ));
    }

    let decoded = hex::decode(bytes).map_err(Error::hex_parsing)?;
    let tokens = ethabi::decode(&[ethabi::ParamType::Int(size)], &decoded[..])
        .map_err(Error::eth_parsing)?;
    if tokens.len() != 1 {
        return Err(Error::parsing("decoded unexpected number of tokens"));
    }

    match tokens
        .get(0)
        .expect("If token decoded successfully there should be one token in the decoded list")
    {
        ethabi::Token::Int(v) => verify_int(v, size),
        _ => Err(Error::parsing("decoded unexpected type for int")),
    }
}

pub(crate) fn decode_bytes(bytes: &[u8], len: usize) -> Result<Vec<u8>, Error> {
    let decoded = hex::decode(bytes).map_err(Error::hex_parsing)?;
    if len > decoded.len() {
        Err(Error::parsing(
            "decoded bytes are smaller than the required length",
        ))
    } else {
        Ok(decoded[..len].to_vec())
    }
}

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

fn verify_int(int: &ethabi::Int, size: usize) -> Result<i64, Error> {
    if int.leading_zeros() > 0 {
        if int.bits() >= size {
            return Err(Error::parsing(
                "decoded integer does not fit in integer of specified size",
            ));
        }
        Ok(int.low_u64() as i64)
    } else {
        let (n, overflows) = int.overflowing_neg();
        if !overflows {
            // if it is a negative integer negating it must overflow
            return Err(Error::parsing(
                "decoded integer does not fit in integer of specified size",
            ));
        }

        let (n, overflows) = n.overflowing_add(ethereum_types::U256::one());
        if overflows || n.bits() > size {
            return Err(Error::parsing(
                "decoded integer does not fit in integer of specified size",
            ));
        }

        let int = n.low_u64() as i64;
        Ok(if int < 0 { int } else { -int })
    }
}

fn verify_uint(u: &ethabi::Uint, size: usize) -> Result<u64, Error> {
    println!("{}, {}", u, size);
    if u.bits() > size {
        Err(Error::parsing(
            "decoded integer does not fit in integer of specified size",
        ))
    } else {
        Ok(u.low_u64())
    }
}

pub(crate) struct DynamicSizedEncoding {
    #[allow(dead_code)]
    offset: String,
    size: String,
    content: String,
}

impl DynamicSizedEncoding {
    #[allow(dead_code)]
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

#[derive(Debug, Clone, Copy)]
pub enum Fixed {
    H256,
    H160,
    U256,
}

impl Fixed {
    pub fn get(name: &str) -> Option<Fixed> {
        match name {
            "H256" => Some(Fixed::H256),
            "H160" => Some(Fixed::H160),
            "U256" => Some(Fixed::U256),
            _ => None,
        }
    }
}
