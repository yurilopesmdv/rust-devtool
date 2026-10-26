package database

import (
	"context"
	"errors"
	"github.com/jackc/pgx/v5/pgxpool"
	"net/url"
	"os"
	"time"
)

func Open(ctx context.Context) (*pgxpool.Pool, error) {
	address := os.Getenv("DATABASE_URL")
	parsed, err := url.Parse(address)
	if err != nil || (parsed.Scheme != "postgres" && parsed.Scheme != "postgresql") || parsed.Host == "" || parsed.Path == "" {
		return nil, errors.New("DATABASE_URL PostgreSQL obrigatória")
	}
	cfg, err := pgxpool.ParseConfig(address)
	if err != nil {
		return nil, errors.New("DATABASE_URL PostgreSQL inválida")
	}
	cfg.MaxConns = 10
	cfg.MinConns = 0
	cfg.MaxConnLifetime = 30 * time.Minute
	cfg.ConnConfig.ConnectTimeout = 3 * time.Second
	pool, err := pgxpool.NewWithConfig(ctx, cfg)
	if err != nil {
		return nil, errors.New("Falha ao criar pool")
	}
	startup, cancel := context.WithTimeout(ctx, 3*time.Second)
	defer cancel()
	if err := pool.Ping(startup); err != nil {
		pool.Close()
		return nil, errors.New("Banco indisponível no startup")
	}
	return pool, nil
}
