use crate::command::base::StrategicBase;
use crate::core::positioning::{calculate_eta_seconds, haversine_distance_km};
use crate::core::types::{Coordinates, DisasterType, PriorityLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentAlert {
    pub incident_id: String,
    pub timestamp: DateTime<Utc>,
    pub disaster_type: DisasterType,
    pub priority: PriorityLevel,
    pub target_coordinates: Coordinates,
    pub reported_severity: f32,
    pub alert_source: String,
    pub initial_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchOrder {
    pub order_id: String,
    pub incident_id: String,
    pub selected_base_id: String,
    pub selected_platform_id: String,
    pub assigned_mode: DisasterType,
    pub target_coordinates: Coordinates,
    pub distance_km: f64,
    pub estimated_transit_seconds: f64,
    pub dispatched_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DispatchEngine {
    pub bases: Vec<StrategicBase>,
}

impl DispatchEngine {
    pub fn new() -> Self {
        Self {
            bases: vec![
                StrategicBase::new(
                    "BASE-ALPHA",
                    "Northern Foothills Rapid Base",
                    Coordinates::new(39.9207, 32.8541, 950.0), // Ankara North
                    65.0,
                    vec!["POD-SENTINEL-01".to_string()],
                ),
                StrategicBase::new(
                    "BASE-BRAVO",
                    "Valley Basin Tactical Station",
                    Coordinates::new(39.7500, 32.7000, 880.0), // Ankara South-West
                    75.0,
                    vec!["POD-SENTINEL-02".to_string()],
                ),
                StrategicBase::new(
                    "BASE-CHARLIE",
                    "Eastern Ridge Emergency Post",
                    Coordinates::new(39.9800, 33.1500, 1120.0), // Ankara East
                    80.0,
                    vec!["POD-SENTINEL-03".to_string()],
                ),
            ],
        }
    }

    /// Select the nearest operational strategic base to the incident coordinates
    pub fn optimize_dispatch(&self, alert: &IncidentAlert) -> Option<DispatchOrder> {
        let mut nearest: Option<(&StrategicBase, f64)> = None;

        for base in &self.bases {
            if !base.is_operational || base.stationed_platform_ids.is_empty() {
                continue;
            }
            let dist = haversine_distance_km(&base.coordinates, &alert.target_coordinates);
            if dist <= base.max_operational_radius_km {
                match nearest {
                    None => nearest = Some((base, dist)),
                    Some((_, best_dist)) if dist < best_dist => nearest = Some((base, dist)),
                    _ => {}
                }
            }
        }

        nearest.map(|(base, dist)| {
            let eta = calculate_eta_seconds(dist, 75.0); // 75 km/h cruise
            let platform_id = base.stationed_platform_ids[0].clone();
            DispatchOrder {
                order_id: format!("ORDER-{}", Utc::now().timestamp_millis()),
                incident_id: alert.incident_id.clone(),
                selected_base_id: base.base_id.clone(),
                selected_platform_id: platform_id,
                assigned_mode: alert.disaster_type,
                target_coordinates: alert.target_coordinates,
                distance_km: dist,
                estimated_transit_seconds: eta,
                dispatched_at: Utc::now(),
            }
        })
    }
}
