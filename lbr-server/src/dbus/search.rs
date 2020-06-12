use anyhow::{anyhow, Result};

use dbus::{
    arg::Variant,
    tree::{Factory, Interface, MTFn, Method},
    Error,
};

use std::collections::HashMap;

pub(super) fn create_interface() -> Interface<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.interface("org.gnome.Shell.SearchProvider2", ())
        .add_m(create_get_initial_result_set())
        .add_m(create_get_subsearch_result_set())
        .add_m(create_get_result_metas())
}

fn create_get_initial_result_set() -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.method("GetInitialResultSet", (), move |m| {
        let terms: Vec<&str> = m.msg.read1()?;

        let hello =
            get_initial_result_set(terms).map_err(|err| Error::new_failed(&err.to_string()))?;

        let results = m.msg.method_return().append1(hello);

        Ok(vec![results])
    })
    .outarg::<Vec<&str>, _>("results")
    .inarg::<Vec<&str>, _>("terms")
}

fn get_initial_result_set(terms: Vec<&str>) -> Result<Vec<&str>> {
    Ok(terms)
}

fn create_get_subsearch_result_set() -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.method("GetSubsearchResultSet", (), move |m| {
        let (previous_results, terms): (Vec<&str>, Vec<&str>) = m.msg.read2()?;

        let hello = get_subsearch_result_set(previous_results, terms)
            .map_err(|err| Error::new_failed(&err.to_string()))?;

        let results = m.msg.method_return().append1(hello);

        Ok(vec![results])
    })
    .outarg::<Vec<&str>, _>("results")
    .inarg::<Vec<&str>, _>("previous_results")
    .inarg::<Vec<&str>, _>("terms")
}

fn get_subsearch_result_set<'a>(
    previous_results: Vec<&str>,
    terms: Vec<&'a str>,
) -> Result<Vec<&'a str>> {
    Ok(terms)
}

fn create_get_result_metas() -> Method<MTFn, ()> {
    let fact = Factory::new_fn::<()>();

    fact.method("GetResultMetas", (), move |m| {
        let identifiers: Vec<&str> = m.msg.read1()?;

        let hello =
            get_result_metas(identifiers).map_err(|err| Error::new_failed(&err.to_string()))?;

        let metas = m.msg.method_return().append1(hello);

        Ok(vec![metas])
    })
    .outarg::<Vec<HashMap<&str, Variant<&str>>>, _>("metas")
    .inarg::<Vec<&str>, _>("identifiers")
}

fn get_result_metas(identifiers: Vec<&str>) -> Result<Vec<HashMap<&str, Variant<String>>>> {
    identifiers
        .into_iter()
        .map(|identifier| {
            let mut meta = HashMap::default();
            meta.insert("id", Variant(identifier.to_string()));
            meta.insert("name", Variant(format!("Name of {}", identifier)));
            meta.insert(
                "description",
                Variant(format!("Description of {}", identifier)),
            );
            Ok(meta)
        })
        .collect()
}
