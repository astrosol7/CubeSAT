package mission

import (
	"context"
	"fmt"
	"log"
	"sync"
	"time"

	"disaster-response/internal/models"
	"disaster-response/internal/storage"
)

// Controller defines the interface for mission control operations
type Controller interface {
	ProcessAlert(ctx context.Context, alert *models.DisasterAlert) (*models.MissionCommand, error)
	GetMission(ctx context.Context, missionID string) (*models.Mission, error)
	UpdateMissionStatus(ctx context.Context, missionID string, status models.MissionStatus) error
	ListActiveMissions(ctx context.Context) ([]*models.Mission, error)
}

// MissionController implements the mission control logic
type MissionController struct {
	store  storage.Store
	config Config
	mu     sync.RWMutex
}

// Config holds configuration for the mission controller
type Config struct {
	DefaultSamplingRateHz map[models.MissionMode]int
	PrioritySensors       map[models.MissionMode][]string
	DefaultRadiusKm       float64
}

// DefaultConfig returns the default mission configuration
func DefaultConfig() Config {
	return Config{
		DefaultSamplingRateHz: map[models.MissionMode]int{
			models.MissionModeFire:      10, // High frequency for fire monitoring
			models.MissionModeFlood:     5,  // Medium frequency for flood
			models.MissionModeLandslide: 2,  // Lower frequency for landslide
			models.MissionModeEarthquake: 1, // Lowest for earthquake aftershocks
		},
		PrioritySensors: map[models.MissionMode][]string{
			models.MissionModeFire:      {"thermal", "temperature", "wind_speed", "air_quality"},
			models.MissionModeFlood:     {"water_level", "visual", "flow_rate", "precipitation"},
			models.MissionModeLandslide: {"accelerometer", "inclination", "soil_moisture", "visual"},
			models.MissionModeEarthquake: {"seismic", "accelerometer", "structural_health", "gas_leak"},
		},
		DefaultRadiusKm: 5.0,
	}
}

// NewMissionController creates a new mission controller
func NewMissionController(store storage.Store, config Config) *MissionController {
	if config.DefaultSamplingRateHz == nil {
		config = DefaultConfig()
	}
	return &MissionController{
		store:  store,
		config: config,
	}
}

// ProcessAlert processes a disaster alert and creates a mission command
func (c *MissionController) ProcessAlert(ctx context.Context, alert *models.DisasterAlert) (*models.MissionCommand, error) {
	log.Printf("Processing alert: %s (type: %s, severity: %s)", alert.ID, alert.Type, alert.Severity)

	// Determine mission mode from disaster type
	mode := c.determineMode(alert.Type)
	if mode == "" {
		return nil, fmt.Errorf("unknown disaster type: %s", alert.Type)
	}

	// Get configuration for this mode
	samplingRate := c.config.DefaultSamplingRateHz[mode]
	if samplingRate == 0 {
		samplingRate = 5 // Default fallback
	}

	prioritySensors := c.config.PrioritySensors[mode]
	if prioritySensors == nil {
		prioritySensors = []string{"visual"}
	}

	// Generate mission ID
	missionID := fmt.Sprintf("mission-%s-%d", alert.Type, time.Now().UnixNano())

	// Create mission command
	command := &models.MissionCommand{
		MissionID:       missionID,
		Mode:            mode,
		SamplingRateHz:  samplingRate,
		PrioritySensors: prioritySensors,
		TargetArea: models.TargetArea{
			CenterLatitude:  alert.Latitude,
			CenterLongitude: alert.Longitude,
			RadiusKm:        c.config.DefaultRadiusKm,
		},
		Timestamp: time.Now(),
	}

	// Create mission record
	mission := &models.Mission{
		ID:        missionID,
		AlertID:   alert.ID,
		Mode:      mode,
		Status:    models.MissionStatusPending,
		Command:   *command,
		CreatedAt: time.Now(),
	}

	// Persist alert and mission
	if err := c.store.CreateAlert(ctx, alert); err != nil {
		return nil, fmt.Errorf("failed to store alert: %w", err)
	}

	if err := c.store.CreateMission(ctx, mission); err != nil {
		return nil, fmt.Errorf("failed to store mission: %w", err)
	}

	log.Printf("Created mission: %s (mode: %s, sampling: %d Hz)", missionID, mode, samplingRate)
	return command, nil
}

