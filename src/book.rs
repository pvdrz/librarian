use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub authors: BTreeSet<String>,
    pub extension: String,
    pub keywords: BTreeSet<String>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct BookHash([u8; 32]);

impl From<[u8; 32]> for BookHash {
    fn from(bytes: [u8; 32]) -> Self {
        BookHash(bytes)
    }
}

impl From<blake3::Hash> for BookHash {
    fn from(hash: blake3::Hash) -> Self {
        BookHash(hash.into())
    }
}

impl From<BookHash> for [u8; 32] {
    fn from(hash: BookHash) -> Self {
        hash.0
    }
}

impl Serialize for BookHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = hex::encode(&self.0);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for BookHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s: String = Deserialize::deserialize(deserializer)?;
        let mut bytes = [0; 32];
        hex::decode_to_slice(s, &mut bytes).map_err(D::Error::custom)?;
        Ok(BookHash(bytes))
    }
}
