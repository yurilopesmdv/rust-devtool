package config

import (
	"fmt"
	"net/url"
	"os"
	"strconv"
)

type Config struct {
	Port           string
	FrontendOrigin string
}

func Load() (Config, error) {
	port := os.Getenv("PORT")
	if port == "" {
		port = "3000"
	}
	number, err := strconv.Atoi(port)
	if err != nil || number < 1 || number > 65535 {
		return Config{}, fmt.Errorf("PORT inválida")
	}
	origin := os.Getenv("FRONTEND_ORIGIN")
	if origin == "" {
		origin = "http://localhost:{{WEB_PORT}}"
	}
	parsed, err := url.Parse(origin)
	if err != nil || (parsed.Scheme != "http" && parsed.Scheme != "https") || parsed.Host == "" || parsed.User != nil || parsed.RawQuery != "" || parsed.Fragment != "" || (parsed.Path != "" && parsed.Path != "/") {
		return Config{}, fmt.Errorf("FRONTEND_ORIGIN inválida")
	}
	return Config{Port: port, FrontendOrigin: origin}, nil
}
