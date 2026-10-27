#!/bin/sh
set -eu
export GOTOOLCHAIN=local
go mod download
go mod verify
unformatted=$(gofmt -l .)
if [ -n "$unformatted" ]; then
    printf '%s\n' "$unformatted" >&2
    exit 1
fi
go test -race ./...
go vet ./...
go build -buildvcs=false -o bin/api ./cmd/api
