use anyhow::{Result, anyhow};

use dbus::{
    tree::{Factory, Interface, MTFn, Method},
    Error,
};

use std::sync::{Mutex, Arc};

use crate::library::Library;

pub(super) fn create_interface(lib: Arc<Mutex<Library>>) -> Interface<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.interface("lbr.edit", ())
        .add_m(create_insert(lib.clone()))
}

fn create_insert(lib: Arc<Mutex<Library>>) -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();
    fact.method("Insert", (), move |m| {
        let doc_str: &str = m.msg.read1()?;

        insert(lib.clone(), doc_str)
            .map_err(|err| Error::new_failed(&err.to_string()))?;

        Ok(vec![])
    })
    .inarg::<&str, _>("doc")
}

fn insert(lib: Arc<Mutex<Library>>, doc_str: &str) -> Result<()> {
    let mut lib = lib.lock().map_err(|e| anyhow!("Failed to lock: {}", e))?;
    let doc = serde_json::from_str(doc_str)?;
    lib.insert(doc)
}
