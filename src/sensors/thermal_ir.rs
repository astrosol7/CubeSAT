use crate::sensors::traits::{DisasterSensor, SensorStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalIrReading {
    pub max_temp_c: f32,
    pub ambient_temp_c: f32,
    pub hotspot_detected: bool,
    pub hotspot_pixel_x: usize,
    pub hotspot_pixel_y: usize,
    pub fire_intensity_kw: f32,
}

#[derive(Debug, Clone)]
pub struct ThermalIrSensor {
    pub status: SensorStatus,
    pub sample_rate_hz: f32,
}

impl Default for ThermalIrSensor {
    fn default() -> Self {
        Self {
            status: SensorStatus::Healthy,
            sample_rate_hz: 4.0,
        }
    }
}

impl DisasterSensor for ThermalIrSensor {
    fn name(&self) -> &'static str {
        "MLX90640-Thermal-Array"
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

impl ThermalIrSensor {
    pub fn sample_simulated(&self, fire_active: bool, progress: f32) -> ThermalIrReading {
        if fire_active {
            let max_c = 180.0 + (progress * 120.0).min(350.0);
            ThermalIrReading {
                max_temp_c: max_c,
                ambient_temp_c: 34.5,
                hotspot_detected: true,
                hotspot_pixel_x: (16.0 + (progress * 6.0).sin() * 8.0) as usize,
                hotspot_pixel_y: (12.0 + (progress * 4.0).cos() * 6.0) as usize,
                fire_intensity_kw: 450.0 + progress * 250.0,
            }
        } else {
            ThermalIrReading {
                max_temp_c: 28.5,
                ambient_temp_c: 24.0,
                hotspot_detected: false,
                hotspot_pixel_x: 0,
                hotspot_pixel_y: 0,
                fire_intensity_kw: 0.0,
            }
        }
    }
}
