use crate::core::types::{Coordinates, PriorityLevel};
use crate::sensors::{OpticalFrameAnalysis, ThermalIrReading};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WildfireAssessment {
    pub active_fire_front_detected: bool,
    pub max_core_temp_c: f32,
    pub fire_intensity_kw: f32,
    pub rate_of_spread_m_s: f32,
    pub propagation_heading_deg: f32,
    pub smoke_dispersion_heading_deg: f32,
    pub estimated_perimeter_radius_m: f32,
    pub threat_to_structures: bool,
    pub primary_evacuation_route: String,
    pub route_status: String, // "CLEAR", "SMOKE_HAZARD", "IMPASSABLE"
    pub priority: PriorityLevel,
    pub responder_action: String,
}

pub fn process_wildfire(
    thermal: &ThermalIrReading,
    optical: &OpticalFrameAnalysis,
    center_coord: &Coordinates,
    wind_heading_deg: f32,
    wind_speed_kmh: f32,
) -> WildfireAssessment {
    let spread_rate = 0.3 + (wind_speed_kmh / 30.0) * 1.2;
    let prop_heading = (wind_heading_deg + 15.0) % 360.0;
    let smoke_heading = wind_heading_deg;

    let route_blocked = thermal.max_temp_c > 120.0 || optical.road_line_continuity_pct < 40.0;
    let route_status = if route_blocked {
        "IMPASSABLE - FIRE CUTOFF".to_string()
    } else if optical.smoke_occlusion_pct > 60.0 {
        "SMOKE_HAZARD - ZERO_VISIBILITY".to_string()
    } else {
        "OPEN - VIABLE EVACUATION CORRIDOR".to_string()
    };

    let priority = if thermal.max_temp_c > 200.0 || route_blocked {
        PriorityLevel::P1Critical
    } else {
        PriorityLevel::P2High
    };

    let responder_action = if route_blocked {
        format!(
            "ALERT: Primary route cut off at [{:.4}, {:.4}]. Divert civ evac north-east. Retarget water-drop tankers to heading {:.0} deg.",
            center_coord.lat, center_coord.lon, prop_heading
        )
    } else {
        format!(
            "Maintain evacuation along Highway 102. Deploy barrier fire line at bearing {:.0} deg.",
            prop_heading
        )
    };

    WildfireAssessment {
        active_fire_front_detected: thermal.hotspot_detected,
        max_core_temp_c: thermal.max_temp_c,
        fire_intensity_kw: thermal.fire_intensity_kw,
        rate_of_spread_m_s: spread_rate,
        propagation_heading_deg: prop_heading,
        smoke_dispersion_heading_deg: smoke_heading,
        estimated_perimeter_radius_m: 350.0 + thermal.fire_intensity_kw * 0.8,
        threat_to_structures: thermal.fire_intensity_kw > 300.0,
        primary_evacuation_route: "Highway-102-Mountain-Pass".to_string(),
        route_status,
        priority,
        responder_action,
    }
}
