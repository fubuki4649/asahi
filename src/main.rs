use std::sync::{LazyLock, Mutex};

use crate::context::Context;
mod dbus_portal;
mod sunrise_watcher;
mod context;
mod location;
mod _utils;
mod config;

static CONTEXT: LazyLock<Mutex<Context>> = LazyLock::new(|| Mutex::new(Context::new()));

fn main() {

}
