use std::error::Error;
use std::future::Future;
use std::pin::Pin;

use tokio::try_join;
use crate::asahi_state::AsahiState;

use crate::sunrise_watcher::observe_sunrise;
use crate::location::location_provider::observe_location;

mod portal;
mod sunrise_watcher;
mod location;
mod asahi_state;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    // Initialize asahi
    let mut state = AsahiState::new();
    
    // Start asahi and location provider
    let location = observe_location(&mut state);
    let sunrise = observe_sunrise(&mut state);

    try_join!(location, sunrise)?;

    Ok(())
    
}
