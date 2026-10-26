package database

import (
	"context"
	"github.com/jackc/pgx/v5/pgxpool"
)

// Prepare is explicit and only bootstraps the demonstration schema.
func Prepare(ctx context.Context, pool *pgxpool.Pool) error {
	_, err := pool.Exec(ctx, "CREATE TABLE IF NOT EXISTS examples (key TEXT PRIMARY KEY, message TEXT NOT NULL)")
	return err
}
