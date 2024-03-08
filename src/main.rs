use std::error::Error;
use std::future::pending;
use zbus::connection;
use crate::portal::Settings;

mod portal;
mod sunrise;
mod location;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let _ = connection::Builder::session()?
        .name("org.freedesktop.impl.portal.desktop.asahi")?
        .serve_at("/org/freedesktop/portal/desktop", Settings::new())?
        .build().await?;

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
    
}
