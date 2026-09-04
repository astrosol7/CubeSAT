pub mod earthquake;
pub mod flood;
pub mod landslide;
pub mod wildfire;

pub use earthquake::process_earthquake;
pub use flood::process_flood;
pub use landslide::process_landslide;
pub use wildfire::process_wildfire;
