use crate::core::power::PowerSubsystem;
use crate::core::types::{CommsLinkQuality, Coordinates, DisasterType, PlatformState, PriorityLevel, SubsystemHealth};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryPacket {
    pub platform_id: String,
    pub timestamp: DateTime<Utc>,
    pub sequence_number: u32,
    pub state: PlatformState,
    pub active_mode: DisasterType,
    pub priority: PriorityLevel,
    pub coordinates: Coordinates,
    pub heading_deg: f32,
    pub speed_kmh: f32,
    pub power: PowerSubsystem,
    pub comms: CommsLinkQuality,
    pub health: SubsystemHealth,
    pub operational_summary: String,
    pub hazard_index: f32, // 0.0 to 10.0 scale
    pub critical_route_blocked: bool,
    pub route_status_desc: String,
    pub specific_data: serde_json::Value,
}

impl TelemetryPacket {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Compress telemetry down to a compact 32-byte binary payload for off-grid LoRa transmission
    pub fn to_lora_binary(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        // Header: [0x53, 0x41, 0x54] ("SAT")
        buf.extend_from_slice(b"SAT");
        // Sequence number (2 bytes, u16 wrapped)
        buf.extend_from_slice(&(self.sequence_number as u16).to_be_bytes());
        // State (1 byte)
        buf.push(self.state as u8);
        // Active mode (1 byte)
        buf.push(self.active_mode as u8);
        // Priority (1 byte)
        buf.push(self.priority as u8);

        // Coordinates: Latitude (i32 scaled by 1e7)
        let lat_scaled = (self.coordinates.lat * 10_000_000.0) as i32;
        buf.extend_from_slice(&lat_scaled.to_be_bytes());

        // Coordinates: Longitude (i32 scaled by 1e7)
        let lon_scaled = (self.coordinates.lon * 10_000_000.0) as i32;
        buf.extend_from_slice(&lon_scaled.to_be_bytes());

        // Altitude (i16 meters)
        let alt_i16 = self.coordinates.alt_meters as i16;
        buf.extend_from_slice(&alt_i16.to_be_bytes());

        // Battery SoC % (1 byte, 0..100)
        buf.push(self.power.battery_soc_pct.round() as u8);

        // Hazard Index (1 byte, scaled 0..100)
        let hazard_scaled = (self.hazard_index * 10.0).clamp(0.0, 100.0) as u8;
        buf.push(hazard_scaled);

        // Flags: bit 0: route blocked, bit 1: gnss lock, bit 2: cam ok, bit 3: thermal ok
        let mut flags: u8 = 0;
        if self.critical_route_blocked { flags |= 1 << 0; }
        if self.health.gnss_lock { flags |= 1 << 1; }
        if self.health.optical_cam_ok { flags |= 1 << 2; }
        if self.health.thermal_sensor_ok { flags |= 1 << 3; }
        buf.push(flags);

        // CRC-16 (IBM) over payload
        let crc = compute_crc16(&buf);
        buf.extend_from_slice(&crc.to_be_bytes());

        buf
    }
}

pub fn compute_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}
