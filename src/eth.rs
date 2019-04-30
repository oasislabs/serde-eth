use super::error::Error;
use oasis_std::types::U256;

fn parse_int(bytes: &[u8], size: usize) -> Result<i64, Error> {
    if bytes.len() != 64 {
        return Err(Error::parsing("invalid byte array size for uint"));
    }

    let decoded = hex::decode(bytes).map_err(Error::hex_parsing)?;
    let value: U256 = (&decoded[..]).into();

    // if value is supposed to be a positive integer
    if value.leading_zeros() > 0 {
        if value.bits() > size {
            return Err(Error::parsing(
                "decoded integer does not fit in integer of specified size",
            ));
        }
        return Ok(value.low_u64() as i64);
    }

    let (n, overflows) = value.overflowing_neg();
    if !overflows {
        // if it is a negative integer negating it must overflow
        return Err(Error::parsing(
            "decoded integer does not fit in integer of specified size",
        ));
    }

    let (n, overflows) = n.overflowing_add(U256::one());
    if overflows || n.bits() > size {
        return Err(Error::parsing(
            "decoded integer does not fit in integer of specified size",
        ));
    }

    let int = n.low_u64() as i64;
    Ok(if int < 0 { int } else { -int })
}

fn parse_uint(bytes: &[u8], size: usize) -> Result<u64, Error> {
    if bytes.len() != 64 {
        return Err(Error::parsing("invalid byte array size for uint"));
    }

    let decoded = hex::decode(bytes).map_err(Error::hex_parsing)?;
    let value: U256 = (&decoded[..]).into();

    // if value is supposed to be a positive integer
    if value.leading_zeros() > 0 {
        if value.bits() > size {
            return Err(Error::parsing(
                "decoded integer does not fit in integer of specified size",
            ));
        }
        Ok(value.low_u64())
    } else {
        Err(Error::parsing(
            "decoded integer does not fit in integer of specified size",
        ))
    }
}

fn gen_uint(value: u64) -> String {
    let uint: U256 = value.into();
    let mut bytes = [0u8; 32];
    uint.to_big_endian(&mut bytes[..]);
    hex::encode(&bytes[..])
}

pub(crate) fn decode_bool(bytes: &[u8]) -> Result<bool, Error> {
    let value = parse_uint(bytes, 1)?;
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Error::parsing("invalid value for boolean")),
    }
}

pub(crate) fn decode_uint(bytes: &[u8], size: usize) -> Result<u64, Error> {
    parse_uint(bytes, size)
}

pub(crate) fn decode_int(bytes: &[u8], size: usize) -> Result<i64, Error> {
    parse_int(bytes, size)
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
    let uint = if value { 1 } else { 0 };
    gen_uint(uint)
}

pub(crate) fn encode_i64(value: i64) -> String {
    if value >= 0 {
        return gen_uint(value as u64);
    }

    let value = (!value as u64) + 1;
    let uint: U256 = value.into();
    let (uint, overflows) = uint.overflowing_sub(U256::one());
    if overflows {
        if uint != U256::zero() {
            panic!("expected overflow from negating uint");
        }
    }

    // if original value was -1, and it is set as 0 for U256,
    // overflowing_neg() leaves 0 unchanged. So we do this,
    // in order to get back the 0xfff..ff (32 f's)
    // value which is the correct encoding
    let uint = if uint == U256::zero() {
        let (uint, _) = U256::one().overflowing_neg();
        let (uint, _) = uint.overflowing_add(U256::one());
        uint
    } else {
        let (uint, overflows) = uint.overflowing_neg();
        if !overflows {
            panic!("expected overflow from negating uint");
        }
        uint
    };

    let mut bytes = [0u8; 32];
    uint.to_big_endian(&mut bytes[..]);
    hex::encode(&bytes[..])
}

pub(crate) fn encode_u64(value: u64) -> String {
    gen_uint(value)
}

pub(crate) fn encode_bytes(value: &[u8]) -> String {
    let encoding = encode_bytes_dynamic(value);

    // just set the default offset to 0x20
    let offset = "0000000000000000000000000000000000000000000000000000000000000020";
    let mut result = String::new();
    result.push_str(offset);
    result.push_str(encoding.size());
    result.push_str(encoding.content());
    result
}

pub(crate) struct DynamicSizedEncoding {
    size: String,
    content: String,
}

impl DynamicSizedEncoding {
    pub(crate) fn size(&self) -> &str {
        &self.size
    }

    pub(crate) fn content(&self) -> &str {
        &self.content
    }
}

pub(crate) fn encode_bytes_dynamic(value: &[u8]) -> DynamicSizedEncoding {
    let len = value.len() as u64;
    let base = (len >> 5) << 5;
    let remain = if len - base == 0u64 { 0 } else { 1 };
    let payload_len = base + (remain << 5);
    let total_len = payload_len; // offset and length
    let mut payload = vec![0u8; total_len as usize];

    for i in 0..value.len() {
        payload[i] = value[i];
    }

    let mut size = vec![0u8; 32];
    let len: U256 = value.len().into();
    len.to_big_endian(&mut size[..]);

    let size = hex::encode(size);
    let content = hex::encode(payload);
    DynamicSizedEncoding { size, content }
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
