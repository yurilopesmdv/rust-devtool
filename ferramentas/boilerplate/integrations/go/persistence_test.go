package httpapi

import (
	"context"
	"encoding/json"
	"net/http/httptest"
	"os"
	"strings"
	"testing"
	"time"
	"{{PROJECT_NAME}}/internal/infrastructure/database"
	"{{PROJECT_NAME}}/internal/modules/example"
	"{{PROJECT_NAME}}/internal/modules/example/repositories"
)

func TestPersistence(t *testing.T) {
	if os.Getenv("DATABASE_URL") == "" {
		t.Skip("Requer DATABASE_URL de teste descartável")
	}
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()
	pool, err := database.Open(ctx)
	if err != nil {
		t.Fatal(err)
	}
	defer pool.Close()
	if err := database.Prepare(ctx, pool); err != nil {
		t.Fatal(err)
	}
	repo := repositories.Repository{Pool: pool}
	previous, err := repo.Read(ctx)
	if err != nil {
		t.Fatal(err)
	}
	// Register restore after Close so restoration runs first, including on assertion failure.
	restored := false
	defer func() {
		if restored {
			return
		}
		restore, cancel := context.WithTimeout(context.Background(), 2*time.Second)
		defer cancel()
		if err := repo.Write(restore, previous); err != nil {
			t.Error("restore failed")
		}
	}()
	handler := New(example.NewService(repo), pool.Ping, "http://localhost:5173")
	for _, item := range []struct {
		message any
		status  int
	}{
		{strings.Repeat("😀", 200), 200},
		{strings.Repeat("😀", 201), 400},
		{123, 400},
	} {
		payload, _ := json.Marshal(map[string]any{"message": item.message})
		response := httptest.NewRecorder()
		handler.ServeHTTP(response, httptest.NewRequest("POST", "/api/v1/example", strings.NewReader(string(payload))))
		if response.Code != item.status {
			t.Fatalf("unexpected status: %d", response.Code)
		}
		if item.status == 200 {
			response = httptest.NewRecorder()
			handler.ServeHTTP(response, httptest.NewRequest("GET", "/api/v1/example", nil))
			var body example.Example
			if err := json.Unmarshal(response.Body.Bytes(), &body); err != nil || body.Message != item.message {
				t.Fatal("write/read mismatch")
			}
		}
	}
	// Restore while the pool is open, then verify shutdown availability separately.
	if err := repo.Write(ctx, previous); err != nil {
		t.Fatal(err)
	}
	restored = true
	pool.Close()
	for path, status := range map[string]int{"/ready": 503, "/health": 200} {
		response := httptest.NewRecorder()
		handler.ServeHTTP(response, httptest.NewRequest("GET", path, nil))
		if response.Code != status {
			t.Fatalf("%s after shutdown: %d", path, response.Code)
		}
	}
}
