use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc {
    pub title: String,
    pub authors: Vec<String>,
    pub keywords: Vec<String>,
    pub extension: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct DocId(#[serde(with = "hex_serde")] [u8; 32]);
