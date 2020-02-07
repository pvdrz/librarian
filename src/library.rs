use serde::{Serialize, Deserialize};

use std::collections::BTreeMap;
use std::fs::{copy, read, File};
use std::path::{Path, PathBuf};

use crate::book::{Book, Extension, BookHash};
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

    pub fn from_file(path: &Path) -> Self {
        serde_json::from_reader(File::open(path).unwrap()).unwrap()
    }

    pub fn persist(&self, path: &Path) {
        serde_json::to_writer(File::create(path).unwrap(), self).unwrap()
    }

    pub fn run_command(&mut self, cmd: Command) {
        match cmd {
            Command::Store{ file, title, authors, keywords } => self.store(file, title, authors, keywords),
            Command::Search { title } => println!("{}", serde_json::to_string_pretty(&self.search(title)).unwrap()),
            Command::Open { id } => self.open(id),
        }
    }

    fn store(&mut self, file: String, title: String, authors: Vec<String>, keywords: Vec<String>) {
        let file = PathBuf::from(file);
        let extension = Extension::from_str(&file.extension().unwrap().to_str().unwrap());

        let hash = md5::compute(read(&file).unwrap());

        let mut path = format!("{:x}", hash);
        path += ".";
        path += extension.to_str();
        let path = self.root.join(path);

        copy(file, path).unwrap();

        let book = Book {
            title,
            authors,
            keywords,
            extension,
        };

        assert!(self.books.insert(BookHash::from(hash.0), book).is_none());
    }

    fn search(&self, title: String) -> Vec<(&BookHash, &Book)> {
        let title = title.to_lowercase();
        self.books.iter().filter(|(_, book)| {
            book.title.to_lowercase().contains(&title)
        }).collect()
    }

    fn open(&self, id: String) {
        let mut hash = [0; 16];
        hex::decode_to_slice(id, &mut hash).unwrap();
        let hash = hash.into();
        let book = self.books.get(&hash).unwrap();
        open::that(self.path(hash, book.extension)).unwrap();

    }

    fn path(&self, hash: BookHash, extension: Extension) -> PathBuf {
        let mut path = hex::encode(&<[u8; 16]>::from(hash));
        path += ".";
        path += extension.to_str();
        self.root.join(path)
    }
}

