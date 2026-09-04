package telemetry

import (
	"context"
	"fmt"
	"log"
	"sync"
	"time"

	"disaster-response/internal/models"
	"disaster-response/internal/storage"
)

// Hub defines the interface for telemetry operations
type Hub interface {
	// Hardware connection management
	RegisterHardware(ctx context.Context, platformID, missionID string) (<-chan *models.MissionCommand, chan<- *models.HardwareTelemetry, error)
	UnregisterHardware(platformID string)

	// Dashboard subscription
	SubscribeDashboard() chan *models.HardwareTelemetry
	UnsubscribeDashboard(ch chan *models.HardwareTelemetry)

	// Direct telemetry ingestion (for testing or non-WS sources)
	IngestTelemetry(ctx context.Context, telemetry *models.HardwareTelemetry) error

	// Get active hardware count
	ActiveHardwareCount() int

	// Mission mode cache for priority routing
	SetMissionMode(missionID string, mode models.MissionMode)
}

// TelemetryHub implements the telemetry hub with async channel processing
type TelemetryHub struct {
	store storage.Store

	// Hardware connections: platformID -> hardware connection
	hardwareConns map[string]*HardwareConnection
	hardwareMu    sync.RWMutex

	// Dashboard subscribers
	dashboardSubscribers map[chan *models.HardwareTelemetry]bool
	dashboardMu          sync.RWMutex

	// Incoming telemetry channel for processing
	ingestChan chan *models.HardwareTelemetry

	// Mission mode cache for priority routing
	missionModes map[string]models.MissionMode
	missionMu    sync.RWMutex

	// Shutdown control
	shutdownCtx context.Context
	shutdownFn  context.CancelFunc
	wg          sync.WaitGroup
}

// HardwareConnection represents an active hardware platform connection
type HardwareConnection struct {
	PlatformID    string
	MissionID     string
	CommandChan   chan *models.MissionCommand
	TelemetryChan chan *models.HardwareTelemetry
	LastSeen      time.Time
	Ctx           context.Context
	Cancel        context.CancelFunc
}

// NewTelemetryHub creates a new telemetry hub
func NewTelemetryHub(store storage.Store) *TelemetryHub {
	ctx, cancel := context.WithCancel(context.Background())
	hub := &TelemetryHub{
		store:                store,
		hardwareConns:        make(map[string]*HardwareConnection),
		dashboardSubscribers: make(map[chan *models.HardwareTelemetry]bool),
		ingestChan:           make(chan *models.HardwareTelemetry, 1000),
		missionModes:         make(map[string]models.MissionMode),
		shutdownCtx:          ctx,
		shutdownFn:           cancel,
	}

	// Start background processor
	hub.wg.Add(1)
	go hub.processTelemetry()

	return hub
}

// RegisterHardware registers a new hardware platform connection
func (h *TelemetryHub) RegisterHardware(ctx context.Context, platformID, missionID string) (<-chan *models.MissionCommand, chan<- *models.HardwareTelemetry, error) {
	h.hardwareMu.Lock()
	defer h.hardwareMu.Unlock()

	// Check if already registered
	if _, exists := h.hardwareConns[platformID]; exists {
		return nil, nil, fmt.Errorf("hardware already registered: %s", platformID)
	}

	// Create connection context with cancel
	connCtx, connCancel := context.WithCancel(ctx)

	conn := &HardwareConnection{
		PlatformID:    platformID,
		MissionID:     missionID,
		CommandChan:   make(chan *models.MissionCommand, 10),
		TelemetryChan: make(chan *models.HardwareTelemetry, 100),
		LastSeen:      time.Now(),
		Ctx:           connCtx,
		Cancel:        connCancel,
	}

	h.hardwareConns[platformID] = conn

	// Start hardware-specific processor
	h.wg.Add(1)
	go h.processHardwareTelemetry(conn)

	log.Printf("Hardware registered: %s (mission: %s)", platformID, missionID)
	return conn.CommandChan, conn.TelemetryChan, nil
}

