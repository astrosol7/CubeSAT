package storage

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"time"

	"disaster-response/internal/models"

	_ "github.com/mattn/go-sqlite3"
)

// Store defines the interface for data persistence
type Store interface {
	// Alert operations
	CreateAlert(ctx context.Context, alert *models.DisasterAlert) error
	GetAlert(ctx context.Context, id string) (*models.DisasterAlert, error)
	ListAlerts(ctx context.Context, limit, offset int) ([]*models.DisasterAlert, error)

	// Mission operations
	CreateMission(ctx context.Context, mission *models.Mission) error
	GetMission(ctx context.Context, id string) (*models.Mission, error)
	UpdateMissionStatus(ctx context.Context, id string, status models.MissionStatus) error
	ListMissions(ctx context.Context, limit, offset int) ([]*models.Mission, error)

	// Telemetry operations
	InsertTelemetry(ctx context.Context, telemetry *models.HardwareTelemetry) error
	GetTelemetryByMission(ctx context.Context, missionID string, limit, offset int) ([]*models.HardwareTelemetry, error)
	GetLatestTelemetry(ctx context.Context, platformID string) (*models.HardwareTelemetry, error)

	// Cleanup
	Close() error
}

// SQLiteStore implements Store using SQLite
type SQLiteStore struct {
	db *sql.DB
}

// NewSQLiteStore creates a new SQLite store and initializes the schema
func NewSQLiteStore(dataSourceName string) (*SQLiteStore, error) {
	db, err := sql.Open("sqlite3", dataSourceName)
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %w", err)
	}

	// Configure connection pool for SQLite
	db.SetMaxOpenConns(1) // SQLite handles one writer at a time
	db.SetMaxIdleConns(1)
	db.SetConnMaxLifetime(time.Hour)

	store := &SQLiteStore{db: db}
	if err := store.initSchema(); err != nil {
		db.Close()
		return nil, fmt.Errorf("failed to initialize schema: %w", err)
	}

	log.Printf("SQLite store initialized: %s", dataSourceName)
	return store, nil
}

// initSchema creates the necessary tables
func (s *SQLiteStore) initSchema() error {
	schema := `
	CREATE TABLE IF NOT EXISTS alerts (
		id TEXT PRIMARY KEY,
		type TEXT NOT NULL,
		latitude REAL NOT NULL,
		longitude REAL NOT NULL,
		severity TEXT NOT NULL,
		timestamp DATETIME NOT NULL
	);

	CREATE INDEX IF NOT EXISTS idx_alerts_timestamp ON alerts(timestamp);

	CREATE TABLE IF NOT EXISTS missions (
		id TEXT PRIMARY KEY,
		alert_id TEXT NOT NULL,
		mode TEXT NOT NULL,
		status TEXT NOT NULL,
		command_json TEXT NOT NULL,
		created_at DATETIME NOT NULL,
		started_at DATETIME,
		completed_at DATETIME,
		FOREIGN KEY (alert_id) REFERENCES alerts(id)
	);

	CREATE INDEX IF NOT EXISTS idx_missions_status ON missions(status);
	CREATE INDEX IF NOT EXISTS idx_missions_alert_id ON missions(alert_id);

	CREATE TABLE IF NOT EXISTS telemetry (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		platform_id TEXT NOT NULL,
		mission_id TEXT NOT NULL,
		latitude REAL NOT NULL,
		longitude REAL NOT NULL,
		temperature REAL,
		humidity REAL,
		water_level REAL,
		image_metadata_json TEXT,
		timestamp DATETIME NOT NULL,
		extra_json TEXT
	);

	CREATE INDEX IF NOT EXISTS idx_telemetry_mission_id ON telemetry(mission_id);
	CREATE INDEX IF NOT EXISTS idx_telemetry_platform_id ON telemetry(platform_id);
	CREATE INDEX IF NOT EXISTS idx_telemetry_timestamp ON telemetry(timestamp);
	`

	_, err := s.db.Exec(schema)
	return err
}

// CreateAlert inserts a new disaster alert
func (s *SQLiteStore) CreateAlert(ctx context.Context, alert *models.DisasterAlert) error {
	query := `INSERT INTO alerts (id, type, latitude, longitude, severity, timestamp) VALUES (?, ?, ?, ?, ?, ?)`
	_, err := s.db.ExecContext(ctx, query, alert.ID, alert.Type, alert.Latitude, alert.Longitude, alert.Severity, alert.Timestamp)
	return err
}

