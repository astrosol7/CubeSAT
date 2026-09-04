# Disaster Response IoT Backend

A Go backend for a Rapid Disaster Response and Situational Awareness System. This MVP handles disaster alerts, creates mission commands for deployed hardware platforms, and manages real-time telemetry streaming via WebSockets.

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│  External   │────▶│  Mission     │────▶│  Hardware      │
│  Alerts     │     │  Controller  │     │  Platforms     │
└─────────────┘     └──────────────┘     └────────────────┘
                           │                     │
                           ▼                     ▼
                    ┌──────────────┐     ┌────────────────┐
                    │   Storage    │◀───▶│  Telemetry Hub │
                    │  (SQLite)    │     │  (Channels)    │
                    └──────────────┘     └────────────────┘
                           │                     │
                           ▼                     ▼
                    ┌─────────────────────────────────────┐
                    │         Dashboard (WS)              │
                    └─────────────────────────────────────┘
```

## Components

### Models (`internal/models/`)
- `DisasterAlert` - Incoming disaster notifications
- `MissionCommand` - Instructions sent to hardware
- `HardwareTelemetry` - Data received from deployed platforms

### Mission Controller (`internal/mission/`)
Deterministic state machine that maps disaster types to operational modes:
- **Wildfire** → `FIRE` mode (10 Hz, thermal/temperature/wind/air_quality sensors)
- **Flood** → `FLOOD` mode (5 Hz, water_level/visual/flow_rate/precipitation sensors)
- **Landslide** → `LANDSLIDE` mode (2 Hz, accelerometer/inclination/soil_moisture/visual sensors)
- **Earthquake** → `EARTHQUAKE` mode (1 Hz, seismic/accelerometer/structural/gas_leak sensors)

### Telemetry Hub (`internal/telemetry/`)
Async channel-based hub for concurrent telemetry handling:
- Non-blocking ingestion from multiple hardware platforms
- Priority-based routing to database and dashboard
- Automatic cleanup on disconnect

### Storage (`internal/storage/`)
SQLite persistence with clean interface:
- Alerts, missions, and telemetry tables
- Proper indexing for time-series queries

### Handlers (`internal/handlers/`)
HTTP/WebSocket endpoints:
- `POST /api/alerts` - Ingest disaster alerts
- `GET /ws/telemetry` - Hardware WebSocket (bidirectional)
- `GET /ws/dashboard` - Dashboard WebSocket (real-time telemetry)
- `GET /api/missions` - List missions
- `GET /api/missions/{id}` - Get mission details
- `POST /api/missions/{id}/activate` - Activate mission
- `POST /api/missions/{id}/complete` - Complete mission

## Prerequisites

- Go 1.22 or later
- SQLite (included via CGO)

## Quick Start

```bash
# Clone and navigate to project
cd disaster-response

# Install dependencies (requires CGO for sqlite3)
go mod tidy

# Run server with defaults
go run ./cmd/server

# Or with custom configuration
go run ./cmd/server -addr :8080 -db ./data/disaster_response.db -read-timeout 15s -write-timeout 15s
```

### Verify Installation

```bash
# Check server health
go run ./cmd/server

# Expected response:
# {"active_hardware":0,"status":"healthy","timestamp":"2026-09-04T12:00:00Z"}
```

## Complete Workflow Example

### 1. Ingest a Disaster Alert

```bash
curl -X POST http://localhost:8080/api/alerts \
  -H "Content-Type: application/json" \
  -d '{
    "id": "alert-wildfire-001",
    "type": "Wildfire",
    "latitude": 37.7749,
    "longitude": -122.4194,
    "severity": "HIGH",
    "timestamp": "2026-09-04T12:00:00Z"
  }'
