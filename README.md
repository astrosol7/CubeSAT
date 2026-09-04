# Rapid Disaster Response & Situational Awareness System

[![Rust](https://img.shields.io/badge/Rust-1.98%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/Systems%20Engineering-Pass-brightgreen.svg)]()
[![Tests](https://img.shields.io/badge/Unit%20%26%20Integration-100%25%20Passed-blue.svg)]()

> **Stage 3 Response Engine:** A high-reliability, deterministic software architecture and modular aerospace sensing platform engineered for rapid situational awareness during catastrophic emergencies.

---

## 1. System Engineering Foundation & Operational Realities

### The Operational Gap
Traditional disaster management relies on macro remote-sensing satellites (e.g., Sentinel-2, Landsat, MODIS, VIIRS) or human-piloted commercial drones (e.g., DJI quadcopters). Both exhibit fatal operational blindspots during the **Golden 72 Hours**:
* **Orbital Satellites:** Constrained by Keplerian orbital mechanics. Revisit times range from 6 to 48 hours; downlink and synthetic-aperture radar (SAR) tasking latencies are high; cloud and dense wildfire smoke occlude optical payloads. An orbital satellite cannot be launched on-demand within 15 minutes.
* **Line-of-Sight Consumer Drones:** Require human flight crews to physically navigate through compromised or hazardous roads to reach the disaster perimeter. Flight times are capped at 20–30 minutes, and commercial firmware depends on active cellular networks or unhardened RF links.

### The Aegis Solution
A standardized, ruggedized **CubeSat-Class Modular Sensing Pod (Aero-Pod)** pre-stationed in automated silos at regional **Strategic Response Bases**. Upon an emergency alert from civil defense or 911 dispatch, the nearest base autonomously deploys the platform towards the incident coordinates. The platform transitions its onboard state machine across **four mission-specific response modes** (Wildfire, Flood, Landslide, Earthquake) while executing edge processing to broadcast compact tactical telemetry (23-byte LoRa binary frames) over off-grid RF channels directly to emergency incident commanders.

---

## 2. Platform Architecture & Subsystem Partitioning

```
                     ┌──────────────────────────────────────────────┐
                     │          NATIONAL EMERGENCY ALERT            │
                     │  (Civil Defense, Satellite Notification)     │
                     └──────────────────────┬───────────────────────┘
                                            │ Incident Alert
                                            ▼
                     ┌──────────────────────────────────────────────┐
                     │        CENTRAL COMMAND ENGINE (RUST)         │
                     │  • Geospatial Triage & Priority Scoring      │
                     │  • Strategic Base Optimization Engine        │
                     │  • Automated Mission Mode Selection          │
                     │  • Tactical Dispatch Instruction Generator   │
                     └──────────────┬───────────────────────────────┘
                                    │ Dispatch Command (Uplink)
                                    ▼
       ┌─────────────────────────────────────────────────────────────┐
       │            STRATEGIC RESPONSE BASE GATEWAY                  │
       │  • Base Readiness & Comms Relays                            │
       │  • Pod Pre-Flight Diagnostic & Activation Trigger           │
       └────────────────────────────┬────────────────────────────────┘
                                    │ RF / LoRa Link
                                    ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │             DEPLOYABLE SENSOR POD (EDGE PLATFORM - RUST)                │
  │                                                                         │
  │  ┌───────────────────────────┐           ┌───────────────────────────┐  │
  │  │   COMMON POWER & EPS      │           │    FAIL-SAFE AUTONOMY     │  │
  │  │  • Li-ion 2S/3S BMS       │           │  • Loss-of-Link Watchdog  │  │
  │  │  • Sensor Power Gating    │           │  • Blackbox Flash Logger  │  │
  │  │  • Brownout Protection    │           │  • Autonomous Return/Hold │  │
  │  └─────────────┬─────────────┘           └─────────────▲─────────────┘  │
  │                │ Power                                 │ Health State   │
  │                ▼                                       │                │
  │  ┌─────────────────────────────────────────────────────┴─────────────┐  │
  │  │            ONBOARD CORE CONTROLLER & MODE ARBITER                 │  │
  │  │  • Telemetry Serialization (Compact Binary / JSON)                │  │
  │  │  • Priority Queue: Critical Alerts > Telemetry > Stills           │  │
  │  └───────────────────────────┬───────────────────────────────────────┘  │
  │                              │ Configures Sensor Pipeline               │
  │                              ▼                                          │
  │  ┌───────────────────────────────────────────────────────────────────┐  │
  │  │               DYNAMIC MISSION SOFTWARE MODES                      │  │
  │  │                                                                   │  │
  │  │   [ WILDFIRE ]     [   FLOOD   ]    [ LANDSLIDE ]   [ EARTHQUAKE] │  │
  │  │   • Thermal IR     • Optical H2O    • Road Cutoff   • Debris Dens │  │
  │  │   • Spread Rate    • Rise Rate      • Slope Tilt    • Route Clear │  │
  │  │   • Smoke Vec      • Island Tag     • Hazard Zone   • USAR Triage │  │
  │  └───────────────────────────────────────────────────────────────────┘  │
  └──────────────────────────────────────┬──────────────────────────────────┘
                                         │ Downlink Telemetry Packets
                                         ▼
                     ┌──────────────────────────────────────────────┐
                     │    CENTRAL TACTICAL OPERATIONS DASHBOARD     │
                     │  • Live Geospatial GIS Map (Leaflet)         │
                     │  • Real-time Telemetry Gauges & Alerts       │
                     │  • Automated Responder Tactical Bulletins    │
                     │  • REST API & WebSocket Streaming            │
                     └──────────────────────────────────────────────┘
```

---

## 3. Four Disaster Operational Modes

| Disaster Mode | Primary Sensors | Key Extracted Metrics | Critical Responder Decision Supported |
| :--- | :--- | :--- | :--- |
| **Wildfire** | Far-IR Thermal Array (MLX90640) + Optical Camera | Core temp (°C), Rate of spread (m/s), Propagation heading, Smoke plume angle | Evacuation corridor closure; retargeting aerial water drops away from shifting flanks. |
| **Flood** | Ultrasonic Range (HC-SR04) + Optical Contrast | Water extent (%), Rise rate (cm/min), Standoff clearance (cm), Bridge inundation | Issuing mandatory levee evacuations; rerouting trauma ambulances before bridges submerge. |
| **Landslide** | 6-DOF IMU (MPU6050) Inclinometer + Edge Camera | Terrain tilt (°), Debris runout length (m), Road severance tag | Establishing safe staging perimeters; preventing ground teams from entering secondary slide zones. |
| **Earthquake** | Triaxial Accelerometer + Structural Contrast | Debris collapse density (%), MedEvac artery clearance, Shock magnitude (g) | Deploying Heavy Urban Search & Rescue (USAR) to priority sectors; verifying clear arterial MedEvac paths. |

---

## 4. Off-Grid Communication & Degraded-Mode Autonomy

### Compact 23-Byte LoRa Binary Packet
When cellular towers collapse, high-bandwidth imagery cannot be downlinked. The platform encodes mission intelligence into a 23-byte binary frame featuring a CRC-16 (IBM) integrity checksum:
* `[0..3]`: Sync Word (`SAT`)
* `[3..5]`: Sequence Number (`u16`)
* `[5..8]`: State, Mode, Priority (`u8` each)
* `[8..16]`: Scaled WGS-84 Coordinates (`i32` Lat, `i32` Lon)
* `[16..18]`: Altitude (`i16` meters)
* `[18..20]`: Battery SoC % & Scaled Hazard Index (`u8` each)
* `[20..21]`: Subsystem & Route Status Bitflags (`u8`)
* `[21..23]`: CRC-16 Checksum (`u16`)

### Degraded-Mode Autonomous Fail-Safe
* **Loss-of-Signal Watchdog:** If uplink heartbeat is absent for >30 seconds, the pod transitions to `AUTONOMOUS_HOLD`.
* **Local Blackbox Logging:** All raw frames are retained in a circular non-volatile memory buffer.
* **Autonomous Emergency Anomaly Detection:** An onboard heuristic classifier scans sensor feeds for extreme emergency events (thermal spike > 75°C, tilt > 28°, shock > 3.2g) and switches modes automatically even without central command instructions.

---

## 5. Verification & Testing

The system includes a full suite of automated unit and integration tests:
```powershell
cargo test
```
**Test Coverage Includes:**
1. `test_haversine_and_bearing`: Geodesic coordinate calculations and forward bearing.
2. `test_power_load_shedding`: Electrical Power Subsystem (EPS) voltage and SoC depletion curves.
3. `test_crc16_and_lora_compression`: 23-byte payload serialization and CRC-16 verification.
4. `test_dispatch_optimization`: Multi-base proximity and transit ETA solver.
5. `test_autonomous_failsafe_trigger`: 30s loss-of-signal watchdog and recovery.
6. `test_emergency_anomaly_detection`: Heuristic mode switching for thermal and seismic shocks.

---

## 6. How to Run the Application

### Option A: Interactive Automated Batch Demonstration (CLI)
Simulates all four disaster scenarios with step-by-step dispatch, telemetry frame generation, compact LoRa packet dumps, and incident commander directives:
```powershell
cargo run -- --demo
```

### Option B: Tactical Operations Center (EOC) Web Dashboard
Starts the multi-threaded Tactical EOC Server and REST API:
```powershell
cargo run
```
Once launched, open your web browser to:
👉 **`http://127.0.0.1:8080`**

### Features of the Web Dashboard:
* **Interactive Tactical Geospatial Map (Leaflet.js):** Displays real-time positions of Strategic Bases, active response pods, and disaster hazard radii.
* **Live Telemetry & Gauges:** Real-time monitoring of Platform State, Mission Mode, Battery SoC %, LoRa RF RSSI, and the Situational Hazard Index (0 to 10 scale).
* **Mission Dispatch Controls:** Instant scenario triggers for Wildfire, Flood, Landslide, and Earthquake incidents.
* **Emergency Off-Grid RF Simulation:** Test Loss-of-Signal to watch the platform autonomously switch into Degraded Mode and log blackbox telemetry.
* **Incident Commander Directives:** Automated tactical bulletins advising on evacuation corridor passability, priority search sectors, and road closures.
