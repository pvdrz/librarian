use anyhow::{anyhow, Result, Context};

use dbus::{
    arg::Variant,
    tree::{Factory, Interface, MTFn, Method},
    Error,
};

use std::collections::HashMap;
use std::sync::Arc;

use crate::Library;

pub(super) fn create_interface(lib: Arc<Library>) -> Interface<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.interface("org.gnome.Shell.SearchProvider2", ())
        .add_m(create_get_initial_result_set(lib.clone()))
        .add_m(create_get_subsearch_result_set(lib.clone()))
        .add_m(create_get_result_metas(lib))
}

fn create_get_initial_result_set(lib: Arc<Library>) -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();
    fact.method("GetInitialResultSet", (), move |m| {
        let terms: Vec<&str> = m.msg.read1()?;

        let hello =
            get_initial_result_set(Arc::clone(&lib), terms).map_err(|err| Error::new_failed(&err.to_string()))?;

        let results = m.msg.method_return().append1(hello);

        Ok(vec![results])
    })
    .outarg::<Vec<&str>, _>("results")
    .inarg::<Vec<&str>, _>("terms")
}

fn get_initial_result_set(lib: Arc<Library>, terms: Vec<&str>) -> Result<Vec<String>> {
    lib.search(&terms.join(" ")).map(|id| serde_json::to_string(&id).context("Could not serialize document ID")).collect()
}

fn create_get_subsearch_result_set(lib: Arc<Library>) -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();
    fact.method("GetSubsearchResultSet", (), move |m| {
        let (previous_results, terms): (Vec<&str>, Vec<&str>) = m.msg.read2()?;

        let hello = get_subsearch_result_set(Arc::clone(&lib), previous_results, terms)
            .map_err(|err| Error::new_failed(&err.to_string()))?;

        let results = m.msg.method_return().append1(hello);

        Ok(vec![results])
    })
    .outarg::<Vec<&str>, _>("results")
    .inarg::<Vec<&str>, _>("previous_results")
    .inarg::<Vec<&str>, _>("terms")
}

fn get_subsearch_result_set<'a>(
    lib: Arc<Library>,
    previous_results: Vec<&str>,
    terms: Vec<&str>,
) -> Result<Vec<String>> {
    get_initial_result_set(lib, terms)
}

fn create_get_result_metas(lib: Arc<Library>) -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.method("GetResultMetas", (), move |m| {
        let identifiers: Vec<&str> = m.msg.read1()?;

        let hello =
            get_result_metas(lib.clone(), identifiers).map_err(|err| Error::new_failed(&err.to_string()))?;

        let metas = m.msg.method_return().append1(hello);

        Ok(vec![metas])
    })
    .outarg::<Vec<HashMap<&str, Variant<&str>>>, _>("metas")
    .inarg::<Vec<&str>, _>("identifiers")
}

fn get_result_metas(lib: Arc<Library>, identifiers: Vec<&str>) -> Result<Vec<HashMap<&str, Variant<String>>>> {
    identifiers
        .into_iter()
        .map(|identifier| {
            let doc = lib.get(&serde_json::from_str(identifier)?);
            let mut meta = HashMap::default();
            meta.insert("id", Variant(identifier.to_string()));
            meta.insert("name", Variant(doc.title.clone()));
            meta.insert(
                "description",
                Variant(doc.authors[0].clone()),
            );
            Ok(meta)
        })
        .collect()
}
