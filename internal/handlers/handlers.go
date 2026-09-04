package handlers

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/gorilla/websocket"

	"disaster-response/internal/mission"
	"disaster-response/internal/models"
	"disaster-response/internal/telemetry"
)

// HandlerConfig holds configuration for HTTP handlers
type HandlerConfig struct {
	MissionController mission.Controller
	TelemetryHub      telemetry.Hub
	ReadTimeout       time.Duration
	WriteTimeout      time.Duration
}

// Handlers holds all HTTP handlers
type Handlers struct {
	config HandlerConfig
}

// NewHandlers creates a new handlers instance
func NewHandlers(config HandlerConfig) *Handlers {
	return &Handlers{config: config}
}

// RegisterRoutes registers all HTTP routes
func (h *Handlers) RegisterRoutes(mux *http.ServeMux) {
	// Alert ingestion endpoint
	mux.HandleFunc("POST /api/alerts", h.handleIngestAlert)

	// WebSocket endpoints
	mux.HandleFunc("GET /ws/telemetry", h.handleTelemetryWS)
	mux.HandleFunc("GET /ws/dashboard", h.handleDashboardWS)

	// Health check
	mux.HandleFunc("GET /health", h.handleHealth)

	// Mission status endpoints
	mux.HandleFunc("GET /api/missions", h.handleListMissions)
	mux.HandleFunc("GET /api/missions/{id}", h.handleGetMission)
	mux.HandleFunc("POST /api/missions/{id}/activate", h.handleActivateMission)
	mux.HandleFunc("POST /api/missions/{id}/complete", h.handleCompleteMission)
}

// handleIngestAlert handles POST /api/alerts
func (h *Handlers) handleIngestAlert(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	var alert models.DisasterAlert
	if err := json.NewDecoder(r.Body).Decode(&alert); err != nil {
		http.Error(w, fmt.Sprintf("invalid request body: %v", err), http.StatusBadRequest)
		return
	}

	// Validate required fields
	if alert.ID == "" {
		http.Error(w, "alert ID is required", http.StatusBadRequest)
		return
	}
	if alert.Type == "" {
		http.Error(w, "alert type is required", http.StatusBadRequest)
		return
	}
	if alert.Latitude == 0 && alert.Longitude == 0 {
		http.Error(w, "coordinates are required", http.StatusBadRequest)
		return
	}
	if alert.Timestamp.IsZero() {
		alert.Timestamp = time.Now()
	}

	// Process alert through mission controller
	command, err := h.config.MissionController.ProcessAlert(ctx, &alert)
	if err != nil {
		log.Printf("Failed to process alert: %v", err)
		http.Error(w, fmt.Sprintf("failed to process alert: %v", err), http.StatusInternalServerError)
		return
	}

	// Cache mission mode in telemetry hub for priority routing
	h.config.TelemetryHub.SetMissionMode(command.MissionID, command.Mode)

	// Return the mission command
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status":  "accepted",
		"mission": command,
	})
}

