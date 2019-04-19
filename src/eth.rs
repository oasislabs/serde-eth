use ethabi;
use hex;

fn encode_bool(value: bool) -> String {
    let abi_encoded = ethabi::encode(&[ethabi::Token::Bool(value)]);
    hex::encode(abi_encoded)
}

fn encode_i64(value: i64) -> String {
    let padded = pad_i64(value);
    let abi_encoded = ethabi::encode(&[ethabi::Token::Int(padded.into())]);
    hex::encode(abi_encoded)
}

fn encode_u64(value: u64) -> String {
    let padded = pad_u64(value);
    let abi_encoded = ethabi::encode(&[ethabi::Token::Uint(padded.into())]);
    hex::encode(abi_encoded)
}

/// Converts u64 to right aligned array of 32 bytes.
pub fn pad_u64(value: u64) -> [u8; 32] {
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
pub fn pad_i64(value: i64) -> [u8; 32] {
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

