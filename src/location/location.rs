pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub last_updated: u64,
}


impl Location {
    pub fn new() -> Self {
        Self {
            lat: 0.0,
            lon: 0.0,
            last_updated: 0,
        }
    }
}