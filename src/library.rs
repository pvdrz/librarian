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
                isbn,
            } => {
                let (title, authors) = match (isbn, title) {
                    (Some(isbn), None) => get_info(&isbn)?,
                    (None, Some(title)) => (title, authors),
                    _ => unreachable!(),
                };
                self.store(file, title, authors, keywords)
            }
            Command::Find { title } => self.find(title),

            Command::Open { hash } => self.open(hash),
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

        let book_json =
            serde_json::to_string_pretty(&book).context("Could not serialize book as JSON")?;

        ensure!(
            self.books.insert(hash, book).is_none(),
            "Book is already in the library"
        );

        copy(file, path).context("Could not copy file to library")?;

        println!(
            "Added book: {}\n with hash: {}",
            book_json,
            serde_json::to_string(&hash).context("Could not serialize book hash")?
        );

        Ok(())
    }

    fn find(&self, title: String) -> Result<()> {
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

    fn open(&self, hash_str: String) -> Result<()> {
        let mut hash = [0; 16];
        hex::decode_to_slice(&hash_str, &mut hash)
            .expect("bug: hash could not be decoded in 16 bytes array");
        let hash = hash.into();
        let book = self
            .books
            .get(&hash)
            .ok_or_else(|| anyhow!("Book with hash {} not found", hash_str))?;
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

fn get_info(isbn: &str) -> Result<(String, Vec<String>)> {
    let isbn = format!(
        "ISBN:{}",
        isbn.chars().filter(|&c| c != '-').collect::<String>()
    );
    let resp = ureq::get("https://openlibrary.org/api/books")
        .query("bibkeys", &isbn)
        .query("jscmd", "data")
        .query("format", "json")
        .call()
        .into_reader();
    let resp = serde_json::from_reader::<_, serde_json::Value>(resp)
        .context("Could not deserialize book info from API")?
        .get(&isbn)
        .ok_or_else(|| anyhow!("Book with {} not found at Open Library", &isbn))?
        .clone();
    let title = resp
        .get("title")
        .expect("bug: unexpected JSON structure")
        .as_str()
        .expect("bug: unexpected JSON structure")
        .to_owned();
    let authors = resp
        .get("authors")
        .expect("bug: unexpected JSON structure")
        .as_array()
        .expect("bug: unexpected JSON structure")
        .into_iter()
        .map(|j| {
            j.get("name")
                .expect("bug: unexpected JSON structure")
                .as_str()
                .expect("bug: unexpected JSON structure")
                .to_owned()
        })
        .collect();

    Ok((title, authors))
}
