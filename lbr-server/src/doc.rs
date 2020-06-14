use serde::{Deserialize, Deserializer, Serialize};

use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc {
    pub title: String,
    pub authors: Vec<String>,
    pub keywords: Vec<String>,
    pub filename: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct DocId(usize);

impl FromStr for DocId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(DocId(s.parse()?))
    }
}

impl ToString for DocId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub fn deserialize_docs<'de, D>(deserializer: D) -> Result<BTreeMap<DocId, Doc>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Vec::deserialize(deserializer)?
        .into_iter()
        .enumerate()
        .map(|(i, doc)| (DocId(i), doc))
        .collect())
}
