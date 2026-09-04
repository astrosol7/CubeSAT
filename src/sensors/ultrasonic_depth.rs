use crate::sensors::traits::{DisasterSensor, SensorStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltrasonicReading {
    pub distance_to_surface_cm: f32,
    pub water_level_rise_rate_cm_min: f32,
    pub critical_submergence_warning: bool,
}

#[derive(Debug, Clone)]
pub struct UltrasonicSensor {
    pub status: SensorStatus,
    pub sample_rate_hz: f32,
    last_distance_cm: f32,
}

impl Default for UltrasonicSensor {
    fn default() -> Self {
        Self {
            status: SensorStatus::Healthy,
            sample_rate_hz: 2.0,
            last_distance_cm: 250.0,
        }
    }
}

impl DisasterSensor for UltrasonicSensor {
    fn name(&self) -> &'static str {
        "HC-SR04-Ultrasonic"
    }
    fn is_operational(&self) -> bool {
        self.status == SensorStatus::Healthy
    }
    fn sample_rate_hz(&self) -> f32 {
        self.sample_rate_hz
    }
    fn status(&self) -> SensorStatus {
        self.status
    }
}

impl UltrasonicSensor {
    pub fn sample_simulated(&mut self, is_flood_mode: bool, severity: f32, dt_secs: f32) -> UltrasonicReading {
        if is_flood_mode {
            // As flood rises, distance from aerial pod/bridge sensor decreases
            let rise_rate_cm_sec = severity * 1.5; // rapid flash flood
            let new_dist = (self.last_distance_cm - rise_rate_cm_sec * dt_secs).max(15.0);
            let rise_rate_cm_min = rise_rate_cm_sec * 60.0;
            self.last_distance_cm = new_dist;

            UltrasonicReading {
                distance_to_surface_cm: new_dist,
                water_level_rise_rate_cm_min: rise_rate_cm_min,
                critical_submergence_warning: new_dist < 45.0,
            }
        } else {
            self.last_distance_cm = 280.0;
            UltrasonicReading {
                distance_to_surface_cm: 280.0,
                water_level_rise_rate_cm_min: 0.0,
                critical_submergence_warning: false,
            }
        }
    }
}
