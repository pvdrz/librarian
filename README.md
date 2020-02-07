# Librarian

This is a tiny command-line application to manage your digital library.

## Why?

I never found a satisfying way of storing and searching my digital library.
Using the documents metadata sounded like a good idea, but then I realized that
editing the metadata for the different file formats would require having more
than one application to do it. So instead I wrote this small app to handle it.

## How?

- Librarian keeps an index with all your books metadata at
  `~/.library/index.json`.

- When you store a book using `librarian store`, the file is copied to the
  `~/.library` folder and the metadata is added to the index file. Each book is
  indexed using the hash of the file.

- Then you can search in your library using `librarian find`.

- Once you found the book, you can open it using `librarian open` with the
  book's hash. This is equivalent to using `open` or `xdg-open`

For more help, run `librarian help`.

## Installation

Clone this repository, build it using `cargo build --release` and put the
binary in your `$PATH`.

## Features

This is just a MVP version. I haven't implemented a lot of functionality I'd
like to have. Currently Librarian can:

- Store documents with a title, multiple authors and multiple keywords.
- Find documents doing a case-insensitive substring search over the title.
- Open documents using your default application for each format.
