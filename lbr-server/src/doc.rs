use serde::{Deserialize, Serialize};

use std::str::FromStr;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc {
    pub title: String,
    pub authors: Vec<String>,
    pub keywords: Vec<String>,
    pub extension: String,
    pub show: bool,
    pub hash: DocHash,
}

impl Doc {
    pub(crate) fn filename(&self) -> String {
        self.hash.to_string() + "." + &self.extension
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DocHash(#[serde(with = "hex")] [u8; 32]);

impl DocHash {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        DocHash(blake3::hash(bytes).into())
    }
}

impl FromStr for DocHash {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; 32];
        hex::decode_to_slice(s, &mut bytes)?;
        Ok(DocHash(bytes))
    }
}

impl fmt::Display for DocHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DocId(pub(crate) usize);

impl FromStr for DocId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DocId(s.parse()?))
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
