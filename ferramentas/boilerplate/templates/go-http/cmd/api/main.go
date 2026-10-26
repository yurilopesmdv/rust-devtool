package main

import (
	"context"
	"errors"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
	"{{PROJECT_NAME}}/internal/config"
	"{{PROJECT_NAME}}/internal/httpapi"
	"{{PROJECT_NAME}}/internal/modules/example"
)

func run() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	service := example.NewService()
	// @boilerplate:resources
	server := &http.Server{Addr: ":" + cfg.Port, Handler: httpapi.New(service, nil, cfg.FrontendOrigin), ReadHeaderTimeout: 5 * time.Second, ReadTimeout: 10 * time.Second, WriteTimeout: 10 * time.Second, IdleTimeout: 60 * time.Second}
	done := make(chan error, 1)
	go func() { done <- server.ListenAndServe() }()
	select {
	case err := <-done:
		if !errors.Is(err, http.ErrServerClosed) {
			return err
		}
		return nil
	case <-ctx.Done():
		shutdown, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := server.Shutdown(shutdown); err != nil {
			_ = server.Close()
			return err
		}
		err := <-done
		if !errors.Is(err, http.ErrServerClosed) {
			return err
		}
		return nil
	}
}
func main() {
	if err := run(); err != nil {
		log.Print("Falha ao iniciar/encerrar API; verifique configuração e recursos")
		os.Exit(1)
	}
}
