use crate::core::types::Coordinates;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicBase {
    pub base_id: String,
    pub name: String,
    pub coordinates: Coordinates,
    pub max_operational_radius_km: f64,
    pub stationed_platform_ids: Vec<String>,
    pub is_operational: bool,
}

impl StrategicBase {
    pub fn new(id: &str, name: &str, coords: Coordinates, max_radius_km: f64, platforms: Vec<String>) -> Self {
        Self {
            base_id: id.to_string(),
            name: name.to_string(),
            coordinates: coords,
            max_operational_radius_km: max_radius_km,
            stationed_platform_ids: platforms,
            is_operational: true,
        }
    }
}
