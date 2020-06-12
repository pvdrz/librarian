use anyhow::Result;

use dbus::{blocking::LocalConnection, tree::Factory};

use std::time::Duration;
use std::sync::Arc;

use crate::Library;

mod search;

pub fn run(lib: Library) -> Result<()> {
    let mut conn = LocalConnection::new_session()?;
    conn.request_name("lbr.server", false, true, false)?;

    let lib = Arc::new(lib);

    let fact = Factory::new_fn::<()>();

    let search_interface = search::create_interface(lib);
    let search_path = fact
        .object_path("/lbr/server/search", ())
        .add(search_interface);

    fact.tree(()).add(search_path).start_receive(&conn);

    loop {
        conn.process(Duration::from_millis(1000))?;
    }
}
