use serde::{Serialize, Deserialize};
use oasis_std::types::{Address, H160, H256, U256};
use std::vec::Vec;

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Simple {
    value1: String,
    value2: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Complex {
    value: String,
    simple: Simple,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Composed {
    field: Vec<Vec<(String, (H256, [u32; 4]))>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct ReversedComposed {
    field: Vec<Vec<((H256, [u32; 4]), String)>>,
}

#[allow(dead_code)]
pub(crate) fn test_h160() -> Vec<(H160, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_address() -> Vec<(Address, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_h256() -> Vec<(H256, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_u256() -> Vec<(U256, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_bool() -> Vec<(bool, &'static str)> {
    vec![
        (
            false,
            "0000000000000000000000000000000000000000000000000000000000000000",
        ),
        (
            true,
            "0000000000000000000000000000000000000000000000000000000000000001",
        ),
    ]
}

#[allow(dead_code)]
pub(crate) fn test_u8() -> Vec<(u8, &'static str)> {
    vec![
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
    ]
}

#[allow(overflowing_literals)]#[allow(dead_code)]
pub(crate) fn test_i8() -> Vec<(i8, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_u16() -> Vec<(u16, &'static str)> {
    vec![
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
    ]
}

#[allow(overflowing_literals)]#[allow(dead_code)]
pub(crate) fn test_i16() -> Vec<(i16, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_u32() -> Vec<(u32, &'static str)> {
    vec![
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
    ]
}

#[allow(overflowing_literals)]#[allow(dead_code)]
pub(crate) fn test_i32() -> Vec<(i32, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_u64() -> Vec<(u64, &'static str)> {
    vec![
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
    ]
}

#[allow(overflowing_literals)]#[allow(dead_code)]
pub(crate) fn test_i64() -> Vec<(i64, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_char() -> Vec<(char, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_string() -> Vec<(String, &'static str)> {
    vec![
        (
            "hello".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000005\
             68656c6c6f000000000000000000000000000000000000000000000000000000",
        ),
        (
            "".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000000",
        ),
        (
            "some long string that takes more than 32 bytes so we can see how eth abi \
             encodes long strings".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000020\
             000000000000000000000000000000000000000000000000000000000000005d\
             736f6d65206c6f6e6720737472696e6720746861742074616b6573206d6f7265\
             207468616e20333220627974657320736f2077652063616e2073656520686f77\
             206574682061626920656e636f646573206c6f6e6720737472696e6773000000",
        ),
    ]
}

#[allow(dead_code)]
pub(crate) fn test_option() -> Vec<(Option<String>, &'static str)> {
    vec![
        (
            None,
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000000",
        ),
        (
            Some("hello".to_string()),
            "0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000001\
             0000000000000000000000000000000000000000000000000000000000000020\
             0000000000000000000000000000000000000000000000000000000000000005\
             68656c6c6f000000000000000000000000000000000000000000000000000000",
        ),
    ]
}

#[allow(dead_code)]
pub(crate) fn test_unit() -> Vec<((), &'static str)> {
    vec![((), "")]
}

#[allow(dead_code)]
pub(crate) fn test_tuple_mixed() -> Vec<((u8, String), &'static str)> {
    vec![(
        (1, "1".to_string()),
        "0000000000000000000000000000000000000000000000000000000000000020\
         0000000000000000000000000000000000000000000000000000000000000001\
         0000000000000000000000000000000000000000000000000000000000000040\
         0000000000000000000000000000000000000000000000000000000000000001\
         3100000000000000000000000000000000000000000000000000000000000000",
    )]
}

#[allow(dead_code)]
pub(crate) fn test_tuple_string() -> Vec<((String, String), &'static str)> {
    vec![(
        ("1".to_string(), "2".to_string()),
        "0000000000000000000000000000000000000000000000000000000000000020\
         0000000000000000000000000000000000000000000000000000000000000040\
         0000000000000000000000000000000000000000000000000000000000000080\
         0000000000000000000000000000000000000000000000000000000000000001\
         3100000000000000000000000000000000000000000000000000000000000000\
         0000000000000000000000000000000000000000000000000000000000000001\
         3200000000000000000000000000000000000000000000000000000000000000",
    )]
}

#[allow(dead_code)]
pub(crate) fn test_seq_int() -> Vec<(Vec<u8>, &'static str)> {
    vec![
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_tuple_u8() -> Vec<([u8;3], &'static str)> {
    vec![(
        [1 as u8; 3],
        "0000000000000000000000000000000000000000000000000000000000000001\
         0000000000000000000000000000000000000000000000000000000000000001\
         0000000000000000000000000000000000000000000000000000000000000001",
    )]
}

#[allow(dead_code)]
pub(crate) fn test_str_seq() -> Vec<(Vec<String>, &'static str)> {
    vec![
        (
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
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
    ]
}

#[allow(dead_code)]
pub(crate) fn test_multiseq() -> Vec<(Vec<Vec<String>>, &'static str)> {
    vec![(
        vec![vec!["1".to_string(), "2".to_string()], vec!["3".to_string(), "4".to_string()]],
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
    )]
}

#[allow(dead_code)]
pub(crate) fn test_simple_struct() -> Vec<(Simple, &'static str)> {
    vec![(
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
    )]
}

#[allow(dead_code)]
pub(crate) fn test_complex_struct() -> Vec<(Complex, &'static str)> {
    vec![(
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
    )]
}

#[allow(dead_code)]
pub(crate) fn test_composed_struct() -> Vec<(Composed, &'static str)> {
    let s = "string".to_string();
    let addr = [1u8; 32];
    let b = [2u32; 4];

    vec![(
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
    )]

}

#[allow(dead_code)]
pub(crate) fn test_reversed_composed_struct() -> Vec<(ReversedComposed, &'static str)> {
    let s = "string".to_string();
    let addr = [1u8; 32];
    let b = [2u32; 4];

    vec![(
        ReversedComposed {
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
    )]
}
