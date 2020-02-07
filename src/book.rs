use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub authors: Vec<String>,
    pub extension: Extension,
    pub keywords: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Extension {
    Pdf,
    Mobi,
    Epub,
    Djvu,
}

impl Extension {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pdf" => Self::Pdf,
            "mobi" => Self::Mobi,
            "epub" => Self::Epub,
            "djvu" => Self::Djvu,
            _ => panic!("invalid extension {}", s),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Pdf => "pdf",
            Self::Mobi => "mobi",
            Self::Epub => "epub",
            Self::Djvu => "djvu",
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct BookHash([u8; 16]);

impl From<[u8; 16]> for BookHash {
    fn from(bytes: [u8; 16]) -> Self {
        BookHash(bytes)
    }
}

impl From<BookHash> for [u8; 16] {
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
        let mut bytes = [0; 16];
        hex::decode_to_slice(s, &mut bytes).map_err(D::Error::custom)?;
        Ok(BookHash(bytes))
    }
}
