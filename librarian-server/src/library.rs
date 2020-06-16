use serde::{Deserialize, Serialize};

use anyhow::{anyhow, Context, Result};

use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::fmt;

use librarian_core::Doc;

use crate::text::SearchEngine;

#[derive(Deserialize, Serialize)]
pub(crate) struct Library {
    docs: BTreeMap<DocId, Doc>,
    last: usize,
    #[serde(skip)]
    path: PathBuf,
    #[serde(skip)]
    engine: SearchEngine,
}

impl Library {
    pub(crate) fn from_file() -> Result<Self> {
        let path = PathBuf::from(std::env::var("LBRPATH")?);
        let file = File::open(&path.join("index.json")).context("Could not open index file")?;
        let mut library: Library =
            serde_json::from_reader(file).context("Could not deserialize index contents")?;
        for (&id, doc) in &library.docs {
            if doc.show {
                library.engine.index(id, doc);
            }
        }
        library.path = path;
        Ok(library)
    }

    pub(crate) fn search(&self, text: &str) -> impl Iterator<Item = DocId> {
        self.engine.search(text, 10).into_iter().map(|(id, _)| id)
    }

    pub(crate) fn get(&self, id: DocId) -> Result<&Doc> {
        self.docs
            .get(&id)
            .and_then(|doc| if doc.show { Some(doc) } else { None })
            .ok_or_else(|| anyhow!("couldn't find document with id {}", id))
    }

    pub(crate) fn get_mut(&mut self, id: DocId) -> Result<&mut Doc> {
        self.docs
            .get_mut(&id)
            .and_then(|doc| if doc.show { Some(doc) } else { None })
            .ok_or_else(|| anyhow!("couldn't find document with id {}", id))
    }

    pub(crate) fn open(&self, id: DocId) -> Result<()> {
        let name = &self.get(id)?.filename();
        let path = self.path.join(name);
        open::that(path)?;
        Ok(())
    }

    pub(crate) fn insert(&mut self, doc: Doc, path: &str) -> Result<()> {
        let id = self.new_id();

        let new_path = self.path.join(&doc.filename());

        std::fs::copy(path, new_path)?;

        self.docs.insert(id, doc);
        self.persist()
    }

    pub(crate) fn remove(&mut self, id: DocId) -> Result<()> {
        self.get_mut(id)?.show = false;
        self.persist()
    }

    fn persist(&self) -> Result<()> {
        let mut file = File::create(self.path.join("index.json"))?;
        serde_json::to_writer(&mut file, self)?;
        Ok(())
    }

    fn new_id(&mut self) -> DocId {
        let id = DocId(self.last);
        self.last += 1;
        id
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
