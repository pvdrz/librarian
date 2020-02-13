use anyhow::{anyhow, bail, ensure, Context, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, BTreeSet};
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
            Command::Add { file, isbn } => self.add(file, isbn),
            Command::Find { pattern, open } => self.find(pattern, open),
            Command::Open { hash } => self.open(hash),
            Command::Edit { hash } => self.edit(hash),
            Command::List => self.list(),
        }
    }

    fn add(&mut self, file: String, isbn: Option<String>) -> Result<()> {
        let file = PathBuf::from(file);

        let hash: BookHash = blake3::hash(&read(&file).context("Could not read file")?).into();

        if self.books.contains_key(&hash) {
            bail!(
                "Document with hash {} already exists",
                serde_json::to_string(&hash).unwrap()
            );
        }

        let extension = file
            .extension()
            .ok_or_else(|| anyhow!("File {:?} has no extension", file))?
            .to_str()
            .ok_or_else(|| anyhow!("Extension is not valid unicode"))?
            .to_lowercase();

        let path = self.path(hash, &extension);

        let mut book = Book {
            title: String::new(),
            authors: BTreeSet::new(),
            keywords: BTreeSet::new(),
            extension,
        };

        if let Some(isbn) = isbn {
            book.set_info_from_api(&isbn)?;
        }

        let book_json = book.edit()?;

        assert!(self.books.insert(hash, book).is_none(),);

        copy(file, path).context("Could not copy file to library")?;

        println!(
            "Added document: {}\n with hash: {}",
            book_json,
            serde_json::to_string(&hash).expect("Bug: Could not serialize document hash")
        );

        Ok(())
    }

    fn list(&self) -> Result<()> {
        show_json(&self.books);
        Ok(())
    }

    fn find(&self, pattern: String, open: bool) -> Result<()> {
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

        if open {
            if let Some(&(&hash, book)) = books.get(0) {
                println!("Opening book {}", serde_json::to_string(&hash).unwrap());
                open::that(self.path(hash, &book.extension)).context("Could not open document")?;
            } else {
                bail!("Search did not return any results");
            }
        } else {
            show_json(&books);
        }

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

    fn edit(&mut self, hash_str: String) -> Result<()> {
        let hash = self.get_hash(&hash_str)?;

        let book = self
            .books
            .get_mut(&hash)
            .ok_or_else(|| anyhow!("Document with hash {} not found", hash_str))?;

        let book_json = book.edit()?;

        println!("Updated document {}: {}", hash_str, book_json);

        Ok(())
    }

    fn path(&self, hash: BookHash, extension: &str) -> PathBuf {
        let mut path = hex::encode(&<[u8; 32]>::from(hash));
        path += ".";
        path += extension;
        self.root.join(path)
    }
}

fn show_json<S: Serialize>(s: &S) {
    println!(
        "{}",
        serde_json::to_string_pretty(&s)
            .expect("Bug: Could not serialize list of documents as JSON")
    );
}
