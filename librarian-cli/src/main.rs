use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use serde::{Deserialize, Serialize};

use structopt::StructOpt;

use librarian_core::{Doc, DocHash};
use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.gnome.Shell.SearchProvider2",
    default_service = "lbr.server",
    default_path = "/lbr/server"
)]
trait Server {
    fn insert_document(&self, doc: &str, path: &str) -> zbus::Result<()> {
        let doc = serde_json::from_str(doc).map_err(|err| Error::Failed(err.to_string()))?;
        self.insert(doc, path)
            .map_err(|err| Error::Failed(err.to_string()))
    }
}

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
    fn run(self, proxy: &ServerProxy) -> Result<()> {
        match self {
            Command::Add { path } => Self::add(path, proxy),
        }
    }

    fn add(path: String, proxy: &ServerProxy) -> Result<()> {
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

        proxy.insert_document(&doc, &path)?;

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

    let connection = zbus::Connection::new_session()?;
    let proxy = ServerProxy::new(&connection)?;

    cmd.run(&proxy)
}
