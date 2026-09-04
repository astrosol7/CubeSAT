use crate::sensors::traits::{DisasterSensor, SensorStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalReading {
    pub baro_pressure_hpa: f32,
    pub baro_altitude_m: f32,
    pub ambient_temp_c: f32,
    pub humidity_pct: f32,
    pub ambient_light_lux: f32,
}

#[derive(Debug, Clone)]
pub struct EnvironmentalSensorSuite {
    pub status: SensorStatus,
    pub sample_rate_hz: f32,
}

impl Default for EnvironmentalSensorSuite {
    fn default() -> Self {
        Self {
            status: SensorStatus::Healthy,
            sample_rate_hz: 1.0,
        }
    }
}

impl DisasterSensor for EnvironmentalSensorSuite {
    fn name(&self) -> &'static str {
        "BMP280-DHT22-Environmental"
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

impl EnvironmentalSensorSuite {
    pub fn sample_simulated(&self, alt_target_m: f32) -> EnvironmentalReading {
        // Barometric formula approximation: P = 1013.25 * (1 - 2.25577e-5 * h)^5.25588
        let pressure = 1013.25 * (1.0 - 0.0000225577 * alt_target_m).powf(5.25588);
        EnvironmentalReading {
            baro_pressure_hpa: pressure,
            baro_altitude_m: alt_target_m,
            ambient_temp_c: 26.5 - (alt_target_m / 1000.0) * 6.5,
            humidity_pct: 54.0,
            ambient_light_lux: 48000.0,
        }
    }
}
