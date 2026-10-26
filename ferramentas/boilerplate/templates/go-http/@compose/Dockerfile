FROM golang:1.27.3-bookworm AS build
WORKDIR /src
COPY . .
RUN go mod download && go mod verify
RUN CGO_ENABLED=0 go build -trimpath -buildvcs=false -o /api ./cmd/api
# @boilerplate:prepare-build
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 app
WORKDIR /app
COPY --from=build /api /app/server
# @boilerplate:prepare-copy
USER 10001
EXPOSE 3000
CMD ["/app/server"]