```

**Response:**
```json
{
  "status": "accepted",
  "mission": {
    "mission_id": "mission-Wildfire-1725451200000000000",
    "mode": "FIRE",
    "sampling_rate_hz": 10,
    "priority_sensors": ["thermal", "temperature", "wind_speed", "air_quality"],
    "target_area": {
      "center_latitude": 37.7749,
      "center_longitude": -122.4194,
      "radius_km": 5
    },
    "timestamp": "2026-09-04T12:00:00Z"
  }
}
```

### 2. Connect Hardware Platform (Drone/Payload)

Open a WebSocket connection with the mission ID from step 1:

```bash
# Using wscat (npm install -g wscat)
wscat -c "ws://localhost:8080/ws/telemetry?platform_id=drone-001&mission_id=mission-Wildfire-1725451200000000000"
```

**On connect, hardware receives the MissionCommand:**
```json
{
  "mission_id": "mission-Wildfire-1725451200000000000",
  "mode": "FIRE",
  "sampling_rate_hz": 10,
  "priority_sensors": ["thermal", "temperature", "wind_speed", "air_quality"],
  "target_area": {...},
  "timestamp": "..."
}
```

### 3. Stream Telemetry from Hardware

Send telemetry data through the WebSocket:

```json
{
  "latitude": 37.7750,
  "longitude": -122.4190,
  "temperature": 85.5,
  "humidity": 12.3,
  "extra": {
    "thermal": {"hotspot_count": 3, "max_temp": 120.0},
    "wind_speed": 15.2,
    "air_quality": {"pm25": 45, "co": 12}
  }
}
```

### 4. Connect Dashboard for Real-time Monitoring

```bash
wscat -c "ws://localhost:8080/ws/dashboard"
```

**Dashboard receives filtered high-priority telemetry:**
```json
{
  "platform_id": "drone-001",
  "mission_id": "mission-Wildfire-1725451200000000000",
  "latitude": 37.7750,
  "longitude": -122.4190,
  "temperature": 85.5,
  "humidity": 12.3,
  "extra": {"thermal": {...}, "wind_speed": 15.2, "air_quality": {...}},
  "timestamp": "2026-09-04T12:00:05Z"
}
```

### 5. Activate Mission (Optional - for tracking)

```bash
curl -X POST http://localhost:8080/api/missions/mission-Wildfire-1725451200000000000/activate
```

### 6. Complete Mission

```bash
curl -X POST http://localhost:8080/api/missions/mission-Wildfire-1725451200000000000/complete
```

## API Reference

### POST /api/alerts

Ingest a disaster alert and create a mission.

**Request:**
```json
{
  "id": "string (required)",
  "type": "Wildfire|Flood|Landslide|Earthquake (required)",
  "latitude": "number (required)",
  "longitude": "number (required)",
  "severity": "LOW|MEDIUM|HIGH|CRITICAL (required)",
  "timestamp": "ISO8601 (optional, defaults to now)"
}
```

**Response:** 200 OK with mission command

### GET /ws/telemetry

Hardware WebSocket endpoint. Bidirectional communication.

**Query Parameters:**
- `platform_id` (required) - Unique hardware identifier
- `mission_id` (required) - Mission to join

**Messages from Server → Hardware:**
- `MissionCommand` (initial, or updated commands)

**Messages from Hardware → Server:**
- `HardwareTelemetry` (sensor readings)

### GET /ws/dashboard

Dashboard WebSocket endpoint. Server → Client only (real-time telemetry feed).

**Optional: Send filter preferences**
```json
{
  "mission_ids": ["mission-1", "mission-2"],
  "platform_ids": ["drone-001"],
  "min_severity": "HIGH"
}
```

### GET /api/missions

List all missions (active + pending).

### GET /api/missions/{id}

Get mission details including full command.

### POST /api/missions/{id}/activate

Change mission status to ACTIVE.

### POST /api/missions/{id}/complete

Change mission status to COMPLETED.

### GET /health

Health check endpoint.

## Configuration

| Flag | Default | Description |
|------|---------|-------------|
| `-addr` | `:8080` | HTTP server address |
| `-db` | `disaster_response.db` | SQLite database path |
| `-read-timeout` | `10s` | HTTP read timeout |
| `-write-timeout` | `10s` | HTTP write timeout |

## Mission Mode Configuration

The mission controller uses configurable defaults. To customize:

```go
config := mission.Config{
    DefaultSamplingRateHz: map[models.MissionMode]int{
        models.MissionModeFire:      20,  // Increase for faster fire detection
        models.MissionModeFlood:     10,
        models.MissionModeLandslide: 5,
        models.MissionModeEarthquake: 2,
    },
    PrioritySensors: map[models.MissionMode][]string{
        models.MissionModeFire:      {"thermal", "temperature", "wind_speed", "air_quality", "humidity"},
        models.MissionModeFlood:     {"water_level", "visual", "flow_rate", "precipitation", "soil_moisture"},
        // ... etc
    },
    DefaultRadiusKm: 10.0,  // Increase coverage area
}
controller := mission.NewMissionController(store, config)
```

## Testing

```bash
# Run all tests
go test ./...

