use serde::{Deserialize, Serialize};

use anyhow::{Context, Result};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;

pub mod dbus;
mod doc;
mod text;

use text::Indices;

use doc::{Doc, DocId};

#[derive(Serialize, Deserialize)]
pub struct Library {
    docs: HashMap<DocId, Doc>,
    root: PathBuf,
    #[serde(skip)]
    indices: Indices,
}

impl Library {
    pub fn from_file(path: &Path) -> Result<Self> {
        let file = File::open(path).context("Could not open index file")?;
        let mut library: Library =
            serde_json::from_reader(file).context("Could not deserialize index contents")?;
        for (id, doc) in &library.docs {
            library.indices.insert(
               * id, doc);
        }
        Ok(library)
    }

    pub fn search(&self, text: &str) -> impl Iterator<Item = DocId> {
        self.indices.search(text, 5).into_iter().map(|(id, _)| id)
    }

    pub fn get(&self, id: &DocId) -> &Doc {
        &self.docs[id]
    }
}

#[test]
fn it_works() {
    let mut library = Library::from_file(&PathBuf::from("/home/christian/MEGAsync/Books/index.json")).unwrap();

    for (id, score) in library.indices.search("typ", 10) {
        println!("{:?} {}", library.docs[&id], score);
    }
}
