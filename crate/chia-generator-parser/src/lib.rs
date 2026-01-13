pub mod error;
pub mod parser;
pub mod types;

pub use error::*;
use hex::FromHexError;
pub use parser::*;
pub use types::*;

pub fn string_to_bytes32(string: &String) -> Result<Bytes32> {
    let string_vec = string.as_bytes();
    if string_vec.len() != 32 {
        return Err(GeneratorParserError::HexDecodingError(
            FromHexError::InvalidStringLength,
        ));
    }
    Bytes32::try_from(string_vec)
        .map_err(|_| GeneratorParserError::HexDecodingError(FromHexError::InvalidStringLength))
}
