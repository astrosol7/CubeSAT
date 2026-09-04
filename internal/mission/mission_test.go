package mission

import (
	"context"
	"sync"
	"testing"
	"time"

	"disaster-response/internal/models"
)

// MockStore implements storage.Store for testing
type MockStore struct {
	alerts   map[string]*models.DisasterAlert
	missions map[string]*models.Mission
	mu       sync.Mutex
}

func NewMockStore() *MockStore {
	return &MockStore{
		alerts:   make(map[string]*models.DisasterAlert),
		missions: make(map[string]*models.Mission),
	}
}

func (m *MockStore) CreateAlert(ctx context.Context, alert *models.DisasterAlert) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.alerts[alert.ID] = alert
	return nil
}

func (m *MockStore) GetAlert(ctx context.Context, id string) (*models.DisasterAlert, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	return m.alerts[id], nil
}

func (m *MockStore) ListAlerts(ctx context.Context, limit, offset int) ([]*models.DisasterAlert, error) {
	return nil, nil
}

func (m *MockStore) CreateMission(ctx context.Context, mission *models.Mission) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.missions[mission.ID] = mission
	return nil
}

func (m *MockStore) GetMission(ctx context.Context, id string) (*models.Mission, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	return m.missions[id], nil
}

func (m *MockStore) UpdateMissionStatus(ctx context.Context, id string, status models.MissionStatus) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m, ok := m.missions[id]; ok {
		m.Status = status
		now := time.Now()
		switch status {
		case models.MissionStatusActive:
			m.StartedAt = &now
		case models.MissionStatusCompleted, models.MissionStatusAborted:
			m.CompletedAt = &now
		}
	}
	return nil
}

func (m *MockStore) ListMissions(ctx context.Context, limit, offset int) ([]*models.Mission, error) {
	return nil, nil
}

func (m *MockStore) InsertTelemetry(ctx context.Context, telemetry *models.HardwareTelemetry) error {
	return nil
}

func (m *MockStore) GetTelemetryByMission(ctx context.Context, missionID string, limit, offset int) ([]*models.HardwareTelemetry, error) {
	return nil, nil
}

func (m *MockStore) GetLatestTelemetry(ctx context.Context, platformID string) (*models.HardwareTelemetry, error) {
	return nil, nil
}

func (m *MockStore) Close() error {
	return nil
}

func TestMissionController_ProcessAlert(t *testing.T) {
	tests := []struct {
		name          string
		alert         *models.DisasterAlert
		expectedMode  models.MissionMode
		expectedRate  int
		expectError   bool
	}{
		{
			name: "wildfire alert creates fire mission",
			alert: &models.DisasterAlert{
				ID:        "alert-1",
				Type:      models.DisasterTypeWildfire,
				Latitude:  37.7749,
				Longitude: -122.4194,
				Severity:  models.SeverityHigh,
				Timestamp: time.Now(),
			},
			expectedMode: models.MissionModeFire,
			expectedRate: 10,
			expectError:  false,
		},
		{
			name: "flood alert creates flood mission",
			alert: &models.DisasterAlert{
				ID:        "alert-2",
				Type:      models.DisasterTypeFlood,
				Latitude:  34.0522,
				Longitude: -118.2437,
				Severity:  models.SeverityMedium,
				Timestamp: time.Now(),
			},
			expectedMode: models.MissionModeFlood,
			expectedRate: 5,
			expectError:  false,
		},
		{
			name: "landslide alert creates landslide mission",
			alert: &models.DisasterAlert{
				ID:        "alert-3",
				Type:      models.DisasterTypeLandslide,
				Latitude:  40.7128,
				Longitude: -74.0060,
				Severity:  models.SeverityCritical,
				Timestamp: time.Now(),
			},
			expectedMode: models.MissionModeLandslide,
			expectedRate: 2,
			expectError:  false,
		},
		{
			name: "earthquake alert creates earthquake mission",
			alert: &models.DisasterAlert{
				ID:        "alert-4",
				Type:      models.DisasterTypeEarthquake,
				Latitude:  35.6762,
				Longitude: 139.6503,
				Severity:  models.SeverityHigh,
				Timestamp: time.Now(),
			},
			expectedMode: models.MissionModeEarthquake,
			expectedRate: 1,
			expectError:  false,
		},
		{
			name: "unknown disaster type returns error",
			alert: &models.DisasterAlert{
				ID:        "alert-5",
				Type:      "Unknown",
				Latitude:  0,
				Longitude: 0,
				Severity:  models.SeverityLow,
				Timestamp: time.Now(),
			},
			expectedMode: "",
			expectedRate: 0,
			expectError:  true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mockStore := NewMockStore()
			controller := NewMissionController(mockStore, DefaultConfig())

			command, err := controller.ProcessAlert(context.Background(), tt.alert)

			if tt.expectError {
				if err == nil {
					t.Errorf("Expected error but got none")
				}
				return
			}

			if err != nil {
				t.Errorf("Unexpected error: %v", err)
				return
			}

			if command == nil {
				t.Errorf("Expected command but got nil")
				return
			}

			if command.Mode != tt.expectedMode {
				t.Errorf("Expected mode %s, got %s", tt.expectedMode, command.Mode)
			}

			if command.SamplingRateHz != tt.expectedRate {
				t.Errorf("Expected sampling rate %d, got %d", tt.expectedRate, command.SamplingRateHz)
			}

			if command.MissionID == "" {
				t.Errorf("Expected mission ID to be set")
			}

			if len(command.PrioritySensors) == 0 {
				t.Errorf("Expected priority sensors to be set")
			}

			// Verify mission was stored
			mission, err := mockStore.GetMission(context.Background(), command.MissionID)
			if err != nil {
				t.Errorf("Failed to retrieve stored mission: %v", err)
			}
			if mission == nil {
				t.Errorf("Mission not stored")
			}
			if mission.Status != models.MissionStatusPending {
				t.Errorf("Expected mission status PENDING, got %s", mission.Status)
			}
		})
	}
}

