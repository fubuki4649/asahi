//! # D-Bus portal definition for: `org.freedesktop.impl.portal.Settings`
use crate::unwrap_or_return;
use log::debug;
use std::collections::HashMap;
use zbus::fdo::Error::UnknownProperty;
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedValue, Value};

pub struct XDGInterfaces {
    /// Hashmap<Namespace, Hashmap<Key, Value>>
    values: HashMap<String, HashMap<String, OwnedValue>>
}


impl XDGInterfaces {
    pub fn new() -> Self {
        Self {
            values: HashMap::from([("org.freedesktop.appearance".to_string(), HashMap::from(
            [("color-scheme".to_string(), OwnedValue::from(0))]
            ))]),
        }
    }

    pub async fn change_setting(&mut self, emitter: &SignalEmitter<'_>, ns: &str, key: &str, value: Value<'_>) {
        self.values.entry(ns.to_string()).or_default().insert(key.to_string(), OwnedValue::try_from(value.clone()).unwrap());

        Self::setting_changed(emitter, ns, key, value).await.expect("Failed to send signal");
        debug!("DBus signal sent");
    }
}

#[interface(name = "org.freedesktop.impl.portal.Settings")]
impl XDGInterfaces {

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


/// Matching helper for ReadAll.
///
/// Per the XDG Desktop Portal Settings spec, a namespace filter matches if:
/// - The patterns array itself is empty (match all)
/// - The pattern is empty (match all)
/// - The pattern ends with `*` and the namespace starts with the prefix
/// - The pattern is an exact match for the namespace
fn glob(patterns: &[&str], namespace: &str) -> bool {
    // An empty filter list means "no filtering", i.e. every namespace matches.
    if patterns.is_empty() {
        return true;
    }

    let mut ret = false;
    patterns.iter().for_each(|&pattern| {
        ret |= pattern.is_empty();
        ret |= pattern == namespace;
        ret |= pattern.ends_with('*') && namespace.starts_with(pattern.trim_end_matches('*'));
    });
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_patterns_array_matches_everything() {
        assert!(glob(&[], "org.freedesktop.appearance"));
    }

    #[test]
    fn single_empty_pattern_matches_everything() {
        assert!(glob(&[""], "org.freedesktop.appearance"));
    }

    #[test]
    fn exact_pattern_matches_only_itself() {
        assert!(glob(&["org.freedesktop.appearance"], "org.freedesktop.appearance"));
        assert!(!glob(&["org.freedesktop.appearance"], "org.freedesktop.other"));
    }

    #[test]
    fn wildcard_pattern_matches_prefix() {
        assert!(glob(&["org.freedesktop.*"], "org.freedesktop.appearance"));
        assert!(!glob(&["org.freedesktop.*"], "org.other.appearance"));
    }

    #[test]
    fn non_matching_pattern_does_not_match() {
        assert!(!glob(&["org.other"], "org.freedesktop.appearance"));
    }
}