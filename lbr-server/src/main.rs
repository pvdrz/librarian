use lbr_server::dbus::run;
use lbr_server::Library;

use std::path::PathBuf;

fn main() {
    let lib =
        Library::from_file(&PathBuf::from("/home/christian/MEGAsync/Books/index.json")).unwrap();
    run(lib).unwrap();
}
