use crate::core::types::{Coordinates, PriorityLevel};
use crate::sensors::{OpticalFrameAnalysis, UltrasonicReading};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloodAssessment {
    pub water_extent_pct: f32,
    pub water_level_rise_rate_cm_min: f32,
    pub current_surface_clearance_cm: f32,
    pub bridge_inundated: bool,
    pub isolated_communities_detected: bool,
    pub evacuation_route_passable: bool,
    pub primary_transport_artery: String,
    pub priority: PriorityLevel,
    pub responder_action: String,
}

pub fn process_flood(
    ultrasonic: &UltrasonicReading,
    optical: &OpticalFrameAnalysis,
    center_coord: &Coordinates,
) -> FloodAssessment {
    let bridge_inundated = ultrasonic.distance_to_surface_cm < 50.0 || optical.water_coverage_pct > 65.0;
    let evac_passable = !bridge_inundated && optical.road_line_continuity_pct > 50.0;

    let priority = if bridge_inundated || ultrasonic.water_level_rise_rate_cm_min > 40.0 {
        PriorityLevel::P1Critical
    } else {
        PriorityLevel::P2High
    };

    let responder_action = if bridge_inundated {
        format!(
            "CRITICAL: Central River Bridge underwater at [{:.4}, {:.4}]. Route severed. Deploy swiftwater rescue teams and divert ground ambulances to Western Levee bypass.",
            center_coord.lat, center_coord.lon
        )
    } else {
        format!(
            "Water level rising at {:.1} cm/min. Bridge clearance: {:.1} cm. Issue precautionary levee evacuation warning.",
            ultrasonic.water_level_rise_rate_cm_min, ultrasonic.distance_to_surface_cm
        )
    };

    FloodAssessment {
        water_extent_pct: optical.water_coverage_pct,
        water_level_rise_rate_cm_min: ultrasonic.water_level_rise_rate_cm_min,
        current_surface_clearance_cm: ultrasonic.distance_to_surface_cm,
        bridge_inundated,
        isolated_communities_detected: optical.isolated_settlement_detected,
        evacuation_route_passable: evac_passable,
        primary_transport_artery: "Valley-River-Crossing-Bridge".to_string(),
        priority,
        responder_action,
    }
}
