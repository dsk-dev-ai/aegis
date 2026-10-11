# Docker

## Development

Run the relay locally in a container:
```bash
docker compose -f docker/compose.dev.yml up
```

Relay will be available at `ws://localhost:8080/v1/ws` with health at `http://localhost:8080/health`.

## Production

Build and run a minimal, hardened image (to be added):
```dockerfile
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p veil-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/veil-server /usr/local/bin/veil-server
USER 1000:1000
EXPOSE 8080
ENV VEIL_RELAY_ADDR=0.0.0.0:8080
ENTRYPOINT ["/usr/local/bin/veil-server"]
```

Notes:
- Run behind a reverse proxy (nginx/Caddy) for TLS termination.
- Persist data (storage to be added).
- Use non-root, read-only FS where possible.
- Limit resources.
