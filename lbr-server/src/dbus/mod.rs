use anyhow::Result;

use dbus::{blocking::LocalConnection, tree::Factory};

use std::sync::{Mutex, Arc};
use std::time::Duration;

use crate::library::Library;

mod edit;
mod search;

pub(crate) fn serve(lib: Library) -> Result<()> {
    let mut conn = LocalConnection::new_session()?;
    conn.request_name("lbr.server", false, true, false)?;

    let lib = Arc::new(Mutex::new(lib));

    let fact = Factory::new_fn::<()>();

    let search_interface = search::create_interface(lib.clone());
    let search_path = fact
        .object_path("/lbr/server/search", ())
        .add(search_interface);

    let edit_interface = edit::create_interface(lib);
    let edit_path = fact.object_path("/lbr/server/edit", ()).add(edit_interface);

    fact.tree(())
        .add(search_path)
        .add(edit_path)
        .start_receive(&conn);

    loop {
        conn.process(Duration::from_millis(1000))?;
    }
}
