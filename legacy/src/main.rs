mod book;
mod cmd;
mod library;

use anyhow::{anyhow, Context, Result};
use structopt::StructOpt;

use library::Library;

fn main() -> Result<()> {
    let library_path = dirs::home_dir()
        .ok_or_else(|| anyhow!("Home directory not found"))?
        .join(".library");

    let index_path = library_path.join("index.json");

    let mut lib = if index_path.exists() {
        Library::from_file(&index_path)?
    } else {
        if !library_path.exists() {
            std::fs::create_dir(&library_path).with_context(|| {
                format!("Could not create library directory at {:?}", library_path)
            })?;
        }
        Library::with_root(library_path)
    };

    let cmd = cmd::Command::from_args();
    lib.run_command(cmd)?;

    lib.persist(&index_path)?;
    Ok(())
}
