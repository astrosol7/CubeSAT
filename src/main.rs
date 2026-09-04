#![allow(dead_code, unused_variables)]
mod command;
mod core;
mod modes;
mod platform;
mod sensors;
mod server;

use crate::command::dispatch::IncidentAlert;
use crate::command::tactical_report::generate_tactical_bulletin;
use crate::command::DispatchEngine;
use crate::core::types::{Coordinates, DisasterType, PriorityLevel};
use crate::platform::PlatformController;
use crate::server::start_tactical_server;
use chrono::Utc;
use std::env;
use std::thread;
use std::time::Duration;

fn main() {
    print_aerospace_banner();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--demo" {
        run_full_system_demo();
    } else {
        let port: u16 = 8080;
        println!("  [SYSTEM MODE] Starting Central Command EOC Web Dashboard on port {}...", port);
        println!("  [INFO] Open your browser and navigate to: http://127.0.0.1:{}", port);
        println!("  [INFO] To run the automated batch CLI demonstration instead, pass: --demo\n");
        start_tactical_server(port);
    }
}

fn print_aerospace_banner() {
    println!(r#"
================================================================================
   AEGIS-SENTINEL: RAPID DISASTER RESPONSE & SITUATIONAL AWARENESS SYSTEM
   Aerospace-Inspired Modular Sensing Platform // Stage 3 Response Engine
================================================================================
"#);
}

fn run_full_system_demo() {
    println!(">>> EXECUTING MULTI-DISASTER RAPID RESPONSE SIMULATION SCENARIOS <<<\n");

    let dispatch_engine = DispatchEngine::new();
    println!("1. STRATEGIC BASE NETWORK INITIALIZED:");
    for base in &dispatch_engine.bases {
        println!("   - [{}] {} | Pos: ({:.4}, {:.4}) | Radius: {} km",
            base.base_id, base.name, base.coordinates.lat, base.coordinates.lon, base.max_operational_radius_km);
    }
    println!();

    let test_scenarios = vec![
        (
            DisasterType::Wildfire,
            Coordinates::new(39.965, 32.890, 150.0),
            0.90,
            "Active flame front threatening Highway 102 mountain pass",
        ),
        (
            DisasterType::Flood,
            Coordinates::new(39.735, 32.720, 150.0),
            0.92,
            "Flash flood breaching central river levee & inundating bridge",
        ),
        (
            DisasterType::Landslide,
            Coordinates::new(39.995, 33.120, 150.0),
            0.82,
            "Massive slope failure severing Canyon Highway 9",
        ),
        (
            DisasterType::Earthquake,
            Coordinates::new(39.930, 32.845, 150.0),
            0.88,
            "M7.1 earthquake rubble blocking downtown MedEvac corridor",
        ),
    ];

    for (i, (dtype, coords, severity, notes)) in test_scenarios.iter().enumerate() {
        println!("--------------------------------------------------------------------------------");
        println!("SCENARIO {}: DISASTER ALERT INGESTED -> {:?} at [{:.4}, {:.4}]", i + 1, dtype, coords.lat, coords.lon);
        println!("Context: {}", notes);

        let alert = IncidentAlert {
            incident_id: format!("INC-TEST-00{}", i + 1),
            timestamp: Utc::now(),
            disaster_type: *dtype,
            priority: PriorityLevel::P1Critical,
            target_coordinates: *coords,
            reported_severity: *severity,
            alert_source: "CIVIL-DEFENSE-HOTLINE".to_string(),
            initial_notes: notes.to_string(),
        };

        let order = dispatch_engine.optimize_dispatch(&alert).expect("Failed to find base");
        println!("   -> DISPATCH OPTIMIZED:");
        println!("      Selected Base: {} ({:.1} km away)", order.selected_base_id, order.distance_km);
        println!("      Assigned Platform: {}", order.selected_platform_id);
        println!("      Estimated Flight Transit: {:.1} seconds\n", order.estimated_transit_seconds);

        let mut pod = PlatformController::new(&order.selected_platform_id, dispatch_engine.bases[0].coordinates);
        pod.dispatch(order.assigned_mode, order.target_coordinates);

        println!("   -> PLATFORM LAUNCHED & SENSOR RECONFIGURATION:");
        println!("      Mode Switched To: {:?}", pod.active_mode);

        // Advance simulation steps
        for step in 1..=3 {
            let packet = pod.simulate_step(2.0, *severity, false);
            println!("      [T+{}s] Telemetry Frame #{} | State: {} | Battery: {:.0}% | LoRa: {} dBm",
                step * 2, packet.sequence_number, packet.state, packet.power.battery_soc_pct, packet.comms.rssi_dbm);

            if step == 3 {
                let lora_bytes = packet.to_lora_binary();
                println!("      [OFF-GRID RF] Compact LoRa Frame ({} bytes):", lora_bytes.len());
                let hex_str = lora_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                println!("      {}", hex_str);

                let bulletin = generate_tactical_bulletin(&packet);
                println!("\n   -> TACTICAL RESPONDER DIRECTIVE:");
                println!("      Route Status: {}", bulletin.evacuation_route_status);
                println!("      Hazard Index: {:.1} / 10.0", bulletin.hazard_score_out_of_10);
                for rec in &bulletin.critical_decision_recommendations {
                    println!("      * {}", rec);
                }
            }
            thread::sleep(Duration::from_millis(50));
        }

        // Test Autonomous Fail-Safe Degraded Mode on Scenario 1
        if i == 0 {
            println!("\n   -> TESTING AUTONOMOUS DEGRADED MODE (Simulating RF Loss-of-Signal):");
            let mut failsafe_pod = pod.clone();
            for t in 1..=4 {
                let packet = failsafe_pod.simulate_step(10.0, *severity, true);
                println!("      [LOS +{}s] State: {} | Autonomous Mode: {} | Blackbox Entries: {}",
                    t * 10, packet.state, failsafe_pod.failsafe.is_autonomous_mode, failsafe_pod.failsafe.blackbox_buffer.len());
            }
        }
        println!();
    }

    println!("================================================================================");
    println!(">>> ALL MULTI-MISSION RESPONSE SCENARIOS COMPLETED & VERIFIED <<<");
    println!("================================================================================\n");
}

