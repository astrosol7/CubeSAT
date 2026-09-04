# Rapid Disaster Response and Situational Awareness System
### Dual-Stack Tactical Reconnaissance Platform: Rust Avionics & Go IoT Ground Station

[![Firmware - Rust](https://img.shields.io/badge/Firmware-Rust%20(no__std%20capable)-orange.svg)](https://www.rust-lang.org/)
[![Backend - Go](https://img.shields.io/badge/Ground%20Station-Go%201.22+-00ADD8.svg)](https://go.dev/)
[![Telemetry - LoRa](https://img.shields.io/badge/Link-Sub--GHz%20LoRa%20(CRC--16)-blueviolet.svg)]()
[![Mission Scope](https://img.shields.io/badge/Disaster%20Framework-Stage%203%20Response-red.svg)]()
[![Build & Test](https://img.shields.io/badge/Tests-Passing-brightgreen.svg)]()

> **Operational Directive:** Bridge the fatal 15-to-60 minute situational awareness gap between high-latency orbital satellites and boots-on-the-ground rescue teams during catastrophic emergencies.

---

### 1. System Architecture Overview

The system operates across two independent, air-gapped hardware tiers connected via a long-range, sub-GHz radio frequency (RF) telemetry link. It is engineered to operate when municipal power grids, commercial internet, and cellular towers have completely failed.

```text
                  AIRBORNE TACTICAL TIER (RUST FIRMWARE)
 ┌─────────────────────────────────────────────────────────────────────────┐
 │ Standardized CubeSat-Class Aero-Pod (1U / 2U Form Factor)               │
 │                                                                         │
 │  ┌─────────────────┐    ┌──────────────────┐    ┌────────────────────┐  │
 │  │ Optical Camera  │    │ Far-IR MLX90640  │    │ MPU6050 IMU &      │  │
 │  │ (Edge Contrasts)│    │ (Thermal Array)  │    │ HC-SR04 Ultrasonic │  │
 │  └────────┬────────┘    └────────┬─────────┘    └─────────┬──────────┘  │
 │           │                      │                        │             │
 │           ▼                      ▼                        ▼             │
 │  ┌───────────────────────────────────────────────────────────────────┐  │
 │  │ RUST AVIONICS ENGINE                                              │  │
 │  │ • Mode State Machine (Wildfire | Flood | Landslide | Quake)       │  │
 │  │ • Electrical Power Subsystem (EPS) & Load Shedding                │  │
 │  │ • 30s Loss-of-Signal (LOS) Watchdog & Circular Blackbox Buffer    │  │
 │  │ • Compact 23-Byte Binary Serialization & CRC-16 (IBM) Framing     │  │
 │  └──────────────────────────────────┬────────────────────────────────┘  │
 └─────────────────────────────────────┼───────────────────────────────────┘
                                       │
                                       │ Direct LoRa RF Burst (9.6 kbps)
                                       │ 433 / 868 / 915 MHz (Line-of-Sight)
                                       ▼
                 GROUND INCIDENT TIER (GO IoT BACKEND & EOC)
 ┌─────────────────────────────────────────────────────────────────────────┐
 │ Field Incident Command Station (Rugged Laptop / Field Gateway)          │
 │                                                                         │
 │  ┌───────────────────────────────────────────────────────────────────┐  │
 │  │ GO HIGH-CONCURRENCY BACKEND                                       │  │
 │  │ • Serial/SPI Radio Ingestion Worker (Goroutines + Channels)       │  │
 │  │ • Binary Packet Deserializer & CRC-16 Integrity Arbiter           │  │
 │  │ • Regional Hub Dispatch Optimizer (Geodesic Haversine / ETA)      │  │
 │  │ • Tactical Incident Bulletin Generator (Natural Language Alerts)  │  │
 │  │ • Native WebSocket Hub & REST API Engine                          │  │
 │  └──────────────────────────────────┬────────────────────────────────┘  │
 │                                     │                                   │
 │                                     │ Localhost / Ad-Hoc LAN WebSocket  │
 │                                     ▼                                   │
 │  ┌───────────────────────────────────────────────────────────────────┐  │
 │  │ BROWSER-BASED TACTICAL EOC CONSOLE (Leaflet GIS)                  │  │
 │  │ • Real-time GPS Track & Hazard Boundary Overlays                  │  │
 │  │ • Road Cutoff / Access Route Vector Scoring                       │  │
 │  │ • One-Click Remote Mode Reconfiguration & RF Degradation Trigger  │  │
 │  └───────────────────────────────────────────────────────────────────┘  │
 └─────────────────────────────────────────────────────────────────────────┘
## 2. Why This Dual-Stack Architecture?

| Subsystem | Implementation Language | Systems Engineering Justification |
| :--- | :--- | :--- |
| **Tactical Aero-Pod (Avionics)** | **Rust** (`no_std` / Embedded) | **Zero-crash memory safety.** Eliminates runtime panics, null pointer dereferences, and data races in critical flight loops without relying on a non-deterministic garbage collector. |
| **Ground Station (IoT Server)** | **Go** (Concurrency Runtime) | **Ultra-efficient I/O multiplexing.** Goroutines and channels handle simultaneous high-frequency serial radio reads, concurrent WebSocket browser clients, and REST dispatch optimization with zero setup friction. |

---

## 3. Aerospace Trade Study & Platform Selection

An exhaustive trade study rejected the notion of an "on-demand orbital CubeSat launch" due to orbital insertion latency, Keplerian revisit mechanics, and multi-thousand-dollar launch logistics. 

The project adapts **CubeSat modularity and bus architecture into an air-dropped / drone-deployed tactical pod**:

| Platform Option | Deployment Speed | Range / Coverage | Weather Resistance | Student Build Feasibility | Operational Verdict |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **Orbital CubeSat** | Weeks to Months | Regional (Periodic) | High (Space) | ❌ Impossible | **REJECTED:** Violates 15-min response window. |
| **High-Altitude Balloon** | 1–2 Hours | High (Unguided Drift) | ❌ Blown off target | ⚠️ Moderate | **REJECTED:** Zero trajectory control. |
| **Consumer Drone (DJI)** | $<5$ Minutes | Micro-Local ($<3\text{ km}$) | ⚠️ Sensitive to wind | ✅ High | **LIMITED:** Short battery; operator-dependent. |
| **CanSat/CubeSat Aero-Pod** | **Immediate** | **Tactical Perimeter** | **High (Ruggedized)** | ✅ **100% Achievable** | **SELECTED ARCHITECTURE:** Standard 1U/2U pod. |

* **Student Prototype:** 1U modular pod deployed via tether, captive carrier drone release, or parachute descent.
* **National Scalable Pathway:** Stored in automated base silos; delivered via long-range fixed-wing carriers to eject over the coordinates.

---

## 4. Operational Modes & Sensor Decision Matrix

One unified hardware platform dynamically switches its firmware state machine into four distinct disaster profiles:

| Disaster Mode | Essential Operational Data | Primary Sensors | Tactical Decision Supported |
| :--- | :--- | :--- | :--- |
| **1. Wildfire** | Active fire front vector, hotspot coordinates, smoke direction | Far-IR Array (MLX90640) + Optical Contrast | Closes highway corridors before vehicles get trapped; routes water drops. |
| **2. Flood** | Inundation perimeter, water rise velocity ($\text{cm/min}$) | Downward Camera + Ultrasonic (HC-SR04) | Flags submerged bridges; dispatches rescue boats to cut-off dry islands. |
| **3. Landslide** | Runout boundary, highway scarp severance, slope creep | Optical Edge Segmentation + 6-DOF IMU | Evacuates search-and-rescue teams from active secondary slide zones. |
| **4. Earthquake** | Structural collapse density, road debris obstruction | Optical Texture Scoring + IMU Shock | Prioritizes heavy USAR teams to high-yield rubble blocks; clears MedEvac paths. |

---

## 5. Low-Bandwidth Telemetry Protocol (23-Byte Frame)

Because LoRa bandwidth is constrained, live video is prohibited. Telemetry is serialized on the pod in Rust and unpacked on the ground in Go:

$$\text{Frame Size} = 23\text{ Bytes} \quad \vert \quad \text{Modulation} = \text{LoRa Chirp Spread Spectrum (CSS)} \quad \vert \quad \text{Integrity} = \text{CRC-16 IBM}$$

| Byte Offset | Field Name | Type | Scaling / Encoding |
| :---: | :--- | :---: | :--- |
| `0x00` | `platform_id` | `uint8` | Unique vehicle ID (`0x01`–`0xFF`) |
| `0x01` | `disaster_mode` | `uint8` | `0`=Idle, `1`=Wildfire, `2`=Flood, `3`=Landslide, `4`=Earthquake |
| `0x02..0x05` | `timestamp` | `uint32` | Unix Epoch timestamp (seconds UTC) |
| `0x06..0x09` | `latitude` | `int32` | Scaled integer degrees ($\text{deg} \times 10^7$) |
| `0x0A..0x0D` | `longitude` | `int32` | Scaled integer degrees ($\text{deg} \times 10^7$) |
| `0x0E..0x0F` | `altitude` | `uint16` | Altitude above sea level in meters ($0 - 65,535\text{ m}$) |
| `0x10` | `battery_soc` | `uint8` | Battery State of Charge ($0 - 100\%$) |
| `0x11` | `hazard_index` | `uint8` | Calculated severity metric ($0 - 255$) |
| `0x12` | `road_clear` | `uint8` | `0`=Blocked/Cut, `1`=Clear, `2`=Indeterminate |
| `0x13..0x14` | `aux_metric` | `uint16` | Mode metric (e.g. Max Temp °C, Rise Rate mm/h) |
| `0x15..0x16` | `checksum` | `uint16` | CRC-16 (IBM polynomial: `0x8005`) |

---

## 6. Repository Layout

```text
.
├── Cargo.toml                 # Rust workspace configuration
├── go.work                    # Go workspace configuration
│
├── firmware/                  # RUST AVIONICS ENGINE
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # Flight loop executive & scheduler
│       ├── core/
│       │   ├── types.rs       # Platform states, modes, coordinates
│       │   ├── power.rs       # EPS state machine & load-shedding
│       │   ├── telemetry.rs   # 23-byte binary packer & CRC-16 generator
│       │   └── failsafe.rs    # 30s LOS watchdog & circular blackbox
│       ├── sensors/
│       │   ├── traits.rs      # Hardware abstraction layer (HAL)
│       │   ├── thermal_ir.rs  # MLX90640 Far-IR driver & hotspot extraction
│       │   ├── optical.rs     # Frame contrast, edge scoring, road checker
│       │   ├── ultrasonic.rs  # HC-SR04 depth & water delta calculator
│       │   └── imu.rs         # MPU6050 pitch/roll inclinometer & shock
│       └── modes/
│           ├── wildfire.rs    # Fire flank propagation vectors
│           ├── flood.rs       # Water extent & isolated pocket mapping
│           ├── landslide.rs   # Slope stability & road severance
│           └── earthquake.rs  # Collapse density scoring
│
├── ground-station/            # GO IoT DISPATCH BACKEND & EOC CONSOLE
│   ├── go.mod
│   ├── go.sum
│   ├── cmd/
│   │   └── server/
│   │       └── main.go        # Ground server entry point & CLI flags
│   ├── internal/
│   │   ├── radio/
│   │   │   └── serial.go      # Serial/SPI LoRa transceiver ingestion worker
│   │   ├── protocol/
│   │   │   └── decoder.go     # 23-byte binary decoder & CRC-16 verification
│   │   ├── dispatch/
│   │   │   └── optimizer.go   # Geodesic Haversine distance, bearing & ETA
│   │   ├── incident/
│   │   │   └── bulletin.go    # Incident Commander automated advisory generator
│   │   └── server/
│   │       ├── router.go      # HTTP router & REST API endpoints
│   │       └── websocket.go   # Real-time 1Hz telemetry broadcaster
│   └── web/                   # EOC WEB INTERFACE
│       ├── index.html         # Single-page tactical console
│       ├── css/style.css      # Dark-mode emergency tactical styling
│       └── js/app.js          # Leaflet GIS mapping & WebSocket receiver
│
└── tests/
    └── system_test.go         # End-to-end integration & RF framing verification
7. Build, Test, and Execution GuidePrerequisitesRust: cargo and rustc (v1.75+)Go: go (v1.22+)Hardware (Optional): LoRa module (SX1262/SX1276) plugged into USB serial. (The system automatically falls back to an internal hardware-in-the-loop simulator if no radio is found).Step 1: Run the Rust Verification SuiteVerify flight math, geodesic distance formulas, binary packing, and fail-safe triggers:Bashcd firmware
cargo test
Expected output:Plaintextrunning 6 tests
test tests::test_crc16_and_lora_compression ... ok
test tests::test_haversine_and_bearing       ... ok
test tests::test_power_load_shedding         ... ok
test tests::test_dispatch_optimization       ... ok
test tests::test_autonomous_failsafe_trigger ... ok
test tests::test_emergency_anomaly_detection ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; finished in 0.00s
Step 2: Run the Go Integration TestsVerify binary decoding, checksum validation, and dispatch routing algorithms:Bashcd ../ground-station
go test -v ./...
Step 3: Launch the Integrated SystemTerminal A: Start the Go Incident Command Ground StationBashcd ground-station
go run cmd/server/main.go --port=8080
The Go server spins up a background serial listener on /dev/ttyUSB0 (or falls back to mock RF), boots the 1Hz WebSocket hub, and serves the UI.Open your browser and navigate to: http://localhost:8080Terminal B: Launch the Rust Avionics Pod SimulationBashcd firmware
# Run automated multi-disaster flight simulation loop
cargo run -- --simulate
8. Fail-Safe Operations & Degraded ModeIf the aerial pod loses its radio uplink during flight:Loss-of-Signal (LOS) Watchdog: After 30 seconds of radio silence from the ground station, the pod autonomously drops into Autonomous Degraded Mode.Onboard Blackbox Buffering: Telemetry, optical frames, and sensor logs are buffered sequentially to onboard non-volatile flash memory.Autonomous Emergency Anomaly Recognition: If local sensors detect a catastrophic spike (thermal $>75^\circ\text{C}$, freefall/shock $>3.2\text{g}$, or ground tilt $>28^\circ$), the pod reconfigures into the relevant disaster mode automatically without waiting for ground authorization.Recovery Beacon: Emits an intermittent acoustic locator tone and low-power RF beacon to aid search teams in recovering the pod.