func TestMissionStateMachine_Transitions(t *testing.T) {
	mockStore := NewMockStore()
	controller := NewMissionController(mockStore, DefaultConfig())
	stateMachine := NewMissionStateMachine(controller)

	// Create a mission first
	alert := &models.DisasterAlert{
		ID:        "alert-test",
		Type:      models.DisasterTypeWildfire,
		Latitude:  37.7749,
		Longitude: -122.4194,
		Severity:  models.SeverityHigh,
		Timestamp: time.Now(),
	}
	command, err := controller.ProcessAlert(context.Background(), alert)
	if err != nil {
		t.Fatalf("Failed to create mission: %v", err)
	}

	// Test valid transitions
	tests := []struct {
		name          string
		fromStatus    models.MissionStatus
		toStatus      models.MissionStatus
		expectError   bool
	}{
		{
			name:        "pending to active",
			fromStatus:  models.MissionStatusPending,
			toStatus:    models.MissionStatusActive,
			expectError: false,
		},
		{
			name:        "active to completed",
			fromStatus:  models.MissionStatusActive,
			toStatus:    models.MissionStatusCompleted,
			expectError: false,
		},
		{
			name:        "pending to aborted",
			fromStatus:  models.MissionStatusPending,
			toStatus:    models.MissionStatusAborted,
			expectError: false,
		},
		{
			name:        "active to aborted",
			fromStatus:  models.MissionStatusActive,
			toStatus:    models.MissionStatusAborted,
			expectError: false,
		},
		{
			name:        "completed to active (invalid)",
			fromStatus:  models.MissionStatusCompleted,
			toStatus:    models.MissionStatusActive,
			expectError: true,
		},
		{
			name:        "aborted to active (invalid)",
			fromStatus:  models.MissionStatusAborted,
			toStatus:    models.MissionStatusActive,
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Set up initial state
			if err := controller.UpdateMissionStatus(context.Background(), command.MissionID, tt.fromStatus); err != nil {
				t.Fatalf("Failed to set initial status: %v", err)
			}

			err := stateMachine.Transition(context.Background(), command.MissionID, tt.toStatus)

			if tt.expectError {
				if err == nil {
					t.Errorf("Expected error for invalid transition")
				}
			} else {
				if err != nil {
					t.Errorf("Unexpected error: %v", err)
				}
				// Verify state was updated
				mission, _ := controller.GetMission(context.Background(), command.MissionID)
				if mission.Status != tt.toStatus {
					t.Errorf("Expected status %s, got %s", tt.toStatus, mission.Status)
				}
			}
		})
	}
}