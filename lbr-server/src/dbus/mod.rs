use anyhow::Result;

use dbus::{blocking::LocalConnection, tree::Factory};

use std::time::Duration;

mod search;

pub fn run() -> Result<()> {
    let mut conn = LocalConnection::new_session()?;
    conn.request_name("lbr.server", false, true, false)?;

    let fact = Factory::new_fn::<()>();

    let search_interface = search::create_interface();
    let search_path = fact
        .object_path("/lbr/server/search", ())
        .add(search_interface);

    fact.tree(()).add(search_path).start_receive(&conn);

    loop {
        conn.process(Duration::from_millis(1000))?;
    }
}