// UnregisterHardware removes a hardware platform connection
func (h *TelemetryHub) UnregisterHardware(platformID string) {
	h.hardwareMu.Lock()
	conn, exists := h.hardwareConns[platformID]
	if exists {
		delete(h.hardwareConns, platformID)
	}
	h.hardwareMu.Unlock()

	if conn != nil {
		conn.Cancel()
		close(conn.CommandChan)
		close(conn.TelemetryChan)
		log.Printf("Hardware unregistered: %s", platformID)
	}
}

// SubscribeDashboard adds a new dashboard subscriber
func (h *TelemetryHub) SubscribeDashboard() chan *models.HardwareTelemetry {
	h.dashboardMu.Lock()
	defer h.dashboardMu.Unlock()

	ch := make(chan *models.HardwareTelemetry, 100)
	h.dashboardSubscribers[ch] = true
	log.Printf("Dashboard subscriber added (total: %d)", len(h.dashboardSubscribers))
	return ch
}

// UnsubscribeDashboard removes a dashboard subscriber
func (h *TelemetryHub) UnsubscribeDashboard(ch chan *models.HardwareTelemetry) {
	h.dashboardMu.Lock()
	defer h.dashboardMu.Unlock()

	// Find and remove the channel
	if _, exists := h.dashboardSubscribers[ch]; exists {
		delete(h.dashboardSubscribers, ch)
		close(ch)
	}
	log.Printf("Dashboard subscriber removed (total: %d)", len(h.dashboardSubscribers))
}

// IngestTelemetry directly ingests telemetry data
func (h *TelemetryHub) IngestTelemetry(ctx context.Context, telemetry *models.HardwareTelemetry) error {
	select {
	case h.ingestChan <- telemetry:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	case <-h.shutdownCtx.Done():
		return fmt.Errorf("hub is shutting down")
	}
}

// ActiveHardwareCount returns the number of active hardware connections
func (h *TelemetryHub) ActiveHardwareCount() int {
	h.hardwareMu.RLock()
	defer h.hardwareMu.RUnlock()
	return len(h.hardwareConns)
}

// SetMissionMode caches the mission mode for priority routing
func (h *TelemetryHub) SetMissionMode(missionID string, mode models.MissionMode) {
	h.missionMu.Lock()
	defer h.missionMu.Unlock()
	h.missionModes[missionID] = mode
}

// GetMissionMode retrieves the cached mission mode
func (h *TelemetryHub) GetMissionMode(missionID string) (models.MissionMode, bool) {
	h.missionMu.RLock()
	defer h.missionMu.RUnlock()
	mode, ok := h.missionModes[missionID]
	return mode, ok
}

// processTelemetry is the main background processor for incoming telemetry
func (h *TelemetryHub) processTelemetry() {
	defer h.wg.Done()

	for {
		select {
		case <-h.shutdownCtx.Done():
			log.Println("Telemetry processor shutting down")
			return
		case telemetry := <-h.ingestChan:
			h.handleTelemetry(telemetry)
		}
	}
}

// processHardwareTelemetry processes telemetry from a specific hardware connection
func (h *TelemetryHub) processHardwareTelemetry(conn *HardwareConnection) {
	defer h.wg.Done()

	for {
		select {
		case <-h.shutdownCtx.Done():
			return
		case <-conn.Ctx.Done():
			return
		case telemetry, ok := <-conn.TelemetryChan:
			if !ok {
				return
			}
			// Enrich with connection info
			telemetry.PlatformID = conn.PlatformID
			telemetry.MissionID = conn.MissionID
			conn.LastSeen = time.Now()

			// Process through main pipeline
			select {
			case h.ingestChan <- telemetry:
			case <-h.shutdownCtx.Done():
				return
			}
		}
	}
}

// handleTelemetry processes a single telemetry reading
func (h *TelemetryHub) handleTelemetry(telemetry *models.HardwareTelemetry) {
	// 1. Persist to database (async, non-blocking)
	h.wg.Add(1)
	go func(t *models.HardwareTelemetry) {
		defer h.wg.Done()
		ctx, cancel := context.WithTimeout(h.shutdownCtx, 5*time.Second)
		defer cancel()
		if err := h.store.InsertTelemetry(ctx, t); err != nil {
			log.Printf("Failed to persist telemetry: %v", err)
		}
	}(telemetry)

	// 2. Check if this is high-priority data based on mission mode
	if h.isHighPriority(telemetry) {
		// 3. Broadcast to dashboard subscribers
		h.broadcastToDashboards(telemetry)
	}
}

