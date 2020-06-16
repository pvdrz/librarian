#![feature(const_generics)]
#![feature(const_generic_impls_guard)]
#![allow(incomplete_features)]

use anyhow::Result;

mod dbus;
mod text;
mod library;

pub fn run() -> Result<()> {
    let lib = library::Library::from_file()?;
    dbus::serve(lib)
}
