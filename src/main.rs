mod book;
mod cmd;
mod library;

use structopt::StructOpt;

use library::Library;

fn main() {
    let library_path = dirs::home_dir().unwrap().join(".library");

    let index_path = library_path.join("index.json");

    let mut lib = if index_path.exists() {
        Library::from_file(&index_path)
    } else {
        if !library_path.exists() {
            std::fs::create_dir(&library_path).unwrap();
        }
        Library::with_root(library_path)
    };

    let cmd = cmd::Command::from_args();
    lib.run_command(cmd);

    lib.persist(&index_path);
}
