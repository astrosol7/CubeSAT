use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisasterType {
    Wildfire,
    Flood,
    Landslide,
    Earthquake,
}

impl std::fmt::Display for DisasterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisasterType::Wildfire => write!(f, "Wildfire"),
            DisasterType::Flood => write!(f, "Flood"),
            DisasterType::Landslide => write!(f, "Landslide"),
            DisasterType::Earthquake => write!(f, "Earthquake"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PriorityLevel {
    P4Low = 4,
    P3Medium = 3,
    P2High = 2,
    P1Critical = 1,
}

impl std::fmt::Display for PriorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PriorityLevel::P1Critical => write!(f, "P1-CRITICAL"),
            PriorityLevel::P2High => write!(f, "P2-HIGH"),
            PriorityLevel::P3Medium => write!(f, "P3-MEDIUM"),
            PriorityLevel::P4Low => write!(f, "P4-LOW"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lon: f64,
    pub alt_meters: f32,
}

impl Coordinates {
    pub fn new(lat: f64, lon: f64, alt_meters: f32) -> Self {
        Self { lat, lon, alt_meters }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformState {
    StandbyOnBase,
    DispatchedInTransit,
    OnStationScanning,
    AutonomousDegradedHold,
    ReturningToBase,
}

impl std::fmt::Display for PlatformState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformState::StandbyOnBase => write!(f, "STANDBY"),
            PlatformState::DispatchedInTransit => write!(f, "TRANSIT"),
            PlatformState::OnStationScanning => write!(f, "ON_STATION"),
            PlatformState::AutonomousDegradedHold => write!(f, "AUTONOMOUS_HOLD"),
            PlatformState::ReturningToBase => write!(f, "RETURNING"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommsLinkQuality {
    pub rssi_dbm: i16,
    pub snr_db: f32,
    pub packet_loss_pct: f32,
    pub link_type: String,
}

impl Default for CommsLinkQuality {
    fn default() -> Self {
        Self {
            rssi_dbm: -78,
            snr_db: 9.2,
            packet_loss_pct: 0.0,
            link_type: "LoRa-915MHz-Direct".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubsystemHealth {
    pub obc_ok: bool,
    pub eps_ok: bool,
    pub comms_ok: bool,
    pub gnss_lock: bool,
    pub optical_cam_ok: bool,
    pub thermal_sensor_ok: bool,
    pub ultrasonic_ok: bool,
    pub imu_ok: bool,
}

impl Default for SubsystemHealth {
    fn default() -> Self {
        Self {
            obc_ok: true,
            eps_ok: true,
            comms_ok: true,
            gnss_lock: true,
            optical_cam_ok: true,
            thermal_sensor_ok: true,
            ultrasonic_ok: true,
            imu_ok: true,
        }
    }
}
