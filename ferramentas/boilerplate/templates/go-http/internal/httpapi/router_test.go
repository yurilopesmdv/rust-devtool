package httpapi

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"{{PROJECT_NAME}}/internal/modules/example"
)

func TestEndpointsAndCORS(t *testing.T) {
	handler := New(example.NewService(), func(context.Context) error { return errors.New("private database details") }, "http://localhost:5173")
	for _, item := range []struct {
		path   string
		status int
	}{
		{"/health", 200},
		{"/ready", 503},
		{"/api/v1/example", 200},
		{"/missing", 404},
	} {
		request := httptest.NewRequest("GET", item.path, nil)
		request.Header.Set("Origin", "http://localhost:5173")
		response := httptest.NewRecorder()
		handler.ServeHTTP(response, request)
		if response.Code != item.status {
			t.Fatalf("%s: %d", item.path, response.Code)
		}
		if response.Header().Get("Access-Control-Allow-Origin") != "http://localhost:5173" {
			t.Fatal("CORS missing")
		}
		if item.path == "/ready" && response.Body.String() == "private database details" {
			t.Fatal("leaked diagnostic")
		}
	}
	response := httptest.NewRecorder()
	request := httptest.NewRequest(http.MethodOptions, "/api/v1/example", nil)
	request.Header.Set("Origin", "https://other.example")
	handler.ServeHTTP(response, request)
	if response.Header().Get("Access-Control-Allow-Origin") != "" {
		t.Fatal("unexpected CORS origin")
	}
}
