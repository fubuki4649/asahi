use std::error::Error;
use std::future::pending;
use zbus::{connection, Connection};
use zbus::zvariant::Value::U32;
use crate::portal::Settings;

mod portal;
mod sunrise;
mod location;


async fn set_darkmode(conn: &Connection, state: bool) {

    let iface_ref = conn
        .object_server()
        .interface::<_, Settings>("/org/freedesktop/portal/desktop").await.unwrap();
    
    let mut meow = iface_ref.get_mut().await;
    meow.change_setting(conn, "org.freedesktop.appearance", "color-scheme", U32(if state {1} else {2})).await;

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let conn = connection::Builder::session()?
        .name("org.freedesktop.impl.portal.desktop.asahi")?
        .serve_at("/org/freedesktop/portal/desktop", Settings::new())?
        .build().await?;
    
    set_darkmode(&conn, true).await;
    
    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
    
}
