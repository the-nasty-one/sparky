# Spark Console

A Rust web dashboard for managing an NVIDIA DGX Spark (aarch64). Built with Leptos + Axum for a single-binary deployment with SSR and WASM hydration.

## Architecture

Five-crate Rust workspace:

- **spark-types** — Shared data structures (compiles for native + wasm32)
- **spark-providers** — System metric collectors (GPU, CPU, memory, disk, uptime, Docker, models)
- **spark-api** — Axum REST API routes
- **spark-ui** — Leptos frontend with SSR and WASM hydration
- **spark-console** — Binary that wires everything into a single server

## Prerequisites

- Rust nightly toolchain (managed by `rust-toolchain.toml`)
- `cargo-leptos`: `cargo install cargo-leptos --locked`
- `wasm32-unknown-unknown` target (managed by `rust-toolchain.toml`)
- For cross-compilation: `aarch64-linux-gnu-gcc`

## Development

```bash
# Copy config
cp config.example.toml config.toml

# Start dev server with auto-reload
cargo leptos watch

# Open http://localhost:3000
```

The dev server runs on x86_64 with mock data for GPU metrics (since nvidia-smi is not available on the dev machine). CPU, memory, disk, and uptime metrics come from live `/proc` data.

## Build

```bash
# Development build
cargo leptos build

# Release build for deployment target
cargo leptos build --release
```

## Deploy to DGX Spark

```bash
# Cross-compile and deploy via scp
./deploy/deploy.sh
```

This builds for `aarch64-unknown-linux-gnu`, copies the binary to the Spark, and restarts the systemd service.

### Manual Setup on Spark

```bash
# Create config directory
sudo mkdir -p /etc/spark-console
sudo cp config.example.toml /etc/spark-console/config.toml

# Install systemd service
sudo cp deploy/spark-console.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now spark-console
```

## Configuration

See `config.example.toml`:

```toml
[server]
bind = "0.0.0.0"
port = 3000
```

The dashboard is designed for LAN-only, single-user operation with no authentication.

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/system` | Full system metrics |
| GET | `/api/v1/system/gpu` | GPU metrics only |
| GET | `/api/v1/system/memory` | Memory metrics only |
| GET | `/api/v1/containers` | List all Docker containers |
| POST | `/api/v1/containers/action` | Start/stop/restart a container |
| GET | `/api/v1/models` | List discovered model files |
