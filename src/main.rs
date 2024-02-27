use std::error::Error;
use std::future::pending;
use zbus::blocking::connection;

mod portal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
 
    let portal_settings = portal::Settings::new();
    let _conn = connection::Builder::session()?
        .name("org.freedesktop.impl.portal.desktop.asahi")?
        .serve_at("/org/freedesktop/portal/desktop", portal_settings)?
        .build()?;

    println!("Hello, World!");

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
    
}