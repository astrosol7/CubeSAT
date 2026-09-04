use crate::core::telemetry::TelemetryPacket;
use crate::core::types::DisasterType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailsafeManager {
    pub seconds_since_last_uplink: f32,
    pub loss_of_signal_threshold_secs: f32,
    pub is_autonomous_mode: bool,
    pub blackbox_buffer: Vec<TelemetryPacket>,
    pub max_blackbox_entries: usize,
    pub emergency_anomaly_detected: bool,
    pub triggered_disaster_mode: Option<DisasterType>,
}

impl Default for FailsafeManager {
    fn default() -> Self {
        Self {
            seconds_since_last_uplink: 0.0,
            loss_of_signal_threshold_secs: 30.0,
            is_autonomous_mode: false,
            blackbox_buffer: Vec::with_capacity(128),
            max_blackbox_entries: 128,
            emergency_anomaly_detected: false,
            triggered_disaster_mode: None,
        }
    }
}

impl FailsafeManager {
    pub fn on_uplink_received(&mut self) {
        self.seconds_since_last_uplink = 0.0;
        self.is_autonomous_mode = false;
    }

    pub fn update(&mut self, dt_secs: f32) {
        self.seconds_since_last_uplink += dt_secs;
        if self.seconds_since_last_uplink >= self.loss_of_signal_threshold_secs {
            self.is_autonomous_mode = true;
        }
    }

    pub fn log_blackbox(&mut self, packet: TelemetryPacket) {
        if self.blackbox_buffer.len() >= self.max_blackbox_entries {
            self.blackbox_buffer.remove(0);
        }
        self.blackbox_buffer.push(packet);
    }

    /// Autonomous anomaly recognition: evaluates sensor inputs without ground commands
    pub fn evaluate_anomalies(
        &mut self,
        max_thermal_temp_c: f32,
        water_proximity_change_rate: f32,
        tilt_degrees: f32,
        shock_magnitude_g: f32,
    ) -> Option<DisasterType> {
        if max_thermal_temp_c > 75.0 {
            self.emergency_anomaly_detected = true;
            self.triggered_disaster_mode = Some(DisasterType::Wildfire);
            return Some(DisasterType::Wildfire);
        }
        if water_proximity_change_rate < -0.15 { // rising rapidly towards sensor
            self.emergency_anomaly_detected = true;
            self.triggered_disaster_mode = Some(DisasterType::Flood);
            return Some(DisasterType::Flood);
        }
        if tilt_degrees > 28.0 {
            self.emergency_anomaly_detected = true;
            self.triggered_disaster_mode = Some(DisasterType::Landslide);
            return Some(DisasterType::Landslide);
        }
        if shock_magnitude_g > 3.2 {
            self.emergency_anomaly_detected = true;
            self.triggered_disaster_mode = Some(DisasterType::Earthquake);
            return Some(DisasterType::Earthquake);
        }
        None
    }
}
