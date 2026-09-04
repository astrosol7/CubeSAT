use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorStatus {
    Healthy,
    Degraded,
    Offline,
}

pub trait DisasterSensor: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_operational(&self) -> bool;
    fn sample_rate_hz(&self) -> f32;
    fn status(&self) -> SensorStatus;
}
