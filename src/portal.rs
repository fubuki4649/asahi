//! # D-Bus interface for: `org.freedesktop.impl.portal.Settings`
use std::collections::HashMap;
use zbus::interface;
use zbus::object_server::SignalContext;
use zbus::zvariant::{OwnedValue, Value};
use zbus::zvariant::Value::U32;

pub(crate) struct Settings {
    pub(crate) values: HashMap<(&'static str, &'static str), Value<'static>>
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::from(
                [(("org.freedesktop.appearance", "color-scheme"), U32(1))]
            )
        }
    }
    
}

#[interface(name = "org.freedesktop.impl.portal.Settings")]
impl Settings {

    /// Read method
    fn read(&self, namespace: &str, key: &str) -> Result<OwnedValue, zbus::fdo::Error> {

        match self.values.get(&(namespace, key)) {
            Some(val) => Ok(val.try_to_owned().unwrap()),
            None => Err(zbus::fdo::Error::UnknownProperty("Requested setting not found".to_string()))
        }

    }

    /// ReadAll method
    fn read_all(&self, namespaces: Box<[&str]>) -> HashMap<&str, HashMap<&str, OwnedValue>> {

        let mut results: HashMap<&str, HashMap<&str, OwnedValue>> = HashMap::new();

        for ns in self.values.iter() {
            
            if glob(&namespaces, ns.0.0) {
                if !results.contains_key(ns.0.0) {
                    results.insert(ns.0.0, HashMap::new());
                }
                
                results.get_mut(ns.0.0).unwrap().insert(ns.0.1, ns.1.try_to_owned().unwrap());
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