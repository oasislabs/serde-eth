# Serde ETH
serde-eth is a [serde](https://serde.rs) data format for the [Ethereum ABI](https://solidity.readthedocs.io/en/develop/abi-spec.html)

---

```toml
[dependencies]
serde-eth = "0.1"
```

The ETH abi is the abi defined by ethereum found [here](https://solidity.readthedocs.io/en/develop/abi-spec.html).

serde-eth allows for easy serialization and deserialization of Rust types into/from eth abi.

## Encoding and Decoding

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

fn encoding(person: Person) -> Result<Vec<u8>> {
  serde_eth::to_vec(person)
}

fn decoding(vec: Vec<u8>) -> Result<Person> {
  // eth abi is defined as a utf8 encoded hex string of
  // the binary representation of the types
  let s = String::from_utf8(vec).unwrap();
  serde_eth::from_str(&s)
}

fn example() {
  let p = Person{
    name: "John",
    lastname: "Smith",
    address: "MyAddress",
    age: 33,
    phones: vec!["1234", "5678"],
  };
  
  let v = encoding(p).unwrap();
  let result = decoding(v).unwrap();
  
  assert_eq!(p, result);
}
```
