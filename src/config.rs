use log::warn;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Loads and merges config from two locations:
///   1. `/etc/asahi/config.toml` - system-wide default
///   2. `~/.config/asahi/config.toml` — user-local overrides (takes priority)
///
/// Where keys present in both files, the config in ~/.config takes priority.
/// If either file is missing or unparseable it is silently skipped,
/// so neither is required to exist.
pub fn load_config() -> toml::Table {
    let system = load_file(&PathBuf::from("/etc/asahi/config.toml"));
    let local = load_file(&local_config_path());

    // Start with system-wide values, then overlay the local ones on top.
    system.into_iter().chain(local).collect()
}

/// Resolves the user-local config path
fn local_config_path() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_default())
        .join(".config/asahi/config.toml")
}

/// Reads a single TOML file and parses it into a [`toml::Table`].
/// Returns an empty table if the file does not exist or cannot be parsed.
fn load_file(path: &PathBuf) -> toml::Table {
    match fs::read_to_string(path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(table) => table,
            Err(e) => {
                warn!("Failed to parse config file {}: {e}", path.display());
                toml::Table::new()
            }
        },
        // Missing file is not an error - just use defaults.
        Err(_) => toml::Table::new(),
    }
}