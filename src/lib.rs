mod custom_de;
mod custom_ser;
pub mod de;
mod error;
mod eth;
pub mod ser;
mod serde_tests;

pub use ser::{to_string, to_vec, to_writer};

pub use de::{from_reader, from_str};
