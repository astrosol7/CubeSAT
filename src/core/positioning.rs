use crate::core::types::Coordinates;

pub const EARTH_RADIUS_KM: f64 = 6371.0;

/// Calculate Great Circle distance between two coordinates in kilometers using Haversine formula.
pub fn haversine_distance_km(coord1: &Coordinates, coord2: &Coordinates) -> f64 {
    let d_lat = (coord2.lat - coord1.lat).to_radians();
    let d_lon = (coord2.lon - coord1.lon).to_radians();

    let lat1 = coord1.lat.to_radians();
    let lat2 = coord2.lat.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_KM * c
}

/// Calculate initial forward bearing from coord1 to coord2 in degrees (0 to 360).
pub fn initial_bearing_degrees(coord1: &Coordinates, coord2: &Coordinates) -> f64 {
    let lat1 = coord1.lat.to_radians();
    let lat2 = coord2.lat.to_radians();
    let d_lon = (coord2.lon - coord1.lon).to_radians();

    let y = d_lon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();

    let bearing_rad = y.atan2(x);
    let bearing_deg = bearing_rad.to_degrees();

    (bearing_deg + 360.0) % 360.0
}

/// Calculate estimated transit time in seconds given distance and average speed (km/h).
pub fn calculate_eta_seconds(distance_km: f64, speed_kmh: f64) -> f64 {
    if speed_kmh <= 0.0 {
        return 0.0;
    }
    (distance_km / speed_kmh) * 3600.0
}
