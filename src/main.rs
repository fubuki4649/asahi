use std::error::Error;
use std::sync::Mutex;

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
    let state = Mutex::new(AsahiState::new());
    
    // Start asahi and location provider
    let location = observe_location(&state);
    let sunrise = observe_sunrise(&state);

    try_join!(location, sunrise)?;

    Ok(())
    
}
