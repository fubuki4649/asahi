//! # D-Bus portal definition for: `org.freedesktop.impl.portal.Settings`
use crate::unwrap_or_return;
use log::debug;
use std::collections::HashMap;
use zbus::fdo::Error::UnknownProperty;
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedValue, Value};

pub struct Portal {
    /// Hashmap<Namespace, Hashmap<Key, Value>>
    pub values: HashMap<String, HashMap<String, OwnedValue>>
}


impl Portal {
    pub fn new() -> Self {
        Self {
            values: HashMap::from([("org.freedesktop.appearance".to_string(), HashMap::from(
            [("color-scheme".to_string(), OwnedValue::from(0))]
            ))]),
        }
    }

    pub async fn change_setting(&mut self, emitter: &SignalEmitter<'_>, ns: &str, key: &str, value: Value<'_>) {
        self.values.entry(ns.to_string()).or_insert_with(HashMap::new).insert(key.to_string(), OwnedValue::try_from(value.clone()).unwrap());

        Self::setting_changed(emitter, ns, key, value).await.expect("Failed to send signal");
        debug!("DBus signal sent");
    }
}

#[interface(name = "org.freedesktop.impl.portal.Settings")]
impl Portal {

    /// Read method
    fn read(&self, ns: &str, key: &str) -> Result<OwnedValue, zbus::fdo::Error> {
        let ns = unwrap_or_return!(self.values.get(ns).ok_or(""), Err(UnknownProperty("Namespace not found".to_string())));
        let value = unwrap_or_return!(ns.get(key).ok_or(""), Err(UnknownProperty("Key not found".to_string())));

        Ok(value.try_to_owned().unwrap())
    }

    /// ReadAll method
    fn read_all(&self, namespaces: Box<[&str]>) -> HashMap<&str, &HashMap<String, OwnedValue>> {

        let mut results: HashMap<&str, &HashMap<String, OwnedValue>> = HashMap::new();

        for ns in self.values.iter() {
            // If namespace matches, insert into results
            if glob(&namespaces, ns.0) {
                results.insert(ns.0, ns.1);
            }
        }

        results
    }

    /// SettingChanged signal
    #[zbus(signal)]
    async fn setting_changed(emitter: &SignalEmitter<'_>, namespace: &str, key: &str, value: Value<'_>) -> zbus::Result<()>;

    /// version property
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 { 0 }

}


/// Matching helper for ReadAll
fn glob(patterns: &[&str], namespace: &str) -> bool {
    let mut ret = false;
    patterns.iter().for_each(|&pattern| {
        ret |= pattern.is_empty();
        ret |= pattern.ends_with('*') &&namespace.contains(pattern.trim_end_matches('*'));
    });
    ret
}