use anyhow::{anyhow, Result};

use dbus::{
    arg::Variant,
    tree::{Factory, Interface, MTFn, Method},
    Error,
};

use std::collections::HashMap;
use std::sync::{Mutex, Arc};

use crate::library::Library;

pub(super) fn create_interface(lib: Arc<Mutex<Library>>) -> Interface<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.interface("org.gnome.Shell.SearchProvider2", ())
        .add_m(create_get_initial_result_set(lib.clone()))
        .add_m(create_get_subsearch_result_set(lib.clone()))
        .add_m(create_get_result_metas(lib.clone()))
        .add_m(create_activate_result(lib))
}

fn create_get_initial_result_set(lib: Arc<Mutex<Library>>) -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();
    fact.method("GetInitialResultSet", (), move |m| {
        let terms: Vec<&str> = m.msg.read1()?;

        let results = get_initial_result_set(Arc::clone(&lib), terms)
            .map_err(|err| Error::new_failed(&err.to_string()))?;

        let results = m.msg.method_return().append1(results);

        Ok(vec![results])
    })
    .outarg::<Vec<&str>, _>("results")
    .inarg::<Vec<&str>, _>("terms")
}

fn get_initial_result_set(lib: Arc<Mutex<Library>>, terms: Vec<&str>) -> Result<Vec<String>> {
    let lib = lib.lock().map_err(|e| anyhow!("Failed to lock: {}", e))?;

    Ok(lib
        .search(&terms.join(" "))
        .map(|id| id.to_string())
        .collect())
}

fn create_get_subsearch_result_set(lib: Arc<Mutex<Library>>) -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();
    fact.method("GetSubsearchResultSet", (), move |m| {
        let (previous_results, terms): (Vec<&str>, Vec<&str>) = m.msg.read2()?;

        let results = get_subsearch_result_set(Arc::clone(&lib), previous_results, terms)
            .map_err(|err| Error::new_failed(&err.to_string()))?;

        let results = m.msg.method_return().append1(results);

        Ok(vec![results])
    })
    .outarg::<Vec<&str>, _>("results")
    .inarg::<Vec<&str>, _>("previous_results")
    .inarg::<Vec<&str>, _>("terms")
}

fn get_subsearch_result_set<'a>(
    lib: Arc<Mutex<Library>>,
    _previous_results: Vec<&str>,
    terms: Vec<&str>,
) -> Result<Vec<String>> {
    get_initial_result_set(lib, terms)
}

fn create_get_result_metas(lib: Arc<Mutex<Library>>) -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.method("GetResultMetas", (), move |m| {
        let identifiers: Vec<&str> = m.msg.read1()?;

        let metas = get_result_metas(lib.clone(), identifiers)
            .map_err(|err| Error::new_failed(&err.to_string()))?;

        let metas = m.msg.method_return().append1(metas);

        Ok(vec![metas])
    })
    .outarg::<Vec<HashMap<&str, Variant<&str>>>, _>("metas")
    .inarg::<Vec<&str>, _>("identifiers")
}

fn get_result_metas(
    lib: Arc<Mutex<Library>>,
    identifiers: Vec<&str>,
) -> Result<Vec<HashMap<&str, Variant<String>>>> {
    let lib = lib.lock().map_err(|e| anyhow!("Failed to lock: {}", e))?;

    identifiers
        .into_iter()
        .map(|identifier| {
            let doc = lib.get(identifier.parse()?);
            let mut meta = HashMap::default();
            meta.insert("id", Variant(identifier.to_string()));
            meta.insert("name", Variant(doc.title.clone()));
            meta.insert("description", Variant(doc.authors[0].clone()));
            Ok(meta)
        })
        .collect()
}

fn create_activate_result(lib: Arc<Mutex<Library>>) -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.method("ActivateResult", (), move |m| {
        let (identifier, terms, timestamp): (&str, Vec<&str>, u32) = m.msg.read3()?;

        activate_result(lib.clone(), identifier, terms, timestamp)
            .map_err(|err| Error::new_failed(&err.to_string()))?;

        Ok(vec![])
    })
    .inarg::<&str, _>("identifier")
    .inarg::<Vec<&str>, _>("terms")
    .inarg::<u32, _>("timestamp")
}

fn activate_result(
    lib: Arc<Mutex<Library>>,
    identifier: &str,
    _terms: Vec<&str>,
    _timestamp: u32,
) -> Result<()> {
    let lib = lib.lock().map_err(|e| anyhow!("Failed to lock: {}", e))?;

    lib.open(identifier.parse()?)
}
