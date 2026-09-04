#[cfg(test)]
mod tests {
    use disaster_response_system::command::dispatch::IncidentAlert;
    use disaster_response_system::command::DispatchEngine;
    use disaster_response_system::core::failsafe::FailsafeManager;
    use disaster_response_system::core::positioning::{haversine_distance_km, initial_bearing_degrees};
    use disaster_response_system::core::power::PowerSubsystem;
    use disaster_response_system::core::telemetry::compute_crc16;
    use disaster_response_system::core::types::{Coordinates, DisasterType, PriorityLevel};
    use disaster_response_system::platform::PlatformController;
    use chrono::Utc;

    #[test]
    fn test_haversine_and_bearing() {
        let ankara_center = Coordinates::new(39.9207, 32.8541, 950.0);
        let ankara_ridge = Coordinates::new(39.9800, 33.1500, 1120.0);

        let dist = haversine_distance_km(&ankara_center, &ankara_ridge);
        assert!(dist > 25.0 && dist < 35.0, "Expected ~26-30 km, got {}", dist);

        let bearing = initial_bearing_degrees(&ankara_center, &ankara_ridge);
        assert!(bearing > 60.0 && bearing < 85.0, "Expected North-East bearing, got {}", bearing);
    }

    #[test]
    fn test_power_load_shedding() {
        let mut power = PowerSubsystem::default();
        assert_eq!(power.battery_soc_pct, 95.0);

        // Simulate 4 hours of flight draw
        power.update(14400.0, true, true);
        assert!(power.battery_soc_pct < 50.0);
    }

    #[test]
    fn test_crc16_and_lora_compression() {
        let home = Coordinates::new(39.92, 32.85, 150.0);
        let mut pod = PlatformController::new("POD-TEST", home);
        let packet = pod.simulate_step(1.0, 0.8, false);

        let lora_bytes = packet.to_lora_binary();
        // Check LoRa header
        assert_eq!(&lora_bytes[0..3], b"SAT");
        // Verify compact payload is 23 bytes
        assert_eq!(lora_bytes.len(), 23);

        // Verify CRC matches
        let payload_len = lora_bytes.len();
        let payload_body = &lora_bytes[..payload_len - 2];
        let expected_crc = compute_crc16(payload_body);
        let actual_crc = u16::from_be_bytes([lora_bytes[payload_len - 2], lora_bytes[payload_len - 1]]);
        assert_eq!(expected_crc, actual_crc, "CRC-16 checksum mismatch");
    }

    #[test]
    fn test_dispatch_optimization() {
        let engine = DispatchEngine::new();
        // Incident near Base Bravo (39.75, 32.70)
        let alert = IncidentAlert {
            incident_id: "INC-TEST".to_string(),
            timestamp: Utc::now(),
            disaster_type: DisasterType::Flood,
            priority: PriorityLevel::P1Critical,
            target_coordinates: Coordinates::new(39.74, 32.71, 100.0),
            reported_severity: 0.9,
            alert_source: "911".to_string(),
            initial_notes: "Test incident".to_string(),
        };

        let order = engine.optimize_dispatch(&alert).expect("Dispatch failed");
        assert_eq!(order.selected_base_id, "BASE-BRAVO");
        assert!(order.distance_km < 10.0);
    }

    #[test]
    fn test_autonomous_failsafe_trigger() {
        let mut failsafe = FailsafeManager::default();
        assert!(!failsafe.is_autonomous_mode);

        // Advance 35 seconds without uplink
        failsafe.update(35.0);
        assert!(failsafe.is_autonomous_mode, "Watchdog failed to trigger autonomous degraded mode");

        // Restore uplink
        failsafe.on_uplink_received();
        assert!(!failsafe.is_autonomous_mode, "Failed to exit autonomous degraded mode on uplink");
    }

    #[test]
    fn test_emergency_anomaly_detection() {
        let mut failsafe = FailsafeManager::default();
        // Thermal heat spike > 75C should trigger Wildfire mode
        let detected = failsafe.evaluate_anomalies(88.0, 0.0, 5.0, 1.0);
        assert_eq!(detected, Some(DisasterType::Wildfire));

        // Sudden ground shock > 3.2g should trigger Earthquake mode
        let detected_eq = failsafe.evaluate_anomalies(25.0, 0.0, 5.0, 4.2);
        assert_eq!(detected_eq, Some(DisasterType::Earthquake));
    }
}
