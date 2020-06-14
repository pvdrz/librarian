
use serde::{Serialize, Deserialize};

use anyhow::{Context, Result};

use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

use crate::text::SearchEngine;
use crate::doc::{deserialize_docs, serialize_docs, Doc, DocId};

#[derive(Deserialize, Serialize)]
pub(crate) struct Library {
    #[serde(skip)]
    path: PathBuf,
    #[serde(deserialize_with = "deserialize_docs")]
    #[serde(serialize_with = "serialize_docs")]
    docs: BTreeMap<DocId, Doc>,
    #[serde(skip)]
    engine: SearchEngine,
    #[serde(skip)]
    last: usize,
}

impl Library {
    pub(crate) fn from_file() -> Result<Self> {
        let path = PathBuf::from(std::env::var("LBRPATH")?);
        let file = File::open(&path).context("Could not open index file")?;
        let mut library: Library =
            serde_json::from_reader(file).context("Could not deserialize index contents")?;
        for (id, doc) in &library.docs {
            library.engine.insert(*id, doc);
        }
        library.last = library.docs.len();
        library.path = path;
        Ok(library)
    }

    pub(crate) fn search(&self, text: &str) -> impl Iterator<Item = DocId> {
        self.engine.search(text, 10).into_iter().map(|(id, _)| id)
    }

    pub(crate) fn get(&self, id: DocId) -> &Doc {
        &self.docs[&id]
    }

    pub(crate) fn open(&self, id: DocId) -> Result<()> {
        let name = &self.get(id).filename;
        let path = self.path.join(name);
        open::that(path)?;
        Ok(())
    }

    pub(crate) fn insert(&mut self, doc: Doc) -> Result<()> {
        let id = DocId(self.last);
        self.last += 1;
        self.engine.insert(id, &doc);
        self.docs.insert(id, doc);
        self.write()
    }

    pub(crate) fn remove(&mut self, id: DocId) -> Result<()> {
        self.engine.remove(id);
        self.docs.remove(&id);
        self.write()
    }

    fn write(&self) -> Result<()> {
        let mut file = File::open(self.path.join("index.json"))?;
        serde_json::to_writer(&mut file, self)?;
        Ok(())
    }
}
