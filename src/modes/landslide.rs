use crate::core::types::{Coordinates, PriorityLevel};
use crate::sensors::{ImuReading, OpticalFrameAnalysis};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LandslideAssessment {
    pub terrain_tilt_deg: f32,
    pub secondary_failure_risk: String, // "CRITICAL", "ELEVATED", "STABLE"
    pub road_severance_detected: bool,
    pub debris_field_length_m: f32,
    pub safe_staging_radius_m: f32,
    pub primary_access_route: String,
    pub priority: PriorityLevel,
    pub responder_action: String,
}

pub fn process_landslide(
    imu: &ImuReading,
    optical: &OpticalFrameAnalysis,
    center_coord: &Coordinates,
) -> LandslideAssessment {
    let road_severed = optical.debris_blockage_detected || optical.road_line_continuity_pct < 20.0;
    let secondary_risk = if imu.tilt_magnitude_deg > 22.0 {
        "CRITICAL - ACTIVE CREEP DETECTED".to_string()
    } else if imu.tilt_magnitude_deg > 10.0 {
        "ELEVATED RISK".to_string()
    } else {
        "MONITORING".to_string()
    };

    let priority = if road_severed || imu.slope_instability_flag {
        PriorityLevel::P1Critical
    } else {
        PriorityLevel::P2High
    };

    let staging_radius = 450.0 + imu.tilt_magnitude_deg * 25.0;

    let responder_action = if road_severed {
        format!(
            "ALERT: Canyon access road completely buried at [{:.4}, {:.4}]. Immediate danger of secondary collapse ({:.1} deg tilt). Fall back staging area to minimum {:.0}m perimeter.",
            center_coord.lat, center_coord.lon, imu.tilt_magnitude_deg, staging_radius
        )
    } else {
        format!(
            "Slope displacement stable at {:.1} deg. Road open under caution. Maintain continuous IMU tilt monitoring.",
            imu.tilt_magnitude_deg
        )
    };

    LandslideAssessment {
        terrain_tilt_deg: imu.tilt_magnitude_deg,
        secondary_failure_risk: secondary_risk,
        road_severance_detected: road_severed,
        debris_field_length_m: 220.0 + imu.tilt_magnitude_deg * 8.0,
        safe_staging_radius_m: staging_radius,
        primary_access_route: "Canyon-Pass-Route-9".to_string(),
        priority,
        responder_action,
    }
}
