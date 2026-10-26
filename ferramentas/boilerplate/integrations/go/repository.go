package repositories

import (
	"context"
	"errors"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

type Repository struct{ Pool *pgxpool.Pool }

func (r Repository) Read(ctx context.Context) (string, error) {
	var message string
	err := r.Pool.QueryRow(ctx, "SELECT message FROM examples WHERE key = $1", "example").Scan(&message)
	if errors.Is(err, pgx.ErrNoRows) {
		return "Olá do Go", nil
	}
	return message, err
}
func (r Repository) Write(ctx context.Context, message string) error {
	_, err := r.Pool.Exec(ctx, "INSERT INTO examples (key,message) VALUES ($1,$2) ON CONFLICT (key) DO UPDATE SET message = EXCLUDED.message", "example", message)
	return err
}
