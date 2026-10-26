package main

import (
	"context"
	"log"
	"os"
	"time"
	"{{PROJECT_NAME}}/internal/infrastructure/database"
)

func run() error {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	pool, err := database.Open(ctx)
	if err != nil {
		return err
	}
	defer pool.Close()
	return database.Prepare(ctx, pool)
}
func main() {
	if err := run(); err != nil {
		log.Print("Falha ao preparar schema; confira banco/configuração")
		os.Exit(1)
	}
}