// determineMode maps disaster type to mission mode
func (c *MissionController) determineMode(disasterType models.DisasterType) models.MissionMode {
	switch disasterType {
	case models.DisasterTypeWildfire:
		return models.MissionModeFire
	case models.DisasterTypeFlood:
		return models.MissionModeFlood
	case models.DisasterTypeLandslide:
		return models.MissionModeLandslide
	case models.DisasterTypeEarthquake:
		return models.MissionModeEarthquake
	default:
		return ""
	}
}

// GetMission retrieves a mission by ID
func (c *MissionController) GetMission(ctx context.Context, missionID string) (*models.Mission, error) {
	return c.store.GetMission(ctx, missionID)
}

// UpdateMissionStatus updates the status of a mission
func (c *MissionController) UpdateMissionStatus(ctx context.Context, missionID string, status models.MissionStatus) error {
	return c.store.UpdateMissionStatus(ctx, missionID, status)
}

// ListActiveMissions returns all active missions
func (c *MissionController) ListActiveMissions(ctx context.Context) ([]*models.Mission, error) {
	// For simplicity, return all missions - in production, filter by status
	missions, err := c.store.ListMissions(ctx, 100, 0)
	if err != nil {
		return nil, err
	}

	var active []*models.Mission
	for _, m := range missions {
		if m.Status == models.MissionStatusActive || m.Status == models.MissionStatusPending {
			active = append(active, m)
		}
	}
	return active, nil
}

// MissionStateMachine provides a more explicit state machine for complex transitions
type MissionStateMachine struct {
	controller *MissionController
	mu         sync.Mutex
	state      map[string]models.MissionStatus
}

// NewMissionStateMachine creates a new state machine
func NewMissionStateMachine(controller *MissionController) *MissionStateMachine {
	return &MissionStateMachine{
		controller: controller,
		state:      make(map[string]models.MissionStatus),
	}
}

// Transition attempts to transition a mission to a new state
func (sm *MissionStateMachine) Transition(ctx context.Context, missionID string, newStatus models.MissionStatus) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	currentStatus, exists := sm.state[missionID]
	if !exists {
		// Load from store
		mission, err := sm.controller.GetMission(ctx, missionID)
		if err != nil {
			return err
		}
		if mission == nil {
			return fmt.Errorf("mission not found: %s", missionID)
		}
		currentStatus = mission.Status
	}

	// Validate transition
	if !isValidTransition(currentStatus, newStatus) {
		return fmt.Errorf("invalid transition from %s to %s", currentStatus, newStatus)
	}

	// Persist transition
	if err := sm.controller.UpdateMissionStatus(ctx, missionID, newStatus); err != nil {
		return err
	}

	sm.state[missionID] = newStatus
	log.Printf("Mission %s transitioned: %s -> %s", missionID, currentStatus, newStatus)
	return nil
}

// isValidTransition checks if a state transition is valid
func isValidTransition(from, to models.MissionStatus) bool {
	validTransitions := map[models.MissionStatus][]models.MissionStatus{
		models.MissionStatusPending:   {models.MissionStatusActive, models.MissionStatusAborted},
		models.MissionStatusActive:    {models.MissionStatusCompleted, models.MissionStatusAborted},
		models.MissionStatusCompleted: {},
		models.MissionStatusAborted:   {},
	}

	allowed, ok := validTransitions[from]
	if !ok {
		return false
	}

	for _, status := range allowed {
		if status == to {
			return true
		}
	}
	return false
}