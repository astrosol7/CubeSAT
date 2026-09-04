use crate::sensors::traits::{DisasterSensor, SensorStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImuReading {
    pub pitch_deg: f32,
    pub roll_deg: f32,
    pub tilt_magnitude_deg: f32,
    pub acceleration_magnitude_g: f32,
    pub shock_event_detected: bool,
    pub slope_instability_flag: bool,
}

#[derive(Debug, Clone)]
pub struct ImuSensor {
    pub status: SensorStatus,
    pub sample_rate_hz: f32,
}

impl Default for ImuSensor {
    fn default() -> Self {
        Self {
            status: SensorStatus::Healthy,
            sample_rate_hz: 10.0,
        }
    }
}

impl DisasterSensor for ImuSensor {
    fn name(&self) -> &'static str {
        "MPU6050-6DOF-IMU"
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

impl ImuSensor {
    pub fn sample_simulated(&self, mode_idx: u8, severity: f32) -> ImuReading {
        match mode_idx {
            2 => { // Landslide: progressive tilt
                let pitch = severity * 35.0;
                let roll = severity * 14.0;
                let tilt = (pitch.powi(2) + roll.powi(2)).sqrt();
                ImuReading {
                    pitch_deg: pitch,
                    roll_deg: roll,
                    tilt_magnitude_deg: tilt,
                    acceleration_magnitude_g: 1.0 + severity * 0.4,
                    shock_event_detected: false,
                    slope_instability_flag: tilt > 18.0,
                }
            }
            3 => { // Earthquake: high acceleration shocks
                let accel = 1.0 + severity * 4.5;
                ImuReading {
                    pitch_deg: severity * 8.0,
                    roll_deg: severity * 6.0,
                    tilt_magnitude_deg: severity * 10.0,
                    acceleration_magnitude_g: accel,
                    shock_event_detected: accel > 2.5,
                    slope_instability_flag: false,
                }
            }
            _ => { // Normal / Wildfire / Flood
                ImuReading {
                    pitch_deg: 1.2,
                    roll_deg: 0.8,
                    tilt_magnitude_deg: 1.4,
                    acceleration_magnitude_g: 1.0,
                    shock_event_detected: false,
                    slope_instability_flag: false,
                }
            }
        }
    }
}