// isHighPriority determines if telemetry should be broadcast to dashboards
func (h *TelemetryHub) isHighPriority(telemetry *models.HardwareTelemetry) bool {
	mode, ok := h.GetMissionMode(telemetry.MissionID)
	if !ok {
		// Default: broadcast all if mode unknown
		return true
	}

	// Define high-priority sensors per mission mode
	highPrioritySensors := map[models.MissionMode][]string{
		models.MissionModeFire:      {"thermal", "temperature", "air_quality"},
		models.MissionModeFlood:     {"water_level", "flow_rate"},
		models.MissionModeLandslide: {"accelerometer", "inclination"},
		models.MissionModeEarthquake: {"seismic", "gas_leak"},
	}

	prioritySensors := highPrioritySensors[mode]
	if len(prioritySensors) == 0 {
		return true
	}

	// Check if telemetry contains any high-priority sensor data
	// This is a simplified check - in production, you'd check the Extra field or specific fields
	extra := telemetry.Extra
	if extra != nil {
		for _, sensor := range prioritySensors {
			if _, exists := extra[sensor]; exists {
				return true
			}
		}
	}

	// Always broadcast if we have image metadata (visual data)
	if telemetry.ImageMetadata != nil {
		return true
	}

	// For fire mode, always broadcast temperature
	if mode == models.MissionModeFire && telemetry.Temperature > 0 {
		return true
	}

	// For flood mode, always broadcast water level
	if mode == models.MissionModeFlood && telemetry.WaterLevel > 0 {
		return true
	}

	return false
}

// broadcastToDashboards sends telemetry to all dashboard subscribers
func (h *TelemetryHub) broadcastToDashboards(telemetry *models.HardwareTelemetry) {
	h.dashboardMu.RLock()
	defer h.dashboardMu.RUnlock()

	for ch := range h.dashboardSubscribers {
		// Non-blocking send with timeout
		select {
		case ch <- telemetry:
		case <-time.After(100 * time.Millisecond):
			log.Printf("Dashboard subscriber slow, dropping telemetry")
		}
	}
}

// SendCommandToHardware sends a mission command to specific hardware
func (h *TelemetryHub) SendCommandToHardware(platformID string, command *models.MissionCommand) error {
	h.hardwareMu.RLock()
	conn, exists := h.hardwareConns[platformID]
	h.hardwareMu.RUnlock()

	if !exists {
		return fmt.Errorf("hardware not connected: %s", platformID)
	}

	select {
	case conn.CommandChan <- command:
		return nil
	case <-conn.Ctx.Done():
		return fmt.Errorf("hardware connection closed: %s", platformID)
	case <-time.After(time.Second):
		return fmt.Errorf("command send timeout: %s", platformID)
	}
}

// BroadcastCommandToMission sends a command to all hardware in a mission
func (h *TelemetryHub) BroadcastCommandToMission(missionID string, command *models.MissionCommand) int {
	h.hardwareMu.RLock()
	defer h.hardwareMu.RUnlock()

	count := 0
	for _, conn := range h.hardwareConns {
		if conn.MissionID == missionID {
			select {
			case conn.CommandChan <- command:
				count++
			default:
				log.Printf("Command channel full for hardware: %s", conn.PlatformID)
			}
		}
	}
	return count
}

// Shutdown gracefully shuts down the telemetry hub
func (h *TelemetryHub) Shutdown() {
	log.Println("Shutting down telemetry hub...")
	h.shutdownFn()
	h.wg.Wait()

	// Close all hardware connections
	h.hardwareMu.Lock()
	for _, conn := range h.hardwareConns {
		conn.Cancel()
		close(conn.CommandChan)
		close(conn.TelemetryChan)
	}
	h.hardwareConns = make(map[string]*HardwareConnection)
	h.hardwareMu.Unlock()

	// Close all dashboard subscribers
	h.dashboardMu.Lock()
	for ch := range h.dashboardSubscribers {
		close(ch)
	}
	h.dashboardSubscribers = make(map[chan *models.HardwareTelemetry]bool)
	h.dashboardMu.Unlock()

	close(h.ingestChan)
	log.Println("Telemetry hub shutdown complete")
}