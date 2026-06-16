## Architecture

The frontend⇆backend API and the backend⇆worker control plane both run over
**gRPC** (tonic). The backend is the gRPC server and the sole DB writer; workers
dial in, register, stream scan results + heartbeats, and receive commands.

- `proto/` — shared `.proto` files (`api.proto`, `worker.proto`) + generated Rust
  (tonic, via `protox` so no `protoc` binary is needed).
- `backend/` — tonic server: `ApiService` (frontend, served as gRPC-web via
  tonic-web) + `WorkerControlService` (workers). In-memory `WorkerRegistry`. Sole
  DB writer: owns the diesel schema/models (`backend/src/{schema,models,chat}.rs`)
  and the embedded migrations (`backend/migrations/`).
- `worker/` — Minecraft scanner. Streams scan results to the backend over gRPC; it
  never touches the database directly.
- `frontend/` — React + Connect-ES gRPC-web client.

## Types: proto → TS/Rust (gRPC)

Wire types come from `proto/proto/*.proto`. The old ts-rs export workflow is
**retired**; `frontend/src/types/*` are now hand-maintained UI view-models that
the adapter in `frontend/src/api/client.ts` maps protobuf messages to.

### Workflow when changing the API

1. Edit `proto/proto/api.proto` (or `worker.proto`).
2. Rust regenerates on build (`cargo build`); update the service impls.
3. Regenerate the frontend client: `cd frontend && npx buf generate` (outputs to
   `src/gen/`). Update the adapter/view-models if shapes changed.

## Type checking for frontend

Always use `npx tsc -b --noEmit` (not `npx tsc --noEmit`)

## Building

`cargo check -p worker` and `cargo check -p backend`. The worker has a single
(gRPC) build configuration.

## Database schema

`backend/src/schema.rs` (diesel migrations in `backend/migrations/`). Run the diesel
CLI from the `backend/` directory (`backend/diesel.toml` configures it).
