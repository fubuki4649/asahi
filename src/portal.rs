//! # D-Bus interface for: `org.freedesktop.impl.portal.Settings`
use std::collections::HashMap;

use zbus::{Connection, interface};
use zbus::object_server::SignalContext;
use zbus::zvariant::{OwnedValue, Value};

pub struct Settings {
    pub values: HashMap<(String, String), OwnedValue>
}

impl Settings {
    pub fn new() -> Self {
        
        Self {
            values: HashMap::from(
                [(("org.freedesktop.appearance".to_string(), "color-scheme".to_string()), OwnedValue::from(1))]
            ),
        }
        
    }
    
    pub async fn change_setting(&mut self, ns: &str, key: &str, value: Value<'_>) {

        let conn = Connection::session().await.unwrap();
        let ctxt = SignalContext::new(&conn, "/org/freedesktop/portal/desktop").unwrap();
        
        self.values.insert((ns.to_string(), key.to_string()), value.try_to_owned().unwrap());
        Self::setting_changed(&ctxt, ns, key, value).await.unwrap();

    }

}

#[interface(name = "org.freedesktop.impl.portal.Settings")]
impl Settings {

    /// Read method
    fn read(&self, namespace: &str, key: &str) -> Result<OwnedValue, zbus::fdo::Error> {

        match self.values.get(&(namespace.to_string(), key.to_string())) {
            Some(val) => Ok(val.try_to_owned().unwrap()),
            None => Err(zbus::fdo::Error::UnknownProperty("Requested setting not found".to_string()))
        }

    }

    /// ReadAll method
    fn read_all(&self, namespaces: Box<[&str]>) -> HashMap<String, HashMap<String, OwnedValue>> {

        let mut results: HashMap<String, HashMap<String, OwnedValue>> = HashMap::new();

        for ns in self.values.iter() {
            
            if glob(&namespaces, &ns.0.0) {
                if !results.contains_key(&ns.0.0) {
                    results.insert(ns.0.0.to_owned(), HashMap::new());
                }
                
                results.get_mut(&ns.0.0).unwrap().insert(ns.0.1.to_owned(), ns.1.try_to_owned().unwrap());
            }

        }
        
        results

    }

    /// SettingChanged signal
    #[zbus(signal)]
    async fn setting_changed(ctxt: &SignalContext<'_>, namespace: &str, key: &str, value: Value<'_>) -> zbus::Result<()>;

    /// version property
    #[zbus(property, name = "version")]
    async fn version(&self) -> u32 { 0 }

}

fn glob(patterns: &[&str], namespace: &str) -> bool {
    
    for &pattern in patterns {
        
        if pattern.is_empty() || pattern.ends_with('*') && namespace.contains(pattern.trim_end_matches('*')) {
            return true
        }
        
    }
    
    false
    
}