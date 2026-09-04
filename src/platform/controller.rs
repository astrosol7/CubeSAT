use crate::core::failsafe::FailsafeManager;
use crate::core::positioning::initial_bearing_degrees;
use crate::core::power::PowerSubsystem;
use crate::core::telemetry::TelemetryPacket;
use crate::core::types::{
    CommsLinkQuality, Coordinates, DisasterType, PlatformState, SubsystemHealth,
};
use crate::modes::{process_earthquake, process_flood, process_landslide, process_wildfire};
use crate::sensors::{
    EnvironmentalSensorSuite, ImuSensor, OpticalCameraSensor, ThermalIrSensor, UltrasonicSensor,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformController {
    pub platform_id: String,
    pub state: PlatformState,
    pub active_mode: DisasterType,
    pub current_coords: Coordinates,
    pub target_coords: Coordinates,
    pub home_coords: Coordinates,
    pub heading_deg: f32,
    pub speed_kmh: f32,
    pub power: PowerSubsystem,
    pub comms: CommsLinkQuality,
    pub health: SubsystemHealth,
    pub failsafe: FailsafeManager,
    pub sequence_number: u32,
    #[serde(skip)]
    pub thermal_sensor: ThermalIrSensor,
    #[serde(skip)]
    pub optical_camera: OpticalCameraSensor,
    #[serde(skip)]
    pub ultrasonic_sensor: UltrasonicSensor,
    #[serde(skip)]
    pub imu_sensor: ImuSensor,
    #[serde(skip)]
    pub env_suite: EnvironmentalSensorSuite,
    pub last_assessment_summary: String,
    pub last_hazard_index: f32,
    pub last_route_blocked: bool,
    pub last_route_status: String,
    pub last_specific_json: serde_json::Value,
}

impl PlatformController {
    pub fn new(platform_id: &str, home: Coordinates) -> Self {
        Self {
            platform_id: platform_id.to_string(),
            state: PlatformState::StandbyOnBase,
            active_mode: DisasterType::Wildfire,
            current_coords: home,
            target_coords: home,
            home_coords: home,
            heading_deg: 0.0,
            speed_kmh: 0.0,
            power: PowerSubsystem::default(),
            comms: CommsLinkQuality::default(),
            health: SubsystemHealth::default(),
            failsafe: FailsafeManager::default(),
            sequence_number: 0,
            thermal_sensor: ThermalIrSensor::default(),
            optical_camera: OpticalCameraSensor::default(),
            ultrasonic_sensor: UltrasonicSensor::default(),
            imu_sensor: ImuSensor::default(),
            env_suite: EnvironmentalSensorSuite::default(),
            last_assessment_summary: "System on standby at regional response base.".to_string(),
            last_hazard_index: 0.0,
            last_route_blocked: false,
            last_route_status: "NOMINAL".to_string(),
            last_specific_json: serde_json::json!({}),
        }
    }

    pub fn dispatch(&mut self, mode: DisasterType, target: Coordinates) {
        self.active_mode = mode;
        self.target_coords = target;
        self.state = PlatformState::DispatchedInTransit;
        self.speed_kmh = 75.0; // Cruise speed towards target
        self.heading_deg = initial_bearing_degrees(&self.current_coords, &self.target_coords) as f32;
        self.failsafe.on_uplink_received();
    }

    pub fn return_to_base(&mut self) {
        self.state = PlatformState::ReturningToBase;
        self.target_coords = self.home_coords;
        self.speed_kmh = 75.0;
        self.heading_deg = initial_bearing_degrees(&self.current_coords, &self.home_coords) as f32;
        self.failsafe.on_uplink_received();
    }

    pub fn simulate_step(&mut self, dt_secs: f32, disaster_severity: f32, sim_uplink_lost: bool) -> TelemetryPacket {
        self.sequence_number += 1;

        if sim_uplink_lost {
            self.failsafe.update(dt_secs);
            if self.failsafe.is_autonomous_mode {
                self.state = PlatformState::AutonomousDegradedHold;
                self.comms.packet_loss_pct = 100.0;
                self.comms.rssi_dbm = -120;
            }
        } else {
            self.failsafe.on_uplink_received();
            self.comms.packet_loss_pct = 0.0;
            self.comms.rssi_dbm = -78;
        }

        // Update Position towards target if in transit
        match self.state {
            PlatformState::DispatchedInTransit => {
                let dist_remaining = crate::core::positioning::haversine_distance_km(&self.current_coords, &self.target_coords);
                if dist_remaining < 0.25 {
                    self.state = PlatformState::OnStationScanning;
                    self.current_coords = self.target_coords;
                    self.speed_kmh = 25.0; // loiter speed
                } else {
                    let step_dist = (self.speed_kmh as f64 / 3600.0) * (dt_secs as f64);
                    let ratio = (step_dist / dist_remaining).min(1.0);
                    self.current_coords.lat += (self.target_coords.lat - self.current_coords.lat) * ratio;
                    self.current_coords.lon += (self.target_coords.lon - self.current_coords.lon) * ratio;
                    self.current_coords.alt_meters = 150.0; // operational aerial altitude
                }
            }
            PlatformState::ReturningToBase => {
                let dist_remaining = crate::core::positioning::haversine_distance_km(&self.current_coords, &self.home_coords);
                if dist_remaining < 0.2 {
                    self.state = PlatformState::StandbyOnBase;
                    self.current_coords = self.home_coords;
                    self.speed_kmh = 0.0;
                } else {
                    let step_dist = (self.speed_kmh as f64 / 3600.0) * (dt_secs as f64);
                    let ratio = (step_dist / dist_remaining).min(1.0);
                    self.current_coords.lat += (self.home_coords.lat - self.current_coords.lat) * ratio;
                    self.current_coords.lon += (self.home_coords.lon - self.current_coords.lon) * ratio;
                }
            }
            _ => {}
        }

        // Update Power Subsystem
        let camera_active = self.state == PlatformState::OnStationScanning;
        self.power.update(dt_secs, camera_active, true);

        // Execute Mission Specific Mode Processing
        let mode_idx = match self.active_mode {
            DisasterType::Wildfire => 0,
            DisasterType::Flood => 1,
            DisasterType::Landslide => 2,
            DisasterType::Earthquake => 3,
        };

        let opt_reading = self.optical_camera.sample_simulated(mode_idx, disaster_severity);
        let thermal_reading = self.thermal_sensor.sample_simulated(self.active_mode == DisasterType::Wildfire, disaster_severity);
        let ultrasonic_reading = self.ultrasonic_sensor.sample_simulated(self.active_mode == DisasterType::Flood, disaster_severity, dt_secs);
        let imu_reading = self.imu_sensor.sample_simulated(mode_idx, disaster_severity);

        // Autonomous Anomaly Recognition check
        if let Some(detected_mode) = self.failsafe.evaluate_anomalies(
            thermal_reading.max_temp_c,
            -ultrasonic_reading.water_level_rise_rate_cm_min / 60.0,
            imu_reading.tilt_magnitude_deg,
            imu_reading.acceleration_magnitude_g,
        ) {
            if self.failsafe.is_autonomous_mode {
                self.active_mode = detected_mode;
            }
        }

        let (summary, hazard, route_blocked, route_desc, specific_val, priority) = match self.active_mode {
            DisasterType::Wildfire => {
                let assessment = process_wildfire(&thermal_reading, &opt_reading, &self.current_coords, 45.0, 32.0);
                let hazard = (thermal_reading.fire_intensity_kw / 100.0).clamp(0.0, 10.0);
                let blocked = thermal_reading.max_temp_c > 120.0;
                let sum = assessment.responder_action.clone();
                let r_desc = assessment.route_status.clone();
                let prio = assessment.priority;
                (sum, hazard, blocked, r_desc, serde_json::to_value(&assessment).unwrap_or_default(), prio)
            }
            DisasterType::Flood => {
                let assessment = process_flood(&ultrasonic_reading, &opt_reading, &self.current_coords);
                let hazard = (opt_reading.water_coverage_pct / 10.0).clamp(0.0, 10.0);
                let blocked = assessment.bridge_inundated;
                let sum = assessment.responder_action.clone();
                let r_desc = if blocked { "BRIDGE_INUNDATED_IMPASSABLE".to_string() } else { "PASSABLE".to_string() };
                let prio = assessment.priority;
                (sum, hazard, blocked, r_desc, serde_json::to_value(&assessment).unwrap_or_default(), prio)
            }
            DisasterType::Landslide => {
                let assessment = process_landslide(&imu_reading, &opt_reading, &self.current_coords);
                let hazard = (imu_reading.tilt_magnitude_deg / 4.0).clamp(0.0, 10.0);
                let blocked = assessment.road_severance_detected;
                let sum = assessment.responder_action.clone();
                let r_desc = if blocked { "ROAD_SEVERED_BY_DEBRIS".to_string() } else { "PASSABLE".to_string() };
                let prio = assessment.priority;
                (sum, hazard, blocked, r_desc, serde_json::to_value(&assessment).unwrap_or_default(), prio)
            }
            DisasterType::Earthquake => {
                let assessment = process_earthquake(&imu_reading, &opt_reading, &self.current_coords);
                let hazard = (assessment.structural_collapse_density_pct / 10.0).clamp(0.0, 10.0);
                let blocked = !assessment.primary_medevac_artery_clear;
                let sum = assessment.responder_action.clone();
                let r_desc = if blocked { "MEDEVAC_BLOCKED_RUBBLE".to_string() } else { "MEDEVAC_CLEAR".to_string() };
                let prio = assessment.priority;
                (sum, hazard, blocked, r_desc, serde_json::to_value(&assessment).unwrap_or_default(), prio)
            }
        };

        self.last_assessment_summary = summary.clone();
        self.last_hazard_index = hazard;
        self.last_route_blocked = route_blocked;
        self.last_route_status = route_desc.clone();
        self.last_specific_json = specific_val.clone();

        let packet = TelemetryPacket {
            platform_id: self.platform_id.clone(),
            timestamp: Utc::now(),
            sequence_number: self.sequence_number,
            state: self.state,
            active_mode: self.active_mode,
            priority,
            coordinates: self.current_coords,
            heading_deg: self.heading_deg,
            speed_kmh: self.speed_kmh,
            power: self.power.clone(),
            comms: self.comms.clone(),
            health: self.health.clone(),
            operational_summary: summary,
            hazard_index: hazard,
            critical_route_blocked: route_blocked,
            route_status_desc: route_desc,
            specific_data: specific_val,
        };

        if self.failsafe.is_autonomous_mode {
            self.failsafe.log_blackbox(packet.clone());
        }

        packet
    }
}
