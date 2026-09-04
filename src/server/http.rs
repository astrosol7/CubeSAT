use crate::command::dispatch::IncidentAlert;
use crate::command::tactical_report::generate_tactical_bulletin;
use crate::command::DispatchEngine;
use crate::core::telemetry::TelemetryPacket;
use crate::core::types::{Coordinates, DisasterType, PriorityLevel};
use crate::platform::PlatformController;
use crate::server::web_ui::get_tactical_dashboard_html;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct DispatchPayload {
    pub disaster_type: String,
    pub lat: f64,
    pub lon: f64,
    pub severity: f32,
}

pub struct AppState {
    pub platform: PlatformController,
    pub dispatch_engine: DispatchEngine,
    pub latest_packet: Option<TelemetryPacket>,
    pub uplink_lost: bool,
    pub active_severity: f32,
}

pub fn start_tactical_server(port: u16) {
    let initial_coords = Coordinates::new(39.9207, 32.8541, 950.0); // Base Alpha
    let platform = PlatformController::new("POD-SENTINEL-01", initial_coords);
    let dispatch_engine = DispatchEngine::new();

    let state = Arc::new(Mutex::new(AppState {
        platform,
        dispatch_engine,
        latest_packet: None,
        uplink_lost: false,
        active_severity: 0.85,
    }));

    // Spawn Background Telemetry Simulation Ticker (1 Hz)
    let ticker_state = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(1000));
            let mut s = ticker_state.lock().unwrap();
            let sev = s.active_severity;
            let lost = s.uplink_lost;
            let packet = s.platform.simulate_step(1.0, sev, lost);
            s.latest_packet = Some(packet);
        }
    });

    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&address).expect("Failed to bind tactical HTTP server");
    println!("[AEGIS-SENTINEL] Tactical Operations Server running on http://127.0.0.1:{}", port);
    println!("[AEGIS-SENTINEL] Listening for emergency response alerts and telemetry...");

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let state_clone = Arc::clone(&state);
            thread::spawn(move || {
                handle_client(stream, state_clone);
            });
        }
    }
}

fn handle_client(mut stream: TcpStream, state: Arc<Mutex<AppState>>) {
    let mut buffer = [0; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let req_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut lines = req_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    if method == "GET" && (path == "/" || path == "/index.html") {
        let html = get_tactical_dashboard_html();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            html.len(),
            html
        );
        let _ = stream.write_all(response.as_bytes());
    } else if method == "GET" && path == "/api/status" {
        let s = state.lock().unwrap();
        let lora_hex = s.latest_packet.as_ref().map(|p| {
            let bytes = p.to_lora_binary();
            bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
        });
        let bulletin = s.latest_packet.as_ref().map(|p| generate_tactical_bulletin(p));

        let resp_json = serde_json::json!({
            "platform": s.platform,
            "bases": s.dispatch_engine.bases,
            "latest_packet": s.latest_packet,
            "latest_lora_hex": lora_hex,
            "latest_bulletin": bulletin,
            "uplink_lost": s.uplink_lost,
        });

        let body = resp_json.to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
    } else if method == "POST" && path == "/api/dispatch" {
        // Extract body after \r\n\r\n
        if let Some(body_start) = req_str.find("\r\n\r\n") {
            let body = &req_str[body_start + 4..];
            if let Ok(payload) = serde_json::from_str::<DispatchPayload>(body) {
                let disaster_type = match payload.disaster_type.as_str() {
                    "Wildfire" => DisasterType::Wildfire,
                    "Flood" => DisasterType::Flood,
                    "Landslide" => DisasterType::Landslide,
                    _ => DisasterType::Earthquake,
                };

                let alert = IncidentAlert {
                    incident_id: format!("INC-{}", Utc::now().timestamp_millis()),
                    timestamp: Utc::now(),
                    disaster_type,
                    priority: PriorityLevel::P1Critical,
                    target_coordinates: Coordinates::new(payload.lat, payload.lon, 150.0),
                    reported_severity: payload.severity,
                    alert_source: "EOC-CENTRAL-COMMAND".to_string(),
                    initial_notes: "Rapid tactical assessment requested by Incident Commander.".to_string(),
                };

                let mut s = state.lock().unwrap();
                s.active_severity = payload.severity;
                s.uplink_lost = false;

                if let Some(order) = s.dispatch_engine.optimize_dispatch(&alert) {
                    println!("[AEGIS-SENTINEL] DISPATCH OPTIMIZED -> Base: {}, Distance: {:.2}km, ETA: {:.1}s",
                        order.selected_base_id, order.distance_km, order.estimated_transit_seconds);
                    s.platform.dispatch(order.assigned_mode, order.target_coordinates);
                }
            }
        }
        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\nOK";
        let _ = stream.write_all(resp.as_bytes());
    } else if method == "POST" && path == "/api/failsafe" {
        let mut s = state.lock().unwrap();
        s.uplink_lost = !s.uplink_lost;
        println!("[AEGIS-SENTINEL] UPLINK STATE TOGGLED -> Off-Grid RF Loss: {}", s.uplink_lost);
        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\nOK";
        let _ = stream.write_all(resp.as_bytes());
    } else if method == "POST" && path == "/api/return" {
        let mut s = state.lock().unwrap();
        s.platform.return_to_base();
        println!("[AEGIS-SENTINEL] PLATFORM COMMANDED TO RETURN TO BASE");
        let resp = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\nOK";
        let _ = stream.write_all(resp.as_bytes());
    } else {
        let not_found = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
        let _ = stream.write_all(not_found.as_bytes());
    }
}
