package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"disaster-response/internal/handlers"
	"disaster-response/internal/mission"
	"disaster-response/internal/storage"
	"disaster-response/internal/telemetry"
)

func main() {
	// Configuration flags
	var (
		addr        = flag.String("addr", ":8080", "HTTP server address")
		dbPath      = flag.String("db", "disaster_response.db", "SQLite database path")
		readTimeout = flag.Duration("read-timeout", 10*time.Second, "HTTP read timeout")
		writeTimeout = flag.Duration("write-timeout", 10*time.Second, "HTTP write timeout")
	)
	flag.Parse()

	log.Printf("Starting Disaster Response Backend...")
	log.Printf("Database: %s", *dbPath)
	log.Printf("Server: %s", *addr)

	// Initialize storage
	store, err := storage.NewSQLiteStore(*dbPath)
	if err != nil {
		log.Fatalf("Failed to initialize storage: %v", err)
	}
	defer func() {
		if err := store.Close(); err != nil {
			log.Printf("Error closing storage: %v", err)
		}
	}()

	// Initialize mission controller
	missionConfig := mission.DefaultConfig()
	missionController := mission.NewMissionController(store, missionConfig)

	// Initialize telemetry hub
	telemetryHub := telemetry.NewTelemetryHub(store)
	defer telemetryHub.Shutdown()

	// Initialize handlers
	handlerConfig := handlers.HandlerConfig{
		MissionController: missionController,
		TelemetryHub:      telemetryHub,
		ReadTimeout:       *readTimeout,
		WriteTimeout:      *writeTimeout,
	}
	h := handlers.NewHandlers(handlerConfig)

	// Setup HTTP server
	mux := http.NewServeMux()
	h.RegisterRoutes(mux)

	server := &http.Server{
		Addr:         *addr,
		Handler:      mux,
		ReadTimeout:  *readTimeout,
		WriteTimeout: *writeTimeout,
		IdleTimeout:  120 * time.Second,
	}

	// Start server in a goroutine
	serverErr := make(chan error, 1)
	go func() {
		log.Printf("HTTP server listening on %s", *addr)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			serverErr <- fmt.Errorf("server error: %w", err)
		}
	}()

	// Wait for interrupt signal or server error
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	select {
	case err := <-serverErr:
		log.Printf("Server error: %v", err)
	case sig := <-sigChan:
		log.Printf("Received signal: %v", sig)
	}

	// Graceful shutdown
	log.Println("Shutting down server...")
	shutdownCtx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		log.Printf("Server shutdown error: %v", err)
	}

	log.Println("Server stopped")
}

// Example usage and testing helper
func init() {
	// This allows the main package to be imported for testing
	// while still being buildable as a binary
}