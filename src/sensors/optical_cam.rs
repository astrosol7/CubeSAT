use crate::sensors::traits::{DisasterSensor, SensorStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpticalFrameAnalysis {
    pub smoke_occlusion_pct: f32,
    pub water_coverage_pct: f32,
    pub debris_blockage_detected: bool,
    pub road_line_continuity_pct: f32, // 100% = clear, < 40% = blocked/destroyed
    pub isolated_settlement_detected: bool,
    pub thumbnail_b64_hash: String,
}

#[derive(Debug, Clone)]
pub struct OpticalCameraSensor {
    pub status: SensorStatus,
    pub sample_rate_hz: f32,
}

impl Default for OpticalCameraSensor {
    fn default() -> Self {
        Self {
            status: SensorStatus::Healthy,
            sample_rate_hz: 1.0,
        }
    }
}

impl DisasterSensor for OpticalCameraSensor {
    fn name(&self) -> &'static str {
        "ESP32-CAM-OV2640"
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

impl OpticalCameraSensor {
    pub fn sample_simulated(&self, mode_idx: u8, severity: f32) -> OpticalFrameAnalysis {
        match mode_idx {
            0 => { // Wildfire
                OpticalFrameAnalysis {
                    smoke_occlusion_pct: (severity * 85.0).clamp(10.0, 95.0),
                    water_coverage_pct: 0.0,
                    debris_blockage_detected: severity > 0.6,
                    road_line_continuity_pct: (100.0 - severity * 80.0).clamp(10.0, 100.0),
                    isolated_settlement_detected: false,
                    thumbnail_b64_hash: format!("OPT-WF-FRAME-{:.0}", severity * 100.0),
                }
            }
            1 => { // Flood
                OpticalFrameAnalysis {
                    smoke_occlusion_pct: 5.0,
                    water_coverage_pct: (severity * 75.0).clamp(5.0, 90.0),
                    debris_blockage_detected: severity > 0.4,
                    road_line_continuity_pct: (100.0 - severity * 90.0).clamp(5.0, 100.0),
                    isolated_settlement_detected: severity > 0.5,
                    thumbnail_b64_hash: format!("OPT-FL-FRAME-{:.0}", severity * 100.0),
                }
            }
            2 => { // Landslide
                OpticalFrameAnalysis {
                    smoke_occlusion_pct: 12.0, // dust plume
                    water_coverage_pct: 0.0,
                    debris_blockage_detected: severity > 0.3,
                    road_line_continuity_pct: (100.0 - severity * 95.0).clamp(0.0, 100.0),
                    isolated_settlement_detected: severity > 0.7,
                    thumbnail_b64_hash: format!("OPT-LS-FRAME-{:.0}", severity * 100.0),
                }
            }
            _ => { // Earthquake
                OpticalFrameAnalysis {
                    smoke_occlusion_pct: 20.0, // rubble dust & urban fires
                    water_coverage_pct: 0.0,
                    debris_blockage_detected: severity > 0.4,
                    road_line_continuity_pct: (100.0 - severity * 70.0).clamp(15.0, 100.0),
                    isolated_settlement_detected: true,
                    thumbnail_b64_hash: format!("OPT-EQ-FRAME-{:.0}", severity * 100.0),
                }
            }
        }
    }
}
