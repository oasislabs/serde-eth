# Serde ETH
[![CircleCI](https://circleci.com/gh/oasislabs/serde-eth.svg?style=svg)](https://circleci.com/gh/oasislabs/serde-eth)
serde-eth is a [serde](https://serde.rs) data format for the [Ethereum ABI](https://solidity.readthedocs.io/en/develop/abi-spec.html)

## Usage

Start by adding the following dependency to your `Cargo.toml`.

```toml
[dependencies]
serde-eth = "0.1"
```


serde-eth allows for easy serialization and deserialization of Rust types into/from eth abi.

### Encoding and Decoding

```rust
use serde::{Deserialize, Serialize};
use serde_eth::Result;

#[derive(Serialize, Deserialize, PartialEq)]
struct Person {
  name: String,
  lastname: String,
  address: String,
  age: u32,
  phones: Vec<String>,
}

fn example() {
  let person = Person{
    name: "John",
    lastname: "Smith",
    address: "MyAddress",
    age: 33,
    phones: vec!["1234", "5678"],
  };
  
  // encode into a Vec<u8> of a ut8 encoded string
  let vec = serde_eth::to_vec(person).unwrap();
  
  // eth abi is defined as a utf8 encoded hex string of
  // the binary representation of the types
  let s = String::from_utf8(vec).unwrap();
  
  serde_eth::from_str(&s).unwrap();
  assert_eq!(p, result);
}
```
