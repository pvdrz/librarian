#![feature(const_generics)]
#![feature(const_generic_impls_guard)]
#![allow(incomplete_features)]

use serde::{Serialize, Deserialize};

use anyhow::{Context, Result};

use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};

pub mod dbus;
mod doc;
mod text;

use text::Indices;

use doc::{deserialize_docs, serialize_docs, Doc, DocId};

#[derive(Deserialize, Serialize)]
pub struct Library {
    #[serde(deserialize_with = "deserialize_docs")]
    #[serde(serialize_with = "serialize_docs")]
    docs: BTreeMap<DocId, Doc>,
    root: PathBuf,
    #[serde(skip)]
    indices: Indices,
    #[serde(skip)]
    last: usize,
}

impl Library {
    pub fn from_file(path: &Path) -> Result<Self> {
        let file = File::open(path).context("Could not open index file")?;
        let mut library: Library =
            serde_json::from_reader(file).context("Could not deserialize index contents")?;
        for (id, doc) in &library.docs {
            library.indices.insert(*id, doc);
        }
        library.last = library.docs.len();
        Ok(library)
    }

    pub fn search(&self, text: &str) -> impl Iterator<Item = DocId> {
        self.indices.search(text, 10).into_iter().map(|(id, _)| id)
    }

    pub fn get(&self, id: DocId) -> &Doc {
        &self.docs[&id]
    }

    pub fn open(&self, id: DocId) -> Result<()> {
        let name = &self.get(id).filename;
        let path = self.root.join(name);
        open::that(path)?;
        Ok(())
    }

    pub fn insert(&mut self, doc: Doc) -> Result<()> {
        let id = DocId(self.last);
        self.last += 1;
        self.indices.insert(id, &doc);
        self.docs.insert(id, doc);

        let mut file = File::open(self.root.join("index.json"))?;
        serde_json::to_writer(&mut file, self)?;
        Ok(())
    }

    pub fn remove(&mut self, id: DocId) -> Result<()> {
        self.indices.remove(id);
        self.docs.remove(&id);

        let mut file = File::open(self.root.join("index.json"))?;
        serde_json::to_writer(&mut file, self)?;
        Ok(())
    }
}