// GetAlert retrieves an alert by ID
func (s *SQLiteStore) GetAlert(ctx context.Context, id string) (*models.DisasterAlert, error) {
	query := `SELECT id, type, latitude, longitude, severity, timestamp FROM alerts WHERE id = ?`
	row := s.db.QueryRowContext(ctx, query, id)

	var alert models.DisasterAlert
	err := row.Scan(&alert.ID, &alert.Type, &alert.Latitude, &alert.Longitude, &alert.Severity, &alert.Timestamp)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	return &alert, err
}

// ListAlerts retrieves a paginated list of alerts
func (s *SQLiteStore) ListAlerts(ctx context.Context, limit, offset int) ([]*models.DisasterAlert, error) {
	query := `SELECT id, type, latitude, longitude, severity, timestamp FROM alerts ORDER BY timestamp DESC LIMIT ? OFFSET ?`
	rows, err := s.db.QueryContext(ctx, query, limit, offset)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var alerts []*models.DisasterAlert
	for rows.Next() {
		var alert models.DisasterAlert
		if err := rows.Scan(&alert.ID, &alert.Type, &alert.Latitude, &alert.Longitude, &alert.Severity, &alert.Timestamp); err != nil {
			return nil, err
		}
		alerts = append(alerts, &alert)
	}
	return alerts, rows.Err()
}

// CreateMission inserts a new mission
func (s *SQLiteStore) CreateMission(ctx context.Context, mission *models.Mission) error {
	commandJSON, err := jsonMarshal(mission.Command)
	if err != nil {
		return fmt.Errorf("failed to marshal command: %w", err)
	}

	query := `INSERT INTO missions (id, alert_id, mode, status, command_json, created_at, started_at, completed_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)`
	_, err = s.db.ExecContext(ctx, query,
		mission.ID,
		mission.AlertID,
		mission.Mode,
		mission.Status,
		commandJSON,
		mission.CreatedAt,
		mission.StartedAt,
		mission.CompletedAt,
	)
	return err
}

// GetMission retrieves a mission by ID
func (s *SQLiteStore) GetMission(ctx context.Context, id string) (*models.Mission, error) {
	query := `SELECT id, alert_id, mode, status, command_json, created_at, started_at, completed_at FROM missions WHERE id = ?`
	row := s.db.QueryRowContext(ctx, query, id)

	var mission models.Mission
	var commandJSON string
	var startedAt, completedAt sql.NullTime

	err := row.Scan(&mission.ID, &mission.AlertID, &mission.Mode, &mission.Status, &commandJSON, &mission.CreatedAt, &startedAt, &completedAt)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}

	if err := jsonUnmarshal(commandJSON, &mission.Command); err != nil {
		return nil, fmt.Errorf("failed to unmarshal command: %w", err)
	}

	if startedAt.Valid {
		mission.StartedAt = &startedAt.Time
	}
	if completedAt.Valid {
		mission.CompletedAt = &completedAt.Time
	}

	return &mission, nil
}

// UpdateMissionStatus updates the status of a mission
func (s *SQLiteStore) UpdateMissionStatus(ctx context.Context, id string, status models.MissionStatus) error {
	var query string
	var args []interface{}

	now := time.Now()
	switch status {
	case models.MissionStatusActive:
		query = `UPDATE missions SET status = ?, started_at = ? WHERE id = ?`
		args = []interface{}{status, now, id}
	case models.MissionStatusCompleted, models.MissionStatusAborted:
		query = `UPDATE missions SET status = ?, completed_at = ? WHERE id = ?`
		args = []interface{}{status, now, id}
	default:
		query = `UPDATE missions SET status = ? WHERE id = ?`
		args = []interface{}{status, id}
	}

	_, err := s.db.ExecContext(ctx, query, args...)
	return err
}

// ListMissions retrieves a paginated list of missions
func (s *SQLiteStore) ListMissions(ctx context.Context, limit, offset int) ([]*models.Mission, error) {
	query := `SELECT id, alert_id, mode, status, command_json, created_at, started_at, completed_at FROM missions ORDER BY created_at DESC LIMIT ? OFFSET ?`
	rows, err := s.db.QueryContext(ctx, query, limit, offset)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var missions []*models.Mission
	for rows.Next() {
		var mission models.Mission
		var commandJSON string
		var startedAt, completedAt sql.NullTime

		if err := rows.Scan(&mission.ID, &mission.AlertID, &mission.Mode, &mission.Status, &commandJSON, &mission.CreatedAt, &startedAt, &completedAt); err != nil {
			return nil, err
		}

		if err := jsonUnmarshal(commandJSON, &mission.Command); err != nil {
			return nil, fmt.Errorf("failed to unmarshal command: %w", err)
		}

		if startedAt.Valid {
			mission.StartedAt = &startedAt.Time
		}
		if completedAt.Valid {
			mission.CompletedAt = &completedAt.Time
		}

		missions = append(missions, &mission)
	}
	return missions, rows.Err()
}

