use log::warn;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub use crate::_utils::boml_ext::Value;

static CONFIG: LazyLock<HashMap<String, Value>> = LazyLock::new(parse_config);

/// Loads and merges config from two locations:
///   1. `/etc/asahi/config.toml` - system-wide default
///   2. `~/.config/asahi/config.toml` — user-local overrides (takes priority)
///
/// Where keys present in both files, the config in ~/.config takes priority.
/// If either file is missing or unparseable it is silently skipped,
/// so neither is required to exist.
pub fn load_config() -> &'static HashMap<String, Value> {
    &CONFIG
}

fn parse_config() -> HashMap<String, Value> {
    let mut config = load_file(Path::new("/etc/asahi/config.toml")).unwrap_or_default();
    if let Some(local) = load_file(&local_config_path()) {
        config.extend(local);
    }
    config
}

/// Resolves the user-local config path
fn local_config_path() -> PathBuf {
    match env::var_os("XDG_CONFIG_HOME") {
        Some(base) => PathBuf::from(base).join("asahi/config.toml"),
        None => PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config/asahi/config.toml"),
    }
}

/// Reads a single TOML file and parses it into a map of keys to values.
/// Returns None if the file does not exist or cannot be parsed.
fn load_file(path: &Path) -> Option<HashMap<String, Value>> {
    let content = fs::read_to_string(path).ok()?;
    match boml::parse(&content) {
        Ok(toml) => Some(
            toml.iter()
                .filter_map(|(k, v)| Value::from_boml(v).map(|val| (k.as_str().to_owned(), val)))
                .collect(),
        ),
        Err(e) => {
            warn!("Failed to parse config file {}: {e}", path.display());
            None
        }
    }
}