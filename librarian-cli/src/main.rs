use anyhow::{anyhow, Context, Result};

use serde::{Deserialize, Serialize};

use dbus::blocking::{Connection, Proxy};

use std::path::PathBuf;
use std::time::Duration;

use librarian_core::{Doc, DocHash};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    about = "\"Each one of these souls is finite and precious. And I'm close... Close to saving them all.\""
)]
pub enum Command {
    #[structopt(about = "Adds a new document into the library")]
    Add {
        #[structopt(help = "Path to the document to be added")]
        path: String,
    },
}

impl Command {
    fn run(self, proxy: &Proxy<&Connection>) -> Result<()> {
        match self {
            Command::Add { path } => Self::add(path, proxy),
        }
    }

    fn add(path: String, proxy: &Proxy<&Connection>) -> Result<()> {
        let path = PathBuf::from(path);
        let bytes = std::fs::read(&path)?;

        let extension = path
            .extension()
            .ok_or_else(|| anyhow!("Path {:?} has no extension", path))?
            .to_str()
            .ok_or_else(|| anyhow!("Extension is not valid unicode"))?
            .to_lowercase();

        let hash = DocHash::from_bytes(&bytes);

        let data = DocData::from_scrawl()?;

        let doc = serde_json::to_string(&Doc {
            title: data.title,
            authors: data.authors,
            keywords: data.keywords,
            hash,
            extension,
            show: true,
        })?;

        let path = path
            .to_str()
            .ok_or_else(|| anyhow!("Path {:?} is not valid unicode", path))?;

        proxy.method_call("lbr.cli", "Insert", (doc, path))?;

        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize)]
struct DocData {
    title: String,
    authors: Vec<String>,
    keywords: Vec<String>,
}

impl DocData {
    fn from_scrawl() -> Result<Self> {
        let text = serde_json::to_string_pretty(&Self::default()).unwrap();
        serde_json::from_str(&scrawl::with(&text)?).context("cannot deserialize document from JSON")
    }
}

fn main() -> Result<()> {
    let cmd = Command::from_args();

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy("lbr.server", "/lbr/server/cli", Duration::from_millis(5000));

    cmd.run(&proxy)
    // let (names,): (Vec<String>,) = proxy.method_call("lbr.cli", "ListNames", ())?;

    // Let's print all the names to stdout.
    // for name in names { println!("{}", name); }
}
