use std::error::Error;
use std::future::{Future, pending};
use std::pin::Pin;

use crate::asahi::Asahi;
use crate::location::location_provider::LocationProvider;

mod portal;
mod asahi;
mod location;

pub type AsyncClosure<T, U> = Box<dyn Fn(T, U) -> Pin<Box<dyn Future<Output = ()>>> + Send>;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    // Start asahi
    let mut asahi = Asahi::new();
    asahi.start();
    
    // Start location provider
    let mut update_latitude = |latitude: f64, longitude: f64| asahi.update_location(latitude, longitude);
    let mut location = LocationProvider::new(&mut update_latitude);
    location.start();
    
    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
    
}
