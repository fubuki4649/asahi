use std::error::Error;
use std::future::{Future, pending};
use std::pin::Pin;

use crate::asahi::Asahi;

mod portal;
mod asahi;
mod location;

pub type AsyncClosure<T, U> = Box<dyn Fn(T, U) -> Pin<Box<dyn Future<Output = ()>>> + Send>;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    let mut asahi = Asahi::new();
    
    asahi.start().await?;
    
    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
    
}
