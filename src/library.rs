use anyhow::{anyhow, bail, ensure, Context, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::fs::{copy, read, File};
use std::path::{Path, PathBuf};

use crate::book::{Book, BookHash};
use crate::cmd::Command;

#[derive(Serialize, Deserialize)]
pub struct Library {
    books: BTreeMap<BookHash, Book>,
    root: PathBuf,
}

impl Library {
    pub fn get_hash(&self, hash_str: &str) -> Result<BookHash> {
        let str_len = hash_str.len();

        if str_len < 64 {
            use std::ops::Bound::Included;
            let mut bot_hash = [0; 32];
            let mut top_hash = [255; 32];

            let mut hash_len = (str_len) / 2;
            if str_len % 2 == 0 {
                hex::decode_to_slice(&hash_str, &mut top_hash[..hash_len])
            } else {
                hash_len = (1 + str_len) / 2;
                let b = &mut top_hash[hash_len - 1];
                *b = b.wrapping_sub(15);
                hex::decode_to_slice(&(hash_str.to_owned() + "f"), &mut top_hash[..hash_len])
            }
            .context("Invalid hash")?;

            for i in 0..hash_len {
                bot_hash[i] += top_hash[i];
            }

            let mut range = self.books.range((
                Included(BookHash::from(bot_hash)),
                Included(BookHash::from(top_hash)),
            ));

            if let Some((&hash, _)) = range.next() {
                ensure!(
                    range.next().is_none(),
                    "Hash collision, please use a longer prefix"
                );
                Ok(hash)
            } else {
                bail!("Hash not found")
            }
        } else if str_len == 64 {
            let mut hash = [0; 32];
            hex::decode_to_slice(&hash_str, &mut hash).context("Invalid hash")?;
            let hash = BookHash::from(hash);
            ensure!(self.books.contains_key(&hash), "Hash not found");
            Ok(hash)
        } else {
            bail!("Hash is longer than expected")
        }
    }

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
            Command::Find { pattern } => self.find(pattern),
            Command::Open { hash } => self.open(hash),
            Command::UpdateAdd {
                hash,
                authors,
                keywords,
            } => self.update_add(hash, authors, keywords),
            Command::Update {
                hash,
                title,
                authors,
                keywords,
            } => self.update(hash, title, authors, keywords),
            Command::List => self.list(),
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
        let extension = file
            .extension()
            .ok_or_else(|| anyhow!("File {:?} has no extension", file))?
            .to_str()
            .ok_or_else(|| anyhow!("Extension is not valid unicode"))?
            .to_lowercase();

        let hash: BookHash = blake3::hash(&read(&file).context("Could not read file")?).into();

        let path = self.path(hash, &extension);

        let book = Book {
            title,
            authors: authors.into_iter().collect(),
            keywords: keywords.into_iter().collect(),
            extension,
        };

        let book_json = serde_json::to_string_pretty(&book)
            .context("Could not serialize document information as JSON")?;

        ensure!(
            self.books.insert(hash, book).is_none(),
            "Document is already in the library"
        );

        copy(file, path).context("Could not copy file to library")?;

        println!(
            "Added document: {}\n with hash: {}",
            book_json,
            serde_json::to_string(&hash).context("Could not serialize document hash")?
        );

        Ok(())
    }

    fn list(&self) -> Result<()> {
        println!(
            "{}",
            serde_json::to_string_pretty(&self.books)
                .context("Could not serialize search results as JSON")?
        );

        Ok(())
    }

    fn find(&self, pattern: String) -> Result<()> {
        let matcher = SkimMatcherV2::default();
        let mut scores = BTreeMap::new();
        let mut books: Vec<_> = self
            .books
            .iter()
            .filter_map(|(hash, book)| {
                let mut score = matcher.fuzzy_match(&book.title, &pattern).unwrap_or(0);
                for author in &book.authors {
                    score += matcher.fuzzy_match(author, &pattern).unwrap_or(0);
                }
                for keyword in &book.keywords {
                    score += matcher.fuzzy_match(keyword, &pattern).unwrap_or(0);
                }
                if score == 0 {
                    return None;
                }
                scores.insert(hash, -(score as isize));
                Some((hash, book))
            })
            .collect();

        books.sort_by_key(|(hash, _)| scores[hash]);

        println!(
            "{}",
            serde_json::to_string_pretty(&books)
                .context("Could not serialize search results as JSON")?
        );

        Ok(())
    }

    fn open(&self, hash_str: String) -> Result<()> {
        let hash = self.get_hash(&hash_str)?;
        let book = self
            .books
            .get(&hash)
            .ok_or_else(|| anyhow!("Document with hash {} not found", hash_str))?;
        open::that(self.path(hash, &book.extension)).context("Could not open document")?;
        Ok(())
    }

    fn update_add(
        &mut self,
        hash_str: String,
        authors: Vec<String>,
        keywords: Vec<String>,
    ) -> Result<()> {
        let hash = self.get_hash(&hash_str)?;

        let book = self
            .books
            .get_mut(&hash)
            .ok_or_else(|| anyhow!("Document with hash {} not found", hash_str))?;

        for author in authors {
            book.authors.insert(author);
        }

        for keyword in keywords {
            book.keywords.insert(keyword);
        }

        println!(
            "Updated book {}: {}",
            hash_str,
            serde_json::to_string_pretty(&book).context("Could not serialize book as JSON")?
        );

        Ok(())
    }

    fn update(
        &mut self,
        hash_str: String,
        title: Option<String>,
        authors: Option<Vec<String>>,
        keywords: Option<Vec<String>>,
    ) -> Result<()> {
        let hash = self.get_hash(&hash_str)?;

        let book = self
            .books
            .get_mut(&hash)
            .ok_or_else(|| anyhow!("Document with hash {} not found", hash_str))?;

        if let Some(title) = title {
            book.title = title;
        }

        if let Some(authors) = authors {
            book.authors = authors.into_iter().collect();
        }

        if let Some(keywords) = keywords {
            book.keywords = keywords.into_iter().collect();
        }

        println!(
            "Updated book {}: {}",
            hash_str,
            serde_json::to_string_pretty(&book).context("Could not serialize book as JSON")?
        );

        Ok(())
    }

    fn path(&self, hash: BookHash, extension: &str) -> PathBuf {
        let mut path = hex::encode(&<[u8; 32]>::from(hash));
        path += ".";
        path += extension;
        self.root.join(path)
    }
}

fn get_info(isbn: &str) -> Result<(String, Vec<String>)> {
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
