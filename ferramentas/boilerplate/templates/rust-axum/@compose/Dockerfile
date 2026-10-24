FROM rust:1.99-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
RUN useradd --system --uid 10001 app
WORKDIR /app
COPY --from=build /app/target/release/{{PROJECT_NAME}} /app/server
USER 10001
EXPOSE 3000
CMD ["/app/server"]