# Run mission tests with verbose output
go test -v ./internal/mission/...

# Run with race detector
go test -race ./internal/mission/...

# Generate coverage report
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out
```

## Building for Production

```bash
# Build binary
go build -o disaster-response ./cmd/server

# Run binary
./disaster-response -addr :8080 -db /var/lib/disaster-response/disaster_response.db
```

### Systemd Service Example

```ini
# /etc/systemd/system/disaster-response.service
[Unit]
Description=Disaster Response Backend
After=network.target

[Service]
Type=simple
User=disaster-response
WorkingDirectory=/opt/disaster-response
ExecStart=/opt/disaster-response/disaster-response -addr :8080 -db /var/lib/disaster-response/disaster_response.db
Restart=on-failure
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
```

## Troubleshooting

### "go: missing go.sum entry" / "go mod tidy" fails
```bash
go mod download
go mod tidy
```

### SQLite CGO Issues
```bash
# Ensure CGO is enabled
export CGO_ENABLED=1

# On Linux, install sqlite3 dev headers
# Ubuntu/Debian: apt-get install libsqlite3-dev
# Alpine: apk add sqlite-dev
```

### WebSocket Connection Rejected
- Check `platform_id` and `mission_id` query parameters are provided
- Verify mission exists: `GET /api/missions/{id}`
- Check server logs for "WebSocket upgrade failed"

### High Memory Usage
- Dashboard subscribers buffer at 100 messages each
- Hardware telemetry channels buffer at 100 messages
- Adjust buffer sizes in `telemetry.go` if needed

### Database Locked Errors
- SQLite uses single writer (MaxOpenConns=1)
- For high throughput, consider PostgreSQL/TimescaleDB
- Current implementation: telemetry writes are async with 5s timeout

## Project Structure

```
disaster-response/
├── cmd/server/main.go           # Entry point, graceful shutdown
├── go.mod                       # Dependencies
├── README.md                    # This file
├── internal/
│   ├── models/
│   │   └── models.go            # DisasterAlert, MissionCommand, HardwareTelemetry, etc.
│   ├── storage/
│   │   └── storage.go           # Store interface + SQLiteStore implementation
│   ├── mission/
│   │   ├── mission.go           # MissionController + MissionStateMachine
│   │   └── mission_test.go      # Unit tests
│   ├── telemetry/
│   │   └── telemetry.go         # TelemetryHub with channel-based processing
│   └── handlers/
│       └── handlers.go          # HTTP/WS route handlers
```

## Concurrency Model

| Component | Strategy |
|-----------|----------|
| Mission Controller | RWMutex for mission state, thread-safe store access |
| Telemetry Hub | 1 goroutine per hardware + 1 central processor + fan-out to dashboards |
| Dashboard | Non-blocking channel sends (100ms timeout) |
| Storage | Single SQLite connection, serialized via database/sql |
| HTTP Server | Standard library, one goroutine per connection |

## Graceful Shutdown

The server handles SIGINT/SIGTERM with a 30-second grace period:
1. Stops accepting new HTTP connections
2. Waits for in-flight requests to complete
3. Closes all WebSocket connections (hardware + dashboard)
4. Flushes telemetry buffers via WaitGroup
5. Closes database connection

## License

MIT License - Feel free to use for disaster response applications.