use crate::core::telemetry::TelemetryPacket;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalResponderBulletin {
    pub bulletin_id: String,
    pub timestamp_utc: String,
    pub platform_id: String,
    pub active_mode: String,
    pub priority_level: String,
    pub target_location: String,
    pub situational_summary: String,
    pub hazard_score_out_of_10: f32,
    pub evacuation_route_status: String,
    pub critical_decision_recommendations: Vec<String>,
}

pub fn generate_tactical_bulletin(packet: &TelemetryPacket) -> TacticalResponderBulletin {
    let mut recommendations = Vec::new();

    if packet.critical_route_blocked {
        recommendations.push("URGENT: Execute immediate route diversion. Target corridor is blocked or destroyed.".to_string());
        recommendations.push("Notify local emergency dispatch to close highway access ramps.".to_string());
    } else {
        recommendations.push("Designated evacuation corridor remains open under continuous surveillance.".to_string());
    }

    if packet.hazard_index > 7.0 {
        recommendations.push("P1 Extreme Hazard: Prohibit ground personnel entry into inner sector.".to_string());
    } else if packet.hazard_index > 4.0 {
        recommendations.push("P2 Moderate Hazard: Heavy vehicle operations permitted with safety buffers.".to_string());
    }

    recommendations.push(packet.operational_summary.clone());

    TacticalResponderBulletin {
        bulletin_id: format!("TAC-BULLETIN-{}", packet.sequence_number),
        timestamp_utc: packet.timestamp.to_rfc3339(),
        platform_id: packet.platform_id.clone(),
        active_mode: packet.active_mode.to_string(),
        priority_level: packet.priority.to_string(),
        target_location: format!("Lat {:.5}, Lon {:.5}, Alt {:.1}m", packet.coordinates.lat, packet.coordinates.lon, packet.coordinates.alt_meters),
        situational_summary: packet.operational_summary.clone(),
        hazard_score_out_of_10: packet.hazard_index,
        evacuation_route_status: packet.route_status_desc.clone(),
        critical_decision_recommendations: recommendations,
    }
}
