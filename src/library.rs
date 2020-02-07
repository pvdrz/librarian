use anyhow::{anyhow, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::fs::{copy, read, File};
use std::path::{Path, PathBuf};

use crate::book::{Book, BookHash, Extension};
use crate::cmd::Command;

#[derive(Serialize, Deserialize)]
pub struct Library {
    books: BTreeMap<BookHash, Book>,
    root: PathBuf,
}

impl Library {
    pub fn with_root(root: PathBuf) -> Self {
        Library {
            books: BTreeMap::new(),
            root,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        serde_json::from_reader(File::open(path).context("Could not open index file")?)
            .context("Could not deserialize index contents")
    }

    pub fn persist(&self, path: &Path) -> Result<()> {
        serde_json::to_writer(
            File::create(path).context("Could not create index file")?,
            self,
        )
        .context("Could not serialize index as JSON")
    }

    pub fn run_command(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::Store {
                file,
                title,
                authors,
                keywords,
            } => self.store(file, title, authors, keywords),
            Command::Search { title } => self.search(title),

            Command::Open { id } => self.open(id),
        }
    }

    fn store(
        &mut self,
        file: String,
        title: String,
        authors: Vec<String>,
        keywords: Vec<String>,
    ) -> Result<()> {
        let file = PathBuf::from(file);
        let extension = Extension::from_str(
            &file
                .extension()
                .ok_or_else(|| anyhow!("File {:?} has no extension", file))?
                .to_str()
                .ok_or_else(|| anyhow!("Extension is not valid unicode"))?,
        )?;

        let hash: BookHash = md5::compute(read(&file).context("Could not read book file")?)
            .0
            .into();

        let path = self.path(hash, extension);

        let book = Book {
            title,
            authors,
            keywords,
            extension,
        };

        ensure!(
            self.books.insert(hash, book).is_none(),
            "Book is already in the library"
        );

        copy(file, path)
            .map(|_| ())
            .context("Could not copy file to library")
    }

    fn search(&self, title: String) -> Result<()> {
        let title = title.to_lowercase();
        let books: Vec<_> = self
            .books
            .iter()
            .filter(|(_, book)| book.title.to_lowercase().contains(&title))
            .collect();

        println!(
            "{}",
            serde_json::to_string_pretty(&books)
                .context("Could not serialize search results as JSON")?
        );
        Ok(())
    }

    fn open(&self, id: String) -> Result<()> {
        let mut hash = [0; 16];
        hex::decode_to_slice(&id, &mut hash)
            .expect("bug: hash could not be decoded in 16 bytes array");
        let hash = hash.into();
        let book = self
            .books
            .get(&hash)
            .ok_or_else(|| anyhow!("Book with id {} not found", id))?;
        open::that(self.path(hash, book.extension)).context("Could not open book")?;
        Ok(())
    }

    fn path(&self, hash: BookHash, extension: Extension) -> PathBuf {
        let mut path = hex::encode(&<[u8; 16]>::from(hash));
        path += ".";
        path += extension.to_str();
        self.root.join(path)
    }
}
