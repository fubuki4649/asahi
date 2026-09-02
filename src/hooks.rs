use log::{info, warn};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runs all executable scripts in the appropriate `[dark|light].d` hook
/// directories for the given mode value (1 = dark, 2 = light).
///
/// Directories checked (in order):
///   /etc/asahi/[dark|light].d
///   ~/.config/asahi/[dark|light].d
///
/// Scripts within each directory are run in alphabetical order.
/// Each script is spawned and not awaited — the daemon does not block on hooks.
pub fn run_hooks(mode: u32) {
    let subdir = match mode {
        1 => "dark.d",
        2 => "light.d",
        _ => return,
    };

    for dir in hook_dirs(subdir) {
        run_dir(&dir);
    }
}

/// Returns the ordered list of hook directories to search.
fn hook_dirs(subdir: &str) -> [PathBuf; 2] {
    let user_base = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut p = PathBuf::from(env::var("HOME").unwrap_or_default());
            p.push(".config");
            p
        });

    [
        PathBuf::from("/etc/asahi").join(subdir),
        user_base.join("asahi").join(subdir),
    ]
}

/// Spawns every executable file in `dir`, sorted alphabetically.
/// Missing or unreadable directories are silently skipped.
fn run_dir(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else { return };

    let mut scripts: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && is_executable(p))
        .collect();

    scripts.sort();

    for script in scripts {
        match Command::new(&script).spawn() {
            Ok(_)  => info!("Hook spawned: {}", script.display()),
            Err(e) => warn!("Failed to spawn hook {}: {e}", script.display()),
        }
    }
}

fn is_executable(path: &Path) -> bool {
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}
