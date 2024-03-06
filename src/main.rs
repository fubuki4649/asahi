use std::error::Error;
use std::future::pending;
use zbus::blocking::connection;
use zbus::Connection;
use crate::portal::Settings;

mod portal;
mod sunrise;
mod location;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
 
    let portal_settings = Settings::new();
    let conn = connection::Builder::session()?
        .name("org.freedesktop.impl.portal.desktop.asahi")?
        .serve_at("/org/freedesktop/portal/desktop", portal_settings)?
        .build()?;

    // Get a SignalContext
    let ctxt = conn.object_server().interface::<_, Settings>("/org/freedesktop/portal/desktop")?.signal_context();

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
    
}