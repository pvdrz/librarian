use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub authors: BTreeSet<String>,
    pub extension: String,
    pub keywords: BTreeSet<String>,
}

impl Book {
    pub fn set_info_from_api(&mut self, isbn: &str) -> Result<()> {
        let isbn = format!(
            "ISBN:{}",
            isbn.chars()
                .filter(|&c| c.is_numeric() || c == 'X')
                .collect::<String>()
        );

        let resp = ureq::get("https://openlibrary.org/api/books")
            .query("bibkeys", &isbn)
            .query("jscmd", "data")
            .query("format", "json")
            .call()
            .into_reader();

        let resp = serde_json::from_reader::<_, serde_json::Value>(resp)
            .context("Could not deserialize document information from the API")?
            .get(&isbn)
            .ok_or_else(|| anyhow!("Document with {} not found at Open Library", &isbn))?
            .clone();

        self.title = resp
            .get("title")
            .map(|value| {
                value
                    .as_str()
                    .expect("Malformed response from API, title is not a string")
                    .to_owned()
            })
            .unwrap_or_else(|| String::new());

        self.authors = resp
            .get("authors")
            .map(|value| {
                value
                    .as_array()
                    .expect("Malformed response from API, authors are not an array")
                    .into_iter()
                    .map(|j| {
                        j.get("name")
                            .expect("Malformed response from API: author does not have a name")
                            .as_str()
                            .expect("Malformed response from API: author's name is not a string")
                            .to_owned()
                    })
                    .collect()
            })
            .unwrap_or_else(|| BTreeSet::new());

        Ok(())
    }

    pub fn edit(&mut self) -> Result<String> {
        let text =
            serde_json::to_string_pretty(&self).context("cannot serialize document to JSON")?;
        let text = scrawl::with(&text)?;
        *self = serde_json::from_str(&text).context("cannot deserialize document from JSON")?;
        Ok(text)
    }
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