// handleTelemetryWS handles WebSocket connections from hardware platforms
func (h *Handlers) handleTelemetryWS(w http.ResponseWriter, r *http.Request) {
	// Upgrade to WebSocket
	upgrader := websocket.Upgrader{
		CheckOrigin: func(r *http.Request) bool { return true }, // Allow all origins for MVP
		ReadBufferSize:  1024,
		WriteBufferSize: 1024,
	}

	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("WebSocket upgrade failed: %v", err)
		return
	}
	defer conn.Close()

	// Parse query parameters for platform ID and mission ID
	query := r.URL.Query()
	platformID := query.Get("platform_id")
	missionID := query.Get("mission_id")

	if platformID == "" {
		log.Println("WebSocket connection missing platform_id")
		conn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.ClosePolicyViolation, "platform_id required"))
		return
	}

	if missionID == "" {
		log.Println("WebSocket connection missing mission_id")
		conn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.ClosePolicyViolation, "mission_id required"))
		return
	}

	log.Printf("Hardware connected: %s (mission: %s)", platformID, missionID)

	// Register with telemetry hub
	cmdChan, telemetryChan, err := h.config.TelemetryHub.RegisterHardware(r.Context(), platformID, missionID)
	if err != nil {
		log.Printf("Failed to register hardware: %v", err)
		conn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseInternalServerErr, err.Error()))
		return
	}
	defer h.config.TelemetryHub.UnregisterHardware(platformID)

	// Send initial mission command if available
	mission, err := h.config.MissionController.GetMission(r.Context(), missionID)
	if err == nil && mission != nil {
		if err := conn.WriteJSON(mission.Command); err != nil {
			log.Printf("Failed to send initial command: %v", err)
			return
		}
	}

	// Create context for this connection
	ctx, cancel := context.WithCancel(r.Context())
	defer cancel()

	// Channel for incoming WebSocket messages
	wsMessages := make(chan []byte, 100)

	// Goroutine to read from WebSocket
	go func() {
		defer close(wsMessages)
		for {
			_, msg, err := conn.ReadMessage()
			if err != nil {
				if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
					log.Printf("WebSocket read error: %v", err)
				}
				cancel()
				return
			}
			select {
			case wsMessages <- msg:
			case <-ctx.Done():
				return
			}
		}
	}()

	// Main connection loop
	for {
		select {
		case <-ctx.Done():
			return

		case <-r.Context().Done():
			return

		case msg := <-wsMessages:
			// Handle incoming telemetry from hardware
			var telemetry models.HardwareTelemetry
			if err := json.Unmarshal(msg, &telemetry); err != nil {
				log.Printf("Failed to parse telemetry: %v", err)
				continue
			}
			telemetry.PlatformID = platformID
			telemetry.MissionID = missionID
			telemetry.Timestamp = time.Now()

			// Send to telemetry hub
			select {
			case telemetryChan <- &telemetry:
			case <-ctx.Done():
				return
			}

		case command := <-cmdChan:
			// Send mission command to hardware
			if err := conn.WriteJSON(command); err != nil {
				log.Printf("Failed to send command to hardware: %v", err)
				return
			}

		case <-time.After(30 * time.Second):
			// Send ping to keep connection alive
			if err := conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

// handleDashboardWS handles WebSocket connections from dashboard clients
func (h *Handlers) handleDashboardWS(w http.ResponseWriter, r *http.Request) {
	upgrader := websocket.Upgrader{
		CheckOrigin: func(r *http.Request) bool { return true },
		ReadBufferSize:  1024,
		WriteBufferSize: 1024,
	}

	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("Dashboard WebSocket upgrade failed: %v", err)
		return
	}
	defer conn.Close()

	log.Println("Dashboard client connected")

	// Subscribe to telemetry hub
	telemetryChan := h.config.TelemetryHub.SubscribeDashboard()
	defer h.config.TelemetryHub.UnsubscribeDashboard(telemetryChan)

	ctx, cancel := context.WithCancel(r.Context())
	defer cancel()

	// Goroutine to read from WebSocket (for ping/pong and filter messages)
	go func() {
		for {
			_, msg, err := conn.ReadMessage()
			if err != nil {
				cancel()
				return
			}
			// Handle filter messages from dashboard if needed
			var filter map[string]interface{}
			if json.Unmarshal(msg, &filter) == nil {
				// Could implement filtering logic here
				log.Printf("Dashboard filter: %v", filter)
			}
		}
	}()

	// Main loop - forward telemetry to dashboard
	for {
		select {
		case <-ctx.Done():
			return
		case <-r.Context().Done():
			return
		case telemetry := <-telemetryChan:
			// Send telemetry to dashboard
			if err := conn.WriteJSON(telemetry); err != nil {
				log.Printf("Failed to send telemetry to dashboard: %v", err)
				return
			}
		}
	}
}

// handleHealth handles GET /health
func (h *Handlers) handleHealth(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"status":           "healthy",
		"timestamp":        time.Now(),
		"active_hardware":  h.config.TelemetryHub.ActiveHardwareCount(),
	})
}

// handleListMissions handles GET /api/missions
func (h *Handlers) handleListMissions(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	missions, err := h.config.MissionController.ListActiveMissions(ctx)
	if err != nil {
		http.Error(w, fmt.Sprintf("failed to list missions: %v", err), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(missions)
}

// handleGetMission handles GET /api/missions/{id}
func (h *Handlers) handleGetMission(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	missionID := r.PathValue("id")

	mission, err := h.config.MissionController.GetMission(ctx, missionID)
	if err != nil {
		http.Error(w, fmt.Sprintf("failed to get mission: %v", err), http.StatusInternalServerError)
		return
	}
	if mission == nil {
		http.Error(w, "mission not found", http.StatusNotFound)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(mission)
}

// handleActivateMission handles POST /api/missions/{id}/activate
func (h *Handlers) handleActivateMission(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	missionID := r.PathValue("id")

	if err := h.config.MissionController.UpdateMissionStatus(ctx, missionID, models.MissionStatusActive); err != nil {
		http.Error(w, fmt.Sprintf("failed to activate mission: %v", err), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "activated"})
}

// handleCompleteMission handles POST /api/missions/{id}/complete
func (h *Handlers) handleCompleteMission(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	missionID := r.PathValue("id")

	if err := h.config.MissionController.UpdateMissionStatus(ctx, missionID, models.MissionStatusCompleted); err != nil {
		http.Error(w, fmt.Sprintf("failed to complete mission: %v", err), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"status": "completed"})
}