// InsertTelemetry inserts a new telemetry record
func (s *SQLiteStore) InsertTelemetry(ctx context.Context, telemetry *models.HardwareTelemetry) error {
	imageMetaJSON := "null"
	if telemetry.ImageMetadata != nil {
		var err error
		imageMetaJSON, err = jsonMarshal(telemetry.ImageMetadata)
		if err != nil {
			return fmt.Errorf("failed to marshal image metadata: %w", err)
		}
	}

	extraJSON := "null"
	if telemetry.Extra != nil {
		var err error
		extraJSON, err = jsonMarshal(telemetry.Extra)
		if err != nil {
			return fmt.Errorf("failed to marshal extra data: %w", err)
		}
	}

	query := `INSERT INTO telemetry (platform_id, mission_id, latitude, longitude, temperature, humidity, water_level, image_metadata_json, timestamp, extra_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`
	_, err := s.db.ExecContext(ctx, query,
		telemetry.PlatformID,
		telemetry.MissionID,
		telemetry.Latitude,
		telemetry.Longitude,
		telemetry.Temperature,
		telemetry.Humidity,
		telemetry.WaterLevel,
		imageMetaJSON,
		telemetry.Timestamp,
		extraJSON,
	)
	return err
}

// GetTelemetryByMission retrieves telemetry for a specific mission
func (s *SQLiteStore) GetTelemetryByMission(ctx context.Context, missionID string, limit, offset int) ([]*models.HardwareTelemetry, error) {
	query := `SELECT platform_id, mission_id, latitude, longitude, temperature, humidity, water_level, image_metadata_json, timestamp, extra_json FROM telemetry WHERE mission_id = ? ORDER BY timestamp DESC LIMIT ? OFFSET ?`
	rows, err := s.db.QueryContext(ctx, query, missionID, limit, offset)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var telemetryList []*models.HardwareTelemetry
	for rows.Next() {
		var t models.HardwareTelemetry
		var imageMetaJSON, extraJSON sql.NullString

		if err := rows.Scan(&t.PlatformID, &t.MissionID, &t.Latitude, &t.Longitude, &t.Temperature, &t.Humidity, &t.WaterLevel, &imageMetaJSON, &t.Timestamp, &extraJSON); err != nil {
			return nil, err
		}

		if imageMetaJSON.Valid && imageMetaJSON.String != "null" {
			if err := jsonUnmarshal(imageMetaJSON.String, &t.ImageMetadata); err != nil {
				return nil, fmt.Errorf("failed to unmarshal image metadata: %w", err)
			}
		}

		if extraJSON.Valid && extraJSON.String != "null" {
			if err := jsonUnmarshal(extraJSON.String, &t.Extra); err != nil {
				return nil, fmt.Errorf("failed to unmarshal extra data: %w", err)
			}
		}

		telemetryList = append(telemetryList, &t)
	}
	return telemetryList, rows.Err()
}

// GetLatestTelemetry retrieves the most recent telemetry for a platform
func (s *SQLiteStore) GetLatestTelemetry(ctx context.Context, platformID string) (*models.HardwareTelemetry, error) {
	query := `SELECT platform_id, mission_id, latitude, longitude, temperature, humidity, water_level, image_metadata_json, timestamp, extra_json FROM telemetry WHERE platform_id = ? ORDER BY timestamp DESC LIMIT 1`
	row := s.db.QueryRowContext(ctx, query, platformID)

	var t models.HardwareTelemetry
	var imageMetaJSON, extraJSON sql.NullString

	err := row.Scan(&t.PlatformID, &t.MissionID, &t.Latitude, &t.Longitude, &t.Temperature, &t.Humidity, &t.WaterLevel, &imageMetaJSON, &t.Timestamp, &extraJSON)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}

	if imageMetaJSON.Valid && imageMetaJSON.String != "null" {
		if err := jsonUnmarshal(imageMetaJSON.String, &t.ImageMetadata); err != nil {
			return nil, fmt.Errorf("failed to unmarshal image metadata: %w", err)
		}
	}

	if extraJSON.Valid && extraJSON.String != "null" {
		if err := jsonUnmarshal(extraJSON.String, &t.Extra); err != nil {
			return nil, fmt.Errorf("failed to unmarshal extra data: %w", err)
		}
	}

	return &t, nil
}

// Close closes the database connection
func (s *SQLiteStore) Close() error {
	return s.db.Close()
}

// Helper functions for JSON marshaling
func jsonMarshal(v interface{}) (string, error) {
	b, err := json.Marshal(v)
	return string(b), err
}

func jsonUnmarshal(data string, v interface{}) error {
	return json.Unmarshal([]byte(data), v)
}