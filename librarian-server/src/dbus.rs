use zbus::fdo::{DBusProxy, Error, RequestNameFlags, Result};
use zbus::{dbus_interface, Connection, ObjectServer};
use zvariant::Value;

use std::collections::HashMap;
use std::convert::TryInto;

use crate::library::{DocId, Library};

pub(crate) fn serve(lib: Library) -> anyhow::Result<()> {
    let connection = Connection::new_session()?;

    DBusProxy::new(&connection)?
        .request_name("lbr.server", RequestNameFlags::ReplaceExisting.into())?;

    let mut object_server = ObjectServer::new(&connection);
    object_server.at(&"/lbr/server".try_into()?, lib)?;

    loop {
        object_server.try_handle_next()?;
    }
}

#[dbus_interface(name = "org.gnome.Shell.SearchProvider2")]
impl Library {
    fn get_initial_result_set(&self, terms: Vec<&str>) -> Vec<String> {
        let query = terms.join(" ");
        self.search(&query).map(|id| id.to_string()).collect()
    }

    fn get_subsearch_result_set(
        &self,
        _previous_results: Vec<&str>,
        terms: Vec<&str>,
    ) -> Vec<String> {
        self.get_initial_result_set(terms)
    }

    fn get_result_metas(&self, identifiers: Vec<&str>) -> Result<Vec<HashMap<String, Value>>> {
        let mut metas = Vec::default();

        for identifier in identifiers {
            let id = identifier
                .parse::<DocId>()
                .map_err(|err| Error::Failed(err.to_string()))?;
            let doc = self.get(id).map_err(|err| Error::Failed(err.to_string()))?;

            let mut meta = HashMap::default();
            meta.insert("id".to_string(), Value::Str(id.to_string().into()));
            meta.insert("name".to_string(), Value::Str(doc.title.clone().into()));
            meta.insert(
                "description".to_string(),
                Value::Str(doc.authors.clone().join(", ").into()),
            );

            metas.push(meta);
        }

        Ok(metas)
    }

    fn activate_result(&self, identifier: &str, _terms: Vec<&str>, _timestamp: u32) -> Result<()> {
        let id = identifier
            .parse::<DocId>()
            .map_err(|err| Error::Failed(err.to_string()))?;

        self.open(id).map_err(|err| Error::Failed(err.to_string()))
    }

    fn launch_search(&self, _terms: Vec<&str>, _timestamp: u32) -> Result<()> {
        Ok(())
    }

    fn insert_document(&mut self, doc: &str, path: &str) -> Result<()> {
        let doc = serde_json::from_str(doc).map_err(|err| Error::Failed(err.to_string()))?;
        self.insert(doc, path)
            .map_err(|err| Error::Failed(err.to_string()))
    }
}
