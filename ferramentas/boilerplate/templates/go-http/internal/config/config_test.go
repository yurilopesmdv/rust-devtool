package config

import "testing"

func TestInvalidConfiguration(t *testing.T) {
	t.Setenv("PORT", "65536")
	if _, err := Load(); err == nil {
		t.Fatal("invalid port accepted")
	}
	t.Setenv("PORT", "3000")
	t.Setenv("FRONTEND_ORIGIN", "https://user:secret@example.com")
	if _, err := Load(); err == nil {
		t.Fatal("credentials accepted in public origin")
	}
}
