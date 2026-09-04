use crate::core::types::{Coordinates, PriorityLevel};
use crate::sensors::{ImuReading, OpticalFrameAnalysis};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EarthquakeAssessment {
    pub structural_collapse_density_pct: f32,
    pub primary_medevac_artery_clear: bool,
    pub primary_medevac_route: String,
    pub highest_urgency_sector: String,
    pub post_shock_acceleration_g: f32,
    pub priority: PriorityLevel,
    pub responder_action: String,
}

pub fn process_earthquake(
    imu: &ImuReading,
    optical: &OpticalFrameAnalysis,
    center_coord: &Coordinates,
) -> EarthquakeAssessment {
    let collapse_density = (100.0 - optical.road_line_continuity_pct).clamp(0.0, 100.0);
    let medevac_clear = !optical.debris_blockage_detected && optical.road_line_continuity_pct > 60.0;

    let priority = if collapse_density > 50.0 || !medevac_clear || imu.shock_event_detected {
        PriorityLevel::P1Critical
    } else {
        PriorityLevel::P2High
    };

    let urgency_sector = if collapse_density > 60.0 {
        "SECTOR-4-DOWNTOWN-HOSPITAL-DISTRICT".to_string()
    } else {
        "SECTOR-2-SUBURBAN-RESIDENTIAL".to_string()
    };

    let responder_action = if !medevac_clear {
        format!(
            "CRITICAL: Primary MedEvac corridor obstructed with masonry rubble at [{:.4}, {:.4}]. Heavy USAR rescue priority directed to {}. Reroute incoming trauma ambulances via Northern Ring Highway.",
            center_coord.lat, center_coord.lon, urgency_sector
        )
    } else {
        format!(
            "MedEvac route confirmed clear. Structural damage localized at {:.1}%. Dispatch light recon teams to {}.",
            collapse_density, urgency_sector
        )
    };

    EarthquakeAssessment {
        structural_collapse_density_pct: collapse_density,
        primary_medevac_artery_clear: medevac_clear,
        primary_medevac_route: "Metropolitan-Main-Boulevard".to_string(),
        highest_urgency_sector: urgency_sector,
        post_shock_acceleration_g: imu.acceleration_magnitude_g,
        priority,
        responder_action,
    }
}
