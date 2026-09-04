package models

import (
	"time"
)

// DisasterType represents the type of disaster
type DisasterType string

const (
	DisasterTypeWildfire  DisasterType = "Wildfire"
	DisasterTypeFlood     DisasterType = "Flood"
	DisasterTypeLandslide DisasterType = "Landslide"
	DisasterTypeEarthquake DisasterType = "Earthquake"
)

// MissionMode represents the operational mode of the hardware platform
type MissionMode string

const (
	MissionModeFire      MissionMode = "FIRE"
	MissionModeFlood     MissionMode = "FLOOD"
	MissionModeLandslide MissionMode = "LANDSLIDE"
	MissionModeEarthquake MissionMode = "EARTHQUAKE"
)

// Severity represents the severity level of an alert
type Severity string

const (
	SeverityLow      Severity = "LOW"
	SeverityMedium   Severity = "MEDIUM"
	SeverityHigh     Severity = "HIGH"
	SeverityCritical Severity = "CRITICAL"
)

// DisasterAlert represents an incoming disaster alert from external sources
type DisasterAlert struct {
	ID        string       `json:"id"`
	Type      DisasterType `json:"type"`
	Latitude  float64      `json:"latitude"`
	Longitude float64      `json:"longitude"`
	Severity  Severity     `json:"severity"`
	Timestamp time.Time    `json:"timestamp"`
}

// MissionCommand represents an outbound instruction to deployed hardware
type MissionCommand struct {
	MissionID       string        `json:"mission_id"`
	Mode            MissionMode   `json:"mode"`
	SamplingRateHz  int           `json:"sampling_rate_hz"`
	PrioritySensors []string      `json:"priority_sensors"`
	TargetArea      TargetArea    `json:"target_area"`
	Timestamp       time.Time     `json:"timestamp"`
}

// TargetArea defines the geographic bounds for the mission
type TargetArea struct {
	CenterLatitude  float64 `json:"center_latitude"`
	CenterLongitude float64 `json:"center_longitude"`
	RadiusKm        float64 `json:"radius_km"`
}

// HardwareTelemetry represents incoming telemetry data from deployed hardware
type HardwareTelemetry struct {
	PlatformID    string                 `json:"platform_id"`
	MissionID     string                 `json:"mission_id"`
	Latitude      float64                `json:"latitude"`
	Longitude     float64                `json:"longitude"`
	Temperature   float64                `json:"temperature,omitempty"`
	Humidity      float64                `json:"humidity,omitempty"`
	WaterLevel    float64                `json:"water_level,omitempty"`
	ImageMetadata *ImageMetadata         `json:"image_metadata,omitempty"`
	Timestamp     time.Time              `json:"timestamp"`
	Extra         map[string]interface{} `json:"extra,omitempty"`
}

// ImageMetadata contains metadata about captured images
type ImageMetadata struct {
	ImageID     string  `json:"image_id"`
	URL         string  `json:"url,omitempty"`
	Width       int     `json:"width"`
	Height      int     `json:"height"`
	Format      string  `json:"format"`
	SizeBytes   int64   `json:"size_bytes"`
	CaptureTime time.Time `json:"capture_time"`
}

// MissionStatus represents the current status of a mission
type MissionStatus string

const (
	MissionStatusPending   MissionStatus = "PENDING"
	MissionStatusActive    MissionStatus = "ACTIVE"
	MissionStatusCompleted MissionStatus = "COMPLETED"
	MissionStatusAborted   MissionStatus = "ABORTED"
)

// Mission represents a full mission record in the system
type Mission struct {
	ID          string        `json:"id"`
	AlertID     string        `json:"alert_id"`
	Mode        MissionMode   `json:"mode"`
	Status      MissionStatus `json:"status"`
	Command     MissionCommand `json:"command"`
	CreatedAt   time.Time     `json:"created_at"`
	StartedAt   *time.Time    `json:"started_at,omitempty"`
	CompletedAt *time.Time    `json:"completed_at,omitempty"`
